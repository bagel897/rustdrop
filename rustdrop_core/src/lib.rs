mod core;
mod mediums;
pub(crate) mod protobuf;
pub(crate) mod runner;
mod ui;
pub use runner::{application::Application, managed::Rustdrop};
pub use ui::{SharedUiHandle, SimpleUI, UiHandle};

pub use crate::core::{
    protocol::{Device, IncomingText, PairingRequest},
    Config,
};
pub use crate::protobuf::sharing::nearby::text_metadata::Type as TextType;
#[cfg(feature = "simple")]
pub use runner::simple::run_simple;
