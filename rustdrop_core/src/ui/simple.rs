use async_trait::async_trait;
use tracing::info;

use super::ui::UiHandle;
use crate::core::protocol::{Device, IncomingText, PairingRequest};

#[derive(Debug, Default)]
pub struct SimpleUI {
    devices: Vec<Device>,
}
#[async_trait]
impl UiHandle for SimpleUI {
    async fn discovered_device(&self, device: Device) {
        todo!();
        self.devices.push(device);
    }
    async fn handle_text(&mut self, text: IncomingText) {
        println!("Recieved {:?}", text);
    }
    async fn handle_pairing_request(&mut self, request: &PairingRequest) -> bool {
        info!("{:?}", request);
        true
    }
    async fn pick_dest(&self) -> Option<Device> {
        todo!();
        self.devices.pop()
    }
}
