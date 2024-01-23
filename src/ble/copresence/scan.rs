use std::error::Error;

use bluer::{monitor::MonitorEvent, Device};
use tokio::select;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::ble::{common::scan::scan_le, copresence::consts::SERVICE_UUID16};

async fn process_device(device: Device) {
    println!(
        "    Address type:       {}",
        device.address_type().await.unwrap()
    );
    println!("    Name:               {:?}", device.name().await.unwrap());
    println!("    Icon:               {:?}", device.icon().await.unwrap());
    println!(
        "    Class:              {:?}",
        device.class().await.unwrap()
    );
    println!(
        "    UUIDs:              {:?}",
        device.uuids().await.unwrap().unwrap_or_default()
    );
    println!(
        "    Paired:             {:?}",
        device.is_paired().await.unwrap()
    );
    println!(
        "    Connected:          {:?}",
        device.is_connected().await.unwrap()
    );
    println!(
        "    Trusted:            {:?}",
        device.is_trusted().await.unwrap()
    );
    println!(
        "    Modalias:           {:?}",
        device.modalias().await.unwrap()
    );
    println!("    RSSI:               {:?}", device.rssi().await.unwrap());
    println!(
        "    TX power:           {:?}",
        device.tx_power().await.unwrap()
    );
    println!(
        "    Manufacturer data:  {:?}",
        device.manufacturer_data().await.unwrap()
    );
    println!(
        "    Service data:       {:?}",
        device.service_data().await.unwrap()
    );
    info!("{:?}", device.all_properties().await.unwrap());
    let mut events = device.events().await.unwrap();
    while let Some(ev) = events.next().await {
        info!("On device {:?}, received event {:?}", device, ev);
    }
}
pub(crate) async fn scan_for_incoming(cancel: CancellationToken) -> Result<(), Box<dyn Error>> {
    let (adapter, _monitor_handle) = scan_le(SERVICE_UUID16).await?;
    let mut monitor_handle = _monitor_handle.fuse();
    loop {
        select! {
            _ = cancel.cancelled() => {
                break;
            }
            Some(mevt) = monitor_handle.next() => {
                if let MonitorEvent::DeviceFound(devid) = mevt {
                    info!("Discovered device {:?}", devid);
                    let dev = adapter.device(devid.device)?;
                    tokio::spawn(process_device(dev));
                }
            }
            else => break,
        }
    }
    info!("Closing BLE scan");
    Ok(())
}
