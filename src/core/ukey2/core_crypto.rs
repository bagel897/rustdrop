use aes::{
    cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit},
    Aes256,
};
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
fn get_aes_key_decrypt(init: [u8; 32], iv: [u8; 16]) -> Aes256CbcDec {
    return Aes256CbcDec::new(&init.into(), &iv.into());
}
fn get_aes_key_encrypt(init: [u8; 32], iv: [u8; 16]) -> Aes256CbcEnc {
    return Aes256CbcEnc::new(&init.into(), &iv.into());
}
pub fn aes_decrypt(init: [u8; 32], iv: [u8; 16], message: Vec<u8>) -> Vec<u8> {
    let key = get_aes_key_decrypt(init, iv);
    return key.decrypt_padded_vec_mut::<Pkcs7>(&message).unwrap();
}
pub fn aes_encrypt(init: [u8; 32], iv: [u8; 16], message: Vec<u8>) -> Vec<u8> {
    let key = get_aes_key_encrypt(init, iv);
    return key.encrypt_padded_vec_mut::<Pkcs7>(&message);
}
#[cfg(test)]
mod tests {

    use crate::core::util::get_random;

    use super::*;
    #[test]
    fn test_keyis_same_aes() {
        let key = [0x42; 32];
        let iv = [0x24; 16];
        let message = get_random(100);
        let encryped = aes_encrypt(key, iv, message);
        let decrypted = aes_decrypt(key, iv, message);
        assert_eq!(message, decrypted);
    }
}
