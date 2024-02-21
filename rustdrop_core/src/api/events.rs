use super::discovery_handle::DiscoveryHandle;
use crate::{IncomingText, IncomingWifi, PairingRequest};

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
    PairingRequest(PairingRequest),
}
#[derive(Debug)]
pub enum SenderEvent {
    AwaitingResponse(),
    Accepted(),
    Rejected(),
    Finished(),
}
