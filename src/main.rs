use std::sync::{Arc, Mutex};

use ui::SimpleUI;

use crate::{core::Config, wlan::WlanAdvertiser};
mod core;
pub(crate) mod protobuf;
mod ui;
mod wlan;
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::default();
    let ui = Arc::new(Mutex::new(SimpleUI::new()));
    let mut handle = WlanAdvertiser::new(&config, ui);
    handle.wait().await;
    handle.stop().await;
}
