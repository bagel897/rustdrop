mod api;
mod core;
mod mediums;
pub(crate) mod protobuf;
pub(crate) mod runner;
pub use runner::{context::Context, managed::Rustdrop};

pub use crate::core::{
    protocol::{Device, PairingRequest},
    Config, IncomingText,
};
pub use crate::protobuf::sharing::nearby::text_metadata::Type as TextType;
