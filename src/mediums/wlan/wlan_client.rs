use std::{
    io::ErrorKind,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use prost::Message;
use tokio::net::TcpStream;
use tracing::info;

use super::stream_handler::StreamHandler;
use crate::{
    core::{
        protocol::{get_paired_frame, get_paired_result, Device},
        ukey2::{get_generic_pubkey, get_public, Crypto, CryptoImpl, Ukey2},
        Config,
    },
    mediums::wlan::{
        mdns::get_dests,
        wlan_common::{get_con_request, get_conn_response, get_ukey_init},
    },
    protobuf::securegcm::{ukey2_message::Type, Ukey2ClientFinished, Ukey2ServerInit},
    ui::UiHandle,
};

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
    stream.unwrap()
}
impl WlanClient {
    pub(crate) async fn new(config: &Config, ui: Arc<Mutex<dyn UiHandle>>) -> Self {
        let mut server: Option<Device> = None;
        while server.is_none() {
            info!("Looking for servers");
            let ips = tokio::task::spawn_blocking(get_dests).await.unwrap();
            server = ui.lock().unwrap().pick_dest(&ips).cloned();
        }
        let ip = server.unwrap().ip;
        let stream = get_stream(&ip).await;
        let handler = StreamHandler::new(stream, ui);
        WlanClient {
            stream_handler: handler,
            config: config.clone(),
        }
    }
    fn get_ukey_finish(&self) -> (Ukey2ClientFinished, <CryptoImpl as Crypto>::SecretKey) {
        let mut res = Ukey2ClientFinished::default();
        let key = CryptoImpl::genkey();
        res.public_key = Some(get_generic_pubkey::<CryptoImpl>(&key).encode_to_vec());
        (res, key)
    }
    async fn handle_init(&mut self) -> Bytes {
        let init = get_con_request(&self.config);
        let ukey_init = get_ukey_init();
        self.stream_handler.send(&init).await;
        let init_raw = self
            .stream_handler
            .send_ukey2(&ukey_init, Type::ClientInit)
            .await;
        info!("Sent messages");
        init_raw
    }
    async fn handle_ukey2_exchange(&mut self, init_raw: Bytes) {
        let (server_resp, resp_raw): (Ukey2ServerInit, Bytes) = self
            .stream_handler
            .next_ukey_message()
            .await
            .expect("Error");
        info!("Recived message {:#?}", server_resp);
        let (finish, key) = self.get_ukey_finish();
        let server_key = get_public::<CryptoImpl>(server_resp.public_key());
        let ukey2 = Ukey2::new(init_raw, key, resp_raw, server_key, true);
        self.stream_handler.setup_ukey2(ukey2);
        self.stream_handler
            .send_ukey2(&finish, Type::ClientFinish)
            .await;
    }
    async fn handle_pairing(&mut self) {
        let _connection_response = self.stream_handler.next_offline().await.expect("Error");
        let c_frame = get_conn_response();
        self.stream_handler.send(&c_frame).await;
        info!("Recived message {:#?}", _connection_response);
        let _server_resp = self.stream_handler.next_payload().await.expect("Error");
        let p_frame = get_paired_frame();
        self.stream_handler.send_payload(&p_frame).await;
        let _server_resp = self.stream_handler.next_payload().await.expect("Error");
        let p_res = get_paired_result();
        self.stream_handler.send_payload(&p_res).await;
    }
    pub async fn run(&mut self) {
        let init_raw = self.handle_init().await;
        self.handle_ukey2_exchange(init_raw).await;
        self.handle_pairing().await;
        self.stream_handler.shutdown().await;
        info!("Shutdown");
    }
}
