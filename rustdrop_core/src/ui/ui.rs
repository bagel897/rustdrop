use async_trait::async_trait;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use crate::core::protocol::{Device, IncomingText, PairingRequest};

#[async_trait]
pub trait UiHandle: Send + Sync + Debug + 'static {
    async fn discovered_device(&self, device: Device);
    async fn handle_url(&mut self, text: IncomingText) {
        self.handle_text(text).await
    }
    async fn handle_address(&mut self, text: IncomingText) {
        self.handle_text(text).await
    }
    async fn handle_phone(&mut self, text: IncomingText) {
        self.handle_text(text).await
    }
    async fn handle_text(&mut self, text: IncomingText);
    async fn handle_pairing_request(&mut self, request: &PairingRequest) -> bool;
    async fn pick_dest(&self) -> Option<Device>;
}
pub type SharedUiHandle = Arc<Mutex<dyn UiHandle>>;
