use rustdrop::Rustdrop;
use tokio::signal;

use crate::ui::DaemonUI;

mod consts;
mod ui;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let mut rustdrop: Rustdrop<DaemonUI> = Rustdrop::default().await.unwrap();
    rustdrop.start_recieving().await.unwrap();
    signal::ctrl_c().await.unwrap();
}
