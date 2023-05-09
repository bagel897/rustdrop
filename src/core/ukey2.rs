use bytes::Buf;
use prost::bytes::{BufMut, Bytes, BytesMut};
use rand_old::rngs::OsRng;
use ring::aead::{LessSafeKey, SealingKey, UnboundKey, AES_256_GCM};
use ring::error::Unspecified;
use ring::hkdf::{KeyType, Salt, HKDF_SHA256};
use ring::hmac::{Key, HMAC_SHA256};
use tracing::info;
use x25519_dalek::{PublicKey, StaticSecret};
const D2D_SALT_RAW: &str = "82AA55A0D397F88346CA1CEE8D3909B95F13FA7DEB1D4AB38376B8256DA85510";
const PT2_SALT_RAW: &str = "BF9D2A53C63616D75DB0A7165B91C1EF73E537F2427405FA23610A4BE657642E";
use crate::protobuf::securegcm::Ukey2ClientFinished;

pub fn get_public_private() -> StaticSecret {
    StaticSecret::new(OsRng)
}
pub fn get_public(raw: &[u8]) -> PublicKey {
    let mut buf = [0u8; 32];
    assert!(raw.len() <= 32);
    raw.clone().copy_to_slice(&mut buf);
    PublicKey::from(buf)
}

fn get_hdkf_key_raw(info: &'static str, key: &[u8], salt: &Salt) -> Result<BytesMut, Unspecified> {
    let extracted = salt.extract(key);
    let info_buf = [info.as_bytes()];
    let key = extracted.expand(&info_buf, HKDF_SHA256)?;
    let mut buffer = BytesMut::zeroed(key.len().len());
    key.fill(&mut buffer)?;
    Ok(buffer)
}

fn get_hmac_key(info: &'static str, key: &[u8], salt: &Salt) -> Key {
    let raw = get_hdkf_key_raw(info, key, salt).unwrap();
    return Key::new(HMAC_SHA256, &raw);
}
fn get_aes_key(info: &'static str, key: &[u8], salt: &Salt) -> LessSafeKey {
    let raw = get_hdkf_key_raw(info, key, salt).unwrap();
    let unbound = UnboundKey::new(&AES_256_GCM, &raw).unwrap();
    return LessSafeKey::new(unbound);
}
pub(crate) struct Ukey2 {
    decrypt_key: LessSafeKey,
    recv_hmac: Key,
    encrypt_key: LessSafeKey,
    send_hmac: Key,
}
fn diffie_hellmen(client_pub: PublicKey, server_key: StaticSecret) -> Bytes {
    let shared = server_key.diffie_hellman(&client_pub);
    return Bytes::copy_from_slice(shared.as_bytes());
}
fn key_echange(
    client_pub: PublicKey,
    server_key: StaticSecret,
    init: BytesMut,
    resp: BytesMut,
) -> (BytesMut, BytesMut) {
    let dhs = diffie_hellmen(client_pub, server_key);
    let mut xor = BytesMut::with_capacity(usize::max(init.len(), resp.len()));
    let default: u8 = 0x0;
    for i in 0..xor.capacity() {
        xor.put_bytes(
            init.get(i).unwrap_or(&default) ^ resp.get(i).unwrap_or(&default),
            1,
        )
    }
    let xor_buf = [xor.as_ref()];
    let prk_auth = Salt::new(HKDF_SHA256, "UKEY2 v1 auth".as_bytes()).extract(&dhs);
    let prk_next = Salt::new(HKDF_SHA256, "UKEY2 v1 next".as_bytes()).extract(&dhs);
    let auth_string = prk_auth.expand(&xor_buf, HKDF_SHA256).unwrap();
    let next_secret = prk_next.expand(&xor_buf, HKDF_SHA256).unwrap();
    let l_auth = auth_string
        .len()
        .hmac_algorithm()
        .digest_algorithm()
        .output_len;
    let l_next = next_secret
        .len()
        .hmac_algorithm()
        .digest_algorithm()
        .output_len;
    let mut auth_buf = BytesMut::zeroed(l_auth);
    auth_string.fill(&mut auth_buf).unwrap();
    let mut next_buf = BytesMut::zeroed(l_next);
    next_secret.fill(&mut next_buf).unwrap();

    (auth_buf, next_buf)
}
impl Ukey2 {
    pub fn new(
        init: BytesMut,
        server_key_pair: StaticSecret,
        resp: &[u8],
        client_resp: Ukey2ClientFinished,
    ) -> Result<Ukey2, Unspecified> {
        let d2d_salt: Salt = Salt::new(HKDF_SHA256, D2D_SALT_RAW.as_bytes());
        let pt2_salt: Salt = Salt::new(HKDF_SHA256, PT2_SALT_RAW.as_bytes());
        let client_pub_key = get_public(client_resp.public_key());
        let resp_buf = BytesMut::from(resp);
        let (_auth_string, next_protocol_secret) =
            key_echange(client_pub_key, server_key_pair, init, resp_buf);
        let d2d_client = get_hdkf_key_raw("client", &next_protocol_secret, &d2d_salt)?;
        let d2d_server = get_hdkf_key_raw("server", &next_protocol_secret, &d2d_salt)?;
        let decrypt_key = get_aes_key("ENC:2", &d2d_client, &pt2_salt);
        let recieve_key = get_hmac_key("SIG_1", &d2d_client, &pt2_salt);
        let encrypt_key = get_aes_key("ENC:2", &d2d_server, &pt2_salt);
        let send_key = get_hmac_key("SIG:1", &d2d_server, &pt2_salt);
        Ok(Ukey2 {
            decrypt_key,
            recv_hmac: recieve_key,
            encrypt_key,
            send_hmac: send_key,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_key_gen() {
        let _keypair = get_public_private();
    }
    #[test]
    fn test_key_exchange() {
        let server_keypair = get_public_private();
        let client_keypair = get_public_private();
        diffie_hellmen(PublicKey::from(&client_keypair), server_keypair);
    }
}
