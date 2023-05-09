use async_stream::stream;
use bytes::{Bytes, BytesMut};
use prost::{decode_length_delimiter, length_delimiter_len, Message};
use rand_new::{thread_rng, RngCore};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::OwnedReadHalf,
};
use tokio_stream::Stream;
use tracing::info;

use crate::protobuf::securegcm::{GcmMetadata, Type};
use crate::protobuf::securemessage::{EncScheme, Header, SigScheme};
pub fn get_header() -> Header {
    let mut metadata = GcmMetadata::default();
    metadata.version = Some(1);
    metadata.r#type = Type::DeviceToDeviceMessage.into();
    let mut header = Header::default();
    header.signature_scheme = SigScheme::HmacSha256.into();
    header.encryption_scheme = EncScheme::Aes256Cbc.into();
    header.iv = Some(get_random(16));
    header.public_metadata = Some(metadata.encode_length_delimited_to_vec());
    return header;
}
pub fn yield_from_stream(stream: &mut OwnedReadHalf) -> impl Stream<Item = Bytes> + '_ {
    stream! {
        let mut buf = BytesMut::with_capacity(1000);
        let s_idx: usize = 0;
        let mut e_idx: usize;
        loop {
            let mut new_data = BytesMut::with_capacity(1000);
            let r = stream.read_buf(&mut new_data).await.unwrap();
            if r == 0 {
                info!("Finished");
                break;
            }
            // info!("Reading {:#X}", new_data);
            buf.extend_from_slice(&new_data);
            let copy: BytesMut = buf.clone();
            if let Ok(len) = decode_length_delimiter(copy) {
                info!("Reading: buf {:#X}", buf);
                e_idx = s_idx + len + length_delimiter_len(len);
                if buf.len() >= e_idx {
                    let other_buf = buf.split_to(e_idx);
                    info!("Yielding {:#X}", other_buf);
                    yield other_buf.into();
                }
            }
        }
    }
}
pub fn get_random(bytes: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut resp_buf = vec![0u8; bytes];
    rng.fill_bytes(&mut resp_buf);
    return resp_buf;
}
pub async fn send<T: Message, S: AsyncWriteExt + Unpin>(stream: &mut S, message: &T) {
    let mut bytes = Bytes::from(message.encode_length_delimited_to_vec());
    info!("Sending {:#X}", bytes);
    stream.write_all_buf(&mut bytes).await.expect("Send Error");
}
