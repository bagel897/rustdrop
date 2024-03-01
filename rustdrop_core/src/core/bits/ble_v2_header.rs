use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Debug)]
struct BleHeader {
    version: B3,
    extended: bool,
    slots: B4,
    bloom_filter: B80,
    adv_hash: B32,
    psm: B16,
}
