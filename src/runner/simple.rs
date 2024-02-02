use tracing_subscriber::{prelude::*, EnvFilter};

use std::sync::{Arc, Mutex};

use clap::Parser;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use super::runner::{run_client, run_server};
use crate::{
    core::Config,
    mediums::ble::{scan_for_incoming, trigger_reciever},
    ui::SimpleUI,
};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    client: bool,
}
pub async fn run_simple() {
    let console_layer = console_subscriber::spawn();

    tracing_subscriber::registry()
        .with(console_layer)
        .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env()))
        //  .with(..potential additional layer..)
        .init();
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
        tracker.spawn(async { scan_for_incoming(c2).await.unwrap() });
        run_server(&config, ui).await;
    }
    cancel.cancel();
    tracker.close();
    tracker.wait().await;
}
