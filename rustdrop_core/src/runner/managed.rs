use crate::{
    core::RustdropError,
    mediums::{
        ble::{scan_for_incoming, trigger_reciever},
        bt::Bluetooth,
        wlan::{start_wlan, WlanClient},
    },
    Application, Config, UiHandle,
};
use tracing::info;
pub struct Rustdrop<U: UiHandle> {
    app: Application<U>,
    bluetooth: Bluetooth<U>,
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
            bluetooth: Bluetooth::new(app.clone()).await?,
            app,
        })
    }
    pub async fn start_recieving(&mut self) -> Result<(), RustdropError> {
        scan_for_incoming(&mut self.app).await.unwrap();
        trigger_reciever(&mut self.app).await.unwrap();
        self.bluetooth.adv_bt().await?;
        info!("Running server");
        start_wlan(&mut self.app).await;
        Ok(())
    }
    pub async fn send_file(&mut self) {
        trigger_reciever(&mut self.app).await.unwrap();
        info!("Running client");
        let mut handle = WlanClient::new(self.app.clone()).await;
        handle.run().await;
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
