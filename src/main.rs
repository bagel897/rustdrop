use crate::{core::Config, wlan::init};
mod core;
pub(crate) mod protobuf;
mod wlan;
fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::default();
    init(&config).unwrap();
}
