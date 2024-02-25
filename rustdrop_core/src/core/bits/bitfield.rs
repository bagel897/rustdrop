use std::io::Cursor;

use base64::prelude::*;

use crate::RustdropResult;

pub trait Bitfield: Sized {
    //fn from_config(config: &Config);
    fn to_vec(self) -> Vec<u8>;
    fn decode(raw: &mut Cursor<&[u8]>) -> RustdropResult<Self>;
    fn decode_raw(raw: &[u8]) -> RustdropResult<Self> {
        Self::decode(&mut Cursor::new(&raw))
    }
    fn to_base64(self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(self.to_vec())
    }
    fn decode_base64(raw: &[u8]) -> RustdropResult<Self> {
        let raw = BASE64_URL_SAFE_NO_PAD.decode(raw)?;
        Self::decode(&mut Cursor::new(&raw))
    }
}
