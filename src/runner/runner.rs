use crate::{
    core::Config,
    ui::SharedUiHandle,
    wlan::{WlanAdvertiser, WlanClient},
};

pub async fn run_client(config: &Config, ui: SharedUiHandle) {
    let mut handle = WlanClient::new(&config, ui).await;
    handle.run().await;
}

pub async fn run_server(config: &Config, ui: SharedUiHandle) {
    let mut handle = WlanAdvertiser::new(&config, ui);
    handle.wait().await;
    handle.stop().await;
}
