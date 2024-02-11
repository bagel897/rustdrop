use crate::event_loop::runtime;
use async_trait::async_trait;
use flume::{Receiver, RecvError, Sender};
use glib::clone;
use rustdrop::{Config, Device, IncomingText, Rustdrop, UiHandle};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
struct ChanUiHandle {
    recv: Receiver<Device>,
    send: Sender<Device>,
}
#[derive(Debug, Clone)]
pub(crate) struct Handler {
    recv: Receiver<Device>,
    send: Sender<Device>,
    cancel: CancellationToken,
}
async fn run_child(child: ChanUiHandle, token: CancellationToken) {
    let mut rustdrop = Rustdrop::from_ui(child, Config::default()).await.unwrap();
    rustdrop.send_file().await.unwrap();
    token.cancelled().await;
    eprintln!("Shutting down daemon");
}
impl Handler {
    pub fn new() -> Self {
        let (send, recv) = flume::unbounded();
        let (send_confirm, recv_confirm) = flume::unbounded();
        let cancel = CancellationToken::new();
        let child = ChanUiHandle {
            send,
            recv: recv_confirm,
        };
        runtime().spawn(clone!(@strong cancel => async move { run_child(child, cancel).await }));
        Self {
            recv,
            send: send_confirm,
            cancel,
        }
    }
    pub async fn get_device(&self) -> Result<Device, RecvError> {
        self.recv.recv_async().await
    }
    pub fn pick_dest(&self, device: Device) {
        self.send.send(device).unwrap()
    }
}
#[async_trait]
impl UiHandle for ChanUiHandle {
    async fn discovered_device(&self, device: Device) {
        self.send.send_async(device).await.unwrap()
    }
    async fn handle_text(&mut self, text: IncomingText) {
        todo!()
    }

    async fn handle_pairing_request(&mut self, request: &rustdrop::PairingRequest) -> bool {
        todo!()
    }

    async fn pick_dest(&self) -> Option<Device> {
        self.recv.recv_async().await.ok()
    }
}
