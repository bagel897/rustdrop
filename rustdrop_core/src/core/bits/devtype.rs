use modular_bitfield::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, BitfieldSpecifier)]
#[bits = 3]
pub enum DeviceType {
    Unknown = 0,
    Phone = 1,
    Tablet = 2,
    Laptop = 3,
}
