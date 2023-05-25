//! Control protocol related module.
//!
//! This modules contains the logic for handling in and out messages through the control plane.
//! Handling of the message itself can be found in the other lib crates.
//!
//! Entrypoint for this module is [PhoenixChannel].
use std::{marker::PhantomData, time::Duration};

use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    TryStreamExt,
};
use futures_util::{Future, SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite};
use tungstenite::Message;
use url::Url;

use crate::Result;

const CHANNEL_SIZE: usize = 1_000;

/// Main struct to interact with the control-protocol channel.
///
/// After creating a new `PhoenixChannel` using [PhoenixChannel::new] you need to
/// use [start][PhoenixChannel::start] for the channel to do anything.
///
/// If you want to send something through the channel you need to obtain a [PhoenixSender] through
/// [PhoenixChannel::sender], this will already clone the sender so no need to clone it after you obtain it.
///
/// When [PhoenixChannel::start] is called a new websocket is created that will listen message from the control plane
/// based on the parameters passed on [new][PhoenixChannel::new], from then on any messages sent with a sender
/// obtained by [PhoenixChannel::sender] will be forwarded to the websocket up to the control plane. Ingress messages
/// will be passed on to the `handler` provided in [PhoenixChannel::new].
///
/// The future returned by [PhoenixChannel::start] will finish when the websocket closes (by an error), meaning that if you
/// `await` it, it will block until you use `close` in a [PhoenixSender], the portal close the connection or something goes wrong.
pub struct PhoenixChannel<F, I> {
    uri: Url,
    handler: F,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    _phantom: PhantomData<I>,
}

impl<F, Fut, I> PhoenixChannel<F, I>
where
    I: DeserializeOwned,
    F: Fn(I) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    /// Starts the tunnel with the parameters given in [Self::new].
    ///
    /// See [struct-level docs][PhoenixChannel] for more info.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn start(&mut self) -> std::result::Result<(), tungstenite::Error> {
        tracing::trace!("Trying to connect to the portal...");

        let (ws_stream, _) = connect_async(&self.uri).await?;

        tracing::trace!("Successfully connected to portal");

        let (write, read) = ws_stream.split();

        let mut sender = self.sender();
        let Self {
            handler, receiver, ..
        } = self;

        let process_messages = read.try_for_each(|message| async {
            Self::message_process(handler, message).await;
            Ok(())
        });

        // TODO: is Forward cancel safe?
        // I would assume it is and that's the advantage over
        // while let Some(item) = reciever.next().await { write.send(item) } ...
        // but double check this!
        // If it's not cancel safe this means an item can be consumed and never sent.
        // Furthermore can this also happen if write errors out? *that* I'd assume is possible...
        // What option is left? write a new future to forward items.
        // For now we should never assume that an item arrived the portal because we sent it!
        let send_messages = receiver.map(Ok).forward(write);

        let phoenix_heartbeat = tokio::spawn(async move {
            let mut timer = tokio::time::interval(Duration::from_secs(30));
            loop {
                timer.tick().await;
                let Ok(_) = sender.send("phoenix", "heartbeat", Empty {}).await else {break};
            }
        });

        futures_util::pin_mut!(process_messages, send_messages);
        // processing messages should be quick otherwise it'd block sending messages.
        // we could remove this limitation by spawning a separate taks for each of these.
        let result = futures::future::select(process_messages, send_messages)
            .await
            .factor_first()
            .0;
        phoenix_heartbeat.abort();
        result?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(handler))]
    async fn message_process(handler: &F, message: tungstenite::Message) {
        tracing::trace!("{message:?}");

        match message.into_text() {
            Ok(m_str) => match serde_json::from_str::<PhoenixMessage<I>>(&m_str) {
                Ok(m) => match m.payload {
                    Payload::Message(m) => handler(m).await,
                    Payload::PhoenixReply { status, .. } => {
                        // TODO: This could be an error
                        tracing::trace!("Recieved phoenix status message: {status}")
                    }
                },
                Err(e) => {
                    tracing::error!("Error deserializing message {m_str}: {e:?}");
                }
            },
            _ => tracing::error!("Recieved message that is not text"),
        }
    }

    /// Obtains a new sender that can be used to send message with this [PhoenixChannel] to the portal.
    ///
    /// Note that for the sender to relay any message will need the future returned [PhoenixChannel::start] to be polled (await it),
    /// and [PhoenixChannel::start] takes `&mut self`, meaning you need to get the sender before running [PhoenixChannel::start].
    pub fn sender(&self) -> PhoenixSender {
        PhoenixSender {
            sender: self.sender.clone(),
        }
    }

    /// Creates a new [PhoenixChannel] not started yet.
    ///
    /// # Parameters:
    /// - `uri`: Portal's websocket uri
    /// - `handler`: The handle that will be called for each recieved message.
    ///
    /// For more info see [struct-level docs][PhoenixChannel].
    pub fn new(uri: Url, handler: F) -> Self {
        let (sender, receiver) = channel(CHANNEL_SIZE);

        Self {
            sender,
            receiver,
            uri,
            handler,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum Payload<T> {
    // TODO: We should be able to extract a Result from this.
    PhoenixReply { response: Empty, status: String },
    Message(T),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
struct PhoenixMessage<T> {
    topic: String,
    event: String,
    payload: Payload<T>,
    #[serde(rename = "ref")]
    reference: Option<i32>,
}

impl<T> PhoenixMessage<T> {
    fn new(topic: impl Into<String>, event: impl Into<String>, payload: T) -> Self {
        Self {
            topic: topic.into(),
            event: event.into(),
            payload: Payload::Message(payload),
            reference: Some(0),
        }
    }
}

// Awful hack to get serde_json to generate an empty "{}" instead of using "null"
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
struct Empty {}

/// You can use this sender to send messages through a `PhoenixChannel`.
///
/// Messages won't be sent unless [PhoenixChannel::start] is running, internally
/// this sends messages through a future channel that are forwrarded then in [PhoenixChannel] event loop
pub struct PhoenixSender {
    sender: Sender<Message>,
}

impl PhoenixSender {
    /// Sends a message upstream to a connected [PhoenixChannel].
    ///
    /// # Parameters
    /// - topic: Phoenix topic
    /// - event: Phoenix event
    /// - payload: Message's payload
    pub async fn send(
        &mut self,
        topic: impl Into<String>,
        event: impl Into<String>,
        payload: impl Serialize,
    ) -> Result<()> {
        let str = serde_json::to_string(&PhoenixMessage::new(topic, event, payload))?;
        self.sender.send(Message::text(str)).await?;
        Ok(())
    }

    /// Join a phoenix topic, meaning that after this method is invoked [PhoenixChannel] will
    /// recieve messages from that topic, given that upstream accepts you into the given topic.
    pub async fn join_topic(&mut self, topic: impl Into<String>) -> Result<()> {
        self.send(topic, "phx_join", Empty {}).await
    }

    /// Closes the [PhoenixChannel]
    pub async fn close(&mut self) -> Result<()> {
        self.sender.send(Message::Close(None)).await?;
        self.sender.close().await?;
        Ok(())
    }
}