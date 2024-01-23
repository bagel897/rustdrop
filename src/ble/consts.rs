use bluer::Uuid;
use bytes::Bytes;
use uuid::uuid;
pub const SERVICE_ID: &str = "Fast Pair";
pub const SERVICE_UUID: Uuid = uuid!("0000FE2C-0000-1000-8000-00805F9B34FB");
pub const SERVICE_UUID16: Bytes = Bytes::from_static(&[0x2c, 0xfe]);
pub const SERVICE_UUID2: Uuid = uuid!("df21fe2c-2515-4fdb-8886-f12c4d67927c");
