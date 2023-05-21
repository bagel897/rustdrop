use tracing::info;

use crate::core::{protocol::PairingRequest, Config};

use super::UiHandle;

pub struct TestUI {
    config: Config,
}
impl UiHandle for TestUI {
    fn pick_dest<'a>(
        &mut self,
        devices: &'a Vec<crate::core::protocol::Device>,
    ) -> Option<&'a crate::core::protocol::Device> {
        return devices.iter().find(|d| d.ip.port() == self.config.port);
    }
    fn handle_error(&mut self, t: String) {
        panic!("{}", t);
    }
    fn handle_pairing_request(&mut self, request: &PairingRequest) -> bool {
        info!("{:?}", request);
        return true;
    }
}
impl TestUI {
    pub fn new(config: &Config) -> Self {
        TestUI {
            config: config.clone(),
        }
    }
}
