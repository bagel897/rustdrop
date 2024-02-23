use openssl::error::ErrorStack;
use thiserror::Error;
use tokio::io;

use crate::protobuf::securegcm::Ukey2Alert;
#[derive(Error, Debug)]
pub enum RustdropError {
    #[error("Encryption Error")]
    Encryption(),
    #[error("Stream closed")]
    StreamClosed(),
    #[error("No Response")]
    NoResponse(),
    #[error("Invalid message recieved")]
    InvalidMessage(String),
    #[error("Invalid endpoint id")]
    InvalidEndpointId(),
    #[error("Connection Error")]
    Connection(),
    #[error("Ukey Error {0:?}")]
    UkeyError(Ukey2Alert),
}
