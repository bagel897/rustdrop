use std::fmt::Debug;

use bytes::Bytes;
use prost::{DecodeError, Message};
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::{
    core::{
        io::{reader::ReaderRecv, writer::WriterSend},
        protocol::{repeat_keep_alive, try_decode_ukey2_alert},
        ukey2::Ukey2,
        util::ukey_alert_to_str,
        Payload, PayloadReciever, PayloadRecieverHandle, PayloadSender, RustdropError,
    },
    protobuf::{
        location::nearby::connections::{DisconnectionFrame, OfflineFrame},
        securegcm::{ukey2_message::Type, Ukey2Message},
        sharing::nearby::Frame,
    },
    runner::application::Application,
    ui::UiHandle,
};

pub(super) struct StreamHandler<U: UiHandle> {
    reader: ReaderRecv,
    write_half: WriterSend,
    app: Application<U>,
    payload_recv: Option<PayloadRecieverHandle>,
    payload_send: Option<PayloadSender>,
    keep_alive: CancellationToken,
}
impl<U: UiHandle> StreamHandler<U> {
    pub fn new(stream: TcpStream, mut app: Application<U>) -> Self {
        let (read_half, write_half) = stream.into_split();
        StreamHandler {
            reader: ReaderRecv::new(read_half, &mut app),
            write_half: WriterSend::new(write_half, &mut app),
            payload_recv: None,
            payload_send: None,
            keep_alive: app.child_token(),
            app,
        }
    }
    pub async fn setup_ukey2(&mut self, ukey2_send: Ukey2, ukey2_recv: Ukey2) {
        self.start_keep_alive().await;
        let encrypted = ukey2_send.start_encrypting(self.write_half.clone(), &mut self.app);
        let decrypted = ukey2_recv.start_decrypting(self.reader.clone(), &mut self.app);
        let payload_recv = PayloadReciever::push_frames(decrypted, &mut self.app);
        self.payload_recv = Some(payload_recv);
        self.payload_send = Some(PayloadSender::new(encrypted));
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
    pub async fn send_payload(&mut self, message: &Frame) {
        info!("Sending payload: {:?}", message);
        self.payload_send.as_mut().unwrap().send_message(message);
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
    pub async fn handle_payload(&mut self, frame: Frame) {
        info!("{:?}", frame);
    }
    pub async fn wait_for_disconnect(self) -> Result<DisconnectionFrame, RustdropError> {
        self.keep_alive.cancel();
        self.payload_recv.unwrap().wait_for_disconnect().await
    }
    pub async fn next_payload_raw(&mut self) -> Result<Payload, RustdropError> {
        self.payload_recv.as_mut().unwrap().get_next_raw().await
    }
    pub async fn next_payload(&mut self) -> Result<Frame, RustdropError> {
        self.payload_recv.as_mut().unwrap().get_next_payload().await
    }
    async fn start_keep_alive(&mut self) {
        let writer = self.write_half.clone();
        let cancel = self.keep_alive.clone();
        self.app
            .spawn(repeat_keep_alive(writer, cancel), "keep-alive");
    }
}
