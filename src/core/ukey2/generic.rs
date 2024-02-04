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
    fn diffie_hellman(secret: Self::SecretKey, public: &Self::PublicKey) -> Vec<u8>;
    fn extract_expand(info: &[u8], key: &[u8], salt: &[u8], len: usize) -> Bytes;
    fn get_aes_decrypt_from_bytes(source: &[u8]) -> Self::AesKey;
    fn get_aes_encrypt_from_bytes(source: &[u8]) -> Self::AesKey;
    fn get_hmac_from_bytes(source: &[u8]) -> Self::HmacKey;
    fn derive_aes_encrypt(info: &[u8], key: &[u8], salt: &[u8], len: usize) -> Self::AesKey {
        let raw = Self::extract_expand(info, key, salt, len);
        Self::get_aes_encrypt_from_bytes(&raw)
    }
    fn derive_aes_decrypt(info: &[u8], key: &[u8], salt: &[u8], len: usize) -> Self::AesKey {
        let raw = Self::extract_expand(info, key, salt, len);
        Self::get_aes_decrypt_from_bytes(&raw)
    }
    fn derive_hmac(info: &[u8], key: &[u8], salt: &[u8], len: usize) -> Self::HmacKey {
        let raw = Self::extract_expand(info, key, salt, len);
        Self::get_hmac_from_bytes(&raw)
    }
    fn decrypt(key: &Self::AesKey, iv: [u8; 16], init: Vec<u8>) -> Vec<u8>;
    fn encrypt(key: &Self::AesKey, iv: [u8; 16], init: Vec<u8>) -> Vec<u8>;
    fn sign(key: &Self::HmacKey, data: &[u8]) -> Vec<u8>;
    fn sha256(data: &[u8]) -> [u8; 32];
    fn sha512(data: &[u8]) -> [u8; 64];
}
