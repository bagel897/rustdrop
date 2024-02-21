use flume::Receiver;
use tokio::runtime::Handle;
use tracing::{error, info};

use crate::{
    core::RustdropError,
    mediums::{Discover, Discovery},
    Context, Device, Outgoing, SenderEvent,
};

#[derive(Debug)]
pub struct DiscoveryHandle {
    device: Device,
    context: Context,
}
impl DiscoveryHandle {
    pub fn new(device: Device, context: Context) -> Self {
        Self { device, context }
    }
    pub fn send_file(
        &self,
        outgoing: Outgoing,
        handle: &Handle,
    ) -> Result<Receiver<SenderEvent>, RustdropError> {
        info!("Running client");
        let (tx, rx) = flume::unbounded();
        let cloned = self.context.clone();
        let device = self.device.clone();
        self.context.spawn_on(
            async move {
                let res = match device.discovery {
                    Discover::Wlan(discovery) => discovery.send_to(cloned, outgoing, tx).await,
                    Discover::Bluetooth(discovery) => discovery.send_to(cloned, outgoing, tx).await,
                };
                if let Err(e) = res {
                    error!("{}", e);
                }
            },
            handle,
        );
        info!("Done sending");
        Ok(rx)
    }
    pub fn device(&self) -> &Device {
        &self.device
    }
}
