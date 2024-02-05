use bytes::{BufMut, Bytes, BytesMut};
use prost::Message;
use tokio::{
    io::{AsyncWrite, AsyncWriteExt, BufWriter},
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tracing::{debug, info};

use crate::{
    protobuf::securegcm::{ukey2_message::Type, Ukey2Message},
    Application, UiHandle,
};

struct WriterRecv<T: AsyncWrite> {
    underlying: BufWriter<T>,
    recv: UnboundedReceiver<Bytes>,
}
impl<T: AsyncWrite + Unpin> WriterRecv<T> {
    async fn write_next(&mut self) {
        while let Some(mut msg) = self.recv.recv().await {
            self.underlying.write_all_buf(&mut msg).await.unwrap();
            self.underlying.flush().await.unwrap();
        }
    }
}
#[derive(Clone, Debug)]
pub struct WriterSend {
    send: UnboundedSender<Bytes>,
}
impl WriterSend {
    pub fn new<T: AsyncWrite + Unpin + Send + 'static, U: UiHandle>(
        underlying: T,
        application: &mut Application<U>,
    ) -> WriterSend {
        let (send, recv) = mpsc::unbounded_channel();
        let writer = WriterSend { send };
        application.spawn(
            async move {
                let mut reciever = WriterRecv {
                    recv,
                    underlying: BufWriter::new(underlying),
                };
                reciever.write_next().await;
            },
            "writer",
        );
        writer
    }
    pub async fn send<T: Message>(&self, message: &T) {
        info!("{:?}", message);
        let mut bytes = BytesMut::with_capacity(message.encoded_len() + 4);
        bytes.put_i32(message.encoded_len().try_into().unwrap());
        bytes.extend_from_slice(message.encode_to_vec().as_slice());
        debug!("Sending {:#X}", bytes);
        let res: Bytes = bytes.into();
        self.send.send(res).unwrap();
    }
    pub async fn send_ukey2<T: Message>(&self, message: &T, message_type: Type) -> Bytes {
        info!("{:?}", message);
        let message_data = Some(message.encode_to_vec());
        let ukey = Ukey2Message {
            message_type: Some(message_type.into()),
            message_data,
        };
        self.send(&ukey).await;
        ukey.encode_to_vec().into()
    }
}
