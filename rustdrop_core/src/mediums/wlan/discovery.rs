use std::{io::ErrorKind, net::SocketAddr};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tracing::info;

use crate::{core::RustdropError, mediums::generic::Discovery};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WlanDiscovery {
    addr: SocketAddr,
}
impl From<SocketAddr> for WlanDiscovery {
    fn from(addr: SocketAddr) -> Self {
        Self { addr }
    }
}
async fn get_stream(ip: &SocketAddr) -> Result<TcpStream, RustdropError> {
    let mut stream;
    let mut counter = 0;
    loop {
        stream = TcpStream::connect(ip).await;
        match stream {
            Ok(ref _s) => break,
            Err(e) => {
                if e.kind() != ErrorKind::ConnectionRefused {
                    return Err(RustdropError::Connection());
                }
                info!("addr: {} {}", ip, e);
            }
        }
        if counter > 10 {
            panic!();
        }
        counter += 1;
    }
    Ok(stream.unwrap())
}
impl Discovery for WlanDiscovery {
    async fn into_socket(self) -> Result<(impl AsyncRead, impl AsyncWrite), RustdropError> {
        let stream = get_stream(&self.addr).await?;
        Ok(stream.into_split())
    }
}
