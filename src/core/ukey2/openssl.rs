use bytes::{Bytes, BytesMut};
use openssl::{
    bn::{BigNum, BigNumContext},
    derive::Deriver,
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    pkey::{Id, PKey, Private, Public},
    pkey_ctx::{HkdfMode, PkeyCtx},
    sign::{Signer, Verifier},
    symm::{decrypt, encrypt, Cipher},
};

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
    type Intermediate = Bytes;
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
            .affine_coordinates_gf2m(&Self::group(), &mut x_num, &mut y_num, &mut ctx)
            .unwrap();
        (Bytes::from(x_num.to_vec()), Bytes::from(y_num.to_vec()))
    }

    fn genkey() -> Self::SecretKey {
        EcKey::generate(&Self::group()).unwrap()
    }

    fn diffie_hellman(secret: Self::SecretKey, public: &Self::PublicKey) -> Self::Intermediate {
        let converted: PKey<Private> = secret.try_into().unwrap();
        let mut deriver = Deriver::new(&converted).unwrap();
        deriver.set_peer(public).unwrap();
        Bytes::from(deriver.derive_to_vec().unwrap())
    }

    fn extract_expand(
        info: &'static str,
        key: &Self::Intermediate,
        salt: &[u8],
        len: usize,
    ) -> Self::Intermediate {
        let mut ctx = PkeyCtx::new_id(Id::HKDF).unwrap();
        ctx.set_hkdf_mode(HkdfMode::EXTRACT_THEN_EXPAND).unwrap();
        ctx.set_hkdf_key(key).unwrap();
        ctx.add_hkdf_info(info.as_bytes()).unwrap();
        ctx.set_hkdf_salt(salt).unwrap();
        let mut result = BytesMut::zeroed(len);
        ctx.derive(Some(&mut result)).unwrap();
        result.into()
    }

    fn get_hmac_from_bytes(source: Self::Intermediate) -> Self::HmacKey {
        PKey::hmac(&source).unwrap()
    }

    fn decrypt(key: &Self::AesKey, iv: [u8; 16], init: Vec<u8>) -> Vec<u8> {
        decrypt(Cipher::aes_256_cbc(), key, Some(&iv), &init).unwrap()
    }

    fn encrypt(key: &Self::AesKey, iv: [u8; 16], init: Vec<u8>) -> Vec<u8> {
        encrypt(Cipher::aes_256_cbc(), key, Some(&iv), &init).unwrap()
    }

    fn verify(key: &Self::HmacKey, data: &[u8], tag: &[u8]) -> bool {
        let mut verifier = Verifier::new(MessageDigest::sha256(), key).unwrap();
        verifier.verify_oneshot(data, tag).unwrap()
    }

    fn sign(key: &Self::HmacKey, data: &[u8]) -> Vec<u8> {
        let mut signer = Signer::new(MessageDigest::sha256(), key).unwrap();
        signer.sign_oneshot_to_vec(data).unwrap()
    }
    fn get_aes_decrypt_from_bytes(source: Self::Intermediate) -> Self::AesKey {
        source
    }
    fn get_aes_encrypt_from_bytes(source: Self::Intermediate) -> Self::AesKey {
        source
    }
}

#[cfg(test)]
mod tests {
    use crate::core::util::get_random;

    use super::*;
    #[test]
    fn test_hkdf() {
        let key = Bytes::from("hi");
        let _ = OpenSSL::extract_expand("info", &key, &get_random(10), 10);
    }
}
