use bytes::Bytes;
use core::fmt::Debug;

pub trait Crypto: Debug + Default {
    type PublicKey;
    type SecretKey;
    type HmacKey;
    type AesKey;
    fn to_pubkey(x: &[u8], y: &[u8]) -> Self::PublicKey;
    fn from_pubkey(pubkey: &Self::SecretKey) -> (Bytes, Bytes);
    fn genkey() -> Self::SecretKey;
    fn diffie_hellman(secret: &Self::SecretKey, public: &Self::PublicKey) -> Bytes;
    fn extract_expand(info: &'static str, key: Bytes, salt: &[u8], len: usize) -> Bytes;
    fn get_aes_from_bytes(source: Bytes) -> Self::AesKey;
    fn get_hmac_from_bytes(source: Bytes) -> Self::HmacKey;
    fn derive_aes(info: &'static str, key: Bytes, salt: &[u8], len: usize) -> Self::AesKey {
        let raw = Self::extract_expand(info, key, salt, len);
        Self::get_aes_from_bytes(raw);
    }
    fn derive_hmac(info: &'static str, key: Bytes, salt: &[u8], len: usize) -> Self::HmacKey {
        let raw = Self::extract_expand(info, key, salt, len);
        Self::get_hmac_from_bytes(raw);
    }
    fn decrypt(key: Self::AesKey, iv: [u8; 16], init: [u8; 32]) -> Bytes;
    fn encrypt(key: Self::AesKey, iv: [u8; 16], init: [u8; 32]) -> Bytes;
    fn verify(key: Self::HmacKey) -> bool;
}
