use flume::Sender;

use crate::{Context, Device, DiscoveryEvent, DiscoveryHandle};

#[derive(Clone)]
pub struct DiscoveringHandle {
    context: Context,
    send: Sender<DiscoveryEvent>,
}
impl DiscoveringHandle {
    pub fn new(context: Context, send: Sender<DiscoveryEvent>) -> Self {
        Self { context, send }
    }
    pub async fn discovered(&self, device: Device) {
        let handle = DiscoveryHandle::new(device, self.context.clone());
        self.send
            .send_async(DiscoveryEvent::Discovered(handle))
            .await
            .unwrap()
    }
}
