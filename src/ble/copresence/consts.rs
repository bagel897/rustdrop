use bluer::Uuid;
use bytes::Bytes;
use uuid::uuid;
pub const SERVICE_ID: &str = "Fast Pair";
// pub const SERVICE_UUID: Uuid = uuid!("0000FEF300001000800000805F9B34FB");
pub const SERVICE_UUID: Uuid = uuid!("0000fe2c-0000-1000-8000-00805f9b34fb");
pub const SERVICE_UUID16: Bytes = Bytes::from_static(&[0x2c, 0xfe]);
