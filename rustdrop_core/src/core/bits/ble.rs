use modular_bitfield::prelude::*;

use crate::{Config, RustdropResult};

use super::{pcp_version::PcpVersion, service::Service, Bitfield};
#[bitfield]
struct BleNameBits {
    unknown: u8,
    service_1: Service,
    unknown2: u32,
    pcp_version: PcpVersion,
    service_2: Service,
    endpoint_id: u32,
}
impl BleNameBits {
    pub fn from_config(config: &Config) -> Self {
        Self::new()
            .with_service_1(Service::default())
            .with_service_2(Service::default())
            .with_pcp_version(PcpVersion::default())
            .with_endpoint_id(config.endpoint_id)
    }
}
pub struct BleName {
    bits: BleNameBits,
    pub name: String,
    mac: String,
}
impl BleName {
    pub(crate) fn new(config: &Config, mac: String) -> Self {
        let bits = BleNameBits::from_config(config);
        let name = config.name.clone();
        Self { bits, mac, name }
    }
}
impl Bitfield for BleName {
    fn to_vec(self) -> Vec<u8> {
        let mut data = self.bits.into_bytes().to_vec();
        // data.extend_from_slice(&self.reserved);
        let mut encoded = self.name.as_bytes().to_vec();
        data.push(encoded.len() as u8);
        data.append(&mut encoded);
        data.append(&mut self.mac.as_bytes().to_vec());
        data
    }
    fn decode(name: &[u8]) -> RustdropResult<Self> {
        todo!()
        // let (first, second) = endpoint_id.split_at(18);
        // let (raw_bits, reserved) = first.split_first().unwrap();
        // let bitfield = BitField::from(*raw_bits);
        // let name =
        //     String::from_utf8(second.to_vec()).map_err(|_| RustdropError::InvalidEndpointId())?;
        // Ok(Self {
        //     bitfield,
        //     reserved: Bytes::copy_from_slice(reserved),
        //     name,
        // })
    }
}
