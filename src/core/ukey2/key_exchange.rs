use num_bigint::BigInt;
use prost::{
    bytes::{Bytes, BytesMut},
    Message,
};
use tracing::info;

use crate::protobuf::securemessage::GenericPublicKey;

use super::generic::Crypto;

fn trim_to_32(raw: &[u8]) -> Bytes {
    let int = BigInt::from_signed_bytes_be(raw);
    let unsigned = int.to_biguint().unwrap();
    Bytes::from(unsigned.to_bytes_be())
}
pub fn get_public<C: Crypto>(raw: &[u8]) -> C::PublicKey {
    let generic = GenericPublicKey::decode(raw).unwrap();
    let key = generic.ec_p256_public_key.as_ref().unwrap();
    info!(
        "Generic Key {:?} x_size {} y_size {}",
        generic,
        key.x.as_slice().len(),
        key.y.as_slice().len()
    );
    let x = trim_to_32(&key.x);
    let y = trim_to_32(&key.y);
    C::to_pubkey(&x, &y)
}
pub fn key_echange<C: Crypto>(
    client_pub: C::PublicKey,
    server_key: C::SecretKey,
    client_init: Bytes,
    server_init: Bytes,
) -> (C::Intermediate, C::Intermediate) {
    let dhs = C::diffie_hellman(server_key, &client_pub);
    let mut xor = BytesMut::new();
    xor.extend_from_slice(&client_init);
    xor.extend_from_slice(&server_init);
    let l_auth = 32;
    let l_next = 32;
    let auth = C::extract_expand(&xor, &dhs, "UKEY2 v1 auth".as_bytes(), l_auth);
    let next = C::extract_expand(&xor, &dhs, "UKEY2 v1 next".as_bytes(), l_next);
    (auth, next)
}
// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn test_key_gen() {
//         let _keypair = get_public_private();
//     }
//     #[test]
//     fn test_diffie_hellman() {
//         let server_keypair = get_public_private();
//         let client_keypair = get_public_private();
//         let server_pubkey = PublicKey::from(&server_keypair);
//         let client_pubkey = PublicKey::from(&client_keypair);
//         // assert_eq!(
//         //     diffie_hellmen(server_pubkey, &client_keypair),
//         //     diffie_hellmen(client_pubkey, &server_keypair)
//         // );
//     }
// }
