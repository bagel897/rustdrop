use std::{collections::HashMap, path::PathBuf};

use async_stream::stream;
use bytes::Bytes;
use futures::Stream;
use tokio::{fs::File, io::AsyncReadExt};

use super::{id::get_payload, traits::IncomingMeta};
use crate::{
    core::protocol::get_online_frame,
    protobuf::nearby::sharing::service::{v1_frame::FrameType, Frame, IntroductionFrame, V1Frame},
    IncomingFile, IncomingText, IncomingWifi,
};
// Metadata for Outgoing media
#[derive(Debug, Clone, Default)]
struct OutgoingMeta {
    pub files: HashMap<i64, IncomingFile>,
    pub text: HashMap<i64, IncomingText>,
    pub wifi: HashMap<i64, IncomingWifi>,
}

#[derive(Debug, Default, Clone)]
pub struct Outgoing {
    meta: OutgoingMeta,
    payloads: HashMap<i64, Bytes>,
    file_payloads: HashMap<i64, PathBuf>,
}
impl Outgoing {
    pub fn add_file(&mut self, path: PathBuf) {
        let payload_id = get_payload();
        let incoming = IncomingFile::from(path.clone());
        self.meta.files.insert(payload_id, incoming);
        self.file_payloads.insert(payload_id, path);
    }
    pub(crate) fn get_frames(self) -> (Frame, impl Stream<Item = (i64, Bytes)>) {
        let intro = self.meta.into();
        let payloads = self.payloads;
        let file_payloads = self.file_payloads;
        let v1 = V1Frame {
            r#type: Some(FrameType::Introduction.into()),
            introduction: Some(intro),
            ..Default::default()
        };
        let frame = get_online_frame(v1);
        (
            frame,
            stream! {
                for (id, payload) in payloads.into_iter() {
                    yield (id, payload)
                }
                for (id, path) in file_payloads.into_iter() {
                    let mut buf = vec![];
                    let mut file = File::open(path).await.unwrap();
                    file.read_to_end(&mut buf).await.unwrap();
                    yield (id, buf.into())
                }
            },
        )
    }
    pub fn len(&self) -> usize {
        self.file_payloads.len() + self.payloads.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl From<OutgoingMeta> for IntroductionFrame {
    fn from(val: OutgoingMeta) -> Self {
        // TODO: App support
        IntroductionFrame {
            start_transfer: Some(true),
            app_metadata: [].into(),
            file_metadata: val
                .files
                .into_iter()
                .map(|(payload_id, data)| data.into_proto_type(payload_id))
                .collect(),
            text_metadata: val
                .text
                .into_iter()
                .map(|(payload_id, data)| data.into_proto_type(payload_id))
                .collect(),
            required_package: None,
            wifi_credentials_metadata: val
                .wifi
                .into_iter()
                .map(|(payload_id, data)| data.into_proto_type(payload_id))
                .collect(),
        }
    }
}
