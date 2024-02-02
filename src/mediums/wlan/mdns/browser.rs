use std::{net::SocketAddr, thread::sleep, time::Duration};

use base64::{engine::general_purpose::URL_SAFE, Engine};
use futures::StreamExt;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use tokio::sync::mpsc::{channel, Sender};
use tracing::info;

use super::constants::TYPE;
use crate::core::protocol::{decode_endpoint_id, Device};

pub(crate) async fn get_dests() -> Vec<Device> {
    let (mut send, mut recv) = channel(10);
    let mut dests = Vec::new();
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let mut reciever = mdns.browse(TYPE).unwrap().into_stream();
    let task = tokio::spawn(async move {
        while let Some(event) = reciever.next().await {
            on_service_discovered(event, &mut send).await;
        }
    });
    sleep(Duration::from_secs(1));
    recv.recv_many(&mut dests, 1).await;
    task.abort();
    dests
}

async fn on_service_discovered(event: ServiceEvent, out: &mut Sender<Device>) {
    match event {
        ServiceEvent::ServiceResolved(info) => {
            info!("Service discovered: {:?}", info);
            let endpoint_info = info.get_property_val("n").unwrap().unwrap();
            for addr in info.get_addresses() {
                let full_addr = SocketAddr::new(*addr, info.get_port());
                let (device_type, name) =
                    decode_endpoint_id(&URL_SAFE.decode(endpoint_info).unwrap());
                let dest = Device {
                    device_type,
                    device_name: name,
                    ip: full_addr,
                };
                out.send(dest).await.unwrap();
            }
        }
        other_event => {
            info!("Received other event: {:?}", &other_event);
        }
    }
}
