use crate::{
    core::{
        bits::{Bitfield, EndpointInfo},
        protocol::get_offline_frame,
        util::{get_osinfo, get_random},
    },
    protobuf::location::nearby::connections::{
        connection_response_frame::ResponseStatus, v1_frame::FrameType, ConnectionRequestFrame,
        ConnectionResponseFrame, KeepAliveFrame, OfflineFrame, V1Frame,
    },
};

pub(crate) fn keep_alive(seq: u32) -> OfflineFrame {
    let keep_alive = KeepAliveFrame {
        ack: Some(true),
        seq_num: Some(seq),
    };
    let v1 = V1Frame {
        r#type: Some(FrameType::KeepAlive.into()),
        keep_alive: Some(keep_alive),
        ..Default::default()
    };
    get_offline_frame(v1)
}
pub fn get_conn_response() -> OfflineFrame {
    let conn = ConnectionResponseFrame {
        response: Some(ResponseStatus::Accept.into()),
        os_info: Some(get_osinfo()),
        handshake_data: Some(get_random(10)),
        nearby_connections_version: Some(1),
        ..Default::default()
    };
    let v1 = V1Frame {
        r#type: Some(FrameType::ConnectionResponse.into()),
        connection_response: Some(conn),
        ..Default::default()
    };
    get_offline_frame(v1)
}
pub(crate) fn get_con_request(endpoint_info: EndpointInfo) -> OfflineFrame {
    let init = ConnectionRequestFrame {
        endpoint_name: Some(endpoint_info.name.clone()),
        endpoint_info: Some(endpoint_info.to_vec()),
        ..Default::default()
    };
    let v1 = V1Frame {
        r#type: Some(FrameType::ConnectionRequest.into()),
        connection_request: Some(init),
        ..Default::default()
    };
    get_offline_frame(v1)
}
