use std::{net::SocketAddr, time::Duration};

use anyhow::Error;
use bytes::Bytes;
use prost::{DecodeError, Message};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;

use super::{errors::RustdropError, io::writer::WriterSend, util::get_random, Config, DeviceType};
use crate::{
    core::handlers::offline::keep_alive,
    protobuf::{
        location::nearby::connections::{OfflineFrame, V1Frame},
        securegcm::{ukey2_message::Type, Ukey2Alert, Ukey2Message},
        sharing::nearby::{
            self, paired_key_result_frame::Status, v1_frame::FrameType, Frame,
            PairedKeyEncryptionFrame, PairedKeyResultFrame,
        },
    },
};

pub(crate) fn decode_endpoint_id(endpoint_id: &[u8]) -> Result<(DeviceType, String), Error> {
    if endpoint_id.len() < 18 {
        return Err(RustdropError::InvalidEndpointId().into());
    }
    let (first, second) = endpoint_id.split_at(18);
    let bits = first.first().unwrap() >> 1 & 0x03;
    let devtype = DeviceType::from(bits);
    let name = String::from_utf8(second.to_vec())?;
    Ok((devtype, name))
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
    let mut encoded = config.name.as_bytes().to_vec();
    data.push(encoded.len() as u8);
    data.append(&mut encoded);
    data
}
pub(crate) fn get_offline_frame(v1: V1Frame) -> OfflineFrame {
    OfflineFrame {
        version: Some(1),
        v1: Some(v1),
    }
}
pub(crate) fn get_online_frame(v1: nearby::V1Frame) -> Frame {
    Frame {
        version: Some(1),
        v1: Some(v1),
    }
}
pub(crate) fn get_paired_result() -> Frame {
    let res = PairedKeyResultFrame {
        status: Some(Status::Unable.into()),
    };
    let v1 = nearby::V1Frame {
        r#type: Some(FrameType::PairedKeyResult.into()),
        paired_key_result: Some(res),
        ..Default::default()
    };
    get_online_frame(v1)
}
pub fn get_paired_frame() -> Frame {
    let p_key = PairedKeyEncryptionFrame {
        secret_id_hash: Some(get_random(6)),
        signed_data: Some(get_random(72)),
        ..Default::default()
    };
    let v1 = nearby::V1Frame {
        r#type: Some(FrameType::PairedKeyEncryption.into()),
        paired_key_encryption: Some(p_key),
        ..Default::default()
    };
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
    pub fn new(endpoint_info: &[u8]) -> Result<Self, Error> {
        let (devtype, name) = decode_endpoint_id(endpoint_info)?;
        Ok(PairingRequest {
            device_name: name,
            device_type: devtype,
        })
    }
    pub fn name(&self) -> String {
        format!("{} wants to share a file with you", self.device_name)
    }
    pub fn body(&self) -> String {
        format!("{} wants to share a file with you", self.device_name)
    }
}
#[derive(Debug, Clone)]
pub struct Device {
    pub device_name: String,
    pub device_type: DeviceType,
    pub ip: SocketAddr,
}
pub(crate) async fn repeat_keep_alive(writer: WriterSend, cancel: CancellationToken) {
    loop {
        select! {
            _ = cancel.cancelled() => { break;},
            _ = sleep(Duration::from_secs(10)) => {
                let msg = keep_alive();
                writer.send(&msg).await;
        },
        }
    }
}
