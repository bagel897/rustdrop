use crate::protobuf::nearby::sharing::service::{
    wifi_credentials_metadata::SecurityType, WifiCredentialsMetadata,
};

use super::traits::IncomingMeta;

#[derive(Debug, Clone)]
pub struct IncomingWifi {
    pub ssid: String,
    pub security_type: SecurityType,
}
impl From<WifiCredentialsMetadata> for IncomingWifi {
    fn from(wifi: WifiCredentialsMetadata) -> Self {
        IncomingWifi {
            ssid: wifi.ssid().into(),
            security_type: wifi.security_type(),
        }
    }
}
impl IncomingMeta for IncomingWifi {
    type ProtoType = WifiCredentialsMetadata;
    fn into_proto_type_with_id(self, payload_id: i64, id: i64) -> Self::ProtoType {
        todo!()
    }
    fn describe(&self, quantity: usize) -> String {
        todo!()
    }
}
