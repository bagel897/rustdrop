use crate::{
    core::{
        ukey2::{get_public, get_public_private, Ukey2},
        util::{get_paired_frame, get_random},
    },
    protobuf::{
        location::nearby::connections::ConnectionRequestFrame,
        securegcm::{Ukey2ClientFinished, Ukey2ClientInit, Ukey2HandshakeCipher, Ukey2ServerInit},
        sharing::nearby::ConnectionResponseFrame,
    },
    wlan::wlan_common::StreamHandler,
};
use bytes::Bytes;
use prost::Message;

use tokio::net::TcpStream;
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
    stream_handler: StreamHandler,
    state: StateMachine,
}

impl WlanReader {
    pub async fn new(stream: TcpStream) -> Self {
        let stream_handler = StreamHandler::new(stream);
        WlanReader {
            stream_handler,
            state: StateMachine::Init,
        }
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
        self.stream_handler.send(&resp).await;
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

        let client_pub_key = get_public(message.public_key());
        let ukey2 = Ukey2::new(
            Bytes::from(init.encode_to_vec()),
            keypair.clone(),
            Bytes::from(resp.encode_to_vec()),
            client_pub_key,
            false,
        );
        self.state = StateMachine::UkeyFinish;
        self.stream_handler
            .send(&ConnectionResponseFrame::default())
            .await;
        self.stream_handler.setup_ukey2(ukey2);
        let p_key = get_paired_frame();
        self.stream_handler.send_frame(&p_key).await;
        // let payload:
        // self.send_encrypted()
    }

    async fn handle_message(&mut self, message_buf: Bytes) {
        info!("Decoding {:#X}", message_buf);
        match &self.state {
            StateMachine::Init => self.handle_con_request(
                ConnectionRequestFrame::decode(message_buf).expect("Decode error"),
            ),
            StateMachine::Request => {
                self.handle_ukey2_clien_init(
                    Ukey2ClientInit::decode(message_buf).expect("Decode error"),
                )
                .await
            }
            StateMachine::UkeyInit {
                init,
                resp,
                keypair,
            } => {
                self.handle_ukey2_client_finish(
                    Ukey2ClientFinished::decode(message_buf).expect("Decode error"),
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
    pub async fn run(&mut self) {
        let span = span!(Level::TRACE, "Handling connection");
        let _enter = span.enter();
        while let Ok(message) = self.stream_handler.next().await {
            self.handle_message(message).await;
        }
    }
}
