use clap::Parser;
use tracing::Level;
use tracing_subscriber::{filter::Targets, prelude::*};

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
    let targets = Targets::new().with_target("rustdrop", Level::DEBUG);
    let console_layer = console_subscriber::spawn();
    let fmt_layer = tracing_subscriber::fmt::layer();
    let filtered = fmt_layer.with_filter(targets);
    tracing_subscriber::registry()
        .with(console_layer)
        .with(filtered)
        //  .with(..potential additional layer..)
        .init();
}
pub async fn run_simple() {
    init_logging();
    let args = Args::parse();
    let mut application: Application<SimpleUI> = Application::default();
    let child = application.child_token();
    if args.client {
        application.spawn(async { trigger_reciever(child).await.unwrap() }, "ble_adv");
        run_client(&mut application).await;
    } else {
        application.spawn(
            async { scan_for_incoming(child).await.unwrap() },
            "ble_scan",
        );
        run_server(&mut application).await;
    }
    application.shutdown().await;
}
