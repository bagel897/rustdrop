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
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    Bluetooth {
        #[from]
        source: bluer::Error,
    },
    #[error(transparent)]
    Decode {
        #[from]
        source: prost::DecodeError,
    },
    #[error(transparent)]
    OpenSSL(#[from] ErrorStack),
}
