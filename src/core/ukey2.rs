use std::error::Error;

use prost::bytes::BytesMut;
use ring::hkdf::{KeyType, Salt, HKDF_SHA256};
use ring::hmac::{self, Key};
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};
const D2D_SALT_RAW: &'static str =
    "82AA55A0D397F88346CA1CEE8D3909B95F13FA7DEB1D4AB38376B8256DA85510";
const PT2_SALT_RAW: &'static str =
    "BF9D2A53C63616D75DB0A7165B91C1EF73E537F2427405FA23610A4BE657642E";
use crate::protobuf::securegcm::Ukey2ClientFinished;

pub fn get_public_private() -> Result<Ed25519KeyPair, Box<dyn Error>> {
    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;
    let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;
    Ok(key_pair)
}
pub fn get_public(raw: &[u8]) -> <Ed25519KeyPair as KeyPair>::PublicKey {
    todo!();
    // let key = PublicKey::public_key_from_der(raw).unwrap();
    // return key;
}

fn get_hdkf_key_raw(info: &'static str, key: &[u8], salt: &Salt) -> BytesMut {
    let extracted = salt.extract(key);
    let info_buf = [info.as_bytes()];
    let key = extracted
        .expand(&info_buf, HKDF_SHA256)
        .expect("Error extracting");
    let mut buffer = BytesMut::with_capacity(key.len().len());
    key.fill(&mut buffer).unwrap();
    return buffer;
}

fn get_hdkf_key(info: &'static str, key: &[u8], salt: &Salt) -> Key {
    // let key = get_okm(info, key, salt);
    // return hmac::Key::from(key);
    todo!();
}
pub(crate) struct Ukey2 {
    decrypt_key: Key,
    recv_hmac: Key,
    encrypt_key: Key,
    send_hmac: Key,
}
fn key_echange(
    client_pub: <Ed25519KeyPair as KeyPair>::PublicKey,
    server_key: Ed25519KeyPair,
) -> (Vec<u8>, Vec<u8>) {
    todo!();
}
impl Ukey2 {
    pub fn new(
        init: BytesMut,
        server_key_pair: Ed25519KeyPair,
        resp: &[u8],
        client_resp: Ukey2ClientFinished,
    ) -> Ukey2 {
        let D2D_SALT: Salt = Salt::new(HKDF_SHA256, D2D_SALT_RAW.as_bytes());
        let PT2_SALT: Salt = Salt::new(HKDF_SHA256, PT2_SALT_RAW.as_bytes());
        let client_pub_key = get_public(client_resp.public_key());
        let (auth_string, next_protocol_secret) = key_echange(client_pub_key, server_key_pair);
        let d2d_client = get_hdkf_key_raw("client", next_protocol_secret.as_slice(), &D2D_SALT);
        let d2d_server = get_hdkf_key_raw("server", next_protocol_secret.as_slice(), &D2D_SALT);
        let decrypt_key = get_hdkf_key("ENC:2", &d2d_client, &PT2_SALT);
        let recieve_key = get_hdkf_key("SIG_1", &d2d_client, &PT2_SALT);
        let encrypt_key = get_hdkf_key("ENC:2", &d2d_server, &PT2_SALT);
        let send_key = get_hdkf_key("SIG:1", &d2d_server, &PT2_SALT);
        Ukey2 {
            decrypt_key,
            recv_hmac: recieve_key,
            encrypt_key,
            send_hmac: send_key,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add() {
        assert_eq!(1 + 2, 3);
    }
    #[test]
    fn test_key_gen() {
        let keypair = get_public_private();
    }
    #[test]
    fn test_key_exchange() {
        let server_keypair = get_public_private().unwrap();
        let client_keypair = get_public_private().unwrap();
        key_echange(*client_keypair.public_key(), server_keypair);
    }
}
