use crate::{
    core::RustdropError,
    mediums::{
        bt::Bluetooth,
        wlan::{get_ips, start_wlan, Mdns, WlanClient},
    },
    Application, Config, UiHandle,
};
use tracing::info;
pub struct Rustdrop<U: UiHandle> {
    app: Application<U>,
    bluetooth: Bluetooth<U>,
    mdns: Mdns<U>,
}
impl<U: UiHandle> Rustdrop<U> {
    pub async fn from_ui(ui: U, config: Config) -> Result<Self, RustdropError> {
        let app = Application::new(ui, config);
        Rustdrop::new(app).await
    }
}
impl<U: UiHandle + From<Config>> Rustdrop<U> {
    pub async fn from_config(config: Config) -> Result<Self, RustdropError> {
        let app = Application::from(config);
        Rustdrop::new(app).await
    }
}
impl<U: UiHandle> Rustdrop<U> {
    async fn new(app: Application<U>) -> Result<Self, RustdropError> {
        Ok(Self {
            mdns: Mdns::new(app.clone()),
            bluetooth: Bluetooth::new(app.clone()).await?,
            app,
        })
    }
    pub async fn start_recieving(&mut self) -> Result<(), RustdropError> {
        self.bluetooth.scan_for_incoming().await?;
        self.bluetooth.adv_bt().await?;
        self.bluetooth.discover_bt_send().await?;
        self.mdns.advertise_mdns(get_ips()).await;
        info!("Running server");
        start_wlan(&mut self.app).await;
        Ok(())
    }
    pub async fn send_file(&mut self) -> Result<(), RustdropError> {
        self.bluetooth.trigger_reciever().await?;
        self.mdns.get_dests().await;
        self.bluetooth.discover_bt_recv().await?;
        info!("Running client");
        let mut handle = WlanClient::new(self.app.clone()).await;
        handle.run().await;
        Ok(())
    }
    pub async fn shutdown(self) {
        self.app.shutdown().await;
    }
}
impl<U: UiHandle + Default> Rustdrop<U> {
    pub async fn default() -> Result<Self, RustdropError> {
        let app = Application::default();
        Self::new(app).await
    }
}
