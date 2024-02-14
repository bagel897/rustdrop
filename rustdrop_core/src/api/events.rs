use tokio::sync::oneshot::Sender;

use crate::{core::IncomingWifi, Device, IncomingText, PairingRequest};

pub enum DiscoveryEvent {
    Discovered(Device),
    Removed(),
    //Device
}
pub enum ReceiveEvent {
    Text(IncomingText),
    Wifi(IncomingWifi),
    PairingRequest {
        request: PairingRequest,
        resp: Sender<bool>,
    },
}
pub enum SenderEvent {
    Accepted(),
    Rejected(),
}
