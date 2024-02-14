use bytes::Bytes;
use prost::Message;
use tokio::net::TcpStream;
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
        securegcm::{ukey2_message::Type, Ukey2Message},
        sharing::nearby::Frame,
    },
    Context,
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
    pub fn new(stream: TcpStream, mut context: Context) -> Self {
        let (read_half, write_half) = stream.into_split();
        StreamHandler {
            reader: ReaderRecv::new(read_half, &mut context),
            write_half: WriterSend::new(write_half, &mut context),
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
                // .map_err(|e| self.try_handle_ukey(e, &raw).await)
                .unwrap(),
            raw,
        ))
    }
    pub async fn handle_payload(&mut self, frame: Frame) {
        info!("{:?}", frame);
    }
    pub async fn wait_for_disconnect(self) {
        debug!("Closing connection");
        self.keep_alive.cancel();
        drop(self.write_half);
        drop(self.payload_send);
        let _ = self.payload_recv.unwrap().wait_for_disconnect().await;
    }
    pub async fn next_payload_raw(&mut self) -> Result<Payload, RustdropError> {
        self.payload_recv.as_mut().unwrap().get_next_raw().await
    }
    pub async fn next_payload(&mut self) -> Result<Frame, RustdropError> {
        self.payload_recv.as_mut().unwrap().get_next_payload().await
    }
    pub fn send_disconnect(&mut self) {
        self.send_encrypted(get_disconnect());
    }
    fn send_encrypted(&mut self, message: OfflineFrame) {
        self.payload_send
            .as_mut()
            .unwrap()
            .send_unencrypted(message)
    }
    async fn start_keep_alive(&mut self) {
        let writer = self.write_half.clone();
        let cancel = self.keep_alive.clone();
        self.context
            .spawn(repeat_keep_alive(writer, cancel), "keep-alive");
    }
}
