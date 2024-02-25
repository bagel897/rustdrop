use std::io::Cursor;

use bytes::Buf;

use super::Bitfield;

#[derive(Debug, Clone)]
pub struct UwbAddress {
    size: u8,
}
impl Bitfield for UwbAddress {
    fn to_vec(self) -> Vec<u8> {
        let res = vec![self.size];
        res
    }
    fn decode(raw: &mut Cursor<&[u8]>) -> crate::RustdropResult<Self> {
        let size = raw.get_u8();
        Ok(UwbAddress { size })
    }
}
impl Default for UwbAddress {
    fn default() -> Self {
        Self { size: 0 }
    }
}
