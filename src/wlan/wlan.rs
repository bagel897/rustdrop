use std::{
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    thread,
};

use crate::core::Config;

use super::mdns::advertise_mdns;
fn handle_connection(stream: TcpStream) {
    println!("CONN {:?}", stream);
    //...
}

pub(crate) fn init(config: &Config) -> std::io::Result<()> {
    let cfg2 = config.clone();
    let mdns_thread = thread::spawn(move || advertise_mdns(&cfg2));
    let listener = TcpListener::bind(SocketAddr::new(
        std::net::IpAddr::V4(Ipv4Addr::from_str(&config.host).unwrap()),
        config.port,
    ))?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(_e) => { /* connection failed */ }
        }
    }
    mdns_thread.join().expect("ERROR");
    Ok(())
}
