use tokio::signal;
use tracing::info;

use crate::{
    core::Config,
    mediums::wlan::{WlanAdvertiser, WlanClient},
    ui::SharedUiHandle,
};

pub async fn run_client(config: &Config, ui: SharedUiHandle) {
    info!("Running client");
    let mut handle = WlanClient::new(config, ui).await;
    handle.run().await;
}

pub async fn run_server(config: &Config, ui: SharedUiHandle) {
    info!("Running server");
    let mut handle = WlanAdvertiser::new(config, ui);
    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = handle.wait() => {},
    };
    handle.stop().await;
}
