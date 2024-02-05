use std::{
    fmt::Debug,
    future::Future,
    sync::{Arc, Mutex},
};

use crate::core::protocol::{Device, PairingRequest};

pub trait UiHandle: Send + Debug + 'static {
    fn handle_error(&mut self, t: String);
    fn handle_pairing_request(
        &mut self,
        request: &PairingRequest,
    ) -> impl Future<Output = bool> + Send;
    fn pick_dest<'a>(&mut self, devices: &'a [Device]) -> Option<&'a Device>;
}
pub type SharedUiHandle = Arc<Mutex<dyn UiHandle>>;
