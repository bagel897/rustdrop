use hkdf::Hkdf;
use p256::{
    ecdh::EphemeralSecret, elliptic_curve::sec1::FromEncodedPoint, AffinePoint, EncodedPoint,
    PublicKey,
};
use prost::{
    bytes::{BufMut, Bytes, BytesMut},
    Message,
};
use rand::rngs::OsRng;
use sha2::Sha256;
use tracing::info;

use crate::protobuf::securemessage::GenericPublicKey;

pub fn get_public_private() -> EphemeralSecret {
    EphemeralSecret::random(&mut OsRng)
}
pub fn get_public(raw: &[u8]) -> PublicKey {
    let generic = GenericPublicKey::decode(raw).unwrap();
    info!("Generic Key {:?}", generic);
    let key = generic.ec_p256_public_key.unwrap();
    let encoded_point = EncodedPoint::from_affine_coordinates(
        key.x.as_slice().into(),
        key.y.as_slice().into(),
        false,
    );
    let affine_point = AffinePoint::from_encoded_point(&encoded_point).unwrap();
    PublicKey::from_affine(affine_point).unwrap()
}
pub fn key_echange(
    client_pub: PublicKey,
    server_key: &EphemeralSecret,
    init: Bytes,
    resp: Bytes,
) -> (BytesMut, BytesMut) {
    let dhs = diffie_hellmen(client_pub, server_key);
    let mut xor = BytesMut::with_capacity(usize::max(init.len(), resp.len()));
    let default: u8 = 0x0;
    for i in 0..xor.capacity() {
        xor.put_bytes(
            init.get(i).unwrap_or(&default) ^ resp.get(i).unwrap_or(&default),
            1,
        )
    }
    let prk_auth = Hkdf::<Sha256>::new(Some("UKEY2 v1 auth".as_bytes()), &dhs);
    let prk_next = Hkdf::<Sha256>::new(Some("UKEY2 v1 next".as_bytes()), &dhs);
    let l_auth = 6;
    let l_next = 32;
    let mut auth_buf = BytesMut::zeroed(l_auth);
    let mut next_buf = BytesMut::zeroed(l_next);
    prk_auth.expand(&xor, &mut auth_buf).unwrap();
    prk_next.expand(&xor, &mut next_buf).unwrap();

    (auth_buf, next_buf)
}
fn diffie_hellmen(client_pub: PublicKey, server_key: &EphemeralSecret) -> Bytes {
    let shared = server_key.diffie_hellman(&client_pub);
    return Bytes::copy_from_slice(shared.raw_secret_bytes().as_slice());
}
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
        assert_eq!(
            diffie_hellmen(server_pubkey, &client_keypair),
            diffie_hellmen(client_pubkey, &server_keypair)
        );
    }
}
