use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes256;
use bytes::Buf;
use cbc::{Decryptor, Encryptor};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use prost::bytes::{BufMut, Bytes, BytesMut};
use prost::Message;
use rand_old::rngs::OsRng;
use sha2::Sha256;
use tracing::info;
use x25519_dalek::{PublicKey, StaticSecret};
const D2D_SALT_RAW: &str = "82AA55A0D397F88346CA1CEE8D3909B95F13FA7DEB1D4AB38376B8256DA85510";
const PT2_SALT_RAW: &str = "BF9D2A53C63616D75DB0A7165B91C1EF73E537F2427405FA23610A4BE657642E";
use crate::core::util::{get_iv, iv_from_vec};
use crate::protobuf::securegcm::{DeviceToDeviceMessage, GcmMetadata, Type};
use crate::protobuf::securemessage::{EncScheme, Header, HeaderAndBody, SecureMessage, SigScheme};

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;
type HmacSha256 = Hmac<Sha256>;
pub fn get_public_private() -> StaticSecret {
    StaticSecret::new(OsRng)
}
pub fn get_public(raw: &[u8]) -> PublicKey {
    let mut buf = [0u8; 32];
    assert!(raw.len() <= 32);
    raw.clone().copy_to_slice(&mut buf);
    PublicKey::from(buf)
}

fn get_hdkf_key_raw(info: &'static str, key: &[u8], salt: &Bytes) -> Bytes {
    let hk = Hkdf::<Sha256>::new(Some(salt), key);
    let mut buf = BytesMut::zeroed(32);
    hk.expand(info.as_bytes(), &mut buf).unwrap();
    return buf.into();
}

