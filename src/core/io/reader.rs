use bytes::{Buf, Bytes, BytesMut};
use flume::{Receiver, Sender};
use prost::Message;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
use tracing::{debug, trace};

use crate::{core::errors::RustdropError, Application, UiHandle};
pub fn decode_32_len(buf: &Bytes) -> Result<usize, ()> {
    if buf.len() < 4 {
        return Err(());
    }
    let mut arr = [0u8; 4];
    for i in 0..4 {
        arr[i] = buf[i];
    }
    Ok(i32::from_be_bytes(arr) as usize)
}

#[derive(Debug)]
struct ReaderSend<R: AsyncRead + Unpin> {
    reader: BufReader<R>,
    buf: BytesMut,
    send: Sender<Bytes>,
}
impl<R: AsyncRead + Unpin> ReaderSend<R> {
    fn new(reader: R, send: Sender<Bytes>) -> Self {
        Self {
            reader: BufReader::new(reader),
            buf: BytesMut::with_capacity(1000),
            send,
        }
    }
    fn try_yield_message(&mut self) -> Option<Bytes> {
        if let Ok(len) = decode_32_len(&self.buf.clone().into()) {
            return self.decode_message(len);
        }
        None
    }
    pub async fn read_messages(&mut self) -> Result<(), RustdropError> {
        loop {
            self.read_data().await?;
            if let Some(bytes) = self.try_yield_message() {
                if self.send.send_async(bytes).await.is_err() {
                    return Ok(());
                }
            }
        }
    }
    async fn read_data(&mut self) -> Result<(), RustdropError> {
        let mut new_data = BytesMut::with_capacity(1000);
        let r = self.reader.read_buf(&mut new_data).await.unwrap();
        if r == 0 {
            return Err(RustdropError::StreamClosed());
        }
        self.buf.extend_from_slice(&new_data);
        Ok(())
    }

    fn decode_message(&mut self, len: usize) -> Option<Bytes> {
        debug!("Reading: buf {:#X}", self.buf);
        let e_idx = len + 4;
        if self.buf.len() >= e_idx {
            let mut other_buf = self.buf.split_to(e_idx);
            other_buf.advance(4);
            trace!("Yielding {:#X} len {}", other_buf, len);
            assert_eq!(other_buf.len(), len);
            return Some(other_buf.into());
        }
        None
    }
}
#[derive(Clone)]
pub struct ReaderRecv {
    recv: Receiver<Bytes>,
}
impl ReaderRecv {
    pub fn new<R: AsyncRead + Unpin + Send + 'static, U: UiHandle>(
        reader: R,
        application: &Application<U>,
    ) -> Self {
        let (send, recv) = flume::unbounded();
        application.tracker.spawn(async move {
            let mut sender = ReaderSend::new(reader, send);
            sender.read_messages().await.unwrap();
        });
        Self { recv }
    }
    pub async fn next(&self) -> Result<Bytes, RustdropError> {
        self.recv
            .recv_async()
            .await
            .map_err(|_| RustdropError::StreamClosed())
    }
    pub async fn next_message<T: Message + Default>(&self) -> Result<T, RustdropError> {
        let raw = self.next().await?;
        Ok(T::decode(raw)?)
    }
}
impl From<Receiver<Bytes>> for ReaderRecv {
    fn from(value: Receiver<Bytes>) -> Self {
        Self { recv: value }
    }
}
