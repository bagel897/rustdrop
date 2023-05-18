use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct EncryptionError {}
impl Error for EncryptionError {}
impl Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encryption error")
    }
}
#[derive(Debug)]
pub struct TcpStreamClosedError {}
impl Error for TcpStreamClosedError {}
impl Display for TcpStreamClosedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TCP stream closed")
    }
}
