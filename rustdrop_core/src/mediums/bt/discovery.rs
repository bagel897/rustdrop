use crate::{
    core::{protocol::Discover::Bluetooth, DeviceType, RustdropError},
    Device,
};
use bytes::Buf;

use super::consts::SERVICE_UUID_SHARING;
struct Advertisment {
    // endpoint_id: i32,
    pub name: String,
    mac: String,
}
impl Advertisment {
    pub fn parse_bytes(raw: &[u8]) -> Self {
        let name_size = raw[35] as usize;
        Self {
            // endpoint_id: raw[13..16].get_i32(),
            name: String::from_utf8(raw[36..(36 + name_size)].into()).unwrap(),
            mac: String::from_utf8(raw[41..46].into()).unwrap(),
        }
    }
}
pub async fn into_device(dev: bluer::Device) -> Result<Device, RustdropError> {
    let mut name = dev.name().await?.unwrap_or(dev.alias().await?);
    let device_type = DeviceType::Unknown;
    if let Some(services) = dev.service_data().await? {
        if let Some(service) = services.get(&SERVICE_UUID_SHARING) {
            // let adv = Advertisment::parse_bytes(service);
            // name = adv.name;
        }
    }

    Ok(Device {
        device_name: name,
        device_type,
        discovery: Bluetooth(dev.address()),
    })
}
