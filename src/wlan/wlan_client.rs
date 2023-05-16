use std::{io::ErrorKind, net::SocketAddr};

use bytes::Bytes;
use prost::Message;
use tokio::net::TcpStream;
use tracing::info;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{
    core::{
        ukey2::{get_public, get_public_private, Ukey2},
        Config,
    },
    protobuf::{
        location::nearby::connections::{OfflineFrame, PairedKeyEncryptionFrame},
        securegcm::{ukey2_message::Type, Ukey2ClientFinished, Ukey2Message, Ukey2ServerInit},
        securemessage::SecureMessage,
    },
    wlan::{
        mdns::get_dests,
        wlan_common::{get_con_request, get_ukey_init},
    },
};

use super::wlan_common::StreamHandler;

pub struct WlanClient {
    stream_handler: StreamHandler,
    config: Config,
}
async fn get_stream(ip: &SocketAddr) -> TcpStream {
    let mut stream;
    let mut counter = 0;
    loop {
        stream = TcpStream::connect(ip).await;
        match stream {
            Ok(ref _s) => break,
            Err(e) => {
                if e.kind() != ErrorKind::ConnectionRefused {
                    panic!("addr: {} {}", ip, e);
                }
                info!("addr: {} {}", ip, e);
            }
        }
        if counter > 10 {
            panic!();
        }
        counter += 1;
    }
    return stream.unwrap();
}
impl WlanClient {
    pub(crate) async fn new(config: &Config) -> Self {
        let ips = get_dests();
        let ip = ips.iter().find(|ip| ip.port() == config.port).unwrap();
        let stream = get_stream(&ip).await;
        let handler = StreamHandler::new(stream);
        WlanClient {
            stream_handler: handler,
            config: config.clone(),
        }
    }
    fn get_ukey_finish(&self) -> (Ukey2ClientFinished, StaticSecret) {
        let mut res = Ukey2ClientFinished::default();
        let key = get_public_private();
        res.public_key = Some(PublicKey::from(&key).to_bytes().to_vec());
        return (res, key);
    }
    pub async fn run(&mut self) {
        let init = get_con_request(&self.config);
        let ukey_init = get_ukey_init();
        self.stream_handler.send(&init).await;
        self.stream_handler
            .send_ukey2(&ukey_init, Type::ClientInit)
            .await;
        info!("Sent messages");
        let server_resp: Ukey2ServerInit = self
            .stream_handler
            .next_ukey_message()
            .await
            .expect("Error");
        info!("Recived message {:#?}", server_resp);
        let (finish, key) = self.get_ukey_finish();
        let server_key = get_public(server_resp.public_key());
        let init_raw = Bytes::from(ukey_init.encode_to_vec());
        let resp_raw = Bytes::from(server_resp.encode_to_vec());
        let ukey2 = Ukey2::new(init_raw, key, resp_raw, server_key, true);
        self.stream_handler.setup_ukey2(ukey2);
        self.stream_handler
            .send_ukey2(&finish, Type::ClientFinish)
            .await;
        let _connection_response: OfflineFrame =
            self.stream_handler.next_message().await.expect("Error");
        info!("Recived message {:#?}", _connection_response);
        let server_resp: SecureMessage = self.stream_handler.next_message().await.expect("Error");
        info!("Recived message {:#?}", server_resp);
        let _decrypted = self
            .stream_handler
            .decrypt_message::<PairedKeyEncryptionFrame>(&server_resp);
        self.stream_handler.shutdown().await;
        info!("Shutdown");
        return;
    }
}
