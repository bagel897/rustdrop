use bluer::adv::Advertisement;
use std::error::Error;

use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::{ble::consts::SERVICE_UUID, core::util::get_random};
pub(crate) async fn trigger_reciever(cancel: CancellationToken) -> Result<(), Box<dyn Error>> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    info!(
        "Advertising on Bluetooth adapter {} with address {}",
        adapter.name(),
        adapter.address().await?
    );
    let mut data: Vec<u8> = vec![
        0xfc, 0x12, 0x8e, 0x01, 0x42, 00, 00, 00, 00, 00, 00, 00, 00, 00,
    ];
    data.extend(get_random(10));
    let le_advertisement = Advertisement {
        advertisement_type: bluer::adv::Type::Peripheral,
        service_uuids: vec![SERVICE_UUID].into_iter().collect(),
        service_data: [(SERVICE_UUID, data)].into(),
        discoverable: Some(true),
        local_name: Some("le_advertise".to_string()),
        ..Default::default()
    };
    println!("{:?}", &le_advertisement);
    let handle = adapter.advertise(le_advertisement).await?;

    cancel.cancelled().await;

    info!("Removing advertisement");
    drop(handle);
    Ok(())
}
