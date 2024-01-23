use base64::{prelude::BASE64_URL_SAFE, Engine};
use bluer::adv::Advertisement;
use bytes::Bytes;
use std::error::Error;
use uuid::Uuid;

use tokio_util::sync::CancellationToken;
use tracing::info;

const MAX_SERVICE_DATA_SIZE: usize = 26;

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
    // let encoded = BASE64_URL_SAFE.encode(adv_data);
    let le_advertisement = Advertisement {
        local_name: Some(service_id),
        advertisement_type: bluer::adv::Type::Peripheral,
        service_uuids: vec![service_uuid].into_iter().collect(),
        service_data: [(service_uuid, adv_data.into())].into(),
        discoverable: Some(true),
        ..Default::default()
    };
    println!("{:?}", &le_advertisement);
    let handle = adapter.advertise(le_advertisement).await?;

    cancel.cancelled().await;

    info!("Removing advertisement");
    drop(handle);
    Ok(())
}
