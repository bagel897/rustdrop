use modular_bitfield::prelude::*;

use crate::Config;

use super::{pcp_version::PcpVersion, service::Service};

#[bitfield]
pub struct Name {
    pcp: PcpVersion,
    endpoint_id: u32,
    service: Service,
    reserved: B16,
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
