use std::{
    fmt::Debug,
    future::Future,
    sync::{Arc, Mutex},
};

use crate::core::protocol::{Device, IncomingText, PairingRequest};
use crate::protobuf::sharing::nearby::text_metadata::Type;

pub trait UiHandle: Send + Debug + 'static {
    fn handle_url(&mut self, text: IncomingText) -> impl Future + Send {
        self.handle_text(text)
    }
    fn handle_address(&mut self, text: IncomingText) -> impl Future + Send {
        self.handle_text(text)
    }
    fn handle_phone(&mut self, text: IncomingText) -> impl Future + Send {
        self.handle_text(text)
    }
    fn handle_text(&mut self, text: IncomingText) -> impl Future + Send;
    fn handle_pairing_request(
        &mut self,
        request: &PairingRequest,
    ) -> impl Future<Output = bool> + Send;
    fn pick_dest<'a>(&mut self, devices: &'a [Device]) -> Option<&'a Device>;
}
pub type SharedUiHandle = Arc<Mutex<dyn UiHandle>>;
