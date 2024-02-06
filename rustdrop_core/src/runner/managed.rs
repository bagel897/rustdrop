use crate::{
    mediums::{
        ble::{scan_for_incoming, trigger_reciever},
        bt::advertise_recv::adv_bt,
        wlan::{start_wlan, WlanClient},
    },
    Application, Config, UiHandle,
};
use tracing::info;
pub struct Rustdrop<U: UiHandle> {
    app: Application<U>,
}
impl<U: UiHandle + From<Config>> Rustdrop<U> {
    pub fn new(config: Config) -> Self {
        Self {
            app: Application::from(config),
        }
    }
}
impl<U: UiHandle> Rustdrop<U> {
    pub async fn start_recieving(&mut self) {
        scan_for_incoming(&mut self.app).await.unwrap();
        trigger_reciever(&mut self.app).await.unwrap();
        adv_bt(&mut self.app).await.unwrap();
        info!("Running server");
        start_wlan(&mut self.app).await;
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
impl<U: UiHandle + Default> Default for Rustdrop<U> {
    fn default() -> Self {
        Self {
            app: Application::default(),
        }
    }
}
