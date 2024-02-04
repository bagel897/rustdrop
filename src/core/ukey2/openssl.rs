use bytes::{Bytes, BytesMut};
use openssl::{
    bn::{BigNum, BigNumContext},
    derive::Deriver,
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    md::Md,
    nid::Nid,
    pkey::{Id, PKey, Private, Public},
    pkey_ctx::{HkdfMode, PkeyCtx},
    sha::{sha256, sha512},
    sign::Signer,
    symm::{decrypt, encrypt, Cipher},
};
use tracing::info;

use super::generic::Crypto;
#[derive(Debug, Default)]
pub struct OpenSSL {}
impl OpenSSL {
    fn group() -> EcGroup {
        EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()
    }
}
impl Crypto for OpenSSL {
    type PublicKey = PKey<Public>;
    type SecretKey = EcKey<Private>;
    type HmacKey = PKey<Private>;
    type AesKey = Bytes;
    fn to_pubkey(x: &[u8], y: &[u8]) -> Self::PublicKey {
        let x_num = BigNum::from_slice(x).unwrap();
        let y_num = BigNum::from_slice(y).unwrap();
        let ec = EcKey::from_public_key_affine_coordinates(&Self::group(), &x_num, &y_num).unwrap();
        ec.try_into().unwrap()
    }

    fn from_pubkey(secret: &Self::SecretKey) -> (Bytes, Bytes) {
        let mut ctx = BigNumContext::new().unwrap();
        let mut x_num = BigNum::new().unwrap();
        let mut y_num = BigNum::new().unwrap();
        let public = secret.public_key();
        public
            .affine_coordinates(&Self::group(), &mut x_num, &mut y_num, &mut ctx)
            .unwrap();
        (Bytes::from(x_num.to_vec()), Bytes::from(y_num.to_vec()))
    }

    fn genkey() -> Self::SecretKey {
        EcKey::generate(&Self::group()).unwrap()
    }

    fn diffie_hellman(secret: Self::SecretKey, public: &Self::PublicKey) -> Vec<u8> {
        let converted: PKey<Private> = secret.try_into().unwrap();
        let mut deriver = Deriver::new(&converted).unwrap();
        deriver.set_peer(public).unwrap();
        deriver.derive_to_vec().unwrap()
    }

    fn extract_expand(info: &[u8], key: &[u8], salt: &[u8], len: usize) -> Bytes {
        let mut ctx = PkeyCtx::new_id(Id::HKDF).unwrap();
        ctx.derive_init().unwrap();
        ctx.set_hkdf_mode(HkdfMode::EXTRACT_THEN_EXPAND).unwrap();
        ctx.set_hkdf_md(Md::sha256()).unwrap();
        ctx.set_hkdf_salt(salt).unwrap();
        ctx.set_hkdf_key(key).unwrap();
        ctx.add_hkdf_info(info).unwrap();
        let mut result = BytesMut::zeroed(len);
        let _ = ctx.derive(Some(&mut result)).unwrap();
        // result.truncate(cnt);
        info!("{:?}", result);
        result.into()
    }

    fn get_hmac_from_bytes(source: &[u8]) -> Self::HmacKey {
        PKey::hmac(&source).unwrap()
    }

    fn decrypt(key: &Self::AesKey, iv: [u8; 16], init: Vec<u8>) -> Vec<u8> {
        decrypt(Cipher::aes_256_cbc(), key, Some(&iv), &init).unwrap()
    }

    fn encrypt(key: &Self::AesKey, iv: [u8; 16], init: Vec<u8>) -> Vec<u8> {
        encrypt(Cipher::aes_256_cbc(), key, Some(&iv), &init).unwrap()
    }

    fn sign(key: &Self::HmacKey, data: &[u8]) -> Vec<u8> {
        let mut signer = Signer::new(MessageDigest::sha256(), key).unwrap();
        signer.sign_oneshot_to_vec(data).unwrap()
    }
    fn get_aes_decrypt_from_bytes(source: &[u8]) -> Self::AesKey {
        Bytes::copy_from_slice(source)
    }
    fn get_aes_encrypt_from_bytes(source: &[u8]) -> Self::AesKey {
        Bytes::copy_from_slice(source)
    }
    fn sha256(data: &[u8]) -> [u8; 32] {
        sha256(data)
    }
    fn sha512(data: &[u8]) -> [u8; 64] {
        sha512(data)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::util::get_random;

    use super::*;
    #[test]
    fn test_hkdf() {
        let key = Bytes::from("hi");
        let _ = OpenSSL::extract_expand("info".as_bytes(), &key, &get_random(10), 10);
    }
}
