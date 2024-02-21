pub mod payload_message;
mod sender;

use std::time::Duration;

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;

use super::{errors::RustdropError, io::writer::WriterSend, util::get_random, Config, DeviceType};
use crate::{
    core::handlers::offline::keep_alive,
    mediums::Discover,
    protobuf::{
        location::nearby::connections::{os_info::OsType, OfflineFrame, V1Frame},
        nearby::sharing::service::{
            self, paired_key_result_frame::Status, v1_frame::FrameType, Frame,
            PairedKeyEncryptionFrame, PairedKeyResultFrame,
        },
    },
    RustdropResult,
};

pub(crate) fn decode_endpoint_id(endpoint_id: &[u8]) -> RustdropResult<(DeviceType, String)> {
    if endpoint_id.len() < 18 {
        return Err(RustdropError::InvalidEndpointId());
    }
    let (first, second) = endpoint_id.split_at(18);
    let bits = first.first().unwrap() >> 1 & 0x03;
    let devtype = DeviceType::from(bits);
    let name =
        String::from_utf8(second.to_vec()).map_err(|e| RustdropError::InvalidEndpointId())?;
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
pub(crate) fn get_endpoint_info(config: &Config) -> Vec<u8> {
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
pub(crate) fn get_online_frame(v1: service::V1Frame) -> Frame {
    Frame {
        version: Some(1),
        v1: Some(v1),
    }
}
pub(crate) fn get_paired_result() -> Frame {
    let res = PairedKeyResultFrame {
        status: Some(Status::Unable.into()),
        os_type: Some(OsType::Linux.into()),
    };
    let v1 = service::V1Frame {
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
    let v1 = service::V1Frame {
        r#type: Some(FrameType::PairedKeyEncryption.into()),
        paired_key_encryption: Some(p_key),
        ..Default::default()
    };
    get_online_frame(v1)
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Device {
    pub device_name: String,
    pub device_type: DeviceType,
    pub discovery: Discover,
}
pub(crate) async fn repeat_keep_alive(writer: WriterSend, cancel: CancellationToken) {
    let mut seq = 0;
    loop {
        select! {
            _ = cancel.cancelled() => { break;},
            _ = sleep(Duration::from_secs(10)) => {
                let msg = keep_alive(seq);
                writer.send(&msg).await;
        },
        };
        seq += 1;
    }
}
