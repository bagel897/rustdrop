use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::core::protocol::try_decode_ukey2_alert;
use crate::core::util::ukey_alert_to_str;
use crate::core::{PayloadHandler, TcpStreamClosedError};
use crate::protobuf::location::nearby::connections::OfflineFrame;
use crate::protobuf::securegcm::Ukey2Message;
use crate::protobuf::securemessage::SecureMessage;
use crate::ui::UiHandle;
use crate::{core::ukey2::Ukey2, protobuf::securegcm::ukey2_message::Type};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use prost::{DecodeError, Message};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};
use tracing::info;

use super::wlan_common::decode_32_len;
pub(super) struct StreamHandler {
    read_half: OwnedReadHalf,
    write_half: OwnedWriteHalf,
    buf: BytesMut,
    ukey2: Option<Ukey2>,
    ui_handle: Arc<Mutex<dyn UiHandle>>,
    payload_handler: PayloadHandler,
}
impl StreamHandler {
    pub fn new(stream: TcpStream, ui_handle: Arc<Mutex<dyn UiHandle>>) -> Self {
        let (read_half, write_half) = stream.into_split();
        let buf = BytesMut::with_capacity(1000);
        StreamHandler {
            read_half,
            write_half,
            buf,
            ukey2: None,
            ui_handle,
            payload_handler: PayloadHandler::default(),
        }
    }
    pub fn setup_ukey2(&mut self, ukey2: Ukey2) {
        self.ukey2 = Some(ukey2);
    }
    pub fn decrypt_message<T: Message + Default>(&mut self, message: &SecureMessage) -> T {
        return self.ukey2.as_mut().unwrap().decrypt_message::<T>(message);
    }
    fn handle_error<T: Debug>(&mut self, error: T) {
        self.ui_handle
            .lock()
            .unwrap()
            .handle_error(format!("{:?}", error));
    }
    fn try_handle_ukey(&mut self, error: DecodeError, raw: &Bytes) {
        match try_decode_ukey2_alert(raw) {
            Ok(a) => self.handle_error(ukey_alert_to_str(a)),
            Err(_e) => self.handle_error(error),
        }
    }
    pub async fn send<T: Message>(&mut self, message: &T) {
        info!("{:?}", message);
        let mut bytes = BytesMut::with_capacity(message.encoded_len() + 4);
        bytes.put_i32(
            message
                .encoded_len()
                .try_into()
                .map_err(|e| self.handle_error(e))
                .unwrap(),
        );
        bytes.extend_from_slice(message.encode_to_vec().as_slice());
        info!("Sending {:#X}", bytes);
        self.write_half
            .write_all_buf(&mut bytes)
            .await
            .expect("Send Error");
    }
    async fn send_securemessage(&mut self, message: &OfflineFrame) {
        info!("{:?}", message);
        let encrypted = self.ukey2.as_mut().unwrap().encrypt_message(message);
        self.send(&encrypted).await;
    }
    pub async fn send_payload<T: Message>(&mut self, message: &T) {
        info!("{:?}", message);
        let chunks = self.payload_handler.send_message(message);
        for chunk in chunks {
            self.send_securemessage(&chunk).await;
        }
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
    async fn read_data(&mut self) -> Result<(), TcpStreamClosedError> {
        let mut new_data = BytesMut::with_capacity(1000);
        let r = self.read_half.read_buf(&mut new_data).await.unwrap();
        if r == 0 {
            info!("No data left");
            return Err(TcpStreamClosedError {});
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
    async fn next(&mut self) -> Result<Bytes, TcpStreamClosedError> {
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
                    return Err(r.err().unwrap());
                }
            }
        }
    }
    pub async fn next_message<T: Message + Default>(&mut self) -> Result<T, TcpStreamClosedError> {
        let raw = self.next().await?;
        Ok(T::decode(raw.clone())
            .map_err(|e| self.try_handle_ukey(e, &raw))
            .unwrap())
    }
    pub async fn next_ukey_message<T: Message + Default>(
        &mut self,
    ) -> Result<(T, Bytes), TcpStreamClosedError> {
        let raw = self.next().await?;
        let ukey = Ukey2Message::decode(raw.clone()).unwrap();
        info!("Recievd ukey2 message {:?}", ukey);
        Ok((
            T::decode(ukey.message_data())
                .map_err(|e| self.try_handle_ukey(e, &raw))
                .unwrap(),
            raw,
        ))
    }
    async fn next_decrypted<T: Message + Default>(&mut self) -> Result<T, TcpStreamClosedError> {
        let secure: SecureMessage = self.next_message().await?;
        Ok(self.decrypt_message::<T>(&secure))
    }
    pub async fn next_payload<T: Message + Default>(&mut self) -> Result<T, TcpStreamClosedError> {
        loop {
            let decrypted = self.next_decrypted().await?;
            self.payload_handler.push_data(decrypted);
            let r = self.payload_handler.get_next_payload();
            if r.is_some() {
                return Ok(r.unwrap());
            }
        }
    }
}
