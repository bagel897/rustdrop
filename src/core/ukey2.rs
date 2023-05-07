use openssl::{
    ec::{EcGroup, EcKey},
    nid::Nid,
    pkey::{Id, PKey, Private, Public},
    pkey_ctx::PkeyCtx,
};
use prost::bytes::BytesMut;
const D2D_SALT: &'static str = "82AA55A0D397F88346CA1CEE8D3909B95F13FA7DEB1D4AB38376B8256DA85510";
const PT2_SALT: &'static str = "BF9D2A53C63616D75DB0A7165B91C1EF73E537F2427405FA23610A4BE657642E";
use crate::protobuf::securegcm::Ukey2ClientFinished;

pub fn get_public_private() -> (EcKey<Private>, EcKey<Public>) {
    let group = EcGroup::from_curve_name(Nid::ECDSA_WITH_RECOMMENDED).unwrap();
    let private_key = EcKey::generate(&group).unwrap();
    let public_key = EcKey::from_public_key(&group, &private_key.public_key()).unwrap();
    (private_key, public_key)
}
fn get_hdkf_key(info: &'static str, key: &[u8], salt: &'static str) -> PKey<Private> {
    let buf = get_hdkf_key_raw(info, key, salt);
    return PKey::private_key_from_raw_bytes(&buf, Id::HKDF).unwrap();
}
fn get_hdkf_key_raw(info: &'static str, key: &[u8], salt: &'static str) -> BytesMut {
    let mut ctx = PkeyCtx::new_id(Id::HKDF).unwrap();
    ctx.set_hkdf_salt(D2D_SALT.as_bytes());
    ctx.add_hkdf_info(info.as_bytes());
    ctx.set_hkdf_key(key);
    let mut buf = BytesMut::with_capacity(ctx.derive(None).unwrap());
    ctx.derive(Some(&mut buf)).unwrap();
    return buf;
}
pub(crate) struct Ukey2 {
    decrypt_key: PKey<Private>,
    recv_hmac: PKey<Private>,
    encrypt_key: PKey<Private>,
    send_hmac: PKey<Private>,
}
fn key_echange() -> (Vec<u8>, Vec<u8>) {}
impl Ukey2 {
    fn new(
        init: &[u8],
        private_key: EcKey<Private>,
        resp: &[u8],
        client_resp: Ukey2ClientFinished,
    ) -> Ukey2 {
        let mut ctx = PkeyCtx::new_id(Id::HKDF).unwrap();
        let (auth_string, next_protocol_secret) = key_echange();
        let d2d_client = get_hdkf_key_raw("client", next_protocol_secret.as_slice(), D2D_SALT);
        let d2d_server = get_hdkf_key_raw("server", next_protocol_secret.as_slice(), D2D_SALT);
        let decrypt_key = get_hdkf_key("ENC:2", &d2d_client, PT2_SALT);
        let recieve_key = get_hdkf_key("SIG_1", &d2d_client, PT2_SALT);
        let encrypt_key = get_hdkf_key("ENC:2", &d2d_server, PT2_SALT);
        let send_key = get_hdkf_key("SIG:1", &d2d_server, PT2_SALT);
        Ukey2 {
            decrypt_key,
            recv_hmac: recieve_key,
            encrypt_key,
            send_hmac: send_key,
        }
    }
}
