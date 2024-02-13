use crate::{core::IncomingWifi, IncomingText, PairingRequest};

pub enum DeviceEvent {
    Discovered(),
    Removed(),
    //Device
}
pub enum ReceiveEvent {
    Text(IncomingText),
    Wifi(IncomingWifi),
}
pub enum ServerEvent {
    PairingRequest(PairingRequest),
}
