mod sender;

use std::{collections::HashMap, net::SocketAddr, time::Duration};

use anyhow::Error;
use bytes::Bytes;
use prost::{DecodeError, Message};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
    select,
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::debug;

use super::{
    errors::RustdropError, io::writer::WriterSend, util::get_random, Config, DeviceType, Payload,
};
use crate::{
    core::handlers::offline::keep_alive,
    protobuf::{
        location::nearby::connections::{OfflineFrame, V1Frame},
        securegcm::{ukey2_message::Type, Ukey2Alert, Ukey2Message},
        sharing::nearby::{
            self, paired_key_result_frame::Status, text_metadata, v1_frame::FrameType,
            wifi_credentials_metadata::SecurityType, Frame, IntroductionFrame,
            PairedKeyEncryptionFrame, PairedKeyResultFrame,
        },
    },
    Application, UiHandle,
};

pub(crate) fn decode_endpoint_id(endpoint_id: &[u8]) -> Result<(DeviceType, String), Error> {
    if endpoint_id.len() < 18 {
        return Err(RustdropError::InvalidEndpointId().into());
    }
    let (first, second) = endpoint_id.split_at(18);
    let bits = first.first().unwrap() >> 1 & 0x03;
    let devtype = DeviceType::from(bits);
    let name = String::from_utf8(second.to_vec())?;
    Ok((devtype, name))
}
fn get_devtype_bit(devtype: DeviceType) -> u8 {
    match devtype {
        DeviceType::Unknown => 0,
        DeviceType::Phone => 1,
        DeviceType::Tablet => 2,
        DeviceType::Laptop => 3,
    }
}
fn get_bitfield(devtype: DeviceType) -> u8 {
    get_devtype_bit(devtype) << 1
}
pub(crate) fn get_endpoint_info(config: &Config) -> Vec<u8> {
    let mut data: Vec<u8> = thread_rng().sample_iter(&Alphanumeric).take(17).collect();
    data[0] = get_bitfield(config.devtype);
    let mut encoded = config.name.as_bytes().to_vec();
    data.push(encoded.len() as u8);
    data.append(&mut encoded);
    data
}
pub(crate) fn get_offline_frame(v1: V1Frame) -> OfflineFrame {
    OfflineFrame {
        version: Some(1),
        v1: Some(v1),
    }
}
pub(crate) fn get_online_frame(v1: nearby::V1Frame) -> Frame {
    Frame {
        version: Some(1),
        v1: Some(v1),
    }
}
pub(crate) fn get_paired_result() -> Frame {
    let res = PairedKeyResultFrame {
        status: Some(Status::Unable.into()),
    };
    let v1 = nearby::V1Frame {
        r#type: Some(FrameType::PairedKeyResult.into()),
        paired_key_result: Some(res),
        ..Default::default()
    };
    get_online_frame(v1)
}
pub fn get_paired_frame() -> Frame {
    let p_key = PairedKeyEncryptionFrame {
        secret_id_hash: Some(get_random(6)),
        signed_data: Some(get_random(72)),
        ..Default::default()
    };
    let v1 = nearby::V1Frame {
        r#type: Some(FrameType::PairedKeyEncryption.into()),
        paired_key_encryption: Some(p_key),
        ..Default::default()
    };
    get_online_frame(v1)
}
pub(crate) fn try_decode_ukey2_alert(raw: &Bytes) -> Result<Ukey2Alert, DecodeError> {
    if let Ok(message) = Ukey2Message::decode(raw.clone()) {
        if message.message_type() == Type::Alert {
            let message = Ukey2Alert::decode(message.message_data())?;
            return Ok(message);
        }
    }
    let message = Ukey2Alert::decode(raw.clone())?;
    Ok(message)
}
#[derive(Debug, Clone)]
pub struct IncomingText {
    pub name: String,
    pub text_type: text_metadata::Type,
    pub size: i64,
    pub text: String,
}
#[derive(Debug, Clone)]
pub struct IncomingFile {
    pub name: String,
    pub mime_type: String,
    pub size: i64,
}
#[derive(Debug, Clone)]
pub struct IncomingWifi {
    pub ssid: String,
    pub security_type: SecurityType,
}
#[derive(Debug)]
pub struct PairingRequest {
    device_name: String,
    device_type: DeviceType,
    files: HashMap<i64, IncomingFile>,
    text: HashMap<i64, IncomingText>,
    wifi: HashMap<i64, IncomingWifi>,
}

