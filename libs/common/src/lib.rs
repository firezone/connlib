//! This crates contains shared types and behavior between all the other libraries.
//!
//! This includes types provided by external crates, i.e. [boringtun] to make sure that
//! we are using the same version across our own crates.

pub mod error;
pub mod error_type;

mod session;

pub mod control;
pub mod messages;

pub use boringtun;
pub use error::ConnlibError as Error;
pub use error::Result;

pub use session::{Callbacks, ControlSession, ResourceList, Session, TunnelAddresses};
