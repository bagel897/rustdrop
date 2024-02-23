use prost::Message;

use crate::{
    core::{
        ukey2::{get_generic_pubkey, Crypto, CryptoImpl},
        util::get_random,
    },
    protobuf::securegcm::{
        ukey2_client_init::CipherCommitment, ukey2_message::Type, Ukey2ClientFinished,
        Ukey2ClientInit, Ukey2HandshakeCipher, Ukey2Message,
    },
};
// pub struct UkeyInitData {
//     init_raw: Bytes,
//     resp_raw: Bytes,
//     keypair: EphemeralSecret,
// }
// pub fn handle_ukey2_client_init(
//     message: Ukey2ClientInit,
//     init_raw: Bytes,
// ) -> RustdropResult<(keypair, Ukey2ServerInit)> {
//     info!("{:?}", message);
//     if message.version() != 1 {
//         return Err(RustdropError::InvalidMessage("Incorrect version".into()));
//     }
//     assert_eq!(message.random().len(), 32);
//     let mut resp = Ukey2ServerInit::default();
//     let keypair = get_public_private();
//     resp.version = Some(1);
//     resp.random = Some(get_random(32));
//     resp.handshake_cipher = Some(Ukey2HandshakeCipher::P256Sha512.into());
//     resp.public_key = Some(get_generic_pubkey(&keypair).encode_to_vec());
//     info!("{:?}", resp);
//     Ok((keypair, resp))
// }
// pub fn handle_ukey2_client_finish(
//     message: Ukey2ClientFinish,
// ) -> RustdropResult<PublicKey<NistP256>> {
//     let client_pub_key = get_public(message.public_key());
//     Ok(client_pub_key)
// }
fn get_ukey_finish(
    cipher: Ukey2HandshakeCipher,
) -> (Ukey2ClientFinished, <CryptoImpl as Crypto>::SecretKey) {
    assert_eq!(cipher, Ukey2HandshakeCipher::P256Sha512);
    let mut res = Ukey2ClientFinished::default();
    let key = CryptoImpl::genkey();
    res.public_key = Some(get_generic_pubkey::<CryptoImpl>(&key).encode_to_vec());
    (res, key)
}

fn get_commitment<C: Crypto>(cipher: Ukey2HandshakeCipher, frame: &[u8]) -> CipherCommitment {
    let sha = C::sha512(frame);
    let mut commitment = CipherCommitment::default();
    commitment.set_handshake_cipher(cipher);
    commitment.commitment = Some(sha.to_vec());
    commitment
}
pub fn get_ukey_init_finish() -> (
    Ukey2ClientInit,
    Ukey2Message,
    <CryptoImpl as Crypto>::SecretKey,
) {
    let cipher: Ukey2HandshakeCipher = Ukey2HandshakeCipher::P256Sha512;
    let (finish, key) = get_ukey_finish(cipher);
    let frame = Ukey2Message {
        message_data: Some(finish.encode_to_vec()),
        message_type: Some(Type::ClientFinish.into()),
    };
    let cipher_commit = get_commitment::<CryptoImpl>(cipher, &frame.encode_to_vec());
    let init = Ukey2ClientInit {
        version: Some(1),
        random: Some(get_random(32)),
        cipher_commitments: vec![cipher_commit],
        next_protocol: Some(cipher.as_str_name().to_string()),
    };
    (init, frame, key)
}
