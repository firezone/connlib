//! Error module.
use base64::{DecodeError, DecodeSliceError};
use boringtun::noise::errors::WireGuardError;
use macros::SwiftEnum;
use thiserror::Error;

/// Unified Result type to use across connlib.
pub type Result<T> = std::result::Result<T, ConnlibError>;

/// Unified error type to use across connlib.
#[derive(Error, Debug, SwiftEnum)]
pub enum ConnlibError {
    /// Standard IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Error while decoding a base64 value.
    #[error("There was an error while decoding a base64 value: `{0}`")]
    Base64DecodeError(#[from] DecodeError),
    /// Error while decoding a base64 value from a slice.
    #[error("There was an error while decoding a base64 value: `{0}`")]
    Base64DecodeSliceError(#[from] DecodeSliceError),
    /// Request error for websocket connection.
    #[error("Error forming request: {0}")]
    RequestError(#[from] tokio_tungstenite::tungstenite::http::Error),
    /// Error during websocket connection.
    #[error("Portal connection error: {0}")]
    PortalConnectionError(#[from] tokio_tungstenite::tungstenite::error::Error),
    /// Provided string was not formatted as a URL.
    #[error("Baddly formatted URI")]
    UriError,
    /// Serde's serialize error.
    #[error(transparent)]
    SerializeError(#[from] serde_json::Error),
    /// Webrtc errror
    #[error("ICE-related error: {0}")]
    IceError(#[from] webrtc::Error),
    /// Webrtc error regarding data channel.
    #[error("ICE-data error: {0}")]
    IceDataError(#[from] webrtc::data::Error),
    /// Error while sending through an async channelchannel.
    #[error("Error sending message through an async channel")]
    SendChannelError,
    /// Error when trying to stablish connection between peers.
    #[error("Error while stablishing connection between peers")]
    ConnectionStablishError,
    /// Error regarding boringtun's devices
    #[error("Error while using boringtun's device")]
    BoringtunError(#[from] boringtun::device::Error),
    /// Error related to wireguard protocol.
    #[error("Wireguard error")]
    WireguardError(WireGuardError),
    /// Expected an initialized runtime but there was none.
    #[error("Expected runtime to be initialized")]
    NoRuntime,
    /// Tried to access a resource which didn't exists.
    #[error("Tried to access an undefined resource")]
    UnknownResource,
    /// Error regarding our own control protocol.
    #[error("Control plane protocol error. Unexpected messages or message order.")]
    ControlProtocolError,
    /// Glob for errors without a type.
    #[error("Other error")]
    Other(&'static str),
}

/// Type auto-generated by [SwiftEnum] intended to be used with rust-swift-bridge.
/// All the variants come from [ConnlibError], reference that for documentaiton.
pub use swift_ffi::SwiftConnlibError;

impl From<WireGuardError> for ConnlibError {
    fn from(e: WireGuardError) -> Self {
        ConnlibError::WireguardError(e)
    }
}

impl From<&'static str> for ConnlibError {
    fn from(e: &'static str) -> Self {
        ConnlibError::Other(e)
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for ConnlibError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        ConnlibError::SendChannelError
    }
}

impl From<futures::channel::mpsc::SendError> for ConnlibError {
    fn from(_: futures::channel::mpsc::SendError) -> Self {
        ConnlibError::SendChannelError
    }
}