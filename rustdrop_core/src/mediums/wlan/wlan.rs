use std::{
    io::{self, ErrorKind},
    net::{IpAddr, SocketAddr},
};

use flume::Sender;
use pnet::datalink;
use tokio::net::TcpListener;
use tracing::info;

use crate::{Context, ReceiveEvent};

use super::wlan_server::WlanReader;
async fn run_listener(
    addr: IpAddr,
    mut context: Context,
    events: Sender<ReceiveEvent>,
) -> io::Result<()> {
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
    while let Ok((stream, addr)) = listener.accept().await {
        let name = format!("Handle {}", addr);
        let child_context = context.clone();
        let events = events.clone();
        context.spawn(
            async {
                WlanReader::new(stream, child_context, events)
                    .await
                    .run()
                    .await
                    .unwrap();
            },
            &name,
        );
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
pub async fn start_wlan(context: &mut Context, events: Sender<ReceiveEvent>) {
    let ips = get_ips();
    for ip in ips {
        let cloned = context.clone();
        let events = events.clone();
        context.spawn(
            async move {
                run_listener(ip, cloned, events)
                    .await
                    .unwrap_or_else(|_| panic!("Error on ip {}", ip));
            },
            "wlan_listener",
        );
    }
}
