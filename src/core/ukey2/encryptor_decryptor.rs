use hmac::Mac;
use p256::ecdh::EphemeralSecret;
use p256::PublicKey;
use prost::bytes::Bytes;
use prost::Message;
use tracing::info;
const D2D_SALT_RAW: &str = "82AA55A0D397F88346CA1CEE8D3909B95F13FA7DEB1D4AB38376B8256DA85510";
const PT2_SALT_RAW: &str = "BF9D2A53C63616D75DB0A7165B91C1EF73E537F2427405FA23610A4BE657642E";
use crate::core::ukey2::core_crypto::{aes_encrypt, get_aes_init, get_hdkf_key_raw, get_hmac_key};
use crate::core::ukey2::key_exchange::key_echange;
use crate::core::ukey2::utils::get_header;
use crate::core::util::{get_iv, iv_from_vec};
use crate::protobuf::securegcm::DeviceToDeviceMessage;
use crate::protobuf::securemessage::{HeaderAndBody, SecureMessage};

use super::core_crypto::{aes_decrypt, HmacSha256};
#[derive(Debug)]
pub(crate) struct Ukey2 {
    decrypt_key: [u8; 32],
    recv_hmac: HmacSha256,
    encrypt_key: [u8; 32],
    send_hmac: HmacSha256,
    seq: i32,
}
impl Ukey2 {
    pub fn new(
        init: Bytes,
        source_key: &EphemeralSecret,
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
        let decrypt_key = get_aes_init("ENC:2", &d2d_client, &pt2_salt);
        let recieve_key = get_hmac_key("SIG:1", &d2d_client, &pt2_salt);
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
        aes_encrypt(self.encrypt_key, iv, message.encode_to_vec())
    }
    fn decrypt(&self, raw: Vec<u8>, iv: [u8; 16]) -> Vec<u8> {
        aes_decrypt(self.decrypt_key, iv, raw)
    }
    pub fn encrypt_message<T: Message>(&mut self, message: &T) -> SecureMessage {
        self.seq += 1;
        let d2d = DeviceToDeviceMessage {
            sequence_number: Some(self.seq),
            message: Some(message.encode_to_vec()),
        };
        self.encrypt_message_d2d(&d2d)
    }
    fn encrypt_message_d2d(&mut self, message: &DeviceToDeviceMessage) -> SecureMessage {
        info!("{:?}", message);
        let iv = get_iv();
        let header = get_header(&iv);
        let body = self.encrypt(message, iv);
        let header_and_body = HeaderAndBody { body, header };
        let raw_hb = header_and_body.encode_to_vec();
        SecureMessage {
            signature: self.sign(&raw_hb),
            header_and_body: raw_hb,
        }
    }
    pub fn decrypt_message<T: Message + Default>(&mut self, message: &SecureMessage) -> T {
        let decrypted = self.decrpyt_message_d2d(message);
        T::decode(decrypted.message()).unwrap()
    }
    fn decrpyt_message_d2d(&mut self, message: &SecureMessage) -> DeviceToDeviceMessage {
        assert!(self.verify(&message.header_and_body, &message.signature));
        let header_body = HeaderAndBody::decode(message.header_and_body.as_slice()).unwrap();
        let iv = iv_from_vec(header_body.header.iv.unwrap());
        let decrypted = self.decrypt(header_body.body, iv);

        return DeviceToDeviceMessage::decode(decrypted.as_slice()).unwrap();
    }

    fn sign(&mut self, data: &[u8]) -> Vec<u8> {
        let mut hmac = self.send_hmac.clone();
        hmac.update(data);

        hmac.finalize().into_bytes().to_vec()
    }
    fn verify(&self, data: &[u8], tag: &[u8]) -> bool {
        let mut hmac = self.recv_hmac.clone();
        hmac.update(data);
        hmac.verify_slice(tag).is_ok()
    }
}
#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use rand::{thread_rng, RngCore};
    use tracing_test::traced_test;

    use crate::{
        core::{protocol::get_paired_frame, ukey2::get_public_private},
        protobuf::sharing::nearby::Frame,
    };

    use super::*;
    fn get_init_resp() -> (Bytes, Bytes) {
        let mut rng = thread_rng();
        let mut init = BytesMut::zeroed(100);
        let mut resp = BytesMut::zeroed(100);
        rng.fill_bytes(&mut init);
        rng.fill_bytes(&mut resp);
        (init.into(), resp.into())
    }
    #[test]
    fn test_unidirectional() {
        let server_keypair = get_public_private();
        let client_keypair = get_public_private();
        let _server_pubkey = PublicKey::from(&server_keypair);
        let client_pubkey = PublicKey::from(&client_keypair);
        let (init, resp) = get_init_resp();
        let mut server_ukey = Ukey2::new(init, &server_keypair, resp, client_pubkey, false);
        let msg = get_paired_frame();
        let _encrypted = server_ukey.encrypt_message(&msg);
    }
    #[traced_test()]
    #[test]
    fn test_bidirectional() {
        let server_keypair = get_public_private();
        let client_keypair = get_public_private();
        let server_pubkey = PublicKey::from(&server_keypair);
        let client_pubkey = PublicKey::from(&client_keypair);
        let (init, resp) = get_init_resp();
        let mut client_ukey = Ukey2::new(
            init.clone(),
            &client_keypair,
            resp.clone(),
            server_pubkey,
            true,
        );
        let mut server_ukey = Ukey2::new(init, &server_keypair, resp, client_pubkey, false);
        info!("Client {:?} Server {:?}", client_ukey, server_ukey);
        let msg = get_paired_frame();
        let encrypted = server_ukey.encrypt_message(&msg);
        let decrypted: Frame = client_ukey.decrypt_message(&encrypted);
        assert_eq!(decrypted, msg);
    }
}
