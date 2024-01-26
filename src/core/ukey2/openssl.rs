use std::thread::yield_now;

use bytes::Bytes;
use openssl::{
    aes::AesKey,
    bn::{BigNum, BigNumContext},
    derive::Deriver,
    ec::{EcGroup, EcKey},
    nid::Nid,
    pkey::{PKey, Private, Public},
};

use super::{generic::Crypto, get_public};
#[derive(Debug, Default)]
pub struct OpenSSL {}
impl OpenSSL {
    fn group() -> EcGroup {
        EcGroup::from_curve_name(Nid::ECDSA_WITH_SHA256).unwrap()
    }
}
impl Crypto for OpenSSL {
    type PublicKey = PKey<Public>;
    type SecretKey = EcKey<Private>;
    type HmacKey = PKey<Private>;
    type AesKey = AesKey;

    fn to_pubkey(x: &[u8], y: &[u8]) -> Self::PublicKey {
        let x_num = BigNum::from_slice(&x).unwrap();
        let y_num = BigNum::from_slice(&y).unwrap();
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
        EcKey::generate(&Self::group()).unwrap().try_into().unwrap()
    }

    fn diffie_hellman(secret: Self::SecretKey, public: &Self::PublicKey) -> Bytes {
        let converted: PKey<Private> = secret.try_into().unwrap();
        let mut deriver = Deriver::new(&converted).unwrap();
        deriver.set_peer(public).unwrap();
        Bytes::from(deriver.derive_to_vec().unwrap())
    }

    fn extract_expand(info: &'static str, key: &Bytes, salt: &[u8], len: usize) -> Bytes {
        todo!()
    }

    fn get_aes_from_bytes(source: Bytes) -> Self::AesKey {
        todo!()
    }

    fn get_hmac_from_bytes(source: Bytes) -> Self::HmacKey {
        todo!()
    }

    fn decrypt(key: &Self::AesKey, iv: [u8; 16], init: Vec<u8>) -> Vec<u8> {
        todo!()
    }

    fn encrypt(key: &Self::AesKey, iv: [u8; 16], init: Vec<u8>) -> Vec<u8> {
        todo!()
    }

    fn verify(key: &Self::HmacKey, data: &[u8], tag: &[u8]) -> bool {
        todo!()
    }

    fn sign(key: &Self::HmacKey, data: &[u8]) -> Vec<u8> {
        todo!()
    }
}
