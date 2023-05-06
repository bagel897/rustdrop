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
}
impl Default for Config {
    fn default() -> Self {
        Config {
            devtype: DeviceType::LAPTOP,
            port: pick_unused_port().expect("No available ports"),
        }
    }
}
