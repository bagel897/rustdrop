use crate::core::io::reader::BufferedReader;
use crate::core::io::writer::SecureWriteExt;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::core::protocol::try_decode_ukey2_alert;
use crate::core::util::ukey_alert_to_str;
use crate::core::{PayloadHandler, TcpStreamClosedError};
use crate::protobuf::location::nearby::connections::OfflineFrame;
use crate::protobuf::securegcm::Ukey2Message;
use crate::protobuf::securemessage::SecureMessage;
use crate::protobuf::sharing::nearby::Frame;
use crate::ui::UiHandle;
use crate::{core::ukey2::Ukey2, protobuf::securegcm::ukey2_message::Type};
use bytes::Bytes;
use prost::{DecodeError, Message};
use tokio::{
    io::AsyncWriteExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};
use tracing::{debug, info};

#[derive(Debug)]
pub(super) struct StreamHandler {
    reader: BufferedReader<OwnedReadHalf>,
    write_half: OwnedWriteHalf,
    ukey2: Option<Ukey2>,
    ui_handle: Arc<Mutex<dyn UiHandle>>,
    payload_handler: PayloadHandler,
}
impl StreamHandler {
    pub fn new(stream: TcpStream, ui_handle: Arc<Mutex<dyn UiHandle>>) -> Self {
        let (read_half, write_half) = stream.into_split();
        StreamHandler {
            reader: BufferedReader::new(read_half),
            write_half,
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
        self.write_half.send(message).await
    }
    async fn send_securemessage(&mut self, message: &OfflineFrame) {
        info!("{:?}", message);
        let encrypted = self.ukey2.as_mut().unwrap().encrypt_message(message);
        self.send(&encrypted).await;
    }
    pub async fn send_payload(&mut self, message: &Frame) {
        info!("{:?}", message);
        let chunks = self.payload_handler.send_message(message);
        info!("Sending {} chunks", chunks.len());
        for chunk in chunks {
            self.send_securemessage(&chunk).await;
        }
    }
    pub async fn send_ukey2<T: Message>(&mut self, message: &T, message_type: Type) -> Bytes {
        self.write_half.send_ukey2(message, message_type).await
    }
    pub async fn shutdown(&mut self) {
        info!("Shutting Down");
        self.write_half.shutdown().await.unwrap();
    }
    pub async fn next_offline(&mut self) -> Result<OfflineFrame, TcpStreamClosedError> {
        self.reader.next_message().await
    }
    // TODO impl as a trait extension
    pub async fn next_ukey_message<T: Message + Default>(
        &mut self,
    ) -> Result<(T, Bytes), TcpStreamClosedError> {
        let raw = self.reader.next().await?;
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
        let secure: SecureMessage = self.reader.next_message().await?;
        debug!("Recieved secure message {:?}", secure);
        Ok(self.decrypt_message::<T>(&secure))
    }
    pub async fn next_payload(&mut self) -> Result<Frame, TcpStreamClosedError> {
        loop {
            let decrypted = self.next_decrypted().await?;
            debug!("Recieved decrypted message {:?}", decrypted);
            self.payload_handler.push_data(decrypted);
            let r = self.payload_handler.get_next_payload();
            if r.is_some() {
                info!("Recievd payload message {:?}", r.as_ref().unwrap());
                return Ok(r.unwrap());
            }
        }
    }
}
