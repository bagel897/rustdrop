use std::{
    io::{self, ErrorKind},
    net::{IpAddr, SocketAddr},
};

use pnet::datalink;
use tokio::{net::TcpListener, select};
use tracing::info;

use super::{mdns::MDNSHandle, wlan_server::WlanReader};
use crate::{runner::application::Application, ui::UiHandle};
async fn run_listener<U: UiHandle>(addr: IpAddr, application: Application<U>) -> io::Result<()> {
    let full_addr = SocketAddr::new(addr, application.config.port);
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
    let token = application.child_token();
    loop {
        select! {
            _ = token.cancelled() => { break;},
            Ok((stream,_addr)) = listener.accept() => {
                let app = application.clone();
                application.tracker.spawn(async { WlanReader::new(stream, app).await.run().await.unwrap();  });}
        }
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
pub async fn start_wlan<U: UiHandle>(application: Application<U>) {
    let ips = get_ips();
    let mdns_handle = MDNSHandle::new(application.clone(), ips.clone());
    application.tracker.spawn(mdns_handle.advertise_mdns());
    for ip in ips {
        let cloned = application.clone();
        application.tracker.spawn(async move {
            run_listener(ip, cloned)
                .await
                .unwrap_or_else(|_| panic!("Error on ip {}", ip));
        });
    }
}
