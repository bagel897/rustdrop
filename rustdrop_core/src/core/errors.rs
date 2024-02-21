use openssl::error::ErrorStack;
use thiserror::Error;
use tokio::io;

use crate::protobuf::securegcm::Ukey2Alert;
#[derive(Error, Debug)]
pub enum RustdropError {
    #[error("Openssl Error {0}")]
    OpenSSL(#[from] ErrorStack),
    #[error("Encryption Error")]
    Encryption(),
    #[error("Decode Error {source}")]
    Decode {
        #[from]
        source: prost::DecodeError,
    },
    #[error("Base64 Decode Error")]
    Base64(#[from] base64::DecodeError),
    #[error("Stream closed")]
    StreamClosed(),
    #[error("No Response")]
    NoResponse(),
    #[error("Invalid message recieved")]
    InvalidMessage(String),
    #[error("Invalid endpoint id")]
    InvalidEndpointId(),
    #[error("Bluetooth Error {source}")]
    Bluetooth {
        #[from]
        source: bluer::Error,
    },
    #[error("Connection Error")]
    Connection(),
    #[error("Ukey Error {0:?}")]
    UkeyError(Ukey2Alert),
    #[error("Io Error {0:?}")]
    Io(#[from] io::Error),
}
