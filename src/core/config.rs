use portpicker::pick_unused_port;

#[derive(Clone, Copy, Debug)]
pub(crate) enum DeviceType {
    UNKNOWN,
    PHONE,
    TABLET,
    LAPTOP,
}
#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub(crate) devtype: DeviceType,
    pub(crate) port: u16,
    pub(crate) host: String,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            devtype: DeviceType::LAPTOP,
            port: pick_unused_port().expect("No available ports"),
            host: "192.168.3.143".to_string(),
        }
    }
}
