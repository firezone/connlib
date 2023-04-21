use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use bytes::Bytes;
use libs_common::{
    boringtun::noise::{Tunn, TunnResult},
    error_type::ErrorType,
    Callbacks,
};
use parking_lot::Mutex;
use webrtc::data::data_channel::DataChannel;

use super::PeerConfig;

pub(crate) struct Peer {
    pub(crate) tunnel: Mutex<Tunn>,
    pub(crate) index: u32,
    preshared_key: [u8; 32],
    pub(crate) allowed_ipv4: Ipv4Addr,
    pub(crate) allowed_ipv6: Ipv6Addr,
    pub(crate) channel: Arc<DataChannel>,
}

impl Peer {
    pub(crate) async fn send_infallible<CB: Callbacks>(&self, data: &[u8]) {
        if let Err(e) = self.channel.write(&Bytes::copy_from_slice(data)).await {
            tracing::error!("Couldn't send  packet to connected peer: {e}");
            CB::on_error(&e.into(), ErrorType::Recoverable);
        }
    }

    pub(crate) fn from_config(
        tunnel: Tunn,
        index: u32,
        config: &PeerConfig,
        channel: Arc<DataChannel>,
    ) -> Self {
        let preshared_key = config.preshared_key.to_bytes();

        Self::new(
            Mutex::new(tunnel),
            index,
            config.ipv4,
            config.ipv6,
            preshared_key,
            channel,
        )
    }

    pub(crate) fn new(
        tunnel: Mutex<Tunn>,
        index: u32,
        ipv4: Ipv4Addr,
        ipv6: Ipv6Addr,
        preshared_key: [u8; 32],
        channel: Arc<DataChannel>,
    ) -> Peer {
        Peer {
            tunnel,
            index,
            allowed_ipv4: ipv4,
            allowed_ipv6: ipv6,
            preshared_key,
            channel,
        }
    }

    pub(crate) fn update_timers<'a>(&self, dst: &'a mut [u8]) -> TunnResult<'a> {
        self.tunnel.lock().update_timers(dst)
    }

    pub(crate) fn is_allowed_ipv4(&self, addr: &Ipv4Addr) -> bool {
        &self.allowed_ipv4 == addr
    }

    pub(crate) fn is_allowed_ipv6(&self, addr: &Ipv6Addr) -> bool {
        &self.allowed_ipv6 == addr
    }
}
