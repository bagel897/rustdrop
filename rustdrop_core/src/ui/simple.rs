use tracing::info;

use super::ui::UiHandle;
use crate::core::protocol::{Device, PairingRequest};

#[derive(Debug, Default)]
pub struct SimpleUI {}
impl UiHandle for SimpleUI {
    fn handle_error(&mut self, t: String) {
        panic!("{}", t);
    }
    async fn handle_pairing_request(&mut self, request: &PairingRequest) -> bool {
        info!("{:?}", request);
        true
    }
    fn pick_dest<'a>(&mut self, devices: &'a [Device]) -> Option<&'a Device> {
        info!("{:#?}", devices);
        return devices.first();
    }
}
