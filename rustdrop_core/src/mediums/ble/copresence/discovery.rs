use std::error::Error;

use tokio::select;
use tracing::info;

use super::consts::{SERVICE_DATA, SERVICE_ID, SERVICE_UUID_RECIEVING, SERVICE_UUID_SHARING};
use crate::{
    mediums::ble::common::{advertise::advertise, scan::scan_le},
    Application, UiHandle,
};

// const SERVICE_DATA: &[u8] = &[252, 18, 142, 1, 66, 0, 0, 0];
// const MAX_SERVICE_DATA_SIZE: usize = 26;
// fn get_service_data() -> Bytes {
//     let mut data: BytesMut = BytesMut::with_capacity(MAX_SERVICE_DATA_SIZE);
//     data.extend_from_slice(&[
//         0xfc, 0x12, 0x8e, 0x01, 0x42, 00, 00, 00, 00, 00, 00, 00, 00, 00,
//     ]);
//     // while data.len() < 128 {
//     //     data.push(0x0);
//     // }
//     data.extend(get_random(10));
//     // data.push(0x0);
//     // data.reverse();
//     data.into()
// }
pub(crate) async fn trigger_reciever<U: UiHandle>(
    app: &mut Application<U>,
) -> Result<(), Box<dyn Error>> {
    advertise(SERVICE_ID.into(), SERVICE_UUID_SHARING, SERVICE_DATA, app).await?;
    let (mut devices, mut events) = scan_le(vec![SERVICE_UUID_RECIEVING], app).await?;
    app.spawn(
        async move {
            loop {
                select! {
                    dev = devices.recv() => {
                        info!("{:?}", dev)
                    }
                    event = events.recv() => {
                        info!("{:?}", event)
                    }
                }
            }
        },
        "discovery_process",
    );
    Ok(())
}
