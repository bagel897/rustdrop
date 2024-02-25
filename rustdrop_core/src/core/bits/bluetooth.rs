use std::io::Cursor;

use bytes::Buf;
use modular_bitfield::prelude::*;

use crate::{Config, RustdropResult};

use super::{pcp_version::PcpVersion, service::Service, Bitfield, EndpointInfo};

#[bitfield]
#[derive(Debug)]
struct NameBits {
    pcp_version: PcpVersion,
    endpoint_id: u32,
    service: Service,
    webrtc_state: u8,
    reserved: B48,
}
impl NameBits {
    fn from_config(config: &Config) -> Self {
        Self::new()
            .with_pcp_version(PcpVersion::default())
            .with_endpoint_id(config.endpoint_id)
            .with_service(Service::default())
            .with_webrtc_state(0x0)
    }
}
#[derive(Debug)]
pub struct Name {
    bits: NameBits,
    endpoint_info: EndpointInfo,
    pub name: String,
}
impl Name {
    pub fn new(config: &Config, endpoint_info: EndpointInfo) -> Self {
        let bits = NameBits::from_config(config);
        let name = config.name.clone();
        Self {
            name,
            endpoint_info,
            bits,
        }
    }
}
impl Bitfield for Name {
    fn to_vec(self) -> Vec<u8> {
        let mut data = self.bits.into_bytes().to_vec();
        let mut encoded = self.endpoint_info.to_vec();
        data.push(encoded.len() as u8);
        data.append(&mut encoded);
        let name = self.name.as_bytes();
        data.push((data.len() + 1).try_into().unwrap());
        data
    }
    fn decode(raw: &mut Cursor<&[u8]>) -> RustdropResult<Self> {
        let mut raw_name: [u8; 15] = [0; 15];
        raw.copy_to_slice(&mut raw_name);
        let bits = NameBits::from_bytes(raw_name);
        let name = todo!();
        let endpoint_info = todo!();
        Ok(Self {
            bits,
            name,
            endpoint_info,
        })
    }
}
// pub(super) fn get_name(config: &Config) -> String {
//     let mut result = BytesMut::new();
//     result.put_u8(PCP);
//     result.extend_from_slice(config.endpoint_id.as_bytes());
//     result.extend_from_slice(&SERVICE_ID);
//     result.put_u8(0x0);
//     result.extend_from_slice(&BytesMut::zeroed(6));
//     let endpoint_info = get_endpoint_info(config);
//     info!("{:?}", endpoint_info);
//     result.put_u8(endpoint_info.len().try_into().unwrap());
//     result.extend_from_slice(&endpoint_info);
//     result.put_u8((result.len() + 1).try_into().unwrap());
//     BASE64_URL_SAFE.encode(result)
// }