fn get_hmac_key(info: &'static str, key: &[u8], salt: &Bytes) -> HmacSha256 {
    let hk = Hkdf::<Sha256>::new(Some(salt), key);
    let mut buf = BytesMut::zeroed(32);
    hk.expand(info.as_bytes(), &mut buf).unwrap();
    let key = HmacSha256::new_from_slice(&buf).unwrap();
    return key;
}
fn get_aes_init(info: &'static str, key: &[u8], salt: &Bytes) -> [u8; 32] {
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
pub(crate) struct Ukey2 {
    decrypt_key: [u8; 32],
    recv_hmac: HmacSha256,
    encrypt_key: [u8; 32],
    send_hmac: HmacSha256,
    seq: i32,
}
fn diffie_hellmen(client_pub: PublicKey, server_key: StaticSecret) -> Bytes {
    let shared = server_key.diffie_hellman(&client_pub);
    return Bytes::copy_from_slice(shared.as_bytes());
}
fn get_header(iv: &[u8; 16]) -> Header {
    let mut metadata = GcmMetadata::default();
    metadata.version = Some(1);
    metadata.r#type = Type::DeviceToDeviceMessage.into();
    let mut header = Header::default();
    header.signature_scheme = SigScheme::HmacSha256.into();
    header.encryption_scheme = EncScheme::Aes256Cbc.into();
    header.iv = Some(iv.to_vec());
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
    let prk_auth = Hkdf::<Sha256>::new(Some("UKEY2 v1 auth".as_bytes()), &dhs);
    let prk_next = Hkdf::<Sha256>::new(Some("UKEY2 v1 next".as_bytes()), &dhs);
    let l_auth = 6;
    let l_next = 32;
    let mut auth_buf = BytesMut::zeroed(l_auth);
    let mut next_buf = BytesMut::zeroed(l_next);
    prk_auth.expand(&xor, &mut auth_buf).unwrap();
    prk_next.expand(&xor, &mut next_buf).unwrap();

    (auth_buf, next_buf)
}
impl Ukey2 {
    pub fn new(
        init: Bytes,
        source_key: StaticSecret,
        resp: Bytes,
        dest_key: PublicKey,
        is_client: bool,
    ) -> Ukey2 {
        let (a, b) = match is_client {
            true => ("server", "client"),
            false => ("client", "server"),
        };

        let d2d_salt: Bytes = D2D_SALT_RAW.as_bytes().into();
        let pt2_salt: Bytes = PT2_SALT_RAW.as_bytes().into();
        let (_auth_string, next_protocol_secret) = key_echange(dest_key, source_key, init, resp);
        let d2d_client = get_hdkf_key_raw(a, &next_protocol_secret, &d2d_salt);
        let d2d_server = get_hdkf_key_raw(b, &next_protocol_secret, &d2d_salt);
        info!("D2D REMOVE {:?} {:?}", d2d_client, d2d_server);
        let decrypt_key = get_aes_init("ENC:2", &d2d_client, &pt2_salt);
        let recieve_key = get_hmac_key("SIG_1", &d2d_client, &pt2_salt);
        let encrypt_key = get_aes_init("ENC:2", &d2d_server, &pt2_salt);
        let send_key = get_hmac_key("SIG:1", &d2d_server, &pt2_salt);
        Ukey2 {
            decrypt_key,
            recv_hmac: recieve_key,
            encrypt_key,
            send_hmac: send_key,
            seq: 0,
        }
    }
    fn encrypt<T: Message>(&self, message: &T, iv: [u8; 16]) -> Vec<u8> {
        let key = get_aes_key_encrypt(self.decrypt_key, iv);
        let raw = message.encode_length_delimited_to_vec();
        return key.encrypt_padded_vec_mut::<Pkcs7>(&raw);
    }
    fn decrypt(&self, raw: Vec<u8>, iv: [u8; 16]) -> Vec<u8> {
        let key = get_aes_key_decrypt(self.decrypt_key, iv);
        return key
            .decrypt_padded_vec_mut::<Pkcs7>(raw.as_slice())
            .unwrap()
            .to_vec();
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
        let iv = get_iv();
        let header = get_header(&iv);
        let body = self.encrypt(message, iv);
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
        let iv = iv_from_vec(header_body.header.iv.unwrap());
        let decrypted = self.decrypt(header_body.body, iv);

        return DeviceToDeviceMessage::decode_length_delimited(decrypted.as_slice()).unwrap();
    }

    pub fn sign(&mut self, data: &Vec<u8>) -> Vec<u8> {
        let mut hmac = self.send_hmac.clone();
        hmac.update(data);
        return hmac.finalize().into_bytes().to_vec();
    }
    pub fn verify(&self, data: &Vec<u8>, tag: &Vec<u8>) -> bool {
        let mut hmac = self.recv_hmac.clone();
        hmac.update(data);
        return hmac.verify_slice(&tag).is_ok();
    }
}
#[cfg(test)]
mod tests {
    use rand_new::{thread_rng, RngCore};

    use crate::{
        core::util::get_paired_frame, protobuf::sharing::nearby::PairedKeyEncryptionFrame,
    };

    use super::*;
    #[test]
    fn test_key_gen() {
        let _keypair = get_public_private();
    }
    fn get_init_resp() -> (Bytes, Bytes) {
        let mut rng = thread_rng();
        let mut init = BytesMut::zeroed(100);
        let mut resp = BytesMut::zeroed(100);
        rng.fill_bytes(&mut init);
        rng.fill_bytes(&mut resp);
        (init.into(), resp.into())
    }
    fn get_iv() -> [u8; 16] {
        let mut buf = [0u8; 16];
        let mut rng = thread_rng();
        rng.fill_bytes(&mut buf);
        return buf;
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
        let (init, resp) = get_init_resp();
        let (client_auth, client_nps) =
            key_echange(server_pubkey, client_keypair, init.clone(), resp.clone());
        let (server_auth, server_nps) = key_echange(client_pubkey, server_keypair, init, resp);
        assert_eq!(client_nps, server_nps);
        assert_eq!(client_auth, server_auth);
    }
    #[test]
    fn test_unidirectional() {
        let server_keypair = get_public_private();
        let client_keypair = get_public_private();
        let server_pubkey = PublicKey::from(&server_keypair);
        let client_pubkey = PublicKey::from(&client_keypair);
        let (init, resp) = get_init_resp();
        let mut server_ukey = Ukey2::new(init, server_keypair, resp, client_pubkey, false);
        let msg = get_paired_frame();
        let encrypted = server_ukey.encrypt_message(&msg);
        let decrypted: PairedKeyEncryptionFrame = server_ukey.decrypt_message(&encrypted);
        assert_eq!(decrypted, msg);
    }
    #[test]
    fn test_bidirectional() {
        let server_keypair = get_public_private();
        let client_keypair = get_public_private();
        let server_pubkey = PublicKey::from(&server_keypair);
        let client_pubkey = PublicKey::from(&client_keypair);
        let (init, resp) = get_init_resp();
        let mut client_ukey = Ukey2::new(
            init.clone(),
            client_keypair,
            resp.clone(),
            server_pubkey,
            true,
        );
        let mut server_ukey = Ukey2::new(init, server_keypair, resp, client_pubkey, false);
        let msg = get_paired_frame();
        let encrypted = server_ukey.encrypt_message(&msg);
        let decrypted: PairedKeyEncryptionFrame = client_ukey.decrypt_message(&encrypted);
        assert_eq!(decrypted, msg);
    }
}
