use crate::core::protocol::get_endpoint_id;
use crate::core::util::get_random;
use crate::core::Config;
use crate::protobuf::location::nearby::connections::v1_frame::FrameType;
use crate::protobuf::location::nearby::connections::ConnectionRequestFrame;
use crate::protobuf::location::nearby::connections::ConnectionResponseFrame;
use crate::protobuf::location::nearby::connections::OfflineFrame;
use crate::protobuf::location::nearby::connections::V1Frame;
use crate::protobuf::securegcm::ukey2_client_init::CipherCommitment;
use crate::protobuf::securegcm::ukey2_message::Type;
use crate::protobuf::securegcm::Ukey2HandshakeCipher;
use crate::protobuf::securegcm::Ukey2Message;
use crate::protobuf::securemessage::SecureMessage;
use crate::{core::ukey2::Ukey2, protobuf::securegcm::Ukey2ClientInit};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};
use tracing::info;
fn decode_32_len(buf: &Bytes) -> Result<usize, ()> {
    if buf.len() < 4 {
        return Err(());
    }
    let mut arr = [0u8; 4];
    for i in 0..4 {
        arr[i] = buf[i];
    }
    return Ok(i32::from_be_bytes(arr) as usize);
}
pub(super) struct StreamHandler {
    read_half: OwnedReadHalf,
    write_half: OwnedWriteHalf,
    buf: BytesMut,
    ukey2: Option<Ukey2>,
}
impl StreamHandler {
    pub fn new(stream: TcpStream) -> Self {
        let (read_half, write_half) = stream.into_split();
        let buf = BytesMut::with_capacity(1000);
        StreamHandler {
            read_half,
            write_half,
            buf,
            ukey2: None,
        }
    }
    pub fn setup_ukey2(&mut self, ukey2: Ukey2) {
        self.ukey2 = Some(ukey2);
    }
    pub fn decrypt_message<T: Message + Default>(&mut self, message: &SecureMessage) -> T {
        return self.ukey2.as_mut().unwrap().decrypt_message::<T>(message);
    }
    pub async fn send<T: Message>(&mut self, message: &T) {
        info!("{:?}", message);
        let mut bytes = BytesMut::with_capacity(message.encoded_len() + 4);
        bytes.put_i32(message.encoded_len().try_into().unwrap());
        bytes.extend_from_slice(message.encode_to_vec().as_slice());
        info!("Sending {:#X}", bytes);
        self.write_half
            .write_all_buf(&mut bytes)
            .await
            .expect("Send Error");
    }
    pub async fn send_securemessage<T: Message>(&mut self, message: &T) {
        info!("{:?}", message);
        let encrypted = self.ukey2.as_mut().unwrap().encrypt_message(message);
        self.send(&encrypted).await;
    }
    pub async fn send_ukey2<T: Message>(&mut self, message: &T, message_type: Type) -> Bytes {
        info!("{:?}", message);
        let message_data = Some(message.encode_to_vec());
        let ukey = Ukey2Message {
            message_type: Some(message_type.into()),
            message_data,
        };
        self.send(&ukey).await;
        return ukey.encode_to_vec().into();
    }
    fn try_yield_message(&mut self) -> Option<Bytes> {
        if let Ok(len) = decode_32_len(&self.buf.clone().into()) {
            return self.decode_message(len);
        }
        None
    }
    async fn read_data(&mut self) -> Result<(), ()> {
        let mut new_data = BytesMut::with_capacity(1000);
        let r = self.read_half.read_buf(&mut new_data).await.unwrap();
        if r == 0 {
            info!("No data left");
            return Err(());
        }
        self.buf.extend_from_slice(&new_data);
        Ok(())
    }

    fn decode_message(&mut self, len: usize) -> Option<Bytes> {
        info!("Reading: buf {:#X}", self.buf);
        let e_idx = len + 4;
        if self.buf.len() >= e_idx {
            let mut other_buf = self.buf.split_to(e_idx);
            other_buf.advance(4);
            info!("Yielding {:#X} len {}", other_buf, len);
            assert_eq!(other_buf.len(), len);
            return Some(other_buf.into());
        }
        None
    }
    pub async fn shutdown(&mut self) {
        info!("Shutting Down");
        self.write_half.shutdown().await.unwrap();
    }
    async fn next(&mut self) -> Result<Bytes, ()> {
        if let Some(bytes) = self.try_yield_message() {
            return Ok(bytes);
        }
        loop {
            let r = self.read_data().await;
            if let Some(bytes) = self.try_yield_message() {
                return Ok(bytes);
            } else {
                if r.is_err() {
                    info!("Stream is finished");
                    return Err(());
                }
            }
        }
    }
    pub async fn next_message<T: Message + Default>(&mut self) -> Result<T, ()> {
        let raw = self.next().await?;
        Ok(T::decode(raw).unwrap())
    }
    pub async fn next_ukey_message<T: Message + Default>(&mut self) -> Result<(T, Bytes), ()> {
        let raw = self.next().await?;
        let ukey = Ukey2Message::decode(raw.clone()).unwrap();
        info!("Recievd ukey2 message {:?}", ukey);
        assert!(Type::is_valid(ukey.message_type.unwrap()));
        Ok((T::decode(ukey.message_data()).unwrap(), raw))
    }
    pub async fn next_decrypted<T: Message + Default>(&mut self) -> Result<T, ()> {
        let secure: SecureMessage = self.next_message().await?;
        Ok(self.decrypt_message::<T>(&secure))
    }
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
