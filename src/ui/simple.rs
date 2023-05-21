use tracing::info;

use crate::core::protocol::{Device, PairingRequest};

use super::ui::UiHandle;

pub(crate) struct SimpleUI {}
impl UiHandle for SimpleUI {
    fn handle_error(&mut self, t: String) {
        panic!("{}", t);
    }
    fn handle_pairing_request(&mut self, request: &PairingRequest) -> bool {
        info!("{:?}", request);
        return true;
    }
    fn pick_dest<'a>(&mut self, devices: &'a Vec<Device>) -> Option<&'a Device> {
        info!("{:#?}", devices);
        return devices.first();
    }
}
impl SimpleUI {
    pub fn new() -> Self {
        return SimpleUI {};
    }
}
