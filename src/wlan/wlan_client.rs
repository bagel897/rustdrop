use std::{io::ErrorKind, net::SocketAddr};

use futures_util::pin_mut;
use prost::Message;
use tokio::{
    io::AsyncWriteExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};
use tokio_stream::StreamExt;
use tracing::info;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{
    core::{ukey2::get_public_private, Config},
    protobuf::{
        location::nearby::connections::ConnectionRequestFrame,
        securegcm::{
            ukey2_client_init::CipherCommitment, Ukey2ClientFinished, Ukey2ClientInit,
            Ukey2HandshakeCipher, Ukey2ServerInit,
        },
    },
    wlan::{
        mdns::get_dests,
        wlan_common::{get_random, send, yield_from_stream},
    },
};

pub struct WlanClient {
    writer: OwnedWriteHalf,
    config: Config,
}
async fn get_stream(ip: &SocketAddr) -> TcpStream {
    let mut stream;
    let mut counter = 0;
    loop {
        stream = TcpStream::connect(ip).await;
        match stream {
            Ok(ref s) => break,
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
        let (reader, writer) = stream.into_split();
        let mut res = WlanClient {
            writer,
            config: config.clone(),
        };
        res.run(reader).await;
        res
    }
    async fn send<T: Message>(&mut self, message: &T) {
        info!("{:?}", message);
        send(&mut self.writer, message).await;
    }
    fn get_con_request(&self) -> ConnectionRequestFrame {
        let mut init = ConnectionRequestFrame::default();
        // init.endpoint_id = Some(self.config.name.to_string());
        init.endpoint_name = Some(self.config.name.to_string());
        return init;
    }
    fn get_ukey_init(&self) -> Ukey2ClientInit {
        let mut ukey_init = Ukey2ClientInit::default();
        ukey_init.version = Some(1);
        ukey_init.random = Some(get_random(10));
        let mut cipher = CipherCommitment::default();
        cipher.handshake_cipher = Some(Ukey2HandshakeCipher::Curve25519Sha512.into());
        ukey_init.cipher_commitments = vec![cipher];
        return ukey_init;
    }
    fn get_ukey_finish(&self) -> (Ukey2ClientFinished, StaticSecret) {
        let mut res = Ukey2ClientFinished::default();
        let key = get_public_private();
        res.public_key = Some(PublicKey::from(&key).to_bytes().to_vec());
        return (res, key);
    }
    pub async fn run(&mut self, mut reader: OwnedReadHalf) {
        let init = self.get_con_request();
        let ukey_init = self.get_ukey_init();
        self.send(&init).await;
        self.send(&ukey_init).await;
        info!("Sent messages");
        let stream = yield_from_stream(&mut reader);
        pin_mut!(stream);
        let message = stream.next().await.expect("Error");
        info!("Recived message {:#X}", message);
        Ukey2ServerInit::decode_length_delimited(message).unwrap();
        let (finish, key) = self.get_ukey_finish();
        self.send(&finish).await;
        self.writer.shutdown().await.unwrap();
        info!("Shutdown");
        return;
    }
}
