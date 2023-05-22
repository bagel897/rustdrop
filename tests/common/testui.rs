use rustdrop::{Config, Device, PairingRequest, UiHandle};
use tracing::info;

#[derive(Debug)]
pub struct TestUI {
    config: Config,
}
impl UiHandle for TestUI {
    fn pick_dest<'a>(&mut self, devices: &'a Vec<Device>) -> Option<&'a Device> {
        return devices.iter().find(|d| d.ip.port() == self.config.port);
    }
    fn handle_error(&mut self, t: String) {
        panic!("{}", t);
    }
    fn handle_pairing_request(&mut self, request: &PairingRequest) -> bool {
        info!("{:?}", request);
        true
    }
}
impl TestUI {
    pub fn new(config: &Config) -> Self {
        TestUI {
            config: config.clone(),
        }
    }
}
