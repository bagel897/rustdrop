use anyhow::Error;
use bluer::monitor::data_type::{
    COMPLETE_LIST_16_BIT_SERVICE_CLASS_UUIDS, INCOMPLETE_LIST_128_BIT_SERVICE_CLASS_UUIDS,
};
use bluer::{
    monitor::{
        data_type::COMPLETE_LIST_128_BIT_SERVICE_CLASS_UUIDS, Monitor, MonitorEvent, Pattern,
        RssiSamplingPeriod,
    },
    Device, DeviceEvent,
};
use bluer::{Uuid, UuidExt};
use tokio::{
    select,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tokio_stream::StreamExt;
use tracing::info;

use crate::{Application, UiHandle};
async fn process_device(
    device: Device,
    devices: &UnboundedSender<Device>,
    events_tx: &UnboundedSender<DeviceEvent>,
) {
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
    info!("{:?}", device.advertising_data().await.unwrap());
    let mut events = device.events().await.unwrap();
    while let Some(ev) = events.next().await {
        info!("On device {:?}, received event {:?}", device, ev);
        events_tx.send(ev).unwrap();
    }
    devices.send(device).unwrap();
}
pub(crate) async fn scan_le<U: UiHandle>(
    services: Vec<Uuid>,
    app: &mut Application<U>,
) -> Result<(UnboundedReceiver<Device>, UnboundedReceiver<DeviceEvent>), Error> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    let mm = adapter.monitor().await?;
    adapter.set_powered(true).await?;
    let pattern = services
        .into_iter()
        .map(|uuid| Pattern {
            data_type: INCOMPLETE_LIST_128_BIT_SERVICE_CLASS_UUIDS,
            start_position: 0x00,
            content: uuid.to_bytes_le().to_vec(),
        })
        .collect();
    info!("Scanning for {:?}", pattern);
    let mut monitor_handle = mm
        .register(Monitor {
            monitor_type: bluer::monitor::Type::OrPatterns,
            rssi_low_threshold: None,
            rssi_high_threshold: None,
            rssi_low_timeout: None,
            rssi_high_timeout: None,
            rssi_sampling_period: Some(RssiSamplingPeriod::All),
            patterns: Some(pattern),
            ..Default::default()
        })
        .await?
        .fuse();
    info!("Scanning BLE");
    let (devices_tx, devices_rx) = mpsc::unbounded_channel();
    let (events_tx, events_rx) = mpsc::unbounded_channel();
    let cancel = app.child_token();
    app.spawn(
        async move {
            loop {
                select! {
                    _ = cancel.cancelled() => {
                        break;
                    }
                    Some(mevt) = monitor_handle.next() => {
                        if let MonitorEvent::DeviceFound(devid) = mevt {
                            info!("Discovered device {:?}", devid);
                            let dev = adapter.device(devid.device).unwrap();
                            process_device(dev, &devices_tx, &events_tx).await;
                        }
                    }
                    else => break,
                }
            }
            info!("Closing BLE scan");
        },
        "ble_advertise",
    );
    Ok((devices_rx, events_rx))
}
