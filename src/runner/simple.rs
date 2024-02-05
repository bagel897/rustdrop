use clap::Parser;

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
    tracing_subscriber::fmt::init()
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
