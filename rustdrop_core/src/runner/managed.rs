use crate::{
    mediums::{
        ble::{scan_for_incoming, trigger_reciever},
        wlan::{start_wlan, WlanClient},
    },
    Application, Config, UiHandle,
};
use tracing::{error, info};
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
        let child = self.app.child_token();
        self.app.spawn(
            async {
                let r = scan_for_incoming(child).await;
                if let Err(e) = r {
                    error!(e);
                }
            },
            "ble_scan",
        );
        info!("Running server");
        start_wlan(&mut self.app).await;
    }
    pub async fn send_file(&mut self) {
        let child = self.app.child_token();
        self.app
            .spawn(async { trigger_reciever(child).await.unwrap() }, "ble_adv");
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
