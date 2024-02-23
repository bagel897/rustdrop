use bytes::Bytes;
use prost::Message;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use crate::{
    core::{
        io::{reader::ReaderRecv, writer::WriterSend},
        protocol::{payload_message::get_disconnect, repeat_keep_alive},
        ukey2::Ukey2,
        Payload, PayloadReciever, PayloadRecieverHandle, PayloadSender, RustdropError,
    },
    protobuf::{
        location::nearby::connections::OfflineFrame,
        nearby::sharing::service::Frame,
        securegcm::{ukey2_message::Type, Ukey2Alert, Ukey2Message},
    },
    Context, RustdropResult,
};

pub(super) struct StreamHandler {
    reader: ReaderRecv,
    write_half: WriterSend,
    context: Context,
    payload_recv: Option<PayloadRecieverHandle>,
    payload_send: Option<PayloadSender>,
    keep_alive: CancellationToken,
}
impl StreamHandler {
    pub fn new(reader: ReaderRecv, writer: WriterSend, context: Context) -> Self {
        StreamHandler {
            reader,
            write_half: writer,
            payload_recv: None,
            payload_send: None,
            keep_alive: CancellationToken::new(),
            context,
        }
    }
    pub async fn setup_ukey2(&mut self, ukey2_send: Ukey2, ukey2_recv: Ukey2) {
        self.start_keep_alive().await;
        let encrypted = ukey2_send.start_encrypting(self.write_half.clone(), &mut self.context);
        let decrypted = ukey2_recv.start_decrypting(self.reader.clone(), &mut self.context);
        let payload_recv = PayloadReciever::push_frames(decrypted, &mut self.context);
        self.payload_recv = Some(payload_recv);
        self.payload_send = Some(PayloadSender::new(encrypted));
    }
    pub async fn send<T: Message>(&self, message: &T) {
        self.write_half.send(message).await;
    }
    pub fn send_payload(&mut self, message: &Frame) {
        info!("Sending payload: {:?}", message);
        self.payload_send.as_mut().unwrap().send_message(message);
    }
    pub fn send_payload_raw(&mut self, raw: Bytes, id: i64) {
        self.payload_send.as_mut().unwrap().send_raw(raw, id);
    }
    pub async fn send_ukey2<T: Message>(&mut self, message: &T, message_type: Type) -> Bytes {
        self.write_half.send_ukey2(message, message_type).await
    }
    pub async fn next_offline(&mut self) -> RustdropResult<OfflineFrame> {
        self.reader.next_message().await
    }
    // TODO impl as a trait extension
    pub async fn next_ukey_message<T: Message + Default>(&mut self) -> RustdropResult<(T, Bytes)> {
        let raw = self.reader.next().await?;
        let ukey = Ukey2Message::decode(raw.clone()).unwrap();
        let ukey_type = ukey.message_type();
        if ukey_type == Type::Alert || ukey_type == Type::UnknownDoNotUse {
            Err(RustdropError::UkeyError(Ukey2Alert::decode(
                ukey.message_data(),
            )?))?;
        }
        info!("Recievd ukey2 message {:?} {:?}", ukey, ukey_type);
        Ok((
            T::decode(ukey.message_data())
                // .map_err(|e| self.try_handle_ukey(e, &raw).await)
                .unwrap(),
            raw,
        ))
    }
    pub async fn handle_payload(&mut self, frame: Frame) {
        info!("{:?}", frame);
    }
    fn pre_shutdown(&self) {
        debug!("Perparing to close connection");
        self.keep_alive.cancel();
    }
    pub async fn wait_for_disconnect(self) {
        self.pre_shutdown();
        debug!("Closing connection");
        drop(self.write_half);
        drop(self.payload_send);
        let _ = self.payload_recv.unwrap().wait_for_disconnect().await;
    }
    pub async fn next_payload_raw(&mut self) -> RustdropResult<Payload> {
        self.payload_recv.as_mut().unwrap().get_next_raw().await
    }
    pub async fn next_payload(&mut self) -> RustdropResult<Frame> {
        self.payload_recv.as_mut().unwrap().get_next_payload().await
    }
    pub fn send_disconnect(mut self) {
        self.pre_shutdown();
        self.send_encrypted(get_disconnect());
        drop(self.write_half);
        drop(self.payload_send);
    }
    fn send_encrypted(&mut self, message: OfflineFrame) {
        self.payload_send.as_mut().unwrap().send_encrypted(message)
    }
    async fn start_keep_alive(&mut self) {
        let writer = self.write_half.clone();
        let cancel = self.keep_alive.clone();
        self.context.spawn(repeat_keep_alive(writer, cancel));
    }
}
