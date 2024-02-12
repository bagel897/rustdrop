use crate::protobuf::location::nearby::connections::{
    v1_frame::FrameType, DisconnectionFrame, OfflineFrame, V1Frame,
};

use super::get_offline_frame;

pub(crate) fn get_disconnect() -> OfflineFrame {
    let disconnect = DisconnectionFrame {
        ..Default::default()
    };
    let v1 = V1Frame {
        r#type: Some(FrameType::Disconnection.into()),
        disconnection: Some(disconnect),
        ..Default::default()
    };
    get_offline_frame(v1)
}
