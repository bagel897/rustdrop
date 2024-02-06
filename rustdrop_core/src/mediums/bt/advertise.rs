use anyhow::Error;
use tracing::info;

use crate::{Application, UiHandle};

pub(crate) async fn adv_bt<U: UiHandle>(app: &mut Application<U>) -> Result<(), Error> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    info!(
        "Advertising on Bluetooth adapter {} with address {}",
        adapter.name(),
        adapter.address().await?
    );
    Ok(())
}
