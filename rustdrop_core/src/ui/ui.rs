use std::{
    fmt::Debug,
    future::Future,
    sync::{Arc, Mutex},
};

use crate::core::protocol::{Device, IncomingText, PairingRequest};

pub trait UiHandle: Send + Sync + Debug + 'static {
    fn discovered_device(&self, device: Device) -> impl Future<Output = ()> + Send;
    fn handle_url(&mut self, text: IncomingText) -> impl Future + Send {
        self.handle_text(text)
    }
    fn handle_address(&mut self, text: IncomingText) -> impl Future + Send {
        self.handle_text(text)
    }
    fn handle_phone(&mut self, text: IncomingText) -> impl Future + Send {
        self.handle_text(text)
    }
    fn handle_text(&mut self, text: IncomingText) -> impl Future<Output = ()> + Send;
    fn handle_pairing_request(
        &mut self,
        request: &PairingRequest,
    ) -> impl Future<Output = bool> + Send;
    fn pick_dest(&self) -> impl Future<Output = Device> + Send;
}
pub type SharedUiHandle = Arc<Mutex<dyn UiHandle>>;
