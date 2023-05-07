use std::{
    io::{self, Read, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    thread,
};

use crate::{
    core::{
        ukey2::{get_public_private, Ukey2},
        Config,
    },
    protobuf::{
        location::nearby::connections::ConnectionRequestFrame,
        securegcm::{Ukey2ClientFinished, Ukey2ClientInit, Ukey2ServerInit},
    },
};
use pnet::datalink;
use prost::{bytes::BytesMut, Message};
use ring::signature::KeyPair;

use super::mdns::advertise_mdns;
fn handle_connection(mut stream: TcpStream) {
    println!("CONN {:?}", stream);
    let mut buf = BytesMut::with_capacity(1000);
    stream.read(&mut buf).expect("Read error");
    let con_request = ConnectionRequestFrame::decode(buf).expect("Con decode error");
    let mut buf_init = BytesMut::with_capacity(1000);
    stream.read(&mut buf_init).expect("Read error");
    let ukey_init = Ukey2ClientInit::decode(buf_init).expect("Ukey decode error");
    assert!(ukey_init.version() == 1);
    println!("con request: {:?}, Ukey init {:?}", con_request, ukey_init);
    let mut resp = Ukey2ServerInit::default();
    let keypair = get_public_private().unwrap();
    resp.public_key = Some(keypair.public_key().as_ref().to_vec());
    stream
        .write(resp.encode_to_vec().as_slice())
        .expect("Send Error");

    let mut buf = BytesMut::with_capacity(1000);
    stream.read(&mut buf).expect("Read error");
    let ukey_finish = Ukey2ClientFinished::decode(buf).expect("Ukey decode error");
    println!("Ukey2 finish {:?}", ukey_finish);
    let ukey2 = Ukey2::new(
        BytesMut::from(ukey_init.encode_to_vec().as_slice()),
        keypair,
        &resp.encode_to_vec(),
        ukey_finish,
    );
    //...
}
fn run_listener(addr: IpAddr, config: &Config) -> io::Result<()> {
    let full_addr = SocketAddr::new(addr, config.port);
    let listener = TcpListener::bind(full_addr)?;
    println!("Bind: {}", full_addr);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                println!("Err: {}", e)
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
    return addrs;
}
pub(crate) fn init(config: &Config) -> std::io::Result<()> {
    let cfg2 = config.clone();
    let mdns_thread = thread::spawn(move || advertise_mdns(&cfg2));
    for ip in get_ips() {
        let cfg = config.clone();
        thread::spawn(move || run_listener(ip, &cfg));
    }

    mdns_thread.join().expect("ERROR");
    Ok(())
}
