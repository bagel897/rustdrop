use crate::protobuf::{
    location::nearby::connections::{OfflineFrame, V1Frame},
    sharing::nearby::{self, v1_frame::FrameType, Frame},
};
use std::net::SocketAddr;

use bytes::Bytes;
use prost::{DecodeError, Message};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::protobuf::{
    securegcm::{ukey2_message::Type, Ukey2Alert, Ukey2Message},
    sharing::nearby::{
        paired_key_result_frame::Status, PairedKeyEncryptionFrame, PairedKeyResultFrame,
    },
};

use super::{util::get_random, Config, DeviceType};

pub(crate) fn decode_endpoint_id(endpoint_id: &[u8]) -> (DeviceType, String) {
    let bits = endpoint_id.first().unwrap() >> 1 & 0x03;
    let devtype = DeviceType::from(bits);
    let name = String::from_utf8(endpoint_id[18..].to_vec()).unwrap();
    (devtype, name)
}
fn get_devtype_bit(devtype: DeviceType) -> u8 {
    match devtype {
        DeviceType::Unknown => 0,
        DeviceType::Phone => 1,
        DeviceType::Tablet => 2,
        DeviceType::Laptop => 3,
    }
}
fn get_bitfield(devtype: DeviceType) -> u8 {
    get_devtype_bit(devtype) << 1
}
pub(crate) fn get_endpoint_id(config: &Config) -> Vec<u8> {
    let mut data: Vec<u8> = thread_rng().sample_iter(&Alphanumeric).take(17).collect();
    data[0] = get_bitfield(config.devtype);
    let mut encoded = config.name.encode_to_vec();
    data.push(encoded.len() as u8);
    data.append(&mut encoded);
    data
}
pub(crate) fn get_offline_frame(v1: V1Frame) -> OfflineFrame {
    let mut offline = OfflineFrame::default();
    offline.version = Some(1);
    offline.v1 = Some(v1);
    offline
}
pub(crate) fn get_online_frame(v1: nearby::V1Frame) -> Frame {
    let mut offline = Frame::default();
    offline.version = Some(1);
    offline.v1 = Some(v1);
    offline
}
pub(crate) fn get_paired_result() -> Frame {
    let res = PairedKeyResultFrame {
        status: Some(Status::Unable.into()),
    };
    let mut v1 = nearby::V1Frame::default();
    v1.r#type = Some(FrameType::PairedKeyResult.into());
    v1.paired_key_result = Some(res);
    get_online_frame(v1)
}
pub fn get_paired_frame() -> Frame {
    let mut p_key = PairedKeyEncryptionFrame::default();
    p_key.secret_id_hash = Some(get_random(6));
    p_key.signed_data = Some(get_random(72));
    let mut v1 = nearby::V1Frame::default();
    v1.r#type = Some(FrameType::PairedKeyEncryption.into());
    v1.paired_key_encryption = Some(p_key);
    get_online_frame(v1)
}
pub(crate) fn try_decode_ukey2_alert(raw: &Bytes) -> Result<Ukey2Alert, DecodeError> {
    if let Ok(message) = Ukey2Message::decode(raw.clone()) {
        if message.message_type() == Type::Alert {
            let message = Ukey2Alert::decode(message.message_data())?;
            return Ok(message);
        }
    }
    let message = Ukey2Alert::decode(raw.clone())?;
    Ok(message)
}
#[derive(Debug)]
pub struct PairingRequest {
    device_name: String,
    device_type: DeviceType,
}
impl PairingRequest {
    pub fn new(endpoint_info: &[u8]) -> Self {
        let (devtype, name) = decode_endpoint_id(endpoint_info);
        PairingRequest {
            device_name: name,
            device_type: devtype,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Device {
    pub device_name: String,
    pub device_type: DeviceType,
    pub ip: SocketAddr,
}
