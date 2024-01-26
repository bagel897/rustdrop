use hkdf::Hkdf;
use p256::{
    ecdh::EphemeralSecret, elliptic_curve::sec1::FromEncodedPoint, EncodedPoint, PublicKey,
};
use prost::{
    bytes::{BufMut, Bytes, BytesMut},
    Message,
};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use tracing::info;

use crate::protobuf::securemessage::GenericPublicKey;

pub fn get_public_private() -> EphemeralSecret {
    EphemeralSecret::random(&mut OsRng)
}
fn trim_to_32(raw: &Vec<u8>) -> &[u8] {
    &raw[raw.len() - 32..]
}
pub fn get_public(raw: &[u8]) -> PublicKey {
    let generic = GenericPublicKey::decode(raw).unwrap();
    let key = generic.ec_p256_public_key.as_ref().unwrap();
    info!(
        "Generic Key {:?} x_size {} y_size {}",
        generic,
        key.x.as_slice().len(),
        key.y.as_slice().len()
    );
    let x = trim_to_32(&key.x).into();
    let y = trim_to_32(&key.y).into();
    let encoded_point = EncodedPoint::from_affine_coordinates(x, y, false);
    PublicKey::from_encoded_point(&encoded_point).unwrap()
}
pub fn key_echange(
    client_pub: PublicKey,
    server_key: &EphemeralSecret,
    init: Bytes,
    resp: Bytes,
) -> (BytesMut, BytesMut) {
    let dhs = server_key.diffie_hellman(&client_pub);
    let mut xor = BytesMut::new();
    xor.extend_from_slice(&init);
    xor.extend_from_slice(&resp);
    let prk_auth = dhs.extract::<Sha256>(Some("UKEY2 v1 auth".as_bytes()));
    let prk_next = dhs.extract::<Sha256>(Some("UKEY2 v1 next".as_bytes()));
    let l_auth = 32;
    let l_next = 32;
    let mut auth_buf = BytesMut::zeroed(l_auth);
    let mut next_buf = BytesMut::zeroed(l_next);
    prk_auth.expand(&xor, &mut auth_buf).unwrap();
    prk_next.expand(&xor, &mut next_buf).unwrap();

    (auth_buf, next_buf)
}
// fn diffie_hellmen(client_pub: PublicKey, server_key: &EphemeralSecret) -> Bytes {
//     let shared = server_key.diffie_hellman(&client_pub);
//     let mut hasher = Sha256::new();
//     hasher.update(shared.raw_secret_bytes());
//     let result = hasher.finalize();
//     Bytes::from_static(result.as_slice()).clone()
// }
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_key_gen() {
        let _keypair = get_public_private();
    }
    #[test]
    fn test_diffie_hellman() {
        let server_keypair = get_public_private();
        let client_keypair = get_public_private();
        let server_pubkey = PublicKey::from(&server_keypair);
        let client_pubkey = PublicKey::from(&client_keypair);
        // assert_eq!(
        //     diffie_hellmen(server_pubkey, &client_keypair),
        //     diffie_hellmen(client_pubkey, &server_keypair)
        // );
    }
}
