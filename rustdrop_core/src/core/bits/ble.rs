use std::io::Cursor;

use bytes::{Buf, BufMut};
use modular_bitfield::prelude::*;
use tracing::info;

use crate::{core::RustdropError, Config, RustdropResult};

use super::{pcp_version::VersionPcp, service::Service, Bitfield, EndpointInfo};
#[bitfield]
#[derive(Debug)]
struct BleNameBits {
    pcp_version: VersionPcp,
    service: Service,
    endpoint_id: u32,
}
#[bitfield]
#[derive(Debug)]
struct BleFastNameBits {
    pcp_version: VersionPcp,
    endpoint_id: u32,
}
impl BleFastNameBits {
    pub fn from_config(config: &Config) -> Self {
        Self::new()
            .with_pcp_version(VersionPcp::default())
            .with_endpoint_id(config.endpoint_id)
    }
}
impl BleNameBits {
    pub fn from_config(config: &Config) -> Self {
        Self::new()
            .with_service(Service::default())
            .with_pcp_version(VersionPcp::default())
            .with_endpoint_id(config.endpoint_id)
    }
}
#[derive(Debug)]
pub struct BleFastName {
    bits: BleFastNameBits,
    pub endpoint_info: EndpointInfo,
    pub name: String,
    mac: String,
}
impl BleFastName {
    pub(crate) fn new(config: &Config, mac: String, endpoint_info: EndpointInfo) -> Self {
        let bits = BleFastNameBits::from_config(config);
        let name = config.name.clone();
        Self {
            bits,
            mac,
            name,
            endpoint_info,
        }
    }
}
impl Bitfield for BleFastName {
    fn to_vec(self) -> Vec<u8> {
        let mut data = self.bits.into_bytes().to_vec();
        // data.extend_from_slice(&self.reserved);
        let mut encoded = self.name.as_bytes().to_vec();
        data.put_u8(encoded.len() as u8);
        data.append(&mut encoded);
        data.append(&mut self.mac.as_bytes().to_vec());
        data
    }
    fn decode(name: &mut Cursor<&[u8]>) -> RustdropResult<Self> {
        // if name.remaining() < 16 {
        //     Err(RustdropError::InvalidEndpointId())?;
        // }
        let mut raw_name: [u8; 5] = [0; 5];
        name.copy_to_slice(&mut raw_name);
        let bits = BleFastNameBits::from_bytes(raw_name);
        info!("{:?}", bits);
        let size = name.get_u8();
        let endpoint_info_raw = name.copy_to_bytes(size as usize);
        let endpoint_info = EndpointInfo::decode_raw(&endpoint_info_raw)?;
        Err(RustdropError::InvalidEndpointId())?;

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
#[derive(Debug)]
pub struct BleName {
    bits: BleNameBits,
    pub endpoint_info: EndpointInfo,
    pub name: String,
    mac: String,
}
impl BleName {
    pub(crate) fn new(config: &Config, mac: String, endpoint_info: EndpointInfo) -> Self {
        let bits = BleNameBits::from_config(config);
        let name = config.name.clone();
        Self {
            bits,
            mac,
            name,
            endpoint_info,
        }
    }
}
impl Bitfield for BleName {
    fn to_vec(self) -> Vec<u8> {
        let mut data = self.bits.into_bytes().to_vec();
        // data.extend_from_slice(&self.reserved);
        let mut encoded = self.name.as_bytes().to_vec();
        data.put_u8(encoded.len() as u8);
        data.append(&mut encoded);
        data.append(&mut self.mac.as_bytes().to_vec());
        data
    }
    fn decode(name: &mut Cursor<&[u8]>) -> RustdropResult<Self> {
        // if name.remaining() < 16 {
        //     Err(RustdropError::InvalidEndpointId())?;
        // }
        let mut raw_name: [u8; 8] = [0; 8];
        name.copy_to_slice(&mut raw_name);
        let bits = BleNameBits::from_bytes(raw_name);
        info!("{:?}", bits);
        let size = name.get_u8();
        let endpoint_info_raw = name.copy_to_bytes(size as usize);
        let endpoint_info = EndpointInfo::decode_raw(&endpoint_info_raw)?;
        Err(RustdropError::InvalidEndpointId())?;

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
#[cfg(test)]
mod tests {
    use tracing::info;
    use tracing_test::traced_test;

    use crate::core::bits::{ble::BleFastName, Bitfield};

    use super::BleName;

    #[traced_test]
    #[test]
    fn test_ble() {
        let raw = [
            74, 23, 35, 48, 77, 88, 70, 17, 50, 27, 93, 179, 71, 45, 235, 48, 174, 253, 127, 79,
            203, 111, 138, 118, 60, 177, 156,
        ];
        BleFastName::decode_raw(&raw).unwrap();
        BleName::decode_raw(&raw).unwrap();
    }
}
