use std::sync::{Arc, Mutex};

use clap::Parser;

use crate::{
    core::Config,
    ui::SimpleUI,
    wlan::{WlanAdvertiser, WlanClient},
};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    client: bool,
}
async fn run_client() {
    let config = Config::default();
    let ui = Arc::new(Mutex::new(SimpleUI::new()));
    let mut handle = WlanClient::new(&config, ui).await;
    handle.run().await;
}

async fn run_server() {
    let config = Config::default();
    let ui = Arc::new(Mutex::new(SimpleUI::new()));
    let mut handle = WlanAdvertiser::new(&config, ui);
    handle.wait().await;
    handle.stop().await;
}
pub async fn run_simple() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    if args.client {
        run_client().await;
    } else {
        run_server().await;
    }
}
