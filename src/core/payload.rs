use std::collections::HashMap;

use bytes::{Bytes, BytesMut};
use prost::Message;

use crate::protobuf::location::nearby::connections::{
    payload_transfer_frame::{
        payload_chunk::{self, Flags},
        payload_header::PayloadType,
        PacketType, PayloadChunk, PayloadHeader,
    },
    v1_frame::{self, FrameType},
    OfflineFrame, PayloadTransferFrame, V1Frame,
};

use super::protocol::get_offline_frame;
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
pub struct PayloadHandler {
    send_next_cnt: usize,
    incoming: HashMap<i64, Incoming>,
}
impl Default for PayloadHandler {
    fn default() -> Self {
        PayloadHandler {
            send_next_cnt: 0,
            incoming: HashMap::new(),
        }
    }
}
fn payload_to_offline(payload: PayloadTransferFrame) -> OfflineFrame {
    let mut v1 = V1Frame::default();
    v1.r#type = Some(FrameType::PayloadTransfer.into());
    v1.payload_transfer = Some(payload);
    return get_offline_frame(v1);
}
fn construct_payload_transfer_first(id: usize, message: &Bytes) -> OfflineFrame {
    let mut data = PayloadChunk::default();
    data.body = Some(message.to_vec());
    data.offset = Some(0);
    let mut payload = PayloadTransferFrame::default();
    payload.packet_type = Some(PacketType::Data.into());
    payload.payload_header = Some(get_payload_header(id, message.len()));
    payload.payload_chunk = Some(data);

    return payload_to_offline(payload);
}
fn construct_payload_transfer_end(id: usize, size: usize) -> OfflineFrame {
    let mut data = PayloadChunk::default();
    data.flags = Some(payload_chunk::Flags::LastChunk.into());
    data.offset = Some(size.try_into().unwrap());
    data.body = Some(Vec::new());
    let mut payload = PayloadTransferFrame::default();
    payload.packet_type = Some(PacketType::Data.into());
    payload.payload_header = Some(get_payload_header(id, size));
    payload.payload_chunk = Some(data);
    return payload_to_offline(payload);
}
fn get_payload_header(id: usize, size: usize) -> PayloadHeader {
    let mut header = PayloadHeader::default();
    header.id = Some(id.try_into().unwrap());
    header.r#type = Some(PayloadType::Bytes.into());
    header.total_size = Some(size.try_into().unwrap());
    return header;
}
impl PayloadHandler {
    pub fn send_message<T: Message>(&mut self, message: &T) -> Vec<OfflineFrame> {
        let id = self.send_next_cnt;
        self.send_next_cnt += 1;
        let body = Bytes::from(message.encode_to_vec());
        let mut res = Vec::new();
        res.push(construct_payload_transfer_first(id, &body));
        res.push(construct_payload_transfer_end(id, body.len()));
        return res;
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
        if !self.incoming.contains_key(&id) {
            self.incoming.insert(id, default);
        }

        let mut incoming = self.incoming.get_mut(&id).unwrap();
        let data = chunk.body.as_ref().unwrap();
        let offset = chunk.offset();
        let len: i64 = data.len().try_into().unwrap();
        incoming.remaining_bytes -= len;
        let start: usize = offset.try_into().unwrap();
        for i in 0..data.len() {
            incoming.data[i + start] = data[i];
        }
        let l_chunk: i32 = Flags::LastChunk.into();
        if l_chunk == chunk.flags() && incoming.remaining_bytes == 0 {
            incoming.is_finished = true;
        }
    }
    pub fn get_next_payload<T: Message + Default>(&mut self) -> Option<T> {
        let mut res: Option<(i64, T)> = None;
        for (id, payload) in self.incoming.iter() {
            if payload.is_finished {
                if let Ok(msg) = T::decode(payload.data.clone()) {
                    res = Some((*id, msg));
                }
            }
        }
        match res {
            None => None,
            Some((id, msg)) => {
                self.incoming.remove(&id);
                return Some(msg);
            }
        }
    }
}
