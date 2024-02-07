use bluer::Uuid;
use hex_literal::hex;
use uuid::uuid;
pub const SERVICE_UUID: Uuid = uuid!("a82efa21-ae5c-3dde-9bbc-f16da7b16c5a");
pub const SERVICE_ID: [u8; 3] = hex!("fc9f5e");
// Full: fc9f5ed42c8a5e9e94684076ef3bf938a809c60ad354992b0435aebbdc58b97b
pub const PCP: u8 = 0x23;
