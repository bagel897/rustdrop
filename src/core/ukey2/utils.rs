use crate::protobuf::securegcm::{GcmMetadata, Type};
use crate::protobuf::securemessage::{
    EcP256PublicKey, EncScheme, GenericPublicKey, Header, PublicKeyType, SigScheme,
};
use p256::ecdh::EphemeralSecret;
use p256::EncodedPoint;
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
pub fn get_generic_pubkey(secret: &EphemeralSecret) -> GenericPublicKey {
    let pubkey = secret.public_key();
    let point = EncodedPoint::from(pubkey);
    let mut pkey = EcP256PublicKey::default();
    pkey.x = point.x().unwrap().to_vec();
    pkey.x = point.y().unwrap().to_vec();
    let mut res = GenericPublicKey::default();
    res.r#type = PublicKeyType::EcP256.into();
    res.ec_p256_public_key = Some(pkey);
    return res;
}
