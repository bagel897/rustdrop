use std::fmt::Display;

use thiserror::Error;
#[derive(Error, Debug)]
pub struct EncryptionError {}
impl Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encryption error")
    }
}
#[derive(Error, Debug)]
pub struct TcpStreamClosedError {}
impl Display for TcpStreamClosedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TCP stream closed")
    }
}
#[derive(Error, Debug)]
pub enum RustdropError {
    #[error("Invalid message recieved")]
    InvalidMessage(String),
}
