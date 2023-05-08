use super::{mdns::MDNSHandle, wlan_server::WlanReader};
use crate::core::Config;
use pnet::datalink;
use std::{
    io,
    net::{IpAddr, SocketAddr},
};
use tokio::{
    net::TcpListener,
    select,
    task::{self, JoinHandle},
};
use tokio_util::sync::CancellationToken;
use tracing::info;
async fn run_listener(addr: IpAddr, config: &Config, token: CancellationToken) -> io::Result<()> {
    let full_addr = SocketAddr::new(addr, config.port);
    let listener = TcpListener::bind(full_addr).await?;
    info!("Bind: {}", full_addr);
    let mut tasks = Vec::new();
    loop {
        select! {
            _ = token.cancelled() => { break;},
            Ok((stream,_addr)) = listener.accept() => {tasks.push(task::spawn(async move { WlanReader::new(stream).await }))},
        }
    }
    info!("Shutting down connection {}", full_addr);
    for task in tasks.iter_mut() {
        task.await.unwrap();
    }
    Ok(())
}
fn get_ips() -> Vec<IpAddr> {
    let mut addrs = Vec::new();
    for iface in datalink::interfaces() {
        for ip in iface.ips {
            addrs.push(ip.ip());
        }
    }
    addrs
}
pub(crate) struct WlanAdvertiser {
    tcp_threads: Vec<JoinHandle<()>>,
    token: CancellationToken,
    mdns_handle: MDNSHandle,
}
impl WlanAdvertiser {
    pub(crate) fn new(config: &Config) -> Self {
        let token = CancellationToken::new();
        let mdns_thread = MDNSHandle::new(config);
        let mut workers = Vec::new();
        for ip in get_ips() {
            let cfg = config.clone();
            let cloned_token = token.clone();
            workers.push(task::spawn(async move {
                run_listener(ip, &cfg, cloned_token)
                    .await
                    .expect(&format!("Error on ip {}", ip));
            }));
        }
        WlanAdvertiser {
            tcp_threads: workers,
            mdns_handle: mdns_thread,
            token,
        }
    }
    pub(crate) async fn wait(&mut self) {
        for task in self.tcp_threads.iter_mut() {
            task.await.unwrap();
        }
    }
    pub(crate) async fn stop(&mut self) {
        info!("Shutting down");
        self.token.cancel();
        self.wait().await;
    }
}
// impl Drop for WlanAdvertiser {
//     fn drop(&mut self) {
//         for thread in self.tcp_threads.iter_mut() {
//             task.await.unwrap();
//         }
//     }
// }
#[cfg(test)]
mod tests {

    use tracing_test::traced_test;

    use crate::wlan::wlan_client::WlanClient;

    use super::*;
    #[traced_test()]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_first_part() {
        let config = Config::default();
        let mut server = WlanAdvertiser::new(&config);
        let _client = WlanClient::new(&config).await;
        server.stop().await;
    }
}
