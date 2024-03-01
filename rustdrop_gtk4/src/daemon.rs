

use async_stream::stream;
use flume::{Receiver, Sender};
use futures::Stream;
use rustdrop::{Config, DiscoveryEvent, DiscoveryHandle, Rustdrop};
use tokio::task::JoinHandle;

use crate::event_loop::runtime;

async fn run_child(send: Sender<Receiver<DiscoveryEvent>>) {
    let mut rustdrop = Rustdrop::new(Config::default()).await.unwrap();
    let discovery = rustdrop.discover().await.unwrap();
    send.send_async(discovery).await.unwrap();
    rustdrop.shutdown().await;
}
#[derive(Debug)]
pub struct DaemonHandle {
    rx: Receiver<DiscoveryEvent>,
    _handle: JoinHandle<()>,
}
impl Default for DaemonHandle {
    fn default() -> Self {
        let (tx, rx) = flume::bounded(1);
        let handle = runtime().spawn(async move { run_child(tx).await });
        let discovery = rx.recv().unwrap();
        Self {
            rx: discovery,
            _handle: handle,
        }
    }
}
impl DaemonHandle {
    pub fn recv(&self) -> impl Stream<Item = DiscoveryHandle> {
        let rx = self.rx.clone();
        stream! {
            while let Ok(DiscoveryEvent::Discovered(handle)) = rx.recv_async().await {
                yield handle;
            }
        }
    }
}
