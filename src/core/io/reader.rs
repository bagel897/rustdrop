use bytes::{Buf, Bytes, BytesMut};
use prost::Message;
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::{debug, info, trace};

use crate::core::{
    protocol::try_decode_ukey2_alert, util::ukey_alert_to_str, TcpStreamClosedError,
};
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
pub struct BufferedReader<R: AsyncRead + Unpin> {
    reader: R,
    buf: BytesMut,
}
impl<R: AsyncRead + Unpin> BufferedReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: BytesMut::with_capacity(1000),
        }
    }
    fn try_yield_message(&mut self) -> Option<Bytes> {
        if let Ok(len) = decode_32_len(&self.buf.clone().into()) {
            return self.decode_message(len);
        }
        None
    }
    pub async fn next(&mut self) -> Result<Bytes, TcpStreamClosedError> {
        if let Some(bytes) = self.try_yield_message() {
            return Ok(bytes);
        }
        loop {
            let r = self.read_data().await;
            if let Some(bytes) = self.try_yield_message() {
                return Ok(bytes);
            }
            if r.is_err() {
                info!("Stream is finished");
                return Err(r.err().unwrap());
            }
        }
    }
    async fn read_data(&mut self) -> Result<(), TcpStreamClosedError> {
        let mut new_data = BytesMut::with_capacity(1000);
        let r = self.reader.read_buf(&mut new_data).await.unwrap();
        if r == 0 {
            info!("No data left");
            return Err(TcpStreamClosedError {});
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
    pub async fn next_message<T: Message + Default>(&mut self) -> Result<T, TcpStreamClosedError> {
        let raw = self.next().await?;
        if let Ok(a) = try_decode_ukey2_alert(&raw) {
            info!("{:?}", ukey_alert_to_str(a))
        }
        // TODO: error handling
        Ok(T::decode(raw.clone())
            // .map_err(|e| self.try_handle_ukey(e, &raw))
            .unwrap())
    }
}
