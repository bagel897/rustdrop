use std::{io::ErrorKind, net::SocketAddr};

use bytes::Bytes;
use openssl::{ec::EcKey, pkey::Private};
use tokio::net::TcpStream;
use tracing::info;

use super::stream_handler::StreamHandler;
use crate::{
    core::{
        protocol::{get_paired_frame, get_paired_result, Device},
        ukey2::{get_public, Crypto, CryptoImpl, Ukey2},
    },
    mediums::wlan::{
        mdns::get_dests,
        wlan_common::{get_con_request, get_conn_response, get_ukey_init_finish},
    },
    protobuf::securegcm::{ukey2_message::Type, Ukey2Message, Ukey2ServerInit},
    runner::application::Application,
    ui::UiHandle,
};

pub struct WlanClient<U: UiHandle> {
    stream_handler: StreamHandler<U>,
    application: Application<U>,
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
impl<U: UiHandle> WlanClient<U> {
    pub(crate) async fn new(application: Application<U>) -> Self {
        let mut server: Option<Device> = None;
        while server.is_none() {
            info!("Looking for servers");
            let ips = tokio::task::spawn_blocking(get_dests).await.unwrap().await;
            server = application.ui().unwrap().pick_dest(&ips).cloned();
        }
        let ip = server.unwrap().ip;
        let stream = get_stream(&ip).await;
        let handler = StreamHandler::new(stream, application.clone());
        WlanClient {
            stream_handler: handler,
            application,
        }
    }
    async fn handle_init(&mut self) -> (Bytes, Ukey2Message, <CryptoImpl as Crypto>::SecretKey) {
        let init = get_con_request(&self.application.config);
        let (ukey_init, finish, key) = get_ukey_init_finish();
        self.stream_handler.send(&init).await;
        let init_raw = self
            .stream_handler
            .send_ukey2(&ukey_init, Type::ClientInit)
            .await;
        info!("Sent messages");
        (init_raw, finish, key)
    }
    async fn handle_ukey2_exchange(
        &mut self,
        init_raw: Bytes,
        finish: Ukey2Message,
        key: EcKey<Private>,
    ) {
        let (server_resp, resp_raw): (Ukey2ServerInit, Bytes) = self
            .stream_handler
            .next_ukey_message()
            .await
            .expect("Error");
        info!("Recived message {:#?}", server_resp);
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
        let (init_raw, finish, key) = self.handle_init().await;
        self.handle_ukey2_exchange(init_raw, finish, key).await;
        self.handle_pairing().await;
    }
}
