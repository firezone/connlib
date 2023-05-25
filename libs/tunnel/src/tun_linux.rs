use futures::{ready, TryStreamExt};
use libs_common::{boringtun::device::tun::TunSocket, Error, Result};
use rtnetlink::{new_connection, packet::nlas::link::Nla, Handle};
use std::{pin::Pin, task::Poll};
use tokio::{
    io::{unix::AsyncFd, AsyncRead, AsyncWrite},
    task::JoinHandle,
};

use libc::{__errno_location, c_short, c_uchar, strerror, IFNAMSIZ};

use super::{resolvconf, InterfaceConfig};

const TUN_DRIVER: &str = "/dev/net/tun";
const TUNSETIFF: u64 = 0x4004_54ca;

// Re-implementing TunSocket would be much better but using it as this for now
#[derive(Debug)]
pub struct DeviceChannel(tokio::io::unix::AsyncFd<TunSocket>);

#[derive(Debug)]
pub struct IfaceDevice {
    device_channel: Option<DeviceChannel>,
    name: String,
    interface_index: u32,
    handle: Handle,
    join_handle: JoinHandle<()>,
}

impl DeviceChannel {
    pub(crate) async fn read(&self, out: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let mut guard = self.0.readable().await?;

            match guard.try_io(|inner| {
                inner.get_ref().read(out).map_err(|err| {
                    if let boringtun::device::Error::IfaceRead(e) = err {
                        std::io::Error::from_raw_os_error(e)
                    } else {
                        panic!("we expect read to only return ifaceread errors")
                    }
                })
            }) {
                Ok(result) => return result.map(|e| e.len()),
                Err(_would_block) => continue,
            }
        }
    }

    pub(crate) async fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        loop {
            let mut guard = self.0.writable().await?;

            // write4 and write6 does the same (must be different in macos or something)
            match guard.try_io(|inner| {
                let res = inner.get_ref().write4(buf);
                if res < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(res)
                }
            }) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncRead for DeviceChannel {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        loop {
            let mut guard = ready!(self.0.poll_read_ready(cx))?;

            let unfilled = buf.initialize_unfilled();
            match guard.try_io(|inner| {
                inner
                    .get_ref()
                    .read(unfilled)
                    .map_err(|err| {
                        if let boringtun::device::Error::IfaceRead(e) = err {
                            std::io::Error::from_raw_os_error(e)
                        } else {
                            panic!("we expect read to only return ifaceread errors")
                        }
                    })
                    .map(|res| res.len())
            }) {
                Ok(Ok(res)) => {
                    buf.advance(res);
                    return Poll::Ready(Ok(()));
                }
                Ok(Err(err)) => return Poll::Ready(Err(err)),
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for DeviceChannel {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        loop {
            let mut guard = ready!(self.0.poll_write_ready(cx))?;

            match guard.try_io(|inner| {
                let res = inner.get_ref().write4(buf);
                if res < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(res)
                }
            }) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        // flush is a no-op
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        // Shutdown is also a no-op
        Poll::Ready(Ok(()))
    }
}

impl Drop for IfaceDevice {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

impl IfaceDevice {
    // TODO: I don't like this kind of API that you can't call this method twice.
    // we could use an enum to represent the device state or return this upon iface creation.
    pub(crate) fn get_device_channel(&mut self) -> Result<DeviceChannel> {
        self.device_channel.take().ok_or_else(|| Error)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) async fn set_iface_config(&self, config: &InterfaceConfig) -> Result<()> {
        let ips = self
            .handle
            .address()
            .get()
            .set_link_index_filter(self.interface_index)
            .execute();

        ips.try_for_each(|ip| self.handle.address().del(ip).execute())
            .await?;

        for addr in config.address.values() {
            self.handle
                .address()
                .add(self.interface_index, addr.addr(), addr.prefix_len())
                .execute()
                .await?
        }

        let name: String = self.name.clone().try_into()?;
        for dns in &config.dns {
            resolvconf::set_dns(&name, dns).await?;
        }

        //nftables::enable_masquerade((config.ipv4_masquerade, config.ipv6_masquerade)).await?;

        Ok(())
    }

    pub(crate) async fn up(&self) -> Result<()> {
        self.handle
            .link()
            .set(self.interface_index)
            .up()
            .execute()
            .await?;

        Ok(())
    }

    pub(crate) async fn mtu(&self) -> Result<u32> {
        while let Ok(Some(msg)) = self
            .handle
            .link()
            .get()
            .match_index(self.interface_index)
            .execute()
            .try_next()
            .await
        {
            for nla in msg.nlas {
                if let Nla::Mtu(mtu) = nla {
                    return Ok(mtu);
                }
            }
        }

        Err(Error::IFaceError)
    }

    // TODO: Do we need to set non-blocking?
    // TODO: More importantly, here we are setting multi-queue
    // however, to read multiqueued data we need to alloc multiple fd
    // see: https://www.kernel.org/doc/Documentation/networking/tuntap.txt multiqueue section
    // maybe we can hold a Vec<File> and that's it? I think so
    pub(crate) async fn new(name: String) -> Result<Self> {
        // TODO unwrap(boringtun's error doesn't implement StdError)
        let dev = TunSocket::new(&name).unwrap().set_non_blocking().unwrap();
        let dev = AsyncFd::new(dev)?;

        let (connection, handle, _) =
            new_connection().context("Couldn't get netlink connection")?;
        let join_handle = tokio::spawn(connection);
        let interface_index = handle
            .link()
            .get()
            .match_name(
                name.clone()
                    .try_into()
                    .context("we are not supporting non utf-8 interface names for now")?,
            )
            .execute()
            .try_next()
            .await
            .context("Couldn't get index of created interface")?
            .ok_or_else(|| Error::IFaceError)?
            .header
            .index;
        Ok(Self {
            device_channel: Some(DeviceChannel(dev)),
            name,
            interface_index,
            handle,
            join_handle,
        })
    }
}

#[repr(C)]
union IfrIfru {
    ifru_flags: c_short,
}

#[repr(C)]
pub struct Ifreq {
    ifr_name: [c_uchar; IFNAMSIZ],
    ifr_ifru: IfrIfru,
}
