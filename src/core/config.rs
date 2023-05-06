pub(crate) enum DeviceType {
    UNKNOWN,
    PHONE,
    TABLET,
    LAPTOP,
}
pub(crate) struct Config {
    pub(crate) devtype: DeviceType,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            devtype: DeviceType::LAPTOP,
        }
    }
}
