mod api;
mod core;
mod mediums;
pub(crate) mod protobuf;
pub(crate) mod runner;
use core::RustdropError;

pub use crate::api::PairingRequest;
pub(crate) use crate::core::Incoming;
pub use crate::core::{
    protocol::Device, Config, IncomingFile, IncomingText, IncomingWifi, Outgoing,
};
pub use crate::protobuf::nearby::sharing::service::text_metadata::Type as TextType;
pub use api::events::{DiscoveryEvent, ReceiveEvent, SenderEvent};
pub use api::DiscoveryHandle;
use color_eyre::eyre;
pub(crate) use runner::context::Context;
pub use runner::managed::Rustdrop;
pub type RustdropResult<T> = eyre::Result<T>;
pub use core::bits::DeviceType;
