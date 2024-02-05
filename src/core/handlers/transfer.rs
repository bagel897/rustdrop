use crate::{
    core::protocol::get_online_frame,
    protobuf::sharing::nearby::{
        connection_response_frame::Status, v1_frame::FrameType, ConnectionResponseFrame, Frame,
        V1Frame,
    },
};

pub(crate) fn transfer_response(accept: bool) -> Frame {
    let status = if accept {
        Status::Accept
    } else {
        Status::Reject
    };
    let resp = ConnectionResponseFrame {
        status: Some(status.into()),
    };
    let v1 = V1Frame {
        r#type: Some(FrameType::Response.into()),
        connection_response: Some(resp),
        ..Default::default()
    };
    get_online_frame(v1)
}
