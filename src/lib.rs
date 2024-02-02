mod core;
mod mediums;
pub(crate) mod protobuf;
pub(crate) mod runner;
mod ui;
pub use runner::{
    application::Application,
    runner::{run_client, run_server},
    simple::run_simple,
};
pub use ui::{SharedUiHandle, SimpleUI, UiHandle};

pub use crate::core::{
    protocol::{Device, PairingRequest},
    Config,
};
