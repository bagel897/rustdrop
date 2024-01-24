use bytes::{Bytes, BytesMut};
use std::error::Error;

use tokio_util::sync::CancellationToken;

use crate::{
    ble::{
        common::advertise::advertise,
        copresence::consts::{SERVICE_ID, SERVICE_UUID},
    },
    core::util::get_random,
};
// const SERVICE_DATA: &[u8] = &[252, 18, 142, 1, 66, 0, 0, 0];
const SERVICE_DATA: &[u8] = &[
    252, 18, 142, 1, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 191, 45, 91, 160, 225, 216, 117, 36, 202, 0,
];
const MAX_SERVICE_DATA_SIZE: usize = 26;
fn get_service_data() -> Bytes {
    let mut data: BytesMut = BytesMut::with_capacity(MAX_SERVICE_DATA_SIZE);
    data.extend_from_slice(&[
        0xfc, 0x12, 0x8e, 0x01, 0x42, 00, 00, 00, 00, 00, 00, 00, 00, 00,
    ]);
    // while data.len() < 128 {
    //     data.push(0x0);
    // }
    data.extend(get_random(10));
    // data.push(0x0);
    // data.reverse();
    data.into()
}
pub(crate) async fn trigger_reciever(cancel: CancellationToken) -> Result<(), Box<dyn Error>> {
    let data = get_service_data();
    advertise(cancel, SERVICE_ID.into(), SERVICE_UUID, SERVICE_DATA.into()).await
}
