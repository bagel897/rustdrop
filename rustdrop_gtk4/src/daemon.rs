use flume::{Receiver, Sender};
use rustdrop::{Config, Device, DiscoveryEvent, Outgoing, Rustdrop};

pub async fn run_child(rx: Receiver<(Device, Outgoing)>, send: Sender<Receiver<DiscoveryEvent>>) {
    let mut rustdrop = Rustdrop::new(Config::default()).await.unwrap();
    let discovery = rustdrop.discover().await.unwrap();
    send.send_async(discovery).await.unwrap();
    while let Ok((dev, outgoing)) = rx.recv_async().await {
        rustdrop.send_file(dev, outgoing).unwrap();
    }
    eprintln!("Shutting down daemon");
}
