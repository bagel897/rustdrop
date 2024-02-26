use rustdrop::{Config, DiscoveryEvent::Discovered, Rustdrop};
use tokio::{signal::ctrl_c, spawn};
use tracing::info;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    tracing_subscriber::fmt().pretty().init();
    let mut rustdrop = Rustdrop::new(Config::default()).await.unwrap();
    let discovery = rustdrop.discover().await.unwrap();
    info!("Started discovery");
    let _handle = spawn(async move {
        while let Ok(event) = discovery.recv_async().await {
            if let Discovered(handle) = event {
                info!("{:?}", handle.device());
            }
        }
    });
    ctrl_c().await.unwrap()
}
