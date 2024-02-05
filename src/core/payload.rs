use std::collections::HashMap;

use bytes::{Bytes, BytesMut};
use prost::Message;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tracing::info;

use super::{protocol::get_offline_frame, RustdropError};
use crate::{
    protobuf::{
        location::nearby::connections::{
            payload_transfer_frame::{
                payload_chunk::{self},
                payload_header::PayloadType,
                PacketType, PayloadChunk, PayloadHeader,
            },
            v1_frame::FrameType,
            KeepAliveFrame, OfflineFrame, PayloadTransferFrame, V1Frame,
        },
        sharing::nearby::Frame,
    },
    Application, UiHandle,
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
#[derive(Debug)]
pub struct PayloadReciever {
    incoming: HashMap<i64, Incoming>,
    send: UnboundedSender<Bytes>,
}
#[derive(Debug)]
pub struct PayloadRecieverHandle {
    recv: UnboundedReceiver<Bytes>,
}
#[derive(Debug)]
pub struct PayloadSender {
    send_next_cnt: i64,
    send: UnboundedSender<OfflineFrame>,
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
impl PayloadSender {
    pub fn new(send: UnboundedSender<OfflineFrame>) -> Self {
        Self {
            send_next_cnt: 0,
            send,
        }
    }
    pub fn send_message(&mut self, message: &Frame) {
        let id = self.send_next_cnt;
        self.send_next_cnt += 1;
        let body = Bytes::from(message.encode_to_vec());
        let len: i64 = body.len().try_into().unwrap();
        let header = get_payload_header(id, len);
        self.send
            .send(construct_payload_transfer_first(&body, header.clone()))
            .unwrap();
        self.send
            .send(construct_payload_transfer_end(header, len))
            .unwrap();
    }
}
impl PayloadReciever {
    pub fn push_frames<U: UiHandle>(
        incoming: UnboundedReceiver<OfflineFrame>,
        app: &mut Application<U>,
    ) -> PayloadRecieverHandle {
        let (send, recv) = mpsc::unbounded_channel();
        let handle = PayloadRecieverHandle { recv };
        app.spawn(
            async {
                let mut reciver = PayloadReciever {
                    incoming: HashMap::default(),
                    send,
                };
                reciver.handle_frames(incoming).await;
            },
            "payload",
        );
        handle
    }
    async fn handle_frames(&mut self, mut incoming: UnboundedReceiver<OfflineFrame>) {
        while let Some(msg) = incoming.recv().await {
            self.handle_frame(msg);
        }
    }

    fn handle_frame(&mut self, frame: OfflineFrame) {
        let v1 = frame.v1.unwrap();
        match v1.r#type() {
            FrameType::PayloadTransfer => self.push_data(v1.payload_transfer.unwrap()),
            FrameType::KeepAlive => self.handle_keep_alive(v1.keep_alive.unwrap()),
            _ => todo!(),
        };
        self.get_next_payload();
    }
    fn handle_keep_alive(&mut self, alive: KeepAliveFrame) {}
    fn push_data(&mut self, data: PayloadTransferFrame) {
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
        }
        if incoming.remaining_bytes == 0 {
            incoming.is_finished = true;
        }
    }
    pub fn get_next_payload(&mut self) {
        let mut to_remove = Vec::default();
        for (id, payload) in self.incoming.iter() {
            if payload.is_finished {
                to_remove.push(*id);
            }
        }
        for id in to_remove {
            let payload = self.incoming.remove(&id).unwrap();
            self.send.send(payload.data.into()).unwrap();
        }
    }
}
impl PayloadRecieverHandle {
    pub async fn get_next_raw(&mut self) -> Result<Bytes, RustdropError> {
        self.recv.recv().await.ok_or(RustdropError::StreamClosed())
    }
    pub async fn get_next_payload(&mut self) -> Result<Frame, RustdropError> {
        let raw = self.get_next_raw().await?;
        let frame = Frame::decode(raw).expect("BRUH");
        info!("Recieved message {:?}", frame);
        Ok(frame)
    }
}
