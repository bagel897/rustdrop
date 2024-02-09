use std::net::{IpAddr, SocketAddr};

use crate::core::protocol::{decode_endpoint_id, Device};
use base64::prelude::*;
use mdns_sd::ServiceInfo;
pub fn parse_device(addr: &IpAddr, info: &ServiceInfo) -> Result<Device, anyhow::Error> {
    let endpoint_info = info.get_property_val("n").unwrap().unwrap();
    let full_addr = SocketAddr::new(*addr, info.get_port());
    let decoded = BASE64_URL_SAFE_NO_PAD.decode(endpoint_info)?;
    let (device_type, name) = decode_endpoint_id(&decoded)?;
    Ok(Device {
        device_type,
        device_name: name,
        ip: full_addr,
    })
}
