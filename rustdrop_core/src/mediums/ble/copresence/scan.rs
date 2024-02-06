use anyhow::Error;

use crate::{
    mediums::ble::{
        common::{advertise::advertise, scan::scan_le},
        copresence::consts::{SERVICE_UUID_RECIEVING, SERVICE_UUID_SHARING},
    },
    Application, UiHandle,
};

use super::consts::{SERVICE_DATA, SERVICE_ID};

pub(crate) async fn scan_for_incoming<U: UiHandle>(app: &mut Application<U>) -> Result<(), Error> {
    advertise(SERVICE_ID.into(), SERVICE_UUID_RECIEVING, SERVICE_DATA, app).await?;
    let (devices, events) = scan_le(vec![SERVICE_UUID_SHARING], app).await?;
    Ok(())
}
