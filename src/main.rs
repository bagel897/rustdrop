use crate::{core::Config, wlan::init};

mod core;
mod wlan;
fn main() {
    println!("Hello, world!");
    init(Config::default());
}
