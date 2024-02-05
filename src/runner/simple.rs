use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;

use super::{
    application::Application,
    runner::{run_client, run_server},
};
use crate::{
    mediums::ble::{scan_for_incoming, trigger_reciever},
    SimpleUI,
};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    client: bool,
}
fn init_logging() {
    let console_layer = console_subscriber::spawn();
    let fmt_layer = tracing_subscriber::fmt::layer();
    let filtered = fmt_layer.with_filter(LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(console_layer)
        .with(filtered)
        //  .with(..potential additional layer..)
        .init();
}
pub async fn run_simple() {
    init_logging();
    let args = Args::parse();
    let application: Application<SimpleUI> = Application::default();
    let child = application.child_token();
    if args.client {
        application
            .tracker
            .spawn(async { trigger_reciever(child).await.unwrap() });
        run_client(application.clone()).await;
    } else {
        application
            .tracker
            .spawn(async { scan_for_incoming(child).await.unwrap() });
        run_server(application.clone()).await;
    }
    application.shutdown().await;
}
