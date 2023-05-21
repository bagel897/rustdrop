use portpicker::pick_unused_port;

#[derive(Clone, Copy, Debug)]
pub(crate) enum DeviceType {
    UNKNOWN,
    PHONE,
    TABLET,
    LAPTOP,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) devtype: DeviceType,
    pub(crate) port: u16,
    pub(crate) name: String,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            devtype: DeviceType::LAPTOP,
            port: pick_unused_port().expect("No available ports"),
            name: "Bagel-Mini".to_string(),
        }
    }
}
impl From<u8> for DeviceType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::UNKNOWN,
            1 => Self::PHONE,
            2 => Self::TABLET,
            3 => Self::LAPTOP,
            _ => panic!(),
        }
    }
}
