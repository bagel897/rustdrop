use std::fmt::Debug;

use bytes::Bytes;
use prost::{DecodeError, Message};
use tokio::net::TcpStream;
use tracing::info;

use crate::{
    core::{
        io::{reader::ReaderRecv, writer::WriterSend},
        protocol::{repeat_keep_alive, try_decode_ukey2_alert},
        ukey2::Ukey2,
        util::ukey_alert_to_str,
        PayloadReciever, PayloadRecieverHandle, PayloadSender, RustdropError,
    },
    protobuf::{
        location::nearby::connections::OfflineFrame,
        securegcm::{ukey2_message::Type, Ukey2Message},
        sharing::nearby::Frame,
    },
    runner::application::Application,
    ui::UiHandle,
};

pub(super) struct StreamHandler<U: UiHandle> {
    reader: ReaderRecv,
    write_half: WriterSend,
    ukey2: Option<Ukey2>,
    app: Application<U>,
    payload_recv: Option<PayloadRecieverHandle>,
    payload_send: PayloadSender,
}
impl<U: UiHandle> StreamHandler<U> {
    pub fn new(stream: TcpStream, app: Application<U>) -> Self {
        let (read_half, write_half) = stream.into_split();
        StreamHandler {
            reader: ReaderRecv::new(read_half, &app),
            write_half: WriterSend::new(write_half, &app),
            ukey2: None,
            app,
            payload_recv: None,
            payload_send: PayloadSender::default(),
        }
    }
    pub async fn setup_ukey2(&mut self, ukey2_send: Ukey2, ukey2_recv: Ukey2) {
        self.ukey2 = Some(ukey2_send);
        self.start_keep_alive().await;
        let (decrypted, decryptor) = ukey2_recv.start_decrypting(self.reader.clone());
        self.app.tracker.track_future(decryptor).await.unwrap();
        let (payload_handler, payload_recv) = PayloadReciever::push_frames(decrypted);
        self.payload_recv = Some(payload_recv);
        self.app
            .tracker
            .track_future(payload_handler)
            .await
            .unwrap();
    }
    fn handle_error<T: Debug>(&mut self, error: T) {
        self.app.ui().unwrap().handle_error(format!("{:?}", error));
    }
    fn try_handle_ukey(&mut self, error: DecodeError, raw: &Bytes) {
        match try_decode_ukey2_alert(raw) {
            Ok(a) => self.handle_error(ukey_alert_to_str(a)),
            Err(_e) => self.handle_error(error),
        }
    }
    pub async fn send<T: Message>(&self, message: &T) {
        self.write_half.send(message).await;
    }
    async fn send_securemessage(&mut self, message: &OfflineFrame) {
        info!("{:?}", message);
        let encrypted = self.ukey2.as_mut().unwrap().encrypt_message(message);
        self.send(&encrypted).await;
    }
    pub async fn send_payload(&mut self, message: &Frame) {
        info!("{:?}", message);
        let chunks = self.payload_send.send_message(message);
        info!("Sending {} chunks", chunks.len());
        for chunk in chunks {
            self.send_securemessage(&chunk).await;
        }
    }
    pub async fn send_ukey2<T: Message>(&mut self, message: &T, message_type: Type) -> Bytes {
        self.write_half.send_ukey2(message, message_type).await
    }
    pub async fn next_offline(&mut self) -> Result<OfflineFrame, RustdropError> {
        self.reader.next_message().await
    }
    // TODO impl as a trait extension
    pub async fn next_ukey_message<T: Message + Default>(
        &mut self,
    ) -> Result<(T, Bytes), RustdropError> {
        let raw = self.reader.next().await?;
        let ukey = Ukey2Message::decode(raw.clone()).unwrap();
        let ukey_type = ukey.message_type();
        if ukey_type == Type::Alert || ukey_type == Type::UnknownDoNotUse {
            todo!();
        }
        info!("Recievd ukey2 message {:?} {:?}", ukey, ukey_type);
        Ok((
            T::decode(ukey.message_data())
                .map_err(|e| self.try_handle_ukey(e, &raw))
                .unwrap(),
            raw,
        ))
    }

    pub async fn next_payload(&mut self) -> Result<Frame, RustdropError> {
        self.payload_recv.as_mut().unwrap().get_next_payload().await
    }
    async fn start_keep_alive(&self) {
        let writer = self.write_half.clone();
        let cancel = self.app.child_token();
        self.app.tracker.spawn(repeat_keep_alive(writer, cancel));
    }
}
