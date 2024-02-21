use modular_bitfield::prelude::*;

use crate::Config;

use super::service::Service;

const PCP: u8 = 0x23;
#[bitfield]
pub struct Name {
    pcp: u8,
    endpoint_id: u32,
    service: Service,
    reserved: B16,
}
impl Name {
    pub fn from_config(config: &Config) -> Self {
        Self::new()
            .with_pcp(PCP)
            .with_endpoint_id(config.endpoint_id)
            .with_service(Service::default())
            .with_reserved(0x0)
    }
}
