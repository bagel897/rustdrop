use anyhow::Error;
use base64::{prelude::BASE64_URL_SAFE, Engine};
use bluer::rfcomm::{Profile, Role::Server};
use bytes::{BufMut, BytesMut};
use tokio_stream::StreamExt;
use tracing::info;

use crate::{
    core::{protocol::get_endpoint_info, util::get_random},
    mediums::bt::consts::{SERVICE_ID, SERVICE_UUID},
    Config, Context,
};

use super::consts::PCP;
pub(super) fn get_name(config: &Config) -> String {
    let mut result = BytesMut::new();
    result.put_u8(PCP);
    result.extend_from_slice(config.endpoint_id.as_bytes());
    result.extend_from_slice(&SERVICE_ID);
    result.put_u8(0x0);
    result.extend_from_slice(&get_random(6));
    let endpoint_info = get_endpoint_info(config);
    info!("{:?}", endpoint_info);
    result.put_u8(endpoint_info.len().try_into().unwrap());
    result.extend_from_slice(&endpoint_info);
    result.put_u8((result.len() + 1).try_into().unwrap());
    BASE64_URL_SAFE.encode(result)
}
pub(crate) async fn adv_bt(context: &mut Context) -> Result<(), Error> {
    let name = get_name(&context.config);
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;
    // adapter.set_discoverable(true).await?;
    let profile = Profile {
        uuid: SERVICE_UUID,
        role: Some(Server),
        // name: Some(name),
        require_authentication: Some(false),
        require_authorization: Some(false),
        channel: Some(0),
        psm: Some(0),
        auto_connect: Some(true),
        ..Default::default()
    };
    let mut handle = session.register_profile(profile).await?;
    // adapter
    //     .register_gatt_profile(bluer::gatt::local::Profile {
    //         uuids: HashSet::from([SERVICE_UUID]),
    //         ..Default::default()
    //     })
    //     .await?;
    let cancel = context.child_token();
    context.spawn(
        async move {
            info!(
                "Advertising on Bluetooth adapter {} with name {}",
                adapter.name(),
                name
            );
            while let Some(req) = handle.next().await {
                info!("{:?}", req);
            }
            info!("No more requests");
            cancel.cancelled().await;
        },
        "BT Adv",
    );

    Ok(())
}
