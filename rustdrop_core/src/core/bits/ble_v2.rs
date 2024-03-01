use std::io::Cursor;

use bytes::{Buf, BufMut, Bytes};
use modular_bitfield::prelude::*;
use tracing::info;

use crate::{core::RustdropError, Config, RustdropResult};

use super::{
    pcp_version::{PcpVersion, VersionPcp},
    service::Service,
    Bitfield, EndpointInfo,
};

#[bitfield]
#[derive(Debug, BitfieldSpecifier)]
struct Header {
    version: B3,
    socket_version: B3,
    fast_adv: bool,
    reserved: B1,
}
impl Header {
    fn default(fast_adv: bool) -> Self {
        Self::new()
            .with_version(2)
            .with_fast_adv(fast_adv)
            .with_socket_version(1)
    }
}
#[bitfield]
#[derive(Debug)]
struct BleNameBits {
    header: Header,
    service: Service,
}
#[bitfield]
#[derive(Debug)]
struct BleFastNameBits {
    header: Header,
}
impl BleFastNameBits {
    pub fn from_config(config: &Config) -> Self {
        Self::new().with_header(Header::default(true))
    }
}
impl BleNameBits {
    pub fn from_config(config: &Config) -> Self {
        Self::new()
            .with_header(Header::default(false))
            .with_service(Service::default())
    }
}

#[derive(Debug)]
pub struct BleFastName {
    bits: BleFastNameBits,
    data: Bytes,
    token: Option<u16>,
}
impl BleFastName {
    pub(crate) fn new(config: &Config, data: Bytes) -> Self {
        let bits = BleFastNameBits::from_config(config);
        let name = config.name.clone();
        Self {
            bits,
            data,
            token: None,
        }
    }
}

impl Bitfield for BleFastName {
    fn to_vec(self) -> Vec<u8> {
        let mut data = self.bits.into_bytes().to_vec();
        let mut encoded = self.data.to_vec();
        data.put_u8(encoded.len() as u8);
        data.append(&mut encoded);
        data
    }
    fn decode(name: &mut Cursor<&[u8]>) -> RustdropResult<Self> {
        let mut raw_name: [u8; 1] = [0; 1];
        name.copy_to_slice(&mut raw_name);
        let bits = BleFastNameBits::from_bytes(raw_name);
        info!("{:?}", bits);
        let size = name.get_i8() as usize;
        if size > name.remaining() {
            Err(RustdropError::InvalidEndpointId())?;
        }
        let data = name.copy_to_bytes(size);
        let token = if 2 <= name.remaining() {
            Some(name.get_u16())
        } else {
            None
        };
        info!("{:?}, {:?}", data, token);
        Ok(Self { bits, data, token })
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
        let mut raw_name: [u8; 4] = [0; 4];
        name.copy_to_slice(&mut raw_name);
        let bits = BleNameBits::from_bytes(raw_name);
        info!("{:?}", bits);
        let size = name.get_u8() as usize;
        if size <= name.remaining() {
            let endpoint_info_raw = name.copy_to_bytes(size);

            let endpoint_info = EndpointInfo::decode_raw(&endpoint_info_raw)?;
        }
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
    use tracing_test::traced_test;

    use crate::core::bits::{ble_v2::BleFastName, Bitfield, EndpointInfo};

    #[traced_test]
    #[test]
    fn test_ble() {
        let raw = [
            74, 23, 35, 48, 77, 88, 70, 17, 50, 27, 93, 179, 71, 45, 235, 48, 174, 253, 127, 79,
            203, 111, 138, 118, 60, 177, 156,
        ];
        let name = BleFastName::decode_raw(&raw).unwrap();
        EndpointInfo::decode_raw(&name.data).unwrap();
        // SocketControlFrame::decode(name.data).unwrap();
    }
}
