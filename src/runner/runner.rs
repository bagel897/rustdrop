use tokio::signal;
use tracing::info;

use crate::{
    mediums::wlan::{start_wlan, WlanClient},
    UiHandle,
};

use super::application::Application;

pub async fn run_client<U: UiHandle>(application: Application<U>) {
    info!("Running client");
    let mut handle = WlanClient::new(application).await;
    handle.run().await;
}

pub async fn run_server<U: UiHandle>(application: Application<U>) {
    info!("Running server");
    start_wlan(application.clone()).await;
    signal::ctrl_c().await.unwrap();
    application.shutdown().await;
}
