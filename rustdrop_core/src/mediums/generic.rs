mod receiver;
mod sender;
mod socket;

use std::{fmt::Debug, hash::Hash};

use flume::Sender;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::error;

use self::{receiver::GenericReciever, sender::GenericSender};
use crate::{
    core::{
        io::{reader::ReaderRecv, writer::WriterSend},
        RustdropError,
    },
    runner::DiscoveringHandle,
    Context, Outgoing, ReceiveEvent, SenderEvent,
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
        context: Context,
        outgoing: Outgoing,
        send: Sender<SenderEvent>,
    ) -> Result<(), RustdropError> {
        let (rx, tx) = self.into_socket().await?;
        let reader = ReaderRecv::new(rx, &context);
        let writer = WriterSend::new(tx, &context);
        GenericSender::send_to(context, reader, writer, outgoing, send).await?;
        Ok(())
    }
}
pub trait Medium {
    type Discovery: Discovery;
    async fn discover(&mut self, send: DiscoveringHandle) -> Result<(), RustdropError>;
    async fn start_recieving(&mut self, send: Sender<ReceiveEvent>) -> Result<(), RustdropError>;
    async fn recieve<
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    >(
        rx: R,
        tx: W,
        context: Context,
        send: Sender<ReceiveEvent>,
    ) {
        let reader = ReaderRecv::new(rx, &context);
        let writer = WriterSend::new(tx, &context);
        let res = GenericReciever::recieve(reader, writer, context, send).await;
        if let Err(e) = res {
            error!("{:?}", e);
        }
    }
}
