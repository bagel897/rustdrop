use std::sync::{Arc, Mutex};
use tokio_util::task::TaskTracker;

use super::runner::{run_client, run_server};
use crate::{
    ble::{scan_for_ble, trigger_reciever},
    core::Config,
    ui::SimpleUI,
};
use clap::Parser;
use tokio_util::sync::CancellationToken;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    client: bool,
}
pub async fn run_simple() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let config = Config::default();
    let ui = Arc::new(Mutex::new(SimpleUI::new()));
    let tracker = TaskTracker::new();
    let cancel = CancellationToken::new();
    let c2 = cancel.clone();
    if args.client {
        tracker.spawn(async { trigger_reciever(c2).await.unwrap() });
        run_client(&config, ui).await;
    } else {
        tracker.spawn(async { scan_for_ble(c2).await.unwrap() });
        run_server(&config, ui).await;
    }
    cancel.cancel();
    tracker.close();
    tracker.wait().await;
}
