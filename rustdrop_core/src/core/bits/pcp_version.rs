use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier, Clone, Debug, Copy)]
#[bits = 5]
enum PCP {
    Unknown = 0,
    P2PStar = 1,
    P2PCluster = 2,
    P2PPointToPoint = 3,
}
#[bitfield]
#[derive(BitfieldSpecifier)]
pub struct PcpVersion {
    version: B3,
    pcp: PCP,
}
// 0x23
impl Default for PcpVersion {
    fn default() -> Self {
        Self::new().with_version(0x0).with_pcp(PCP::P2PPointToPoint)
    }
}
