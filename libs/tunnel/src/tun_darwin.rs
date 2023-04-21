use libs_common::{boringtun::device::tun::TunSocket, Result};
use std::os::fd::{AsRawFd, RawFd};
use tokio::io::unix::AsyncFd;

use super::InterfaceConfig;

// TODO: we have to replace TunSocket because we need to use netpacketprovider to get approved in the app store
#[derive(Debug)]
pub(crate) struct DeviceChannel(tokio::io::unix::AsyncFd<TunSocket>);

#[derive(Debug)]
pub(crate) struct IfaceDevice {
    fd: RawFd,
}

impl DeviceChannel {
    pub(crate) async fn mtu(&self) -> Result<usize> {
        Ok(self.0.get_ref().mtu()?)
    }

    pub(crate) async fn read(&self, out: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let mut guard = self.0.readable().await?;

            match guard.try_io(|inner| {
                inner.get_ref().read(out).map_err(|err| match err {
                    libs_common::boringtun::device::Error::IfaceRead(e) => e,
                    _ => panic!("Unexpected error while trying to read network interface"),
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

            // write4 and write6 does the same
            match guard.try_io(|inner| match inner.get_ref().write4(buf) {
                0 => Err(std::io::Error::last_os_error()),
                i => Ok(i),
            }) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }
}

impl IfaceDevice {
    // It's easier to not make these functions async, setting these should not block the thread for too long
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn set_iface_config(&mut self, config: &InterfaceConfig) -> Result<()> {
        // TODO

        Ok(())
    }

    pub fn up(&mut self) -> Result<()> {
        // TODO
        Ok(())
    }
}

pub(crate) async fn create_iface() -> Result<(IfaceDevice, DeviceChannel)> {
    let dev = TunSocket::new("utun").unwrap().set_non_blocking().unwrap();
    let fd = dev.as_raw_fd();
    tracing::trace!("Started new interface with name: {:?}", dev.name());
    let dev = AsyncFd::new(dev)?;

    Ok((IfaceDevice { fd }, DeviceChannel(dev)))
}
