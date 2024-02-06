use tracing::info;

use super::ui::UiHandle;
use crate::core::protocol::{Device, IncomingText, PairingRequest};

#[derive(Debug, Default)]
pub struct SimpleUI {}
impl UiHandle for SimpleUI {
    async fn handle_text(&mut self, text: IncomingText) {
        println!("Recieved {:?}", text);
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
