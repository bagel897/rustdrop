use std::{collections::HashMap, net::IpAddr};

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use mdns_sd::ServiceInfo;

use crate::{
    core::{
        bits::{Bitfield, EndpointInfo, MdnsName},
        Config,
    },
    mediums::wlan::mdns::constants::TYPE,
};
fn encode(data: &[u8]) -> String {
    BASE64_URL_SAFE_NO_PAD.encode(data)
}

pub fn get_service_info(
    config: &Config,
    endpoint_info: EndpointInfo,
    ips: Vec<IpAddr>,
    port: u16,
) -> ServiceInfo {
    let name_raw = MdnsName::from_config(config).into_bytes();
    let name = encode(&name_raw);
    let txt = endpoint_info.to_base64();
    let mut txt_record = HashMap::new();
    txt_record.insert("n".to_string(), txt);
    let service = ServiceInfo::new(TYPE, &name, &name, &*ips, port, txt_record).unwrap();
    service.enable_addr_auto()
}
