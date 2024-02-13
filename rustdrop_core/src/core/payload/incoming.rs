use std::collections::HashMap;

use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};
use tracing::debug;

use crate::{
    core::{IncomingFile, IncomingWifi},
    protobuf::sharing::nearby::{text_metadata, IntroductionFrame},
    Context, IncomingText,
};

use super::{traits::IncomingMeta, Payload};
#[derive(Debug, Clone, Default)]
pub struct Incoming {
    files: HashMap<i64, IncomingFile>,
    text: HashMap<i64, IncomingText>,
    wifi: HashMap<i64, IncomingWifi>,
}

impl Incoming {
    pub(crate) fn process_introduction(&mut self, introduction: IntroductionFrame) {
        for file in introduction.file_metadata {
            self.files.insert(file.payload_id(), file.into());
        }
        for text in introduction.text_metadata {
            self.text.insert(text.payload_id(), text.into());
        }
        for wifi in introduction.wifi_credentials_metadata {
            self.wifi.insert(wifi.payload_id(), wifi.into());
        }
    }
    async fn write_file(&mut self, payload: &mut Payload, context: &Context) {
        debug!("Writing payload {:?}", payload.id);
        let incoming = self.files.remove(&payload.id).unwrap();
        let dest = context.config.dest.clone();
        create_dir_all(dest.clone()).await.unwrap();
        let filepath = dest.join(incoming.name);
        let mut file = File::create(filepath).await.unwrap();
        file.write_all_buf(&mut payload.data).await.unwrap();
    }
    pub(crate) async fn process_payload(&mut self, payload: &mut Payload, context: &Context) -> bool {
        if self.files.contains_key(&payload.id) {
            self.write_file(payload, context).await;
            return true;
        }
        if let Some(mut incoming) = self.text.remove(&payload.id) {
            incoming
                .text
                .extend(String::from_utf8(payload.data.to_vec()));
            let mut ui = context.ui_write().await;
            match incoming.text_type {
                text_metadata::Type::Unknown => todo!(),
                text_metadata::Type::Text => {
                    ui.handle_text(incoming).await;
                }
                text_metadata::Type::Url => {
                    ui.handle_url(incoming).await;
                }
                text_metadata::Type::Address => {
                    ui.handle_address(incoming).await;
                }
                text_metadata::Type::PhoneNumber => {
                    ui.handle_phone(incoming).await;
                }
            };
            return true;
        }
        if self.wifi.contains_key(&payload.id) {
            todo!();
            return true;
        }
        false
    }
    pub(crate) fn is_finished(&self) -> bool {
        self.files.is_empty() && self.wifi.is_empty() && self.text.is_empty()
    }
    pub(crate) fn meta_type(&self) -> String {
        if let Some(text) = self.text.values().next() {
            return text.describe(self.text.len());
        }
        if let Some(file) = self.files.values().next() {
            return file.describe(self.files.len());
        }
        todo!();
    }
}
impl From<IntroductionFrame> for Incoming {
    fn from(value: IntroductionFrame) -> Self {
        let mut incoming = Self::default();
        incoming.process_introduction(value);
        incoming
    }
}