impl PairingRequest {
    pub fn new(endpoint_info: &[u8]) -> Result<Self, Error> {
        let (devtype, name) = decode_endpoint_id(endpoint_info)?;
        Ok(PairingRequest {
            device_name: name,
            device_type: devtype,
            files: HashMap::new(),
            text: HashMap::new(),
            wifi: HashMap::new(),
        })
    }
    pub(crate) async fn process_payload<U: UiHandle>(
        &mut self,
        payload: &mut Payload,
        app: &Application<U>,
    ) -> bool {
        if self.files.contains_key(&payload.id) {
            self.write_file(payload, app).await;
            return true;
        }
        if let Some(mut incoming) = self.text.remove(&payload.id) {
            incoming
                .text
                .extend(String::from_utf8(payload.data.to_vec()));
            let mut ui = app.ui_write().await;
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
    async fn write_file<U: UiHandle>(&mut self, payload: &mut Payload, app: &Application<U>) {
        debug!("Writing payload {:?}", payload.id);
        let incoming = self.files.remove(&payload.id).unwrap();
        let dest = app.config.dest.clone();
        create_dir_all(dest.clone()).await.unwrap();
        let filepath = dest.join(incoming.name);
        let mut file = File::create(filepath).await.unwrap();
        file.write_all_buf(&mut payload.data).await.unwrap();
    }
    pub(crate) fn process_introduction(&mut self, introduction: IntroductionFrame) {
        for file in introduction.file_metadata {
            self.files.insert(
                file.payload_id(),
                IncomingFile {
                    name: file.name().to_string(),
                    mime_type: file.mime_type().into(),
                    size: file.size(),
                },
            );
        }
        for text in introduction.text_metadata {
            self.text.insert(
                text.payload_id(),
                IncomingText {
                    name: text.text_title().into(),
                    text_type: text.r#type(),
                    size: text.size(),
                    text: String::new(),
                },
            );
        }
        for wifi in introduction.wifi_credentials_metadata {
            self.wifi.insert(
                wifi.payload_id(),
                IncomingWifi {
                    ssid: wifi.ssid().into(),
                    security_type: wifi.security_type(),
                },
            );
        }
    }
    pub(crate) fn is_finished(&self) -> bool {
        self.files.is_empty() && self.wifi.is_empty() && self.text.is_empty()
    }
    pub fn name(&self) -> String {
        "Nearby Sharing".into()
    }
    pub fn body(&self) -> String {
        if let Some(text) = self.text.values().next() {
            let text = match text.text_type {
                crate::TextType::Unknown => todo!(),
                crate::TextType::Text => "some text",
                crate::TextType::Url => "a link",
                crate::TextType::Address => "an address",
                crate::TextType::PhoneNumber => "a phone number",
            };
            return format!("{} wants to share {} with you", self.device_name, text);
        }
        format!("{} wants to share a file with you", self.device_name)
    }
}
#[derive(Debug, Clone)]
pub struct Device {
    pub device_name: String,
    pub device_type: DeviceType,
    pub ip: SocketAddr,
}
pub(crate) async fn repeat_keep_alive(writer: WriterSend, cancel: CancellationToken) {
    loop {
        select! {
            _ = cancel.cancelled() => { break;},
            _ = sleep(Duration::from_secs(10)) => {
                let msg = keep_alive();
                writer.send(&msg).await;
        },
        }
    }
}
