pub mod file;
mod id;
pub mod incoming;
pub mod outgoing;
pub mod text;
pub mod traits;
pub mod wifi;
use std::collections::HashMap;

use bytes::{Bytes, BytesMut};
use prost::Message;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    oneshot::{self, Receiver, Sender},
};
use tracing::{debug, error, info};

use self::id::get_payload;
use super::{protocol::get_offline_frame, RustdropError};
use crate::{
    protobuf::{
        location::nearby::connections::{
            payload_transfer_frame::{
                payload_chunk::{self, Flags},
                payload_header::PayloadType,
                PacketType, PayloadChunk, PayloadHeader,
            },
            v1_frame::FrameType,
            DisconnectionFrame, KeepAliveFrame, OfflineFrame, PayloadTransferFrame, V1Frame,
        },
        sharing::nearby::Frame,
    },
    Context,
};
#[derive(Debug)]
struct Incoming {
    pub data: BytesMut,
    pub remaining_bytes: i64,
    pub is_finished: bool,
}
#[derive(Debug)]
pub struct Payload {
    pub data: Bytes,
    pub id: i64,
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
    send: UnboundedSender<Payload>,
    disconnect: Sender<DisconnectionFrame>,
}
#[derive(Debug)]
pub struct PayloadRecieverHandle {
    recv: UnboundedReceiver<Payload>,
    disconnect: Receiver<DisconnectionFrame>,
}
#[derive(Debug)]
pub struct PayloadSender {
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
        Self { send }
    }
    pub fn send_unencrypted(&mut self, message: OfflineFrame) {
        self.send.send(message).unwrap();
    }
    pub fn send_raw(&mut self, data: Bytes, payload_id: i64) {
        let len: i64 = data.len().try_into().unwrap();
        let header = get_payload_header(payload_id, len);
        self.send
            .send(construct_payload_transfer_first(&data, header.clone()))
            .unwrap();
        self.send
            .send(construct_payload_transfer_end(header, len))
            .unwrap();
    }
    pub fn send_message(&mut self, message: &Frame) {
        let id = get_payload();
        let body = Bytes::from(message.encode_to_vec());
        self.send_raw(body, id);
    }
}
impl PayloadReciever {
    pub fn push_frames(
        incoming: UnboundedReceiver<OfflineFrame>,
        context: &mut Context,
    ) -> PayloadRecieverHandle {
        let (send, recv) = mpsc::unbounded_channel();
        let (tx, rx) = oneshot::channel();
        let handle = PayloadRecieverHandle {
            recv,
            disconnect: rx,
        };
        context.spawn(
            async {
                let reciver = PayloadReciever {
                    incoming: HashMap::default(),
                    send,
                    disconnect: tx,
                };
                reciver.handle_frames(incoming).await;
            },
            "payload",
        );
        handle
    }
    async fn handle_frames(mut self, mut incoming: UnboundedReceiver<OfflineFrame>) {
        while let Some(msg) = incoming.recv().await {
            let v1 = msg.v1.unwrap();
            match v1.r#type() {
                FrameType::PayloadTransfer => self.push_data(v1.payload_transfer.unwrap()),
                FrameType::KeepAlive => self.handle_keep_alive(v1.keep_alive.unwrap()),
                FrameType::Disconnection => {
                    self.disconnect.send(v1.disconnection.unwrap()).unwrap();
                    info!("Disconnecting");
                    return;
                }
                _ => {
                    error!("Recieved unhandlable frame {:?}", v1);
                    todo!()
                }
            };
            self.get_next_payload();
        }
        debug!("No more frames to handle");
    }

    fn handle_keep_alive(&mut self, _alive: KeepAliveFrame) {}
    fn push_data(&mut self, data: PayloadTransferFrame) {
        let header = data.payload_header.unwrap();
        let chunk = data.payload_chunk.unwrap();
        let id = header.id();
        let size = header.total_size();
        let default = Incoming::new(size);
        self.incoming.entry(id).or_insert(default);

        let incoming = self.incoming.get_mut(&id).unwrap();
        let offset = chunk.offset();
        let flags = chunk.flags();
        if let Some(data) = chunk.body {
            let len: i64 = data.len().try_into().unwrap();
            incoming.remaining_bytes -= len;
            let start: usize = offset.try_into().unwrap();
            for i in 0..data.len() {
                incoming.data[i + start] = data[i];
            }
        }
        if flags == i32::from(Flags::LastChunk) && incoming.remaining_bytes == 0 {
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
            let incoming = self.incoming.remove(&id).unwrap();
            let payload = Payload {
                data: incoming.data.into(),
                id,
            };
            self.send.send(payload).unwrap();
        }
    }
}
impl PayloadRecieverHandle {
    pub async fn get_next_raw(&mut self) -> Result<Payload, RustdropError> {
        self.recv.recv().await.ok_or(RustdropError::StreamClosed())
    }
    pub async fn wait_for_disconnect(self) -> Result<DisconnectionFrame, RustdropError> {
        self.disconnect
            .await
            .map_err(|_| RustdropError::StreamClosed())
    }
    pub async fn get_next_payload(&mut self) -> Result<Frame, RustdropError> {
        let raw = self.get_next_raw().await?;
        let frame = Frame::decode(raw.data)?;
        info!("Recieved message {:?}", frame);
        Ok(frame)
    }
}
