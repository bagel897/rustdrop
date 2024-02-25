pub mod payload_message;
mod sender;

use std::time::Duration;

use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;

use super::{io::writer::WriterSend, util::get_random};
use crate::{
    core::handlers::offline::keep_alive,
    mediums::Discover,
    protobuf::{
        location::nearby::connections::{os_info::OsType, OfflineFrame, V1Frame},
        nearby::sharing::service::{
            paired_key_result_frame::Status, v1_frame::FrameType, Frame, PairedKeyEncryptionFrame,
            PairedKeyResultFrame, V1Frame as V1FrameOnline,
        },
    },
    DeviceType,
};

pub(crate) fn get_offline_frame(v1: V1Frame) -> OfflineFrame {
    OfflineFrame {
        version: Some(1),
        v1: Some(v1),
    }
}
pub(crate) fn get_online_frame(v1: V1FrameOnline) -> Frame {
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
    let v1 = V1FrameOnline {
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
    let v1 = V1FrameOnline {
        r#type: Some(FrameType::PairedKeyEncryption.into()),
        paired_key_encryption: Some(p_key),
        ..Default::default()
    };
    get_online_frame(v1)
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Device {
    pub endpoint_id: u32,
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
