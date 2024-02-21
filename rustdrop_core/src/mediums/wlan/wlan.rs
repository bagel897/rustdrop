use std::{
    io::{self, ErrorKind},
    net::{IpAddr, SocketAddr},
};

use flume::Sender;
use pnet::datalink;
use tokio::net::TcpListener;
use tracing::info;

use super::{mdns::Mdns, WlanDiscovery};
use crate::{
    core::RustdropError, mediums::Medium, runner::DiscoveringHandle, Context, ReceiveEvent,
};

pub struct Wlan {
    mdns: Mdns,
    context: Context,
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
impl Wlan {
    pub fn new(context: Context) -> Self {
        Self {
            mdns: Mdns::new(context.clone()),
            context,
        }
    }
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
            let (rx, tx) = stream.into_split();
            context.spawn(
                async {
                    Self::recieve(rx, tx, child_context, events).await;
                },
                &name,
            );
        }
        Ok(())
    }
    pub async fn start_wlan(context: &mut Context, events: Sender<ReceiveEvent>) {
        let ips = get_ips();
        for ip in ips {
            let cloned = context.clone();
            let events = events.clone();
            context.spawn(
                async move {
                    Self::run_listener(ip, cloned, events)
                        .await
                        .unwrap_or_else(|_| panic!("Error on ip {}", ip));
                },
                "wlan_listener",
            );
        }
    }
}
impl Medium for Wlan {
    type Discovery = WlanDiscovery;
    async fn discover(&mut self, send: DiscoveringHandle) -> Result<(), RustdropError> {
        self.mdns.get_dests(send).await;
        Ok(())
    }

    async fn start_recieving(&mut self, send: Sender<ReceiveEvent>) -> Result<(), RustdropError> {
        self.mdns.advertise_mdns(get_ips()).await;
        Self::start_wlan(&mut self.context, send).await;
        Ok(())
    }
}
