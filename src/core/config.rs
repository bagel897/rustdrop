use std::{path::PathBuf, time::Duration};

use portpicker::pick_unused_port;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceType {
    Unknown,
    Phone,
    Tablet,
    Laptop,
}

#[derive(Clone, Debug)]
pub struct Mdns {
    pub poll_interval: Duration,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub devtype: DeviceType,
    pub port: u16,
    pub name: String,
    pub mdns: Mdns,
    pub dest: PathBuf,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            devtype: DeviceType::Laptop,
            port: pick_unused_port().expect("No available ports"),
            name: hostname::get().unwrap().to_str().unwrap().into(),
            mdns: Mdns {
                poll_interval: Duration::from_millis(100),
            },
            dest: dirs::download_dir().unwrap().join("nearby"),
        }
    }
}
impl From<u8> for DeviceType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::Phone,
            2 => Self::Tablet,
            3 => Self::Laptop,
            _ => panic!(),
        }
    }
}
