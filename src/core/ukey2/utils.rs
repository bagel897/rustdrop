use prost::Message;

use crate::protobuf::{
    securegcm::{GcmMetadata, Type},
    securemessage::{
        EcP256PublicKey, EncScheme, GenericPublicKey, Header, PublicKeyType, SigScheme,
    },
};

use super::generic::Crypto;

pub fn get_header(iv: &[u8; 16]) -> Header {
    let mut metadata = GcmMetadata::default();
    metadata.version = Some(1);
    metadata.r#type = Type::DeviceToDeviceMessage.into();
    let mut header = Header::default();
    header.signature_scheme = SigScheme::HmacSha256.into();
    header.encryption_scheme = EncScheme::Aes256Cbc.into();
    header.iv = Some(iv.to_vec());
    header.public_metadata = Some(metadata.encode_to_vec());
    header
}
fn arr_to_protobuf(arr: &[u8]) -> Vec<u8> {
    let mut v = vec![0u8];
    v.extend_from_slice(arr);
    v
}
pub fn get_generic_pubkey<C: Crypto>(secretkey: &C::SecretKey) -> GenericPublicKey {
    let (x, y) = C::from_pubkey(secretkey);
    let pkey = EcP256PublicKey {
        x: x.to_vec(),
        y: y.to_vec(),
    };
    GenericPublicKey {
        r#type: PublicKeyType::EcP256.into(),
        ec_p256_public_key: Some(pkey),
        ..Default::default()
    }
}
