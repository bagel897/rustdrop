use std::{
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use flume::Sender;
use tokio::net::TcpListener;
use tracing::{info, span, Level};

use super::{mdns::Mdns, WlanDiscovery};
use crate::{
    core::RustdropError, mediums::Medium, runner::DiscoveringHandle, Context, ReceiveEvent,
};

pub struct Wlan {
    mdns: Mdns,
    context: Context,
}
impl Wlan {
    pub fn new(context: Context) -> Self {
        Self {
            mdns: Mdns::new(context.clone()),
            context,
        }
    }
    async fn run_listener(
        &self,
        addr: IpAddr,
        events: Sender<ReceiveEvent>,
    ) -> Result<(), RustdropError> {
        let full_addr = SocketAddr::new(addr, 0);
        let listener = match TcpListener::bind(full_addr).await {
            Ok(l) => l,
            Err(e) => {
                if e.kind() == ErrorKind::InvalidInput {
                    return Ok(());
                }
                return Err(e.into());
            }
        };
        let addr = listener.local_addr()?;
        info!("Bind: {}", addr);
        let child = self.context.clone();
        self.context.spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                let span = span!(Level::INFO, "Handle", addr = format!("{}", addr));
                let child_context = child.clone();
                let events = events.clone();
                let (rx, tx) = stream.into_split();
                child.spawn(async {
                    Self::recieve(rx, tx, child_context, events).await;
                    drop(span)
                });
            }
        });
        self.mdns.advertise_mdns(vec![addr.ip()], addr.port()).await;
        Ok(())
    }
    pub async fn start_wlan(&self, events: Sender<ReceiveEvent>) -> Result<(), RustdropError> {
        let events = events.clone();
        self.run_listener(Ipv4Addr::new(0, 0, 0, 0).into(), events)
            .await
    }
}
impl Medium for Wlan {
    type Discovery = WlanDiscovery;
    async fn discover(&mut self, send: DiscoveringHandle) -> Result<(), RustdropError> {
        self.mdns.get_dests(send).await;
        Ok(())
    }

    async fn start_recieving(&mut self, send: Sender<ReceiveEvent>) -> Result<(), RustdropError> {
        self.start_wlan(send).await?;
        Ok(())
    }
}
