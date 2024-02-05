use std::error::Error;

use bluer::adv::{Advertisement, SecondaryChannel};
use bytes::Bytes;
use tokio_util::sync::CancellationToken;
use tracing::info;
use uuid::Uuid;

pub(crate) async fn advertise(
    cancel: CancellationToken,
    service_id: String,
    service_uuid: Uuid,
    adv_data: Bytes,
) -> Result<(), Box<dyn Error>> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    info!(
        "Advertising on Bluetooth adapter {} with address {}",
        adapter.name(),
        adapter.address().await?
    );
    let le_advertisement = Advertisement {
        local_name: Some(service_id),
        advertisement_type: bluer::adv::Type::Peripheral,
        service_uuids: vec![service_uuid].into_iter().collect(),
        service_data: [(service_uuid, adv_data.into())].into(),
        discoverable: Some(true),
        secondary_channel: Some(SecondaryChannel::OneM),
        ..Default::default()
    };
    println!("{:?}", &le_advertisement);
    let handle = adapter.advertise(le_advertisement).await?;

    cancel.cancelled().await;

    info!("Removing advertisement");
    drop(handle);
    Ok(())
}
