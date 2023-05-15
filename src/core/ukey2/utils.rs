use crate::protobuf::securegcm::{GcmMetadata, Type};
use crate::protobuf::securemessage::{EncScheme, Header, SigScheme};
use prost::Message;

pub fn get_header(iv: &[u8; 16]) -> Header {
    let mut metadata = GcmMetadata::default();
    metadata.version = Some(1);
    metadata.r#type = Type::DeviceToDeviceMessage.into();
    let mut header = Header::default();
    header.signature_scheme = SigScheme::HmacSha256.into();
    header.encryption_scheme = EncScheme::Aes256Cbc.into();
    header.iv = Some(iv.to_vec());
    header.public_metadata = Some(metadata.encode_length_delimited_to_vec());
    return header;
}
