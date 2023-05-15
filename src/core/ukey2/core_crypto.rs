use aes::{cipher::KeyIvInit, Aes256};
use cbc::{Decryptor, Encryptor};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use prost::bytes::{Bytes, BytesMut};
use sha2::Sha256;

pub type Aes256CbcEnc = Encryptor<Aes256>;
pub type Aes256CbcDec = Decryptor<Aes256>;
pub type HmacSha256 = Hmac<Sha256>;
pub fn get_hdkf_key_raw(info: &'static str, key: &[u8], salt: &Bytes) -> Bytes {
    let hk = Hkdf::<Sha256>::new(Some(salt), key);
    let mut buf = BytesMut::zeroed(32);
    hk.expand(info.as_bytes(), &mut buf).unwrap();
    return buf.into();
}

pub fn get_hmac_key(info: &'static str, key: &[u8], salt: &Bytes) -> HmacSha256 {
    let hk = Hkdf::<Sha256>::new(Some(salt), key);
    let mut buf = BytesMut::zeroed(32);
    hk.expand(info.as_bytes(), &mut buf).unwrap();
    let key = HmacSha256::new_from_slice(&buf).unwrap();
    return key;
}
pub fn get_aes_init(info: &'static str, key: &[u8], salt: &Bytes) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(salt), key);
    let mut buf = [0u8; 32];
    hk.expand(info.as_bytes(), &mut buf).unwrap();
    return buf;
}
pub fn get_aes_key_decrypt(init: [u8; 32], iv: [u8; 16]) -> Aes256CbcDec {
    return Aes256CbcDec::new(&init.into(), &iv.into());
}
pub fn get_aes_key_encrypt(init: [u8; 32], iv: [u8; 16]) -> Aes256CbcEnc {
    return Aes256CbcEnc::new(&init.into(), &iv.into());
}
