use crate::{
    core::protocol::get_online_frame,
    protobuf::nearby::sharing::service::{
        connection_response_frame::Status, v1_frame::FrameType, ConnectionResponseFrame, Frame,
        V1Frame,
    },
};
pub(crate) fn process_transfer_response(frame: Frame) -> bool {
    let resp = frame.v1.unwrap().connection_response.unwrap();
    match resp.status() {
        Status::Accept => true,
        _ => false,
    }
}

pub(crate) fn transfer_response(accept: bool) -> Frame {
    let status = if accept {
        Status::Accept
    } else {
        Status::Reject
    };
    // TODO: attachment_details
    let resp = ConnectionResponseFrame {
        status: Some(status.into()),
        ..Default::default()
    };
    let v1 = V1Frame {
        r#type: Some(FrameType::Response.into()),
        connection_response: Some(resp),
        ..Default::default()
    };
    get_online_frame(v1)
}
