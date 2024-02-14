mod api;
mod core;
mod mediums;
pub(crate) mod protobuf;
pub(crate) mod runner;
pub use crate::core::{
    protocol::{Device, PairingRequest},
    Config, IncomingText,
};
pub use crate::protobuf::sharing::nearby::text_metadata::Type as TextType;
pub use api::events::{DiscoveryEvent, ReceiveEvent, SenderEvent};
pub(crate) use runner::context::Context;
pub use runner::managed::Rustdrop;
