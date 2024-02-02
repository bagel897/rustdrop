use prost::{bytes::Bytes, Message};
use tracing::info;

use super::{
    consts::{D2D_SALT, PT2_SALT},
    openssl::OpenSSL,
};
use crate::{
    core::{
        ukey2::{generic::Crypto, key_exchange::key_echange, utils::get_header},
        util::{get_iv, iv_from_vec},
    },
    protobuf::{
        securegcm::DeviceToDeviceMessage,
        securemessage::{HeaderAndBody, SecureMessage},
    },
};
type CryptoImpl = OpenSSL;
pub(crate) struct Ukey2<C: Crypto> {
    pub crypto: C,
    decrypt_key: C::AesKey,
    recv_hmac: C::HmacKey,
    encrypt_key: C::AesKey,
    send_hmac: C::HmacKey,
    seq: i32,
}
impl<C: Crypto> Ukey2<C> {
    pub fn new(
        client_init: Bytes,
        source_key: C::SecretKey,
        server_init: Bytes,
        dest_key: C::PublicKey,
        is_client: bool,
    ) -> Self {
        info!("{:X?}", D2D_SALT);
        let (_auth_string, next_protocol_secret) =
            key_echange::<C>(dest_key, source_key, client_init, server_init);
        let d2d_client = C::extract_expand("client", &next_protocol_secret, &D2D_SALT, 32);
        let d2d_server = C::extract_expand("server", &next_protocol_secret, &D2D_SALT, 32);
        let client_key = C::derive_aes_decrypt("ENC:2", &d2d_client, &PT2_SALT, 32);
        let client_hmac = C::derive_hmac("SIG:1", &d2d_client, &PT2_SALT, 32);
        let server_key = C::derive_aes_encrypt("ENC:2", &d2d_server, &PT2_SALT, 32);
        let server_hmac = C::derive_hmac("SIG:1", &d2d_server, &PT2_SALT, 32);
        if is_client {
            Ukey2 {
                crypto: C::default(),
                decrypt_key: server_key,
                recv_hmac: server_hmac,
                encrypt_key: client_key,
                send_hmac: client_hmac,
                seq: 0,
            }
        } else {
            Ukey2 {
                crypto: C::default(),
                decrypt_key: client_key,
                recv_hmac: client_hmac,
                encrypt_key: server_key,
                send_hmac: server_hmac,
                seq: 0,
            }
        }
    }
    fn encrypt<T: Message>(&self, message: &T, iv: [u8; 16]) -> Vec<u8> {
        C::encrypt(&self.encrypt_key, iv, message.encode_to_vec())
    }
    fn decrypt(&self, raw: Vec<u8>, iv: [u8; 16]) -> Vec<u8> {
        C::decrypt(&self.decrypt_key, iv, raw)
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
        assert_eq!(&self.verify(&message.header_and_body), &message.signature);
        let header_body = HeaderAndBody::decode(message.header_and_body.as_slice()).unwrap();
        let iv = iv_from_vec(header_body.header.iv.unwrap());
        let decrypted = self.decrypt(header_body.body, iv);

        return DeviceToDeviceMessage::decode(decrypted.as_slice()).unwrap();
    }

    fn sign(&mut self, data: &[u8]) -> Vec<u8> {
        C::sign(&self.send_hmac, data)
    }
    fn verify(&self, data: &[u8]) -> Vec<u8> {
        C::sign(&self.recv_hmac, data)
    }
}
#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use rand::{thread_rng, RngCore};
    use tracing_test::traced_test;

    use crate::{core::protocol::get_paired_frame, protobuf::sharing::nearby::Frame};

    use super::*;
    fn get_init_resp() -> (Bytes, Bytes) {
        let mut rng = thread_rng();
        let mut init = BytesMut::zeroed(100);
        let mut resp = BytesMut::zeroed(100);
        rng.fill_bytes(&mut init);
        rng.fill_bytes(&mut resp);
        (init.into(), resp.into())
    }
    fn to_pubkey<C: Crypto>(private: &C::SecretKey) -> C::PublicKey {
        let (x, y) = C::from_pubkey(private);
        C::to_pubkey(&x, &y)
    }
    #[test]
    fn test_unidirectional() {
        let server_keypair = OpenSSL::genkey();
        let client_keypair = OpenSSL::genkey();

        // let _server_pubkey = to_pubkey(&server_keypair);
        let client_pubkey = to_pubkey::<OpenSSL>(&client_keypair);
        let (init, resp) = get_init_resp();
        let mut server_ukey: Ukey2<OpenSSL> =
            Ukey2::new(init, server_keypair, resp, client_pubkey, false);
        let msg = get_paired_frame();
        let _encrypted = server_ukey.encrypt_message(&msg);
    }
    #[traced_test()]
    #[test]
    fn test_bidirectional() {
        let server_keypair = OpenSSL::genkey();
        let client_keypair = OpenSSL::genkey();

        let server_pubkey = to_pubkey::<OpenSSL>(&server_keypair);
        let client_pubkey = to_pubkey::<OpenSSL>(&client_keypair);
        let (init, resp) = get_init_resp();
        let mut client_ukey: Ukey2<OpenSSL> = Ukey2::new(
            init.clone(),
            client_keypair,
            resp.clone(),
            server_pubkey,
            true,
        );
        let mut server_ukey: Ukey2<OpenSSL> =
            Ukey2::new(init, server_keypair, resp, client_pubkey, false);
        // info!("Client {:?} Server {:?}", client_ukey, server_ukey);
        let msg = get_paired_frame();
        let encrypted = server_ukey.encrypt_message(&msg);
        let decrypted: Frame = client_ukey.decrypt_message(&encrypted);
        assert_eq!(decrypted, msg);
    }
}
