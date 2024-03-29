use flume::Receiver;
use tracing::info;

use crate::{
    mediums::{bt::Bluetooth, wlan::Wlan, Medium},
    Config, Context, DiscoveryEvent, ReceiveEvent, RustdropResult,
};

use super::DiscoveringHandle;
pub struct Rustdrop {
    context: Context,
    bluetooth: Bluetooth,
    wlan: Wlan,
}
impl Rustdrop {
    pub async fn new(config: Config) -> RustdropResult<Self> {
        let context = Context::from(config);
        Ok(Self {
            wlan: Wlan::new(context.clone()),
            bluetooth: Bluetooth::new(context.clone()).await?,
            context,
        })
    }
    pub async fn start_recieving(&mut self) -> RustdropResult<Receiver<ReceiveEvent>> {
        let (tx, rx) = flume::unbounded();
        info!("Running server");
        self.wlan.start_recieving(tx.clone()).await?;
        self.bluetooth.start_recieving(tx).await?;
        Ok(rx)
    }
    pub async fn discover(&mut self) -> RustdropResult<Receiver<DiscoveryEvent>> {
        let (tx, rx) = flume::unbounded();
        let handle = DiscoveringHandle::new(self.context.clone(), tx);
        self.wlan.discover(handle.clone()).await?;
        self.bluetooth.discover(handle).await?;
        Ok(rx)
    }
    pub async fn shutdown(self) {
        self.context.shutdown().await;
    }
}
