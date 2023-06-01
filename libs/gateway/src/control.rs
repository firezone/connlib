use std::{sync::Arc, time::Duration};

use firezone_tunnel::{ControlSignal, Tunnel};
use libs_common::{
    boringtun::x25519::StaticSecret,
    error_type::ErrorType::{Fatal, Recoverable},
    messages::ResourceDescription,
    Callbacks, ControlSession, Result,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use super::messages::{
    ConnectionReady, ConnectionRequest, EgressMessages, IngressMessages, InitGateway, Resource,
};

use async_trait::async_trait;

const INTERNAL_CHANNEL_SIZE: usize = 256;

pub struct ControlPlane<C: Callbacks> {
    tunnel: Arc<Tunnel<ControlSignaler, C>>,
    control_signaler: ControlSignaler,
}

#[derive(Clone)]
struct ControlSignaler {
    internal_sender: Arc<Sender<EgressMessages>>,
}

#[async_trait]
impl ControlSignal for ControlSignaler {
    async fn signal_connection_to(&self, resource: &ResourceDescription) -> Result<()> {
        tracing::warn!("A message to network resource: {resource:?} was discarded, gateways aren't meant to be used as clients.");
        Ok(())
    }
}

impl<C: Callbacks> ControlPlane<C>
where
    C: Send + Sync + 'static,
{
    #[tracing::instrument(level = "trace", skip(self))]
    async fn start(mut self, mut receiver: Receiver<IngressMessages>) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            tokio::select! {
                Some(msg) = receiver.recv() => self.handle_message(msg).await,
                _ = interval.tick() => self.stats_event().await,
                else => break
            }
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn init(&mut self, init: InitGateway) {
        if let Err(e) = self.tunnel.set_interface(&init.interface).await {
            tracing::error!("Couldn't intialize interface: {e}");
            C::on_error(&e, Fatal);
            return;
        }

        // TODO: Enable masquerading here.
        tracing::info!("Firezoned Started!");
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn connection_request(&self, connection_request: ConnectionRequest) {
        let tunnel = Arc::clone(&self.tunnel);
        let control_signaler = self.control_signaler.clone();
        tokio::spawn(async move {
            match tunnel
                .set_peer_connection_request(
                    connection_request.rtc_sdp,
                    connection_request.client.peer.into(),
                    connection_request.relays,
                    connection_request.client.id,
                )
                .await
            {
                Ok(gateway_rtc_sdp) => {
                    if let Err(err) = control_signaler
                        .internal_sender
                        .send(EgressMessages::ConnectionReady(ConnectionReady {
                            client_id: connection_request.client.id,
                            gateway_rtc_sdp,
                        }))
                        .await
                    {
                        tunnel.cleanup_peer_connection(connection_request.client.id);
                        C::on_error(&err.into(), Recoverable);
                    }
                }
                Err(err) => {
                    tunnel.cleanup_peer_connection(connection_request.client.id);
                    C::on_error(&err, Recoverable);
                }
            }
        });
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn add_resource(&self, resource: Resource) {
        todo!()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub(super) async fn handle_message(&mut self, msg: IngressMessages) {
        match msg {
            IngressMessages::Init(init) => self.init(init).await,
            IngressMessages::ConnectionRequest(connection_request) => {
                self.connection_request(connection_request)
            }
            IngressMessages::AddResource(resource) => self.add_resource(resource),
            IngressMessages::RemoveResource(_) => todo!(),
            IngressMessages::UpdateResource(_) => todo!(),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub(super) async fn stats_event(&mut self) {
        tracing::debug!("TODO: STATS EVENT");
    }
}

#[async_trait]
impl<C: Callbacks> ControlSession<IngressMessages, EgressMessages> for ControlPlane<C>
where
    C: Send + Sync + 'static,
{
    #[tracing::instrument(level = "trace", skip(private_key))]
    async fn start(
        private_key: StaticSecret,
    ) -> Result<(Sender<IngressMessages>, Receiver<EgressMessages>)> {
        // This is kinda hacky, the buffer size is 1 so that we make sure that we
        // process one message at a time, blocking if a previous message haven't been processed
        // to force queue ordering.
        // (couldn't find any other guarantee of the ordering of message)
        let (sender, receiver) = channel::<IngressMessages>(1);

        let (internal_sender, internal_receiver) = channel(INTERNAL_CHANNEL_SIZE);
        let internal_sender = Arc::new(internal_sender);
        let control_signaler = ControlSignaler { internal_sender };
        let tunnel = Arc::new(Tunnel::<_, C>::new(private_key, control_signaler.clone()).await?);

        let control_plane = ControlPlane {
            tunnel,
            control_signaler,
        };

        // TODO: We should have some kind of callback from clients to surface errors here
        tokio::spawn(async move { control_plane.start(receiver).await });

        Ok((sender, internal_receiver))
    }

    fn socket_path() -> &'static str {
        "gateway"
    }
}
