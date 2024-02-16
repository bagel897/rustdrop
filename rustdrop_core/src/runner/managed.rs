use flume::Receiver;
use tracing::{error, info};

use crate::{
    core::RustdropError,
    mediums::{bt::Bluetooth, wlan::Wlan, Discover, Discovery, Medium},
    Config, Context, Device, DiscoveryEvent, Outgoing, ReceiveEvent, SenderEvent,
};
pub struct Rustdrop {
    context: Context,
    bluetooth: Bluetooth,
    wlan: Wlan,
}
impl Rustdrop {
    pub async fn new(config: Config) -> Result<Self, RustdropError> {
        let context = Context::from(config);
        Ok(Self {
            wlan: Wlan::new(context.clone()),
            bluetooth: Bluetooth::new(context.clone()).await?,
            context,
        })
    }
    pub async fn start_recieving(&mut self) -> Result<Receiver<ReceiveEvent>, RustdropError> {
        let (tx, rx) = flume::unbounded();
        info!("Running server");
        self.bluetooth.scan_for_incoming().await?;
        self.bluetooth.adv_bt().await?;
        // self.bluetooth.discover_bt_send(tx).await?;
        self.wlan.start_recieving(tx.clone()).await?;
        Ok(rx)
    }
    pub async fn discover(&mut self) -> Result<Receiver<DiscoveryEvent>, RustdropError> {
        let (tx, rx) = flume::unbounded();
        self.bluetooth.trigger_reciever().await?;
        self.wlan.discover(tx.clone()).await?;
        self.bluetooth.discover_bt_recv(tx).await?;
        Ok(rx)
    }
    pub fn send_file(
        &mut self,
        device: Device,
        outgoing: Outgoing,
    ) -> Result<Receiver<SenderEvent>, RustdropError> {
        info!("Running client");
        let (tx, rx) = flume::unbounded();
        let cloned = self.context.clone();
        self.context.spawn(
            async move {
                let res = match device.discovery {
                    Discover::Wlan(discovery) => discovery.send_to(cloned, outgoing, tx).await,
                    Discover::Bluetooth(discovery) => discovery.send_to(cloned, outgoing, tx).await,
                };
                if let Err(e) = res {
                    error!("{}", e);
                }
            },
            "Sending",
        );
        info!("Done sending");
        Ok(rx)
    }
    pub async fn shutdown(self) {
        self.context.shutdown().await;
    }
}
