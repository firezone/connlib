use super::InterfaceConfig;
use libs_common::Result;

// This is an stubbed out module to be able to compile on windows.
#[derive(Debug)]
pub(crate) struct DeviceChannel;

#[derive(Debug)]
pub(crate) struct IfaceDevice;

impl DeviceChannel {
    pub(crate) async fn mtu(&self) -> Result<usize> {
        todo!()
    }

    pub(crate) async fn read(&self, out: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }

    pub(crate) async fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }
}

impl IfaceDevice {
    // It's easier to not make these functions async, setting these should not block the thread for too long
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn set_iface_config(&mut self, config: &InterfaceConfig) -> Result<()> {
        todo!()
    }

    pub fn up(&mut self) -> Result<()> {
        todo!()
    }
}

pub(crate) async fn create_iface() -> Result<(IfaceDevice, DeviceChannel)> {
    todo!()
}
