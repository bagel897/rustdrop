use std::io::Cursor;

use bytes::Buf;
use modular_bitfield::prelude::*;

use crate::{Config, RustdropResult};

use super::{pcp_version::PcpVersion, service::Service, Bitfield};

#[bitfield]
#[derive(Debug)]
pub struct Name {
    pcp: PcpVersion,
    pub endpoint_id: u32,
    service: Service,
    uwb_address: B1,
    reserved: B15,
}
impl Name {
    pub fn from_config(config: &Config) -> Self {
        Self::new()
            .with_pcp(PcpVersion::default())
            .with_endpoint_id(config.endpoint_id)
            .with_service(Service::default())
            .with_reserved(0x0)
    }
}
impl Bitfield for Name {
    fn to_vec(self) -> Vec<u8> {
        self.into_bytes().to_vec()
    }
    fn decode(raw: &mut Cursor<&[u8]>) -> RustdropResult<Self> {
        let mut arr: [u8; 10] = [0; 10];
        raw.copy_to_slice(&mut arr);
        Ok(Self::from_bytes(arr))
    }
}
#[cfg(test)]
mod tests {
    // #[test]
    // fn test_win_name() {
    //     let raw = "IzNnN2X8n14AAA._FC9F5ED42C8A._tcp.local.";
    //     let
    //     unimplemented!();
    // }
}
