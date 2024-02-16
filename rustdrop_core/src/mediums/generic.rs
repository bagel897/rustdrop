mod receiver;
mod sender;
mod socket;

use std::{fmt::Debug, hash::Hash};

use flume::Sender;
use tokio::io::{AsyncRead, AsyncWrite};

use self::{receiver::GenericReciever, sender::GenericSender};
use crate::{
    core::{
        io::{reader::ReaderRecv, writer::WriterSend},
        RustdropError,
    },
    Context, DiscoveryEvent, Outgoing, ReceiveEvent, SenderEvent,
};

pub trait Discovery: Debug + Clone + PartialEq + Hash + Eq + 'static {
    async fn into_socket(
        self,
    ) -> Result<
        (
            impl AsyncRead + Send + Sync + Unpin,
            impl AsyncWrite + Send + Sync + Unpin,
        ),
        RustdropError,
    >;
    async fn send_to(
        self,
        mut context: Context,
        outgoing: Outgoing,
        send: Sender<SenderEvent>,
    ) -> Result<(), RustdropError> {
        let (rx, tx) = self.into_socket().await?;
        let reader = ReaderRecv::new(rx, &mut context);
        let writer = WriterSend::new(tx, &mut context);
        GenericSender::send_to(context, reader, writer, outgoing, send).await?;
        Ok(())
    }
}
pub trait Medium {
    type Discovery: Discovery;
    async fn discover(&mut self, send: Sender<DiscoveryEvent>) -> Result<(), RustdropError>;
    async fn start_recieving(&mut self, send: Sender<ReceiveEvent>) -> Result<(), RustdropError>;
    async fn recieve<
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    >(
        rx: R,
        tx: W,
        mut context: Context,
        send: Sender<ReceiveEvent>,
    ) {
        let reader = ReaderRecv::new(rx, &mut context);
        let writer = WriterSend::new(tx, &mut context);
        GenericReciever::recieve(reader, writer, context, send).await;
    }
}
