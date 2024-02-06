use anyhow::Error;
use bluer::adv::{Advertisement, SecondaryChannel};
use bluer::Uuid;
use bytes::Bytes;
use tracing::info;

use crate::{Application, UiHandle};

pub(crate) async fn advertise<U: UiHandle>(
    service_id: String,
    service_uuid: Uuid,
    adv_data: Bytes,
    app: &mut Application<U>,
) -> Result<(), Error> {
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
    let cancel = app.child_token();
    info!("{:?}", &le_advertisement);
    app.spawn(
        async move {
            let handle = adapter.advertise(le_advertisement).await.unwrap();
            cancel.cancelled().await;
            info!("Removing advertisement");
            drop(handle);
        },
        "ble advertise",
    );
    Ok(())
}
