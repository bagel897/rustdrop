use crate::core::protocol::{Device, PairingRequest};

pub(crate) trait UiHandle: Send {
    fn handle_error(&mut self, t: String);
    fn handle_pairing_request(&mut self, request: PairingRequest) -> bool;
    fn pick_dest<'a>(&mut self, devices: &'a Vec<Device>) -> Option<&'a Device>;
}
