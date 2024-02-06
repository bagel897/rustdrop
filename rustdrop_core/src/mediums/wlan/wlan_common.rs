use openssl::sha::sha512;
use prost::Message;

use crate::{
    core::{
        protocol::{get_endpoint_info, get_offline_frame},
        ukey2::{get_generic_pubkey, Crypto, CryptoImpl},
        util::{get_osinfo, get_random},
        Config,
    },
    protobuf::{
        location::nearby::connections::{
            connection_response_frame::ResponseStatus, v1_frame::FrameType, ConnectionRequestFrame,
            ConnectionResponseFrame, OfflineFrame, V1Frame,
        },
        securegcm::{
            ukey2_client_init::CipherCommitment, ukey2_message::Type, Ukey2ClientFinished,
            Ukey2ClientInit, Ukey2HandshakeCipher, Ukey2Message,
        },
    },
};
fn get_ukey_finish(
    cipher: Ukey2HandshakeCipher,
) -> (Ukey2ClientFinished, <CryptoImpl as Crypto>::SecretKey) {
    assert_eq!(cipher, Ukey2HandshakeCipher::P256Sha512);
    let mut res = Ukey2ClientFinished::default();
    let key = CryptoImpl::genkey();
    res.public_key = Some(get_generic_pubkey::<CryptoImpl>(&key).encode_to_vec());
    (res, key)
}

fn get_commitment(cipher: Ukey2HandshakeCipher, frame: &[u8]) -> CipherCommitment {
    let sha = sha512(frame);
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
    let cipher_commit = get_commitment(cipher, &frame.encode_to_vec());
    let init = Ukey2ClientInit {
        version: Some(1),
        random: Some(get_random(32)),
        cipher_commitments: vec![cipher_commit],
        next_protocol: Some(cipher.as_str_name().to_string()),
    };
    (init, frame, key)
}
pub fn get_conn_response() -> OfflineFrame {
    let conn = ConnectionResponseFrame {
        response: Some(ResponseStatus::Accept.into()),
        os_info: Some(get_osinfo()),
        handshake_data: Some(get_random(10)),
        nearby_connections_version: Some(1),
        ..Default::default()
    };
    let v1 = V1Frame {
        r#type: Some(FrameType::ConnectionResponse.into()),
        connection_response: Some(conn),
        ..Default::default()
    };
    get_offline_frame(v1)
}
pub(crate) fn get_con_request(config: &Config) -> OfflineFrame {
    let init = ConnectionRequestFrame {
        endpoint_info: Some(get_endpoint_info(config)),
        endpoint_name: Some(config.name.to_string()),
        ..Default::default()
    };
    let v1 = V1Frame {
        r#type: Some(FrameType::ConnectionRequest.into()),
        connection_request: Some(init),
        ..Default::default()
    };
    get_offline_frame(v1)
}
