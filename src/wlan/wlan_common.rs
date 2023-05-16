use crate::core::ukey2::Ukey2;
use crate::protobuf::securemessage::SecureMessage;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};
use tracing::info;
fn decode_32_len(buf: &Bytes) -> Result<usize, ()> {
    if buf.len() < 4 {
        return Err(());
    }
    let mut arr = [0u8; 4];
    for i in 0..4 {
        arr[i] = buf[i];
    }
    return Ok(i32::from_be_bytes(arr) as usize);
}
pub(super) struct StreamHandler {
    read_half: OwnedReadHalf,
    write_half: OwnedWriteHalf,
    buf: BytesMut,
    ukey2: Option<Ukey2>,
}
impl StreamHandler {
    pub fn new(stream: TcpStream) -> Self {
        let (read_half, write_half) = stream.into_split();
        let buf = BytesMut::with_capacity(1000);
        StreamHandler {
            read_half,
            write_half,
            buf,
            ukey2: None,
        }
    }
    pub fn setup_ukey2(&mut self, ukey2: Ukey2) {
        self.ukey2 = Some(ukey2);
    }
    pub fn decrypt_message<T: Message + Default>(&mut self, message: &SecureMessage) -> T {
        return self.ukey2.as_mut().unwrap().decrypt_message::<T>(message);
    }
    pub async fn send<T: Message>(&mut self, message: &T) {
        info!("{:?}", message);
        let mut bytes = BytesMut::with_capacity(message.encoded_len() + 4);
        bytes.put_i32(message.encoded_len().try_into().unwrap());
        bytes.extend_from_slice(message.encode_to_vec().as_slice());
        info!("Sending {:#X}", bytes);
        self.write_half
            .write_all_buf(&mut bytes)
            .await
            .expect("Send Error");
    }
    pub async fn send_frame<T: Message>(&mut self, message: &T) {
        info!("{:?}", message);
        let encrypted = self.ukey2.as_mut().unwrap().encrypt_message(message);
        self.send(&encrypted).await;
    }
    fn try_yield_message(&mut self) -> Option<Bytes> {
        if let Ok(len) = decode_32_len(&self.buf.clone().into()) {
            return self.decode_message(len);
        }
        None
    }
    async fn read_data(&mut self) -> Result<(), ()> {
        let mut new_data = BytesMut::with_capacity(1000);
        let r = self.read_half.read_buf(&mut new_data).await.unwrap();
        if r == 0 {
            info!("Finished");
            return Err(());
        }
        self.buf.extend_from_slice(&new_data);
        Ok(())
    }

    fn decode_message(&mut self, len: usize) -> Option<Bytes> {
        info!("Reading: buf {:#X}", self.buf);
        let e_idx = len + 4;
        if self.buf.len() >= e_idx {
            let mut other_buf = self.buf.split_to(e_idx);
            other_buf.advance(4);
            info!("Yielding {:#X}", other_buf);
            return Some(other_buf.into());
        }
        None
    }
    pub async fn shutdown(&mut self) {
        self.write_half.shutdown().await.unwrap();
    }
    pub async fn next(&mut self) -> Result<Bytes, ()> {
        if let Some(bytes) = self.try_yield_message() {
            return Ok(bytes);
        }
        loop {
            self.read_data().await?;
            if let Some(bytes) = self.try_yield_message() {
                return Ok(bytes);
            }
        }
    }
    pub async fn next_message<T: Message + Default>(&mut self) -> Result<T, ()> {
        let raw = self.next().await?;
        Ok(T::decode(raw).unwrap())
    }
}
