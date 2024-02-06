use anyhow::Error;
use base64::{prelude::BASE64_URL_SAFE, Engine};
use bluer::rfcomm::Profile;
use bytes::{BufMut, BytesMut};
use tracing::info;

use crate::{
    core::{protocol::get_endpoint_info, util::get_random},
    mediums::bt::consts::{SERVICE_ID, SERVICE_UUID},
    Application, Config, UiHandle,
};

use super::consts::PCP;
fn get_name(config: &Config) -> String {
    let mut result = BytesMut::new();
    result.put_u8(PCP);
    result.extend_from_slice(config.endpoint_id.as_bytes());
    result.extend_from_slice(&SERVICE_ID);
    result.put_u8(0x0);
    result.extend_from_slice(&get_random(6));
    let endpoint_info = get_endpoint_info(config);
    result.put_u8(endpoint_info.len() as u8);
    result.extend_from_slice(&endpoint_info);
    result.put_u8((result.len() + 1) as u8);
    BASE64_URL_SAFE.encode(result)
}
pub(crate) async fn adv_bt<U: UiHandle>(app: &mut Application<U>) -> Result<(), Error> {
    let name = get_name(&app.config);
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;
    let profile = Profile {
        uuid: SERVICE_UUID,
        name: Some(name),
        ..Default::default()
    };
    let cancel = app.child_token();
    app.spawn(
        async move {
            let handle = session.register_profile(profile).await;
            info!(
                "Advertising on Bluetooth adapter {} with address {}",
                adapter.name(),
                adapter.address().await.unwrap()
            );
            cancel.cancelled().await;
        },
        "BT Adv",
    );

    Ok(())
}
