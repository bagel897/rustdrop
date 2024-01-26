use std::{collections::HashMap, vec};

use bytes::{Bytes, BytesMut};
use prost::Message;

use super::protocol::get_offline_frame;
use crate::protobuf::{
    location::nearby::connections::{
        payload_transfer_frame::{
            payload_chunk::{self, Flags},
            payload_header::PayloadType,
            PacketType, PayloadChunk, PayloadHeader,
        },
        v1_frame::{self, FrameType},
        OfflineFrame, PayloadTransferFrame, V1Frame,
    },
    sharing::nearby::Frame,
};
#[derive(Debug)]
struct Incoming {
    pub data: BytesMut,
    pub remaining_bytes: i64,
    pub is_finished: bool,
}
impl Incoming {
    pub fn new(size: i64) -> Self {
        Incoming {
            data: BytesMut::zeroed(size.try_into().unwrap()),
            remaining_bytes: size,
            is_finished: false,
        }
    }
}
#[derive(Default, Debug)]
pub struct PayloadHandler {
    send_next_cnt: i64,
    incoming: HashMap<i64, Incoming>,
}

fn payload_to_offline(payload: PayloadTransferFrame) -> OfflineFrame {
    get_offline_frame(V1Frame {
        r#type: Some(FrameType::PayloadTransfer.into()),
        payload_transfer: Some(payload),
        ..Default::default()
    })
}
fn construct_payload_transfer_first(message: &Bytes, header: PayloadHeader) -> OfflineFrame {
    let data = PayloadChunk {
        body: Some(message.to_vec()),
        offset: Some(0),
        flags: Some(0),
    };

    let payload = PayloadTransferFrame {
        packet_type: Some(PacketType::Data.into()),
        payload_header: Some(header),
        payload_chunk: Some(data),
        ..Default::default()
    };

    payload_to_offline(payload)
}
fn construct_payload_transfer_end(header: PayloadHeader, size: i64) -> OfflineFrame {
    let data = PayloadChunk {
        body: None,
        offset: Some(size),
        flags: Some(payload_chunk::Flags::LastChunk.into()),
    };

    let payload = PayloadTransferFrame {
        packet_type: Some(PacketType::Data.into()),
        payload_header: Some(header),
        payload_chunk: Some(data),
        ..Default::default()
    };
    payload_to_offline(payload)
}
fn get_payload_header(id: i64, size: i64) -> PayloadHeader {
    PayloadHeader {
        id: Some(id),
        r#type: Some(PayloadType::Bytes.into()),
        total_size: Some(size),
        is_sensitive: Some(false),
        ..Default::default()
    }
}
impl PayloadHandler {
    pub fn send_message(&mut self, message: &Frame) -> Vec<OfflineFrame> {
        let id = self.send_next_cnt;
        self.send_next_cnt += 1;
        let body = Bytes::from(message.encode_to_vec());
        let len: i64 = body.len().try_into().unwrap();
        let header = get_payload_header(id, len);
        let first = construct_payload_transfer_first(&body, header.clone());
        let second = construct_payload_transfer_end(header, len);
        vec![first, second]
    }
    pub fn push_data(&mut self, frame: OfflineFrame) {
        let v1 = frame.v1.unwrap();
        assert!(v1.r#type() == v1_frame::FrameType::PayloadTransfer);
        let data = v1.payload_transfer.unwrap();
        let header = data.payload_header.unwrap();
        let chunk = data.payload_chunk.unwrap();
        let id = header.id();
        let size = header.total_size();
        let default = Incoming::new(size);
        self.incoming.entry(id).or_insert(default);

        let incoming = self.incoming.get_mut(&id).unwrap();
        let offset = chunk.offset();
        if let Some(data) = chunk.body {
            let len: i64 = data.len().try_into().unwrap();
            incoming.remaining_bytes -= len;
            let start: usize = offset.try_into().unwrap();
            for i in 0..data.len() {
                incoming.data[i + start] = data[i];
            }
        } else {
            let l_chunk: i32 = Flags::LastChunk.into();
            if l_chunk == chunk.flags() && incoming.remaining_bytes == 0 {
                incoming.is_finished = true;
            }
        }
    }
    pub fn get_next_payload(&mut self) -> Option<Frame> {
        let mut res: Option<(i64, Frame)> = None;
        for (id, payload) in self.incoming.iter() {
            if payload.is_finished {
                if let Ok(msg) = Frame::decode(payload.data.clone()) {
                    res = Some((*id, msg));
                }
            }
        }
        match res {
            None => None,
            Some((id, msg)) => {
                self.incoming.remove(&id);
                Some(msg)
            }
        }
    }
}
