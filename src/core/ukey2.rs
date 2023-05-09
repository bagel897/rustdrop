use bytes::Buf;
use prost::bytes::{BufMut, Bytes, BytesMut};
use prost::Message;
use rand_new::{thread_rng, RngCore};
use rand_old::rngs::OsRng;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::error::Unspecified;
use ring::hkdf::{KeyType, Salt, HKDF_SHA256};
use ring::hmac::{Key, HMAC_SHA256};
use tracing::info;
use x25519_dalek::{PublicKey, StaticSecret};
const D2D_SALT_RAW: &str = "82AA55A0D397F88346CA1CEE8D3909B95F13FA7DEB1D4AB38376B8256DA85510";
const PT2_SALT_RAW: &str = "BF9D2A53C63616D75DB0A7165B91C1EF73E537F2427405FA23610A4BE657642E";
use crate::protobuf::securegcm::{DeviceToDeviceMessage, GcmMetadata, Type};
use crate::protobuf::securemessage::{EncScheme, Header, HeaderAndBody, SecureMessage, SigScheme};

use super::util::get_random;

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
    seq: i32,
}
fn diffie_hellmen(client_pub: PublicKey, server_key: StaticSecret) -> Bytes {
    let shared = server_key.diffie_hellman(&client_pub);
    return Bytes::copy_from_slice(shared.as_bytes());
}
fn get_header() -> Header {
    let mut metadata = GcmMetadata::default();
    metadata.version = Some(1);
    metadata.r#type = Type::DeviceToDeviceMessage.into();
    let mut header = Header::default();
    header.signature_scheme = SigScheme::HmacSha256.into();
    header.encryption_scheme = EncScheme::Aes256Cbc.into();
    header.iv = Some(get_random(16));
    header.public_metadata = Some(metadata.encode_length_delimited_to_vec());
    return header;
}
fn key_echange(
    client_pub: PublicKey,
    server_key: StaticSecret,
    init: Bytes,
    resp: Bytes,
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
        init: Bytes,
        source_key: StaticSecret,
        resp: Bytes,
        dest_key: PublicKey,
        is_client: bool,
    ) -> Result<Ukey2, Unspecified> {
        let (a, b) = match is_client {
            true => ("server", "client"),
            false => ("client", "server"),
        };

        let d2d_salt: Salt = Salt::new(HKDF_SHA256, D2D_SALT_RAW.as_bytes());
        let pt2_salt: Salt = Salt::new(HKDF_SHA256, PT2_SALT_RAW.as_bytes());
        let (_auth_string, next_protocol_secret) = key_echange(dest_key, source_key, init, resp);
        let d2d_client = get_hdkf_key_raw(a, &next_protocol_secret, &d2d_salt)?;
        let d2d_server = get_hdkf_key_raw(b, &next_protocol_secret, &d2d_salt)?;
        info!("D2D REMOVE {:?} {:?}", d2d_client, d2d_server);
        let decrypt_key = get_aes_key("ENC:2", &d2d_client, &pt2_salt);
        let recieve_key = get_hmac_key("SIG_1", &d2d_client, &pt2_salt);
        let encrypt_key = get_aes_key("ENC:2", &d2d_server, &pt2_salt);
        let send_key = get_hmac_key("SIG:1", &d2d_server, &pt2_salt);
        Ok(Ukey2 {
            decrypt_key,
            recv_hmac: recieve_key,
            encrypt_key,
            send_hmac: send_key,
            seq: 0,
        })
    }
    fn encrypt<T: Message>(&self, message: &T) -> Vec<u8> {
        let mut raw = message.encode_length_delimited_to_vec();
        let aad = Aad::empty();
        let mut rng = thread_rng();
        let mut buf = BytesMut::zeroed(12);
        rng.fill_bytes(&mut buf);
        let nonce = Nonce::try_assume_unique_for_key(&buf).unwrap();
        self.encrypt_key
            .seal_in_place_append_tag(nonce, aad, &mut raw)
            .unwrap();
        return raw;
    }
    fn decrypt(&self, mut raw: Vec<u8>) -> Vec<u8> {
        let aad = Aad::empty();
        let mut rng = thread_rng();
        let mut buf = BytesMut::zeroed(12);
        rng.fill_bytes(&mut buf);
        let nonce = Nonce::try_assume_unique_for_key(&buf).unwrap();
        self.decrypt_key
            .open_in_place(nonce, aad, &mut raw)
            .unwrap();
        return raw;
    }
    pub fn encrypt_message<T: Message>(&mut self, message: &T) -> SecureMessage {
        let mut d2d = DeviceToDeviceMessage::default();
        d2d.sequence_number = Some(self.seq);
        self.seq += 1;
        d2d.message = Some(message.encode_length_delimited_to_vec());
        self.encrypt_message_d2d(&d2d)
    }
    fn encrypt_message_d2d(&mut self, message: &DeviceToDeviceMessage) -> SecureMessage {
        info!("{:?}", message);
        let header = get_header();
        let body = self.encrypt(message);
        let mut header_and_body = HeaderAndBody::default();
        header_and_body.header = header;
        header_and_body.body = body;
        let raw_hb = header_and_body.encode_length_delimited_to_vec();
        let mut msg = SecureMessage::default();
        msg.signature = self.sign(&raw_hb);
        msg.header_and_body = raw_hb;
        return msg;
    }
    pub fn decrypt_message<T: Message + Default>(&mut self, message: &SecureMessage) -> T {
        let decrypted = self.decrpyt_message_d2d(message);
        T::decode_length_delimited(decrypted.message.unwrap().as_slice()).unwrap()
    }
    fn decrpyt_message_d2d(&mut self, message: &SecureMessage) -> DeviceToDeviceMessage {
        assert!(self.verify(&message.header_and_body, &message.signature));
        let header_body =
            HeaderAndBody::decode_length_delimited(message.header_and_body.as_slice()).unwrap();
        let decrypted = self.decrypt(header_body.body);

        return DeviceToDeviceMessage::decode_length_delimited(decrypted.as_slice()).unwrap();
    }
    pub fn sign(&self, data: &Vec<u8>) -> Vec<u8> {
        return ring::hmac::sign(&self.send_hmac, data.as_slice())
            .as_ref()
            .to_vec();
    }
    pub fn verify(&self, data: &Vec<u8>, tag: &Vec<u8>) -> bool {
        return ring::hmac::verify(&self.recv_hmac, data.as_slice(), tag).is_ok();
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        core::util::get_paired_frame, protobuf::sharing::nearby::PairedKeyEncryptionFrame,
    };

    use super::*;
    #[test]
    fn test_key_gen() {
        let _keypair = get_public_private();
    }
    #[test]
    fn test_diffie_hellman() {
        let server_keypair = get_public_private();
        let client_keypair = get_public_private();
        let server_pubkey = PublicKey::from(&server_keypair);
        let client_pubkey = PublicKey::from(&client_keypair);
        assert_eq!(
            diffie_hellmen(server_pubkey, client_keypair),
            diffie_hellmen(client_pubkey, server_keypair)
        );
    }
    #[test]
    fn test_key_exchange() {
        let server_keypair = get_public_private();
        let client_keypair = get_public_private();
        let server_pubkey = PublicKey::from(&server_keypair);
        let client_pubkey = PublicKey::from(&client_keypair);
        let init = BytesMut::zeroed(100);
        let resp = BytesMut::zeroed(100);
        let (client_auth, client_nps) = key_echange(
            server_pubkey,
            client_keypair,
            init.clone().into(),
            resp.clone().into(),
        );
        let (server_auth, server_nps) =
            key_echange(client_pubkey, server_keypair, init.into(), resp.into());
        assert_eq!(client_nps, server_nps);
        assert_eq!(client_auth, server_auth);
    }
    #[test]
    fn test_birectional() {
        let server_keypair = get_public_private();
        let client_keypair = get_public_private();
        let server_pubkey = PublicKey::from(&server_keypair);
        let client_pubkey = PublicKey::from(&client_keypair);
        let init = BytesMut::zeroed(100);
        let resp = BytesMut::zeroed(100);
        let mut client_ukey = Ukey2::new(
            init.clone().into(),
            client_keypair,
            resp.clone().into(),
            server_pubkey,
            true,
        )
        .unwrap();
        let mut server_ukey = Ukey2::new(
            init.into(),
            server_keypair,
            resp.into(),
            client_pubkey,
            false,
        )
        .unwrap();
        let msg = get_paired_frame();
        let encrypted = server_ukey.encrypt_message(&msg);
        let decrypted: PairedKeyEncryptionFrame = client_ukey.decrypt_message(&encrypted);
        assert_eq!(decrypted, msg);
    }
}
