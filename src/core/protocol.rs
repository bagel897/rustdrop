use prost::Message;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::protobuf::sharing::nearby::{
    paired_key_result_frame::Status, PairedKeyEncryptionFrame, PairedKeyResultFrame,
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
        DeviceType::UNKNOWN => 0,
        DeviceType::PHONE => 1,
        DeviceType::TABLET => 2,
        DeviceType::LAPTOP => 3,
    }
}
fn get_bitfield(devtype: DeviceType) -> u8 {
    return get_devtype_bit(devtype) << 1;
}
pub(crate) fn get_endpoint_id(config: &Config) -> Vec<u8> {
    let mut data: Vec<u8> = thread_rng().sample_iter(&Alphanumeric).take(17).collect();
    data[0] = get_bitfield(config.devtype);
    let mut encoded = config.name.encode_to_vec();
    data.push(encoded.len() as u8);
    data.append(&mut encoded);
    return data;
}
pub(crate) fn get_paired_result() -> PairedKeyResultFrame {
    let res = PairedKeyResultFrame {
        status: Some(Status::Unknown.into()),
    };
    return res;
}
pub fn get_paired_frame() -> PairedKeyEncryptionFrame {
    let mut p_key = PairedKeyEncryptionFrame::default();
    p_key.secret_id_hash = Some(get_random(6));
    p_key.signed_data = Some(get_random(72));
    p_key
}
