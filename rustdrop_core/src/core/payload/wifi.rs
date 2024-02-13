use crate::protobuf::sharing::nearby::{
    wifi_credentials_metadata::SecurityType, WifiCredentialsMetadata,
};

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
