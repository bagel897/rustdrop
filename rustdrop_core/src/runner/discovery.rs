use crate::mediums::Discover;
use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap,
};

use flume::Sender;

use crate::{Context, Device, DiscoveryEvent, DiscoveryHandle};

#[derive(Clone)]
pub struct DiscoveringHandle {
    context: Context,
    send: Sender<DiscoveryEvent>,
    tx_map: HashMap<u32, Sender<Discover>>,
}
impl DiscoveringHandle {
    pub fn new(context: Context, send: Sender<DiscoveryEvent>) -> Self {
        Self {
            context,
            send,
            tx_map: HashMap::new(),
        }
    }
    pub async fn discovered(&mut self, device: Device, discovery: Discover) {
        let send = self.tx_map.entry(device.endpoint_id).or_insert_with(|| {
            let (tx, rx) = flume::unbounded();
            let handle = DiscoveryHandle::new(device, self.context.clone(), rx);
            self.send.send(DiscoveryEvent::Discovered(handle)).unwrap();
            tx
        });
        send.send_async(discovery).await.unwrap();
    }
}
