use tokio::sync::oneshot::Sender;

use crate::{core::IncomingWifi, Device, IncomingText, PairingRequest};

#[derive(Debug)]
pub enum DiscoveryEvent {
    Discovered(Device),
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
    Accepted(),
    Rejected(),
}
