use crate::{core::Config, wlan::WlanAdvertiser};
mod core;
pub(crate) mod protobuf;
mod wlan;
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::default();
    let mut handle = WlanAdvertiser::new(&config);
    handle.wait().await;
    handle.stop().await;
}
