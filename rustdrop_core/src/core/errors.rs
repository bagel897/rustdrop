use openssl::error::ErrorStack;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum RustdropError {
    #[error("Openssl Error")]
    OpenSSL(#[from] ErrorStack),
    #[error("Encryption Error")]
    Encryption(),
    #[error("Decode Error")]
    Decode {
        #[from]
        source: prost::DecodeError,
    },
    #[error("Stream closed")]
    StreamClosed(),
    #[error("Invalid message recieved")]
    InvalidMessage(String),
    #[error("Invalid endpoint id")]
    InvalidEndpointId(),
    #[error("Bluetooth Error")]
    Bluetooth {
        #[from]
        source: bluer::Error,
    },
    #[error("Connection Error")]
    Connection(),
}
