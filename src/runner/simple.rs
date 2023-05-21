use std::sync::{Arc, Mutex};

use super::runner::{run_client, run_server};
use crate::{core::Config, ui::SimpleUI};
use clap::Parser;
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
    if args.client {
        run_client(&config, ui).await;
    } else {
        run_server(&config, ui).await;
    }
}
