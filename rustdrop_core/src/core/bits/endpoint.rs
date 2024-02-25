use std::io::Cursor;

use super::{bitfield::Bitfield, uwb_address::UwbAddress};
use crate::{core::RustdropError, Config, DeviceType, RustdropResult};
use bytes::{Buf, Bytes};
use modular_bitfield::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use tracing::{info, instrument};

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
    #[instrument]
    fn decode(endpoint_id: &mut Cursor<&[u8]>) -> RustdropResult<Self> {
        info!("{:?}", endpoint_id);
        // if endpoint_id.remaining() < 18 {
        //     Err(RustdropError::InvalidEndpointId())?;
        // }
        let raw_bits = endpoint_id.get_u8();
        let reserved = endpoint_id.copy_to_bytes(16);
        let bitfield = BitField::from_bytes([raw_bits]);
        let name = if endpoint_id.has_remaining() {
            let size = endpoint_id.get_u8() as usize;
            let raw_name = endpoint_id.copy_to_bytes(size);
            String::from_utf8(raw_name.to_vec()).map_err(|_| RustdropError::InvalidEndpointId())?
        } else {
            "".into()
        };
        Ok(Self {
            bitfield,
            reserved,
            name,
        })
    }
}
#[cfg(test)]
mod tests {
    use crate::core::bits::{Bitfield, EndpointInfo};

    #[test]
    fn test_win() {
        let raw = "Fjo2V4BhObmdc29KmxN5k8Y";
        EndpointInfo::decode_base64(raw.as_bytes()).unwrap();
    }
    #[test]
    fn test_android() {
        let raw = "MnOfGXWt4xYOEvqYHBptzjc";
        EndpointInfo::decode_base64(raw.as_bytes()).unwrap();
    }
}
