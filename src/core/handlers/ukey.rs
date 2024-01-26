use crate::{
    core::errors::RustdropError,
    protobuf::securegcm::{Ukey2ClientFinished, Ukey2ClientInit, Ukey2ServerInit},
};
use bytes::Bytes;
use p256::{elliptic_curve::PublicKey, NistP256};
use tracing::info;
pub struct UkeyInitData {
    init_raw: Bytes,
    resp_raw: Bytes,
    keypair: EphemeralSecret,
}
pub fn handle_ukey2_client_init(
    message: Ukey2ClientInit,
    init_raw: Bytes,
) -> Result<(keypair, Ukey2ServerInit), RustdropError> {
    info!("{:?}", message);
    if message.version() != 1 {
        return Err(RustdropError::InvalidMessage("Incorrect version".into()));
    }
    assert_eq!(message.random().len(), 32);
    let mut resp = Ukey2ServerInit::default();
    let keypair = get_public_private();
    resp.version = Some(1);
    resp.random = Some(get_random(32));
    resp.handshake_cipher = Some(Ukey2HandshakeCipher::P256Sha512.into());
    resp.public_key = Some(get_generic_pubkey(&keypair).encode_to_vec());
    info!("{:?}", resp);
    Ok((keypair, resp))
}
pub fn handle_ukey2_client_finish(
    message: Ukey2ClientFinish,
) -> Result<PublicKey<NistP256>, RustdropError> {
    let client_pub_key = get_public(message.public_key());
    Ok(client_pub_key)
}
