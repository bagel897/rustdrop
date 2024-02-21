use tokio::sync::oneshot::{self, Receiver, Sender};

use crate::{
    core::{protocol::decode_endpoint_id, DeviceType, RustdropError},
    Incoming, RustdropResult,
};

#[derive(Debug)]
pub struct PairingRequest {
    device_name: String,
    device_type: DeviceType,
    incoming: Incoming,
    tx: Sender<bool>,
}

impl PairingRequest {
    pub fn new(
        endpoint_info: &[u8],
        incoming: Incoming,
    ) -> RustdropResult<(Self, PairingResponse)> {
        let (tx, rx) = oneshot::channel();
        let (devtype, name) = decode_endpoint_id(endpoint_info)?;
        Ok((
            PairingRequest {
                device_name: name,
                device_type: devtype,
                incoming,
                tx,
            },
            PairingResponse { rx },
        ))
    }
    pub fn name(&self) -> String {
        "Nearby Sharing".into()
    }
    pub fn body(&self) -> String {
        format!(
            "{} wants to share {} with you",
            self.device_name,
            self.incoming.meta_type()
        )
    }
    pub fn respond(self, response: bool) {
        self.tx.send(response).unwrap()
    }
}
pub struct PairingResponse {
    rx: Receiver<bool>,
}
impl PairingResponse {
    pub async fn get_response(self) -> RustdropResult<bool> {
        self.rx.await.map_err(|_| RustdropError::NoResponse())
    }
}
