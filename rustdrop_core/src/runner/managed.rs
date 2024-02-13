use crate::{
    core::RustdropError,
    mediums::{
        bt::Bluetooth,
        wlan::{get_ips, start_wlan, Mdns, WlanClient},
    },
    Config, Context, Device,
};
use flume::Receiver;
use tracing::info;
pub struct Rustdrop {
    context: Context,
    bluetooth: Bluetooth,
    mdns: Mdns,
}
impl Rustdrop {
    pub async fn from_ui(config: Config) -> Result<Self, RustdropError> {
        let context = Context::new(config);
        Rustdrop::new(context).await
    }
}
impl Rustdrop {
    pub async fn from_config(config: Config) -> Result<Self, RustdropError> {
        let context = Context::from(config);
        Rustdrop::new(context).await
    }
}
impl Rustdrop {
    async fn new(context: Context) -> Result<Self, RustdropError> {
        Ok(Self {
            mdns: Mdns::new(context.clone()),
            bluetooth: Bluetooth::new(context.clone()).await?,
            context,
        })
    }
    pub async fn start_recieving(&mut self) -> Result<(), RustdropError> {
        self.bluetooth.scan_for_incoming().await?;
        self.bluetooth.adv_bt().await?;
        self.bluetooth.discover_bt_send().await?;
        self.mdns.advertise_mdns(get_ips()).await;
        info!("Running server");
        start_wlan(&mut self.context).await;
        Ok(())
    }
    pub async fn send_file(&mut self) -> Result<Receiver<Device>, RustdropError> {
        self.bluetooth.trigger_reciever().await?;
        self.mdns.get_dests().await;
        self.bluetooth.discover_bt_recv().await?;
        let ui = self.context.ui_ref();
        while let Some(dest) = ui.read().await.pick_dest().await {
            info!("Running client");
            match dest.discovery {
                crate::core::protocol::Discover::Wlan(ip) => {
                    WlanClient::send_to(&mut self.context, ip);
                }
                crate::core::protocol::Discover::Bluetooth(_) => todo!(),
            };
        }

        info!("Done sending");
        Ok(())
    }
    pub async fn shutdown(self) {
        self.context.shutdown().await;
    }
}
impl Default for Rustdrop {
    async fn default() -> Result<Self, RustdropError> {
        let context = Context::default();
        Self::new(context).await
    }
}
