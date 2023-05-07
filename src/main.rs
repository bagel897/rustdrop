use crate::{core::Config, wlan::init};

mod core;
pub(crate) mod protobuf;
mod wlan;
fn main() {
    println!("Hello, world!");
    let config = Config::default();
    init(&config);
}
