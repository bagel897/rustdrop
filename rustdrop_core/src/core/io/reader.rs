use bytes::{Bytes, BytesMut};
use flume::{Receiver, Sender};
use prost::Message;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
use tracing::{debug, trace};

use crate::{core::errors::RustdropError, Context};
#[derive(Debug)]
struct ReaderSend<R: AsyncRead + Unpin> {
    reader: BufReader<R>,
    send: Sender<Bytes>,
}
impl<R: AsyncRead + Unpin> ReaderSend<R> {
    fn new(reader: R, send: Sender<Bytes>) -> Self {
        Self {
            reader: BufReader::new(reader),
            send,
        }
    }
    pub async fn read_messages(&mut self) {
        while let Ok(bytes) = self.read_data().await {
            if self.send.send_async(bytes).await.is_err() {
                break;
            }
        }
    }
    async fn read_data(&mut self) -> Result<Bytes, RustdropError> {
        let size: i32 = self
            .reader
            .read_i32()
            .await
            .map_err(|_| RustdropError::StreamClosed())?;
        let mut buf = BytesMut::zeroed(size.try_into().unwrap());
        self.reader
            .read_exact(&mut buf)
            .await
            .map_err(|_| RustdropError::StreamClosed())?;
        Ok(buf.into())
    }
}
#[derive(Clone)]
pub struct ReaderRecv {
    recv: Receiver<Bytes>,
}
impl ReaderRecv {
    pub fn new<R: AsyncRead + Unpin + Send + 'static>(reader: R, context: &Context) -> Self {
        let (send, recv) = flume::unbounded();
        context.spawn(
            async move {
                let mut sender = ReaderSend::new(reader, send);
                sender.read_messages().await;
            },
            "reader",
        );
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
        trace!("Raw message {:?}", raw);
        let res = T::decode(raw)?;
        if res.encoded_len() < 1000 {
            debug!("Recieved {:?}", res);
        }
        Ok(res)
    }
}
impl From<Receiver<Bytes>> for ReaderRecv {
    fn from(value: Receiver<Bytes>) -> Self {
        Self { recv: value }
    }
}
