use crate::{
    core::ukey2::{get_public_private, Ukey2},
    protobuf::{
        location::nearby::connections::{
            v1_frame, ConnectionRequestFrame, OfflineFrame, PayloadTransferFrame, V1Frame,
        },
        securegcm::{
            DeviceToDeviceMessage, Ukey2ClientFinished, Ukey2ClientInit, Ukey2HandshakeCipher,
            Ukey2ServerInit,
        },
        securemessage::{HeaderAndBody, SecureMessage},
        sharing::nearby::{ConnectionResponseFrame, PairedKeyEncryptionFrame},
    },
    wlan::wlan_common::{get_header, get_random, send, yield_from_stream},
};
use bytes::{Bytes, BytesMut};
use prost::Message;

use futures_util::pin_mut;
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};
use tokio_stream::StreamExt;
use tracing::{info, span, Level};
use x25519_dalek::{PublicKey, StaticSecret};
enum StateMachine {
    Init,
    Request,
    UkeyInit {
        init: Ukey2ClientInit,
        resp: Ukey2ServerInit,
        keypair: StaticSecret,
    },
    UkeyFinish,
}
pub struct WlanReader {
    writer: OwnedWriteHalf,
    state: StateMachine,
    ukey2: Option<Ukey2>,
    seq: i32,
}

impl WlanReader {
    pub async fn new(stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        let mut res = WlanReader {
            writer,
            state: StateMachine::Init,
            ukey2: None,
            seq: 0,
        };
        res.run(reader).await;
        res
    }
    fn handle_con_request(&mut self, message: ConnectionRequestFrame) {
        info!("{:?}", message);
        self.state = StateMachine::Request;
    }
    async fn handle_ukey2_clien_init(&mut self, message: Ukey2ClientInit) {
        info!("{:?}", message);
        self.state = StateMachine::Request;
        let mut resp = Ukey2ServerInit::default();
        let keypair = get_public_private();
        resp.version = Some(1);
        resp.random = Some(get_random(10));
        resp.handshake_cipher = Some(Ukey2HandshakeCipher::Curve25519Sha512.into());
        resp.public_key = Some(PublicKey::from(&keypair).as_bytes().to_vec());
        info!("{:?}", resp);
        self.send(&resp).await;
        self.state = StateMachine::UkeyInit {
            init: message,
            resp,
            keypair,
        };
    }

    async fn handle_ukey2_client_finish(
        &mut self,
        message: Ukey2ClientFinished,
        keypair: &StaticSecret,
        init: &Ukey2ClientInit,
        resp: &Ukey2ServerInit,
    ) {
        info!("{:?}", message);

        let ukey2 = Ukey2::new(
            BytesMut::from(init.encode_to_vec().as_slice()),
            keypair.clone(),
            &resp.encode_to_vec(),
            message,
        )
        .expect("Encryption error");
        self.state = StateMachine::UkeyFinish;
        self.send(&ConnectionResponseFrame::default()).await;
        self.ukey2 = Some(ukey2);
        let mut p_key = PairedKeyEncryptionFrame::default();
        p_key.secret_id_hash = Some(get_random(6));
        p_key.signed_data = Some(get_random(72));
        self.send_frame(&p_key).await;
        // let payload:
        // self.send_encrypted()
    }
    async fn send<T: Message>(&mut self, message: &T) {
        info!("{:?}", message);
        send(&mut self.writer, message).await;
    }
    async fn send_frame<T: Message>(&mut self, message: &T) {
        info!("{:?}", message);
        let mut d2d = DeviceToDeviceMessage::default();
        d2d.sequence_number = Some(self.seq);
        self.seq += 1;
        d2d.message = Some(message.encode_length_delimited_to_vec());
        self.send_encrypted(&d2d).await;
    }
    async fn send_encrypted(&mut self, message: &DeviceToDeviceMessage) {
        info!("{:?}", message);
        let header = get_header();
        let ukey2 = self.ukey2.as_mut().unwrap();
        let body = ukey2.encrypt(message);
        let mut header_and_body = HeaderAndBody::default();
        header_and_body.header = header;
        header_and_body.body = body;
        let raw_hb = header_and_body.encode_length_delimited_to_vec();
        let mut msg = SecureMessage::default();
        msg.signature = ukey2.sign(&raw_hb);
        msg.header_and_body = raw_hb;
        self.send(&msg).await;
    }

    async fn handle_message(&mut self, message_buf: Bytes) {
        info!("Decoding {:#X}", message_buf);
        match &self.state {
            StateMachine::Init => self.handle_con_request(
                ConnectionRequestFrame::decode_length_delimited(message_buf).expect("Decode error"),
            ),
            StateMachine::Request => {
                self.handle_ukey2_clien_init(
                    Ukey2ClientInit::decode_length_delimited(message_buf).expect("Decode error"),
                )
                .await
            }
            StateMachine::UkeyInit {
                init,
                resp,
                keypair,
            } => {
                self.handle_ukey2_client_finish(
                    Ukey2ClientFinished::decode_length_delimited(message_buf)
                        .expect("Decode error"),
                    &keypair.clone(),
                    &init.clone(),
                    &resp.clone(),
                )
                .await
            }
            StateMachine::UkeyFinish => todo!(),
        }
        info!("Handled message");
    }
    async fn run(&mut self, mut reader: OwnedReadHalf) {
        let span = span!(Level::TRACE, "Handling connection");
        let _enter = span.enter();
        info!("CONN {:?}", reader);
        let stream = yield_from_stream(&mut reader);
        pin_mut!(stream);
        while let Some(message) = stream.next().await {
            self.handle_message(message).await;
        }
    }
}
