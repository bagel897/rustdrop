use std::{
    net::{IpAddr, SocketAddr},
    thread::sleep,
    time::Duration,
};

use base64::prelude::*;
use futures::StreamExt;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use tokio::sync::mpsc::{channel, Sender};
use tracing::{debug, error, info};

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
fn parse_device(addr: &IpAddr, info: &ServiceInfo) -> Result<Device, anyhow::Error> {
    let endpoint_info = info.get_property_val("n").unwrap().unwrap();
    let full_addr = SocketAddr::new(*addr, info.get_port());
    let decoded = BASE64_URL_SAFE_NO_PAD.decode(endpoint_info)?;
    let (device_type, name) = decode_endpoint_id(&decoded)?;
    Ok(Device {
        device_type,
        device_name: name,
        ip: full_addr,
    })
}
async fn on_service_discovered(event: ServiceEvent, out: &mut Sender<Device>) {
    match event {
        ServiceEvent::ServiceResolved(info) => {
            debug!("Service discovered: {:?}", info);
            for addr in info.get_addresses() {
                match parse_device(addr, &info) {
                    Ok(dest) => out.send(dest).await.unwrap(),
                    Err(e) => error!("Error while parsing endpoint {:?}: {}", info, e),
                };
            }
        }
        other_event => {
            info!("Received other event: {:?}", &other_event);
        }
    }
}
