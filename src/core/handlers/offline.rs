use crate::{
    core::protocol::get_offline_frame,
    protobuf::location::nearby::connections::{
        v1_frame::FrameType, KeepAliveFrame, OfflineFrame, V1Frame,
    },
};

pub(crate) fn keep_alive() -> OfflineFrame {
    let keep_alive = KeepAliveFrame { ack: Some(true) };
    let v1 = V1Frame {
        r#type: Some(FrameType::KeepAlive.into()),
        keep_alive: Some(keep_alive),
        ..Default::default()
    };
    get_offline_frame(v1)
}
