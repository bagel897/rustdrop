use std::net::{IpAddr, SocketAddr};

use mdns_sd::ServiceInfo;

use crate::{
    core::{
        bits::{Bitfield, EndpointInfo},
        protocol::Device,
    },
    mediums::{wlan::WlanDiscovery, Discover},
    RustdropResult,
};
pub fn parse_device(addr: &IpAddr, info: &ServiceInfo) -> RustdropResult<Device> {
    let raw_info = info.get_property_val("n").unwrap().unwrap();
    let full_addr = SocketAddr::new(*addr, info.get_port());
    let endpoint_info = EndpointInfo::decode_base64(raw_info)?;
    Ok(Device {
        device_type: endpoint_info.devtype(),
        device_name: endpoint_info.name,
        discovery: Discover::Wlan(WlanDiscovery::from(full_addr)),
    })
}
