mod common;

use common::testui::TestUI;
use rustdrop::{run_client, run_server, Context};
use tracing::info;
use tracing_test::traced_test;

#[traced_test()]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_bidirectional() {
    let context: Context<TestUI> = Context::default();
    let context_clone = context.clone();
    context.tracker.spawn(async move {
        run_server(context_clone).await;
    });
    let context_clone = context.clone();
    context.tracker.spawn(async move {
        run_client(context_clone).await;
    });
    info!("Started client and server");
    context.shutdown().await;
}
