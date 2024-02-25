use std::net::SocketAddr;

use mdns_sd::ServiceInfo;
use tracing::info;

use crate::{
    core::{
        bits::{Bitfield, EndpointInfo, MdnsName},
        protocol::Device,
    },
    mediums::{wlan::WlanDiscovery, Discover},
    runner::DiscoveringHandle,
    RustdropResult,
};
pub async fn parse_device(
    info: &ServiceInfo,
    handle: &mut DiscoveringHandle,
) -> RustdropResult<()> {
    let raw_info = info.get_property_val("n").unwrap().unwrap();
    let split_name = info.get_fullname().split_once('.').unwrap().0;
    let name = MdnsName::decode_base64(split_name.as_bytes())?;
    let endpoint_info = EndpointInfo::decode_base64(raw_info)?;
    info!(
        "Found Wlan Device with name {:?} and info {:?}",
        name, endpoint_info
    );
    let device = Device {
        endpoint_id: name.endpoint_id(),
        device_type: endpoint_info.devtype(),
        device_name: endpoint_info.name,
    };
    for addr in info.get_addresses() {
        let full_addr = SocketAddr::new(*addr, info.get_port());
        let discovery = Discover::Wlan(WlanDiscovery::from(full_addr));
        handle.discovered(device.clone(), discovery).await;
    }
    Ok(())
}
