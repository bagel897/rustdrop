use crate::core::protocol::get_endpoint_id;
use crate::core::protocol::get_offline_frame;
use crate::core::util::get_osinfo;
use crate::core::util::get_random;
use crate::core::Config;
use crate::protobuf::location::nearby::connections::connection_response_frame::ResponseStatus;
use crate::protobuf::location::nearby::connections::v1_frame::FrameType;
use crate::protobuf::location::nearby::connections::ConnectionRequestFrame;
use crate::protobuf::location::nearby::connections::ConnectionResponseFrame;
use crate::protobuf::location::nearby::connections::OfflineFrame;
use crate::protobuf::location::nearby::connections::V1Frame;
use crate::protobuf::securegcm::ukey2_client_init::CipherCommitment;
use crate::protobuf::securegcm::Ukey2ClientInit;
use crate::protobuf::securegcm::Ukey2HandshakeCipher;
pub fn get_ukey_init() -> Ukey2ClientInit {
    let cipher = CipherCommitment {
        handshake_cipher: Some(Ukey2HandshakeCipher::P256Sha512.into()),
        commitment: None,
    };
    Ukey2ClientInit {
        version: Some(1),
        random: Some(get_random(32)),
        cipher_commitments: vec![cipher],
        next_protocol: Some(Ukey2HandshakeCipher::P256Sha512.as_str_name().to_string()),
    }
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
        endpoint_info: Some(get_endpoint_id(config)),
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
