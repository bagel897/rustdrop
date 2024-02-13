use std::{
    io::{self, ErrorKind},
    net::{IpAddr, SocketAddr},
};

use pnet::datalink;
use tokio::{net::TcpListener, select};
use tracing::info;

use crate::Context;

use super::wlan_server::WlanReader;
async fn run_listener(addr: IpAddr, mut context: Context) -> io::Result<()> {
    let full_addr = SocketAddr::new(addr, context.config.port);
    let listener = match TcpListener::bind(full_addr).await {
        Ok(l) => l,
        Err(e) => {
            if e.kind() == ErrorKind::InvalidInput {
                return Ok(());
            }
            return Err(e);
        }
    };
    info!("Bind: {}", full_addr);
    let token = context.child_token();
    loop {
        select! {
            _ = token.cancelled() => { break;},
            Ok((stream,addr)) = listener.accept() => {
                let name = format!("Handle {}",addr);
                let context = context.clone();
                context.spawn(async { WlanReader::new(stream, context).await.run().await.unwrap();  },&name );}
        }
    }
    Ok(())
}
pub fn get_ips() -> Vec<IpAddr> {
    let mut addrs = Vec::new();
    for iface in datalink::interfaces() {
        for ip in iface.ips {
            addrs.push(ip.ip());
        }
    }
    addrs
}
pub async fn start_wlan(context: &mut Context) {
    let ips = get_ips();
    for ip in ips {
        let cloned = context.clone();
        context.spawn(
            async move {
                run_listener(ip, cloned)
                    .await
                    .unwrap_or_else(|_| panic!("Error on ip {}", ip));
            },
            "wlan_listener",
        );
    }
}
