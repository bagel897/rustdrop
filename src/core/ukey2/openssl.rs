use openssl::{aes::AesKey, dh::Dh};

use super::generic::Crypto;
#[derive(Debug)]
pub struct OpenSSL {}
impl Crypto for OpenSSL {
    type AesKey = AesKey;
    type PublicKey = Dh;
    type SecretKey = Dh;
}
