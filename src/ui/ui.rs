use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use crate::core::protocol::{Device, PairingRequest};

pub trait UiHandle: Send + Debug {
    fn handle_error(&mut self, t: String);
    fn handle_pairing_request(&mut self, request: &PairingRequest) -> bool;
    fn pick_dest<'a>(&mut self, devices: &'a Vec<Device>) -> Option<&'a Device>;
}
pub type SharedUiHandle = Arc<Mutex<dyn UiHandle>>;
