use std::collections::HashSet;

use async_stream::stream;
use flume::{Receiver, Sender};
use futures_util::Stream;
use rustdrop::{Config, Device, DiscoveryEvent, Outgoing, Rustdrop};
use tokio::task::JoinHandle;

use crate::event_loop::runtime;

async fn run_child(rx: Receiver<(Device, Outgoing)>, send: Sender<Receiver<DiscoveryEvent>>) {
    let mut rustdrop = Rustdrop::new(Config::default()).await.unwrap();
    let discovery = rustdrop.discover().await.unwrap();
    send.send_async(discovery).await.unwrap();
    while let Ok((dev, outgoing)) = rx.recv_async().await {
        rustdrop.send_file(dev, outgoing).unwrap();
    }
    eprintln!("Shutting down daemon");
}
#[derive(Debug)]
pub struct DiscoveryHandle {
    tx: Sender<(Device, Outgoing)>,
    pub device: Device,
}
impl DiscoveryHandle {
    pub fn send(&self, outgoing: Outgoing) {
        self.tx.send((self.device.clone(), outgoing)).unwrap()
    }
}
#[derive(Debug)]
pub struct DaemonHandle {
    tx: Sender<(Device, Outgoing)>,
    rx: Receiver<DiscoveryEvent>,
    _handle: JoinHandle<()>,
}
impl Default for DaemonHandle {
    fn default() -> Self {
        let (tx, rx) = flume::bounded(1);
        let (tx_send, rx_send) = flume::unbounded();
        let handle = runtime().spawn(async move { run_child(rx_send, tx).await });
        let discovery = rx.recv().unwrap();
        Self {
            tx: tx_send,
            rx: discovery,
            _handle: handle,
        }
    }
}
impl DaemonHandle {
    pub fn recv(&self) -> impl Stream<Item = DiscoveryHandle> {
        let rx = self.rx.clone();
        let tx = self.tx.clone();
        stream! {
            let mut seen = HashSet::new();
            while let Ok(DiscoveryEvent::Discovered(dev)) = rx.recv_async().await {
                if seen.contains(&dev) {
                    continue;
                }
                seen.insert(dev.clone());
                let tx = tx.clone();
                yield DiscoveryHandle { device: dev, tx };
            }
        }
    }
}
