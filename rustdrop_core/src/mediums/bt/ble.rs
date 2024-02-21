use bluer::{
    adv::{Advertisement, Feature, SecondaryChannel},
    monitor::{
        data_type::INCOMPLETE_LIST_128_BIT_SERVICE_CLASS_UUIDS, Monitor, Pattern,
        RssiSamplingPeriod,
    },
    Device, DeviceEvent, Uuid,
};
use bytes::Bytes;
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
pub fn get_monitor(services: Vec<Uuid>) -> Monitor {
    let pattern = services
        .into_iter()
        .map(|uuid| Pattern {
            data_type: INCOMPLETE_LIST_128_BIT_SERVICE_CLASS_UUIDS,
            start_position: 0x00,
            content: uuid.to_bytes_le().to_vec(),
        })
        .collect();
    info!("Scanning for {:?}", pattern);
    Monitor {
        monitor_type: bluer::monitor::Type::OrPatterns,
        rssi_low_threshold: None,
        rssi_high_threshold: None,
        rssi_low_timeout: None,
        rssi_high_timeout: None,
        rssi_sampling_period: Some(RssiSamplingPeriod::All),
        patterns: Some(pattern),
        ..Default::default()
    }
}
pub fn get_advertisment(service_id: String, service_uuid: Uuid, adv_data: Bytes) -> Advertisement {
    Advertisement {
        // local_name: Some(service_id),
        advertisement_type: bluer::adv::Type::Broadcast,
        service_uuids: vec![service_uuid].into_iter().collect(),
        service_data: [(service_uuid, adv_data.into())].into(),
        // discoverable: Some(true),
        secondary_channel: Some(SecondaryChannel::OneM),
        // duration: Some(Duration::from_secs(60)),
        // timeout: Some(Duration::from_secs(60)),
        system_includes: [Feature::TxPower].into(),
        tx_power: Some(20),
        ..Default::default()
    }
}
pub async fn process_device(
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
