use bluer::Uuid;
use bytes::Bytes;
use uuid::uuid;
pub const SERVICE_UUID: Uuid = uuid!("a82efa21-ae5c-3dde-9bbc-f16da7b16c5a");
pub const SERVICE_UUID_NEW: Uuid = uuid!("00001101-0000-1000-8000-00805F9B34FB");
// Full: fc9f5ed42c8a5e9e94684076ef3bf938a809c60ad354992b0435aebbdc58b97b
pub const SERVICE_ID_BLE: &str = "NearbySharing";
pub const SERVICE_UUID_RECIEVING: Uuid = uuid!("0000FEF300001000800000805F9B34FB"); // Device is
                                                                                    // receiving
pub const SERVICE_UUID_SHARING: Uuid = uuid!("0000fe2c-0000-1000-8000-00805f9b34fb"); // device is
                                                                                      // sharing
pub const SERVICE_DATA: Bytes = Bytes::from_static(&[
    252, 18, 142, 1, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 191, 45, 91, 160, 225, 216, 117, 36, 202, 0,
]);
