use crate::{core::Config, wlan::init};
use tracing::subscriber::set_global_default;
mod core;
pub(crate) mod protobuf;
mod wlan;
fn main() {
    let tracing_subscriber = tracing_subscriber::fmt::init();
    let subscriber = tracing_subscriber::fmt().finish();
    set_global_default(subscriber);
    let config = Config::default();
    init(&config);
}
