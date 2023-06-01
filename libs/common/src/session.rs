use async_trait::async_trait;
use backoff::{backoff::Backoff, ExponentialBackoffBuilder};
use boringtun::x25519::{PublicKey, StaticSecret};
use rand_core::OsRng;
use std::{
    marker::PhantomData,
    net::{Ipv4Addr, Ipv6Addr},
};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{Receiver, Sender},
};
use url::Url;

use crate::{control::PhoenixChannel, error_type::ErrorType, messages::Key, Error, Result};

// TODO: Not the most tidy trait for a control-plane.
/// Trait that represents a control-plane.
#[async_trait]
pub trait ControlSession<T, U> {
    /// Start control-plane with the given private-key in the background.
    async fn start(private_key: StaticSecret) -> Result<(Sender<T>, Receiver<U>)>;

    /// Either "gateway" or "client" used to get the control-plane URL.
    fn socket_path() -> &'static str;
}

// TODO: Currently I'm using Session for both gateway and clients
// however, gateway could use the runtime directly and could make things easier
// so revisit this.
/// A session is the entry-point for connlib, mantains the runtime and the tunnel.
///
/// A session is created using [Session::connect], then to stop a session we use [Session::disconnect].
pub struct Session<T, U, V> {
    runtime: Option<Runtime>,
    _phantom: PhantomData<(T, U, V)>,
}

/// Resource list that will be displayed to the users.
pub struct ResourceList {
    pub resources: Vec<String>,
}

/// Tunnel addresses to be surfaced to the client apps.
pub struct TunnelAddresses {
    /// IPv4 Address.
    pub address4: Ipv4Addr,
    /// IPv6 Address.
    pub address6: Ipv6Addr,
}

// Evaluate doing this not static
/// Traits that will be used by connlib to callback the client upper layers.
pub trait Callbacks {
    /// Called when there's a change in the resource list.
    fn on_update_resources(resource_list: ResourceList);
    /// Called when the tunnel address is set.
    fn on_set_tunnel_adresses(tunnel_addresses: TunnelAddresses);
    /// Called when there's an error.
    ///
    /// # Parameters
    /// - `error`: The actual error that happened.
    /// - `error_type`: Wether the error should terminate the session or not.
    fn on_error(error: &Error, error_type: ErrorType);
}

macro_rules! fatal_error {
    ($result:expr, $c:ty) => {
        match $result {
            Ok(res) => res,
            Err(e) => {
                <$c>::on_error(&e, ErrorType::Fatal);
                return;
            }
        }
    };
}

