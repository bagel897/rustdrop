use modular_bitfield::prelude::*;
const SERVICE_1: u8 = 0xFC;
const SERVICE_2: u8 = 0x9F;
const SERVICE_3: u8 = 0x5E;
#[bitfield]
#[derive(BitfieldSpecifier, Debug)]
pub struct Service {
    service_1: u8,
    service_2: u8,
    service_3: u8,
}
impl Default for Service {
    fn default() -> Self {
        Self::new()
            .with_service_1(SERVICE_1)
            .with_service_2(SERVICE_2)
            .with_service_3(SERVICE_3)
    }
}
