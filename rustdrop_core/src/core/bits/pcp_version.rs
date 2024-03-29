use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier, Clone, Debug, Copy)]
#[bits = 5]
enum PCP {
    Unknown = 0,
    P2PStar = 1,
    P2PCluster = 2,
    P2PPointToPoint = 3,
}
#[bitfield(bits = 8)]
#[derive(BitfieldSpecifier, Debug)]
pub struct PcpVersion {
    pcp: PCP,
    version: B3,
}
// 0x23
impl Default for PcpVersion {
    fn default() -> Self {
        Self::new().with_version(0x1).with_pcp(PCP::P2PPointToPoint)
    }
}
#[bitfield(bits = 8)]
#[derive(BitfieldSpecifier, Debug)]
pub struct VersionPcp {
    version: B3,
    pcp: PCP,
}
// 0x23
impl Default for VersionPcp {
    fn default() -> Self {
        Self::new().with_version(0x1).with_pcp(PCP::P2PPointToPoint)
    }
}