impl<T, U, V> Session<T, U, V>
where
    T: ControlSession<U, V>,
    U: for<'de> serde::Deserialize<'de> + std::fmt::Debug + Send + 'static,
    V: serde::Serialize + Send + 'static,
{
    /// Block on waiting for ctrl+c to terminate the runtime.
    /// (Used for the gateways).
    pub fn wait_for_ctrl_c(&mut self) -> Result<()> {
        self.runtime
            .as_ref()
            .ok_or(Error::NoRuntime)?
            .block_on(async {
                tokio::signal::ctrl_c().await?;
                Ok(())
            })
    }

    /// Starts a session in the background.
    ///
    /// This will:
    /// 1. Create and start a tokio runtime
    /// 2. Connect to the control plane to the portal
    /// 3. Start the tunnel in the background and forward control plane messages to it.
    ///
    /// The generic parameter `C` should implement all the handlers and that's how errors will be surfaced.
    ///
    /// On a fatal error you should call `[Session::disconnect]` and start a new one.
    // TODO: token should be something like SecretString but we need to think about FFI compatibiltiy
    pub fn connect<C: Callbacks>(portal_url: impl TryInto<Url>, token: String) -> Result<Self> {
        // TODO: We could use tokio::runtime::current() to get the current runtime
        // which could work with swif-rust that already runs a runtime. But IDK if that will work
        // in all pltaforms, a couple of new threads shouldn't bother none.
        // Big question here however is how do we get the result? We could block here await the result and spawn a new task.
        // but then platforms should know that this function is blocking.

        let portal_url = portal_url.try_into().map_err(|_| Error::UriError)?;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        runtime.spawn(async move {
                let private_key = StaticSecret::random_from_rng(OsRng);
                let self_id = uuid::Uuid::new_v4();

                let connect_url = fatal_error!(get_websocket_path(portal_url, token, T::socket_path(), &Key(PublicKey::from(&private_key).to_bytes()), &self_id.to_string()), C);

                let (sender, mut receiver) = fatal_error!(T::start(private_key).await, C);

                let mut connection = PhoenixChannel::new(connect_url, move |msg| {
                    let sender = sender.clone();
                    async move {
                        tracing::trace!("Recieved message: {msg:?}");
                        if let Err(e) = sender.send(msg).await {
                            tracing::warn!("Recieved a message after handler already closed: {e}. Probably message recieved during session clean up.");
                        }
                    }
                });

                // Used to send internal messages
                let mut internal_sender = connection.sender();
                let topic = T::socket_path().to_string();
                let topic_send = topic.clone();

                tokio::spawn(async move {
                    let mut exponential_backoff = ExponentialBackoffBuilder::default().build();
                    loop {
                        let result = connection.start(vec![topic.clone()]).await;
                        if let Some(t) = exponential_backoff.next_backoff() {
                            tracing::warn!("Error during connection to the portal, retrying in {} seconds", t.as_secs());
                            tokio::time::sleep(t).await;
                            match result {
                                Ok(()) => C::on_error(&tokio_tungstenite::tungstenite::Error::ConnectionClosed.into(), ErrorType::Recoverable),
                                Err(e) => C::on_error(&e, ErrorType::Recoverable)
                            }
                        } else {
                            tracing::error!("Conneciton to the portal error, check your internet or the status of the portal.\nDisconnecting interface.");
                            match result {
                                Ok(()) => C::on_error(&crate::Error::PortalConnectionError(tokio_tungstenite::tungstenite::Error::ConnectionClosed), ErrorType::Fatal),
                                Err(e) => C::on_error(&e, ErrorType::Fatal)
                            }
                            break;
                        }
                    }

                });

                // TODO: Implement Sink for PhoenixEvent (created from a PhoenixSender event + topic)
                // that way we can simply do receiver.forward(sender)
                tokio::spawn(async move {
                    while let Some(message) = receiver.recv().await {
                        if let Err(err) = internal_sender.send(&topic_send, message).await {
                            tracing::error!("Channel already closed when trying to send message: {err}. Probably trying to send a message during session clean up.");
                        }
                    }
                });
        });

        Ok(Self {
            runtime: Some(runtime),
            _phantom: PhantomData,
        })
    }

    /// Cleanup a [Session].
    ///
    /// For now this just drops the runtime, which should drop all pending tasks.
    /// Further cleanup should be done here. (Otherwise we can just drop [Session]).
    pub fn disconnect(&mut self) -> bool {
        // 1. Close the websocket connection
        // 2. Free the device handle (UNIX)
        // 3. Close the file descriptor (UNIX)
        // 4. Remove the mapping

        // The way we cleanup the tasks is we drop the runtime
        // this means we don't need to keep track of different tasks
        // but if any of the tasks never yields this will block forever!
        // So always yield and if you spawn a blocking tasks rewrite this.
        // Furthermore, we will depend on Drop impls to do the list above so,
        // implement them :)
        self.runtime = None;
        true
    }

    /// TODO
    pub fn bump_sockets(&self) -> bool {
        true
    }

    /// TODO
    pub fn disable_some_roaming_for_broken_mobile_semantics(&self) -> bool {
        true
    }
}

fn get_websocket_path(
    mut url: Url,
    secret: String,
    mode: &str,
    public_key: &Key,
    external_id: &str,
) -> Result<Url> {
    {
        let mut paths = url.path_segments_mut().map_err(|_| Error::UriError)?;
        paths.pop_if_empty();
        paths.push(mode);
        paths.push("websocket");
    }

    {
        let mut query_pairs = url.query_pairs_mut();
        query_pairs.clear();
        query_pairs.append_pair("token", &secret);
        query_pairs.append_pair("public_key", &public_key.to_string());
        query_pairs.append_pair("external_id", external_id);
    }

    Ok(url)
}
