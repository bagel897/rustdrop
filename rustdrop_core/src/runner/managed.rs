use crate::{
    core::RustdropError,
    mediums::{
        bt::Bluetooth,
        wlan::{get_ips, start_wlan, Mdns, WlanClient},
    },
    Config, Context, Device, DiscoveryEvent, ReceiveEvent, SenderEvent,
};
use flume::Receiver;
use tracing::info;
pub struct Rustdrop {
    context: Context,
    bluetooth: Bluetooth,
    mdns: Mdns,
}
impl Rustdrop {
    async fn new(config: Config) -> Result<Self, RustdropError> {
        let context = Context::from(config);
        Ok(Self {
            mdns: Mdns::new(context.clone()),
            bluetooth: Bluetooth::new(context.clone()).await?,
            context,
        })
    }
    pub async fn start_recieving(&mut self) -> Result<Receiver<ReceiveEvent>, RustdropError> {
        let (tx, rx) = flume::unbounded();
        self.bluetooth.scan_for_incoming().await?;
        self.bluetooth.adv_bt().await?;
        // self.bluetooth.discover_bt_send(tx).await?;
        self.mdns.advertise_mdns(get_ips()).await;
        info!("Running server");
        start_wlan(&mut self.context, tx).await;
        Ok(rx)
    }
    pub async fn discover(&mut self) -> Result<Receiver<DiscoveryEvent>, RustdropError> {
        let (tx, rx) = flume::unbounded();
        self.bluetooth.trigger_reciever().await?;
        self.mdns.get_dests(tx.clone()).await;
        self.bluetooth.discover_bt_recv(tx).await?;
        Ok(rx)
    }
    pub async fn send_file(
        &mut self,
        device: Device,
    ) -> Result<Receiver<SenderEvent>, RustdropError> {
        info!("Running client");
        let (tx, rx) = flume::unbounded();
        match device.discovery {
            crate::core::protocol::Discover::Wlan(ip) => {
                WlanClient::send_to(&mut self.context, ip);
            }
            crate::core::protocol::Discover::Bluetooth(_) => todo!(),
        };
        info!("Done sending");
        Ok(rx)
    }
    pub async fn shutdown(self) {
        self.context.shutdown().await;
    }
}
