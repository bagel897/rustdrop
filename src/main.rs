use runner::simple::run_simple;

mod core;
pub(crate) mod protobuf;
mod runner;
mod ui;
mod wlan;
#[tokio::main]
async fn main() {
    run_simple().await;
}
