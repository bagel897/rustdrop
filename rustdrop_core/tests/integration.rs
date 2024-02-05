mod common;

use common::testui::TestUI;
use rustdrop::{run_client, run_server, Application};
use tracing::info;
use tracing_test::traced_test;

#[traced_test()]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_bidirectional() {
    let app: Application<TestUI> = Application::default();
    let app_clone = app.clone();
    app.tracker.spawn(async move {
        run_server(app_clone).await;
    });
    let app_clone = app.clone();
    app.tracker.spawn(async move {
        run_client(app_clone).await;
    });
    info!("Started client and server");
    app.shutdown().await;
}
