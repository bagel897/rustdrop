mod common;

use std::sync::{Arc, Mutex};

use rustdrop::{run_client, run_server, Config};
use tracing_test::traced_test;

use common::testui::TestUI;

#[traced_test()]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_bidirectional() {
    let config = Config::default();
    let ui = Arc::new(Mutex::new(TestUI::new(&config)));
    let config_clone = config.clone();
    let ui_clone = ui.clone();
    let server = tokio::task::spawn(async move {
        run_server(&config_clone, ui_clone).await;
    });
    let config_clone = config.clone();
    let ui_clone = ui.clone();
    let client = tokio::task::spawn(async move {
        run_client(&config_clone, ui_clone).await;
    });
    client.await.unwrap();
    server.await.unwrap();
}
