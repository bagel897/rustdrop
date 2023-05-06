use std::{
    net::{TcpListener, TcpStream},
    thread,
};

use crate::core::Config;

use super::mdns::advertise_mdns;
fn handle_connection(stream: TcpStream) {
    println!("CONN {:?}", stream);
    //...
}

pub(crate) fn init(config: Config) -> std::io::Result<()> {
    let mdns_thread = thread::spawn(move || advertise_mdns(&config));
    let host = format!("192.168.3.143:{}", config.port);
    println!("Host: {}", host);
    let listener = TcpListener::bind(host)?;
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
