use libc::{
    close, fcntl, ioctl, sockaddr, sockaddr_in, socket, write, AF_INET, F_GETFL, F_SETFL,
    IFF_MULTI_QUEUE, IFF_NO_PI, IFF_TUN, IFNAMSIZ, IF_NAMESIZE, IPPROTO_IP, O_NONBLOCK, O_RDWR,
    SIOCGIFMTU, SOCK_STREAM,
};
use libs_common::{Error, Result};
use std::{
    ffi::{c_int, c_short, c_uchar},
    io,
    os::fd::{AsRawFd, RawFd},
    sync::Arc,
};

use super::InterfaceConfig;

#[derive(Debug)]
pub(crate) struct IfaceConfig(pub(crate) Arc<IfaceDevice>);

const TUNSETIFF: u64 = 0x4004_54ca;
const TUN_FILE: &[u8] = b"/dev/net/tun\0";

#[repr(C)]
union IfrIfru {
    ifru_addr: sockaddr,
    ifru_addr_v4: sockaddr_in,
    ifru_addr_v6: sockaddr_in,
    ifru_dstaddr: sockaddr,
    ifru_broadaddr: sockaddr,
    ifru_flags: c_short,
    ifru_metric: c_int,
    ifru_mtu: c_int,
    ifru_phys: c_int,
    ifru_media: c_int,
    ifru_intval: c_int,
    ifru_wake_flags: u32,
    ifru_route_refcnt: u32,
    ifru_cap: [c_int; 2],
    ifru_functional_type: u32,
}

#[repr(C)]
pub struct ifreq {
    ifr_name: [c_uchar; IFNAMSIZ],
    ifr_ifru: IfrIfru,
}

#[derive(Default, Debug)]
pub struct IfaceDevice {
    fd: RawFd,
    name: String,
}

impl Drop for IfaceDevice {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}

impl AsRawFd for IfaceDevice {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl IfaceDevice {
    fn write(&self, buf: &[u8]) -> usize {
        match unsafe { write(self.fd, buf.as_ptr() as _, buf.len() as _) } {
            -1 => 0,
            n => n as usize,
        }
    }

    pub fn new(name: &str) -> Result<IfaceDevice> {
        let fd = match unsafe { open(TUN_FILE.as_ptr() as _, O_RDWR) } {
            -1 => return Err(get_last_error()),
            fd => fd,
        };

        let iface_name = name.as_bytes();
        let mut ifr = ifreq {
            ifr_name: [0; IFNAMSIZ],
            ifr_ifru: IfrIfru {
                ifru_flags: (IFF_TUN | IFF_NO_PI | IFF_MULTI_QUEUE) as _,
            },
        };

        if iface_name.len() >= ifr.ifr_name.len() {
            return Err(Error::InvalidTunnelName);
        }

        ifr.ifr_name[..iface_name.len()].copy_from_slice(iface_name);

        if unsafe { ioctl(fd, TUNSETIFF as _, &ifr) } < 0 {
            return Err(get_last_error());
        }

        let name = name.to_string();
        Ok(TunSocket { fd, name })
    }

    pub fn set_non_blocking(self) -> Result<Self> {
        match unsafe { fcntl(self.fd, F_GETFL) } {
            -1 => Err(get_last_error()),
            flags => match unsafe { fcntl(self.fd, F_SETFL, flags | O_NONBLOCK) } {
                -1 => Err(get_last_error()),
                _ => Ok(self),
            },
        }
    }

    pub fn name(&self) -> Result<String> {
        Ok(self.name.clone())
    }

    /// Get the current MTU value
    pub fn mtu(&self) -> Result<usize> {
        let fd = match unsafe { socket(AF_INET, SOCK_STREAM, IPPROTO_IP) } {
            -1 => return Err(get_last_error()),
            fd => fd,
        };

        let name = self.name()?;
        let iface_name: &[u8] = name.as_ref();
        let mut ifr = ifreq {
            ifr_name: [0; IF_NAMESIZE],
            ifr_ifru: IfrIfru { ifru_mtu: 0 },
        };

        ifr.ifr_name[..iface_name.len()].copy_from_slice(iface_name);

        if unsafe { ioctl(fd, SIOCGIFMTU as _, &ifr) } < 0 {
            return Err(get_last_error());
        }

        unsafe { close(fd) };

        Ok(unsafe { ifr.ifr_ifru.ifru_mtu } as _)
    }

    pub fn write4(&self, src: &[u8]) -> usize {
        self.write(src)
    }

    pub fn write6(&self, src: &[u8]) -> usize {
        self.write(src)
    }

    pub fn read<'a>(&self, dst: &'a mut [u8]) -> Result<&'a mut [u8]> {
        match unsafe { read(self.fd, dst.as_mut_ptr() as _, dst.len()) } {
            -1 => Err(Error::IfaceRead(io::Error::last_os_error())),
            n => Ok(&mut dst[..n as usize]),
        }
    }
}

fn get_last_error() -> Error {
    Error::Io(io::Error::last_os_error())
}

impl IfaceConfig {
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn set_iface_config(&mut self, config: &InterfaceConfig) -> Result<()> {
        todo!()
    }

    pub fn up(&mut self) -> Result<()> {
        todo!()
    }
}
