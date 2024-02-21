use tokio::sync::oneshot::Sender;

use super::discovery_handle::DiscoveryHandle;
use crate::{core::IncomingWifi, IncomingText, PairingRequest};

#[derive(Debug)]
pub enum DiscoveryEvent {
    Discovered(DiscoveryHandle),
    Removed(),
    //Device
}
#[derive(Debug)]
pub enum ReceiveEvent {
    Text(IncomingText),
    Wifi(IncomingWifi),
    PairingRequest {
        request: PairingRequest,
        resp: Sender<bool>,
    },
}
#[derive(Debug)]
pub enum SenderEvent {
    AwaitingResponse(),
    Accepted(),
    Rejected(),
    Finished(),
}
