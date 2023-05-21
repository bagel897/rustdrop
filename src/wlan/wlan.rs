use super::{mdns::MDNSHandle, wlan_server::WlanReader};
use crate::{core::Config, ui::UiHandle};
use pnet::datalink;
use std::{
    io::{self, ErrorKind},
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
};
use tokio::{
    net::TcpListener,
    select,
    task::{self, spawn_blocking, JoinHandle},
};
use tokio_util::sync::CancellationToken;
use tracing::info;
async fn run_listener(
    addr: IpAddr,
    config: &Config,
    token: CancellationToken,
    ui: Arc<Mutex<dyn UiHandle>>,
) -> io::Result<()> {
    let full_addr = SocketAddr::new(addr, config.port);
    let listener = match TcpListener::bind(full_addr).await {
        Ok(l) => l,
        Err(e) => {
            if e.kind() == ErrorKind::InvalidInput {
                return Ok(());
            }
            return Err(e);
        }
    };
    info!("Bind: {}", full_addr);
    let mut tasks = Vec::new();
    loop {
        select! {
            _ = token.cancelled() => { break;},
            Ok((stream,_addr)) = listener.accept() => {
                let ui_clone = ui.clone();
                tasks.push(task::spawn(async move { WlanReader::new(stream, ui_clone).await.run().await.unwrap();  }))},
        }
    }
    info!("Shutting down connection {}", full_addr);
    for task in tasks.iter_mut() {
        task.await.unwrap();
    }
    Ok(())
}
pub fn get_ips() -> Vec<IpAddr> {
    let mut addrs = Vec::new();
    for iface in datalink::interfaces() {
        for ip in iface.ips {
            addrs.push(ip.ip());
        }
    }
    addrs
}
pub(crate) struct WlanAdvertiser {
    tasks: Vec<JoinHandle<()>>,
    token: CancellationToken,
}
impl WlanAdvertiser {
    pub(crate) fn new(config: &Config, ui: Arc<Mutex<dyn UiHandle>>) -> Self {
        let token = CancellationToken::new();
        let mut mdns_handle = MDNSHandle::new(config, token.clone());
        let mut workers = Vec::new();
        workers.push(spawn_blocking(move || mdns_handle.run()));
        for ip in get_ips() {
            let cfg = config.clone();
            let cloned_token = token.clone();
            let clone_ui = ui.clone();
            workers.push(task::spawn(async move {
                run_listener(ip, &cfg, cloned_token, clone_ui)
                    .await
                    .expect(&format!("Error on ip {}", ip));
            }));
        }
        WlanAdvertiser {
            tasks: workers,
            token,
        }
    }
    pub(crate) async fn wait(&mut self) {
        for task in self.tasks.iter_mut() {
            task.await.unwrap();
        }
    }
    pub(crate) async fn stop(&mut self) {
        info!("Shutting down");
        self.token.cancel();
        self.wait().await;
    }
}
impl Drop for WlanAdvertiser {
    fn drop(&mut self) {
        self.token.cancel();
    }
}
#[cfg(test)]
mod tests {

    use tracing_test::traced_test;

    use crate::{ui::TestUI, wlan::wlan_client::WlanClient};

    use super::*;
    #[traced_test()]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_bidirectional() {
        let config = Config::default();
        let ui = Arc::new(Mutex::new(TestUI::new(&config)));
        let mut server = WlanAdvertiser::new(&config, ui.clone());
        let _client = WlanClient::new(&config, ui).await.run().await;
        server.stop().await;
    }
}
