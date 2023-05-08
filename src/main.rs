use std::{thread, time::Duration};

use crate::{core::Config, wlan::WlanAdvertiser};
mod core;
pub(crate) mod protobuf;
mod wlan;
fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::default();
    let _handle = WlanAdvertiser::new(&config);
    loop {
        thread::sleep(Duration::default());
    }
}
