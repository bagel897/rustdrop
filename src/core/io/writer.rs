use bytes::{BufMut, Bytes, BytesMut};
use prost::Message;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tracing::info;

use crate::protobuf::securegcm::{ukey2_message::Type, Ukey2Message};

pub trait SecureWriteExt: AsyncWrite + Unpin {
    async fn send<T: Message>(&mut self, message: &T)
    where
        Self: Sized,
    {
        info!("{:?}", message);
        let mut bytes = BytesMut::with_capacity(message.encoded_len() + 4);
        bytes.put_i32(message.encoded_len().try_into().unwrap());
        bytes.extend_from_slice(message.encode_to_vec().as_slice());
        info!("Sending {:#X}", bytes);
        self.write_all_buf(&mut bytes).await.expect("Send Error");
    }
    async fn send_ukey2<T: Message>(&mut self, message: &T, message_type: Type) -> Bytes
    where
        Self: Sized,
    {
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
impl<T: AsyncWrite + Unpin> SecureWriteExt for T {}
