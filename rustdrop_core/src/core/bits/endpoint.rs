use super::bitfield::Bitfield;
use crate::{core::RustdropError, Config, DeviceType, RustdropResult};
use bytes::Bytes;
use modular_bitfield::prelude::*;
use rand::{distributions::Alphanumeric, Rng};

#[bitfield(bits = 8)]
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[repr(u8)]
struct BitField {
    version: B3,
    visibility: B1,
    devtype: DeviceType,
    reserved: B1,
}
impl BitField {
    pub fn from_config(config: &Config) -> Self {
        Self::new()
            .with_version(0)
            .with_visibility(0)
            .with_devtype(config.devtype)
            .with_reserved(0)
    }
}
#[derive(Debug, Clone)]
pub struct EndpointInfo {
    bitfield: BitField,
    reserved: Bytes,
    pub name: String,
}
impl EndpointInfo {
    pub(crate) fn new<R: Rng>(config: &Config, rng: &mut R) -> Self {
        let reserved = rng.sample_iter(&Alphanumeric).take(16).collect();
        let bitfield = BitField::from_config(config);
        let name = config.name.clone();
        Self {
            bitfield,
            reserved,
            name,
        }
    }
    pub fn devtype(&self) -> DeviceType {
        self.bitfield.devtype()
    }
}
impl Bitfield for EndpointInfo {
    fn to_vec(self) -> Vec<u8> {
        let mut data = self.bitfield.into_bytes().to_vec();
        data.extend_from_slice(&self.reserved);
        let mut encoded = self.name.as_bytes().to_vec();
        data.push(encoded.len() as u8);
        data.append(&mut encoded);
        data
    }
    fn decode(endpoint_id: &[u8]) -> RustdropResult<Self> {
        if endpoint_id.len() < 18 {
            Err(RustdropError::InvalidEndpointId())?;
        }
        let (first, second) = endpoint_id.split_at(18);
        let (raw_bits, reserved) = first.split_first().unwrap();
        let bitfield = BitField::from(*raw_bits);
        let name =
            String::from_utf8(second.to_vec()).map_err(|_| RustdropError::InvalidEndpointId())?;
        Ok(Self {
            bitfield,
            reserved: Bytes::copy_from_slice(reserved),
            name,
        })
    }
}
