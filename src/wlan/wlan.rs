use std::{
    io,
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    thread,
};

use pnet::datalink;

use crate::core::Config;

use super::mdns::advertise_mdns;
fn handle_connection(stream: TcpStream) {
    println!("CONN {:?}", stream);
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
