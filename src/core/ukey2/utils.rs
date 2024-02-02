use bytes::{BufMut, Bytes, BytesMut};
use num_bigint::{BigUint, ToBigInt};
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
fn _encode(unsigned: Bytes) -> Vec<u8> {
    let mut res = BytesMut::with_capacity(unsigned.len());
    res.put_u8(0x0);
    res.extend_from_slice(&unsigned);
    return res.to_vec();
    // let u = BigUint::from_bytes_be(&unsigned);
    // let i = u.to_bigint().unwrap();
    // i.to_signed_bytes_be()
}
pub fn get_generic_pubkey<C: Crypto>(secretkey: &C::SecretKey) -> GenericPublicKey {
    let (x, y) = C::from_pubkey(secretkey);
    let pkey = EcP256PublicKey {
        x: _encode(x),
        y: _encode(y),
    };
    GenericPublicKey {
        r#type: PublicKeyType::EcP256.into(),
        ec_p256_public_key: Some(pkey),
        ..Default::default()
    }
}
