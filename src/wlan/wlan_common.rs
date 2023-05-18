use crate::core::protocol::get_endpoint_id;
use crate::core::util::get_random;
use crate::core::Config;
use crate::protobuf::location::nearby::connections::v1_frame::FrameType;
use crate::protobuf::location::nearby::connections::ConnectionRequestFrame;
use crate::protobuf::location::nearby::connections::ConnectionResponseFrame;
use crate::protobuf::location::nearby::connections::OfflineFrame;
use crate::protobuf::location::nearby::connections::V1Frame;
use crate::protobuf::securegcm::ukey2_client_init::CipherCommitment;
use crate::protobuf::securegcm::Ukey2ClientInit;
use crate::protobuf::securegcm::Ukey2HandshakeCipher;
use bytes::Bytes;
pub fn decode_32_len(buf: &Bytes) -> Result<usize, ()> {
    if buf.len() < 4 {
        return Err(());
    }
    let mut arr = [0u8; 4];
    for i in 0..4 {
        arr[i] = buf[i];
    }
    return Ok(i32::from_be_bytes(arr) as usize);
}
pub fn get_ukey_init() -> Ukey2ClientInit {
    let mut ukey_init = Ukey2ClientInit::default();
    ukey_init.version = Some(1);
    ukey_init.random = Some(get_random(32));
    let mut cipher = CipherCommitment::default();
    cipher.handshake_cipher = Some(Ukey2HandshakeCipher::P256Sha512.into());
    ukey_init.cipher_commitments = vec![cipher];
    return ukey_init;
}
pub fn get_conn_response() -> OfflineFrame {
    let conn = ConnectionResponseFrame::default();
    let mut v1 = V1Frame::default();
    v1.r#type = Some(FrameType::ConnectionResponse.into());
    v1.connection_response = Some(conn);
    let mut offline = OfflineFrame::default();
    offline.version = Some(1);
    offline.v1 = Some(v1);
    return offline;
}
pub(crate) fn get_con_request(config: &Config) -> OfflineFrame {
    let mut init = ConnectionRequestFrame::default();
    init.endpoint_info = Some(get_endpoint_id(config));
    // init.endpoint_id = Some(self.config.name.to_string());
    init.endpoint_name = Some(config.name.to_string());
    let mut v1 = V1Frame::default();
    v1.r#type = Some(FrameType::ConnectionRequest.into());
    v1.connection_request = Some(init);
    let mut frame = OfflineFrame::default();
    frame.version = Some(1);
    frame.v1 = Some(v1);
    return frame;
}
