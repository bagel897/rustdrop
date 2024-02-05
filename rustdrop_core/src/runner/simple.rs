use clap::Parser;
use tokio::signal;
use tracing::Level;
use tracing_subscriber::{filter::Targets, prelude::*};

use super::managed::Rustdrop;
use crate::SimpleUI;
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
    let mut runner: Rustdrop<SimpleUI> = Rustdrop::default();
    if args.client {
        runner.send_file().await;
    } else {
        runner.start_recieving().await;
        signal::ctrl_c().await.unwrap();
    }
    runner.shutdown().await;
}
