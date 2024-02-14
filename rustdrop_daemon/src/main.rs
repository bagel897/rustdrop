use handlers::handle_event;
use rustdrop::{Config, Rustdrop};
use tokio::signal;

mod consts;
mod handlers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::default();
    let mut rustdrop = Rustdrop::new(config).await.unwrap();
    let events = rustdrop.start_recieving().await.unwrap();
    tokio::spawn(async move {
        while let Ok(event) = events.recv_async().await {
            handle_event(event);
        }
    });
    signal::ctrl_c().await.unwrap();
}
