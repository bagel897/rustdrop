use bluer::Uuid;
use bytes::Bytes;
use uuid::uuid;
pub const SERVICE_ID: &str = "Fast Pair";
pub const SERVICE_UUID_COPRESENCE: Uuid = uuid!("0000FEF300001000800000805F9B34FB");
pub const SERVICE_UUID: Uuid = uuid!("0000fe2c-0000-1000-8000-00805f9b34fb");
pub const SERVICE_DATA: Bytes = Bytes::from_static(&[
    252, 18, 142, 1, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 191, 45, 91, 160, 225, 216, 117, 36, 202, 0,
]);
