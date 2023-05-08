use std::{
    io::{self, Read, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    thread::{self, JoinHandle},
};

use super::mdns::MDNSHandle;
use crate::{
    core::{
        ukey2::{get_public_private, Ukey2},
        Config,
    },
    protobuf::{
        location::nearby::connections::ConnectionRequestFrame,
        securegcm::{Ukey2ClientFinished, Ukey2ClientInit, Ukey2HandshakeCipher, Ukey2ServerInit},
        sharing::nearby::ConnectionResponseFrame,
    },
};
use bytes::Bytes;
use pnet::datalink;
use prost::{bytes::BytesMut, decode_length_delimiter, Message};
use rand_new::thread_rng;
use rand_new::{rngs::ThreadRng, RngCore};
use tracing::{error, info, span, Level};
use x25519_dalek::{PublicKey, StaticSecret};
#[derive(Clone)]
enum StateMachine {
    Init,
    Request,
    UkeyInit {
        init: Ukey2ClientInit,
        resp: Ukey2ServerInit,
        keypair: StaticSecret,
    },
    UkeyFinish {
        ukey2: Ukey2,
    },
}
struct WlanReader {
    stream: TcpStream,
    state: StateMachine,
    rng: ThreadRng,
}

impl WlanReader {
    fn new(stream: TcpStream) -> Self {
        WlanReader {
            stream,
            state: StateMachine::Init,
            rng: thread_rng(),
        }
    }
    fn handle_con_request(&mut self, message: ConnectionRequestFrame) {
        info!("{:?}", message);
        self.state = StateMachine::Request;
    }
    fn handle_ukey2_clien_init(&mut self, message: Ukey2ClientInit) {
        info!("{:?}", message);
        self.state = StateMachine::Request;
        let mut resp = Ukey2ServerInit::default();
        let keypair = get_public_private();
        resp.version = Some(1);
        let mut resp_buf = vec![0u8; 10];
        self.rng.fill_bytes(&mut resp_buf);
        resp.random = Some(resp_buf);
        resp.handshake_cipher = Some(Ukey2HandshakeCipher::Curve25519Sha512.into());
        resp.public_key = Some(PublicKey::from(&keypair).as_bytes().to_vec());
        info!("{:?}", resp);
        self.send(&resp);
        self.state = StateMachine::UkeyInit {
            init: message,
            resp,
            keypair,
        };
    }

    fn handle_ukey2_client_finish(
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
        );
        self.state = StateMachine::UkeyFinish { ukey2 };
        self.send(&ConnectionResponseFrame::default());
    }
    fn send<T: Message>(&mut self, message: &T) {
        info!("{:?}", message);
        self.stream
            .write_all(message.encode_length_delimited_to_vec().as_slice())
            .expect("Send Error");
    }
    fn handle_message(&mut self, message_buf: Bytes) {
        match &self.state.clone() {
            StateMachine::Init => self.handle_con_request(
                ConnectionRequestFrame::decode(message_buf).expect("Decode error"),
            ),
            StateMachine::Request => self.handle_ukey2_clien_init(
                Ukey2ClientInit::decode(message_buf).expect("Decode error"),
            ),
            StateMachine::UkeyInit {
                init,
                resp,
                keypair,
            } => self.handle_ukey2_client_finish(
                Ukey2ClientFinished::decode(message_buf).expect("Decode error"),
                keypair,
                init,
                resp,
            ),
            StateMachine::UkeyFinish { ukey2 } => todo!(),
        }
    }
    fn run(&mut self) {
        let span = span!(Level::TRACE, "Handling connection");
        let _enter = span.enter();
        info!("CONN {:?}", self.stream);
        let mut buf = BytesMut::with_capacity(1000);
        let s_idx: usize = 0;
        let mut e_idx: usize;
        loop {
            let mut new_data = BytesMut::with_capacity(1000);
            self.stream.read(&mut new_data).expect("Read err");
            buf.extend_from_slice(&new_data);
            let copy: BytesMut = buf.clone();
            if let Ok(len) = decode_length_delimiter(copy) {
                info!("Reading: buf {:?}", buf);
                e_idx = s_idx + len;
                if buf.len() >= e_idx {
                    let other_buf = buf.split_to(e_idx);
                    self.handle_message(other_buf.into());
                }
            }
        }
    }
}
fn handle_connection(stream: TcpStream) {
    let mut handler = WlanReader::new(stream);
    handler.run();
}
fn run_listener(addr: IpAddr, config: &Config) -> io::Result<()> {
    let full_addr = SocketAddr::new(addr, config.port);
    let listener = TcpListener::bind(full_addr)?;
    info!("Bind: {}", full_addr);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                error!("Err: {}", e)
            }
        }
    }
    Ok(())
}
fn get_ips() -> Vec<IpAddr> {
    let mut addrs = Vec::new();
    for iface in datalink::interfaces() {
        for ip in iface.ips {
            addrs.push(ip.ip());
        }
    }
    addrs
}
pub(crate) struct WlanAdvertiser {
    tcp_threads: Vec<JoinHandle<io::Result<()>>>,
    mdns_handle: MDNSHandle,
}
impl WlanAdvertiser {
    pub(crate) fn new(config: &Config) -> Self {
        let mdns_thread = MDNSHandle::new(config);
        let mut workers = Vec::new();
        for ip in get_ips() {
            let cfg = config.clone();
            workers.push(thread::spawn(move || run_listener(ip, &cfg)));
        }
        WlanAdvertiser {
            tcp_threads: workers,
            mdns_handle: mdns_thread,
        }
    }
}
#[cfg(test)]
mod tests {
    use std::io::ErrorKind;
    use std::time::Duration;

    use tracing_test::traced_test;

    use crate::{
        protobuf::location::nearby::connections::bandwidth_upgrade_negotiation_frame::ClientIntroduction,
        wlan::mdns::get_dests,
    };

    use super::*;
    fn get_stream(ip: &SocketAddr) -> TcpStream {
        let mut stream;
        let mut counter = 0;
        loop {
            stream = TcpStream::connect(ip);
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
            thread::sleep(Duration::from_millis(100));
        }
        return stream.unwrap();
    }
    #[traced_test()]
    #[test]
    fn test_first_part() {
        let handle = WlanAdvertiser::new(&Config::default());
        let ips = get_dests();
        let ip = ips.first().unwrap();
        let mut stream = get_stream(&ip);
        let init = ClientIntroduction::default();
        let ukey_init = Ukey2ClientInit::default();
        stream
            .write_all(init.encode_length_delimited_to_vec().as_slice())
            .unwrap();
        stream
            .write_all(ukey_init.encode_length_delimited_to_vec().as_slice())
            .unwrap();
        let mut buf = BytesMut::with_capacity(1000);
        loop {
            let mut new_data = BytesMut::with_capacity(1000);
            stream.read(&mut new_data).expect("Read err");
            buf.extend_from_slice(&new_data);
            let copy: BytesMut = buf.clone();
            if let Ok(len) = decode_length_delimiter(copy) {
                info!("Reading (client): buf {:?}", buf);
                if buf.len() >= len {
                    Ukey2ServerInit::decode_length_delimited(buf).unwrap();
                    return;
                }
            }
        }
    }
}
