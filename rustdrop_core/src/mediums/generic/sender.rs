use bytes::Bytes;
use flume::Sender;
use futures::pin_mut;
use futures::StreamExt;
use openssl::{ec::EcKey, pkey::Private};
use tracing::{debug, info};

use super::socket::StreamHandler;
use crate::RustdropResult;
use crate::{
    core::{
        handlers::{
            offline::{get_con_request, get_conn_response},
            transfer::process_transfer_response,
            ukey::get_ukey_init_finish,
        },
        io::{reader::ReaderRecv, writer::WriterSend},
        protocol::{get_paired_frame, get_paired_result},
        ukey2::{get_public, Crypto, CryptoImpl, Ukey2},
        RustdropError,
    },
    protobuf::securegcm::{ukey2_message::Type, Ukey2Message, Ukey2ServerInit},
    Context, Outgoing, SenderEvent,
};

pub struct GenericSender {
    stream_handler: StreamHandler,
    context: Context,
    outgoing: Outgoing,
    send: Sender<SenderEvent>,
}
impl GenericSender {
    pub(crate) async fn send_to(
        context: Context,
        reader: ReaderRecv,
        writer: WriterSend,
        outgoing: Outgoing,
        send: Sender<SenderEvent>,
    ) -> RustdropResult<()> {
        let sender = GenericSender {
            stream_handler: StreamHandler::new(reader, writer, context.clone()),
            context,
            outgoing,
            send,
        };
        sender.run().await?;
        Ok(())
    }
    async fn handle_init(
        &mut self,
    ) -> RustdropResult<(Bytes, Ukey2Message, <CryptoImpl as Crypto>::SecretKey)> {
        let init = get_con_request(self.context.endpoint_info.clone());
        let (ukey_init, finish, key) = get_ukey_init_finish();
        self.stream_handler.send(&init).await;
        let init_raw = self
            .stream_handler
            .send_ukey2(&ukey_init, Type::ClientInit)
            .await;
        debug!("Sent messages");
        Ok((init_raw, finish, key))
    }
    async fn handle_ukey2_exchange(
        &mut self,
        init_raw: Bytes,
        finish: Ukey2Message,
        key: EcKey<Private>,
    ) -> RustdropResult<()> {
        let (server_resp, resp_raw): (Ukey2ServerInit, Bytes) =
            self.stream_handler.next_ukey_message().await?;
        debug!("Recived message {:#?}", server_resp);
        let server_key = get_public::<CryptoImpl>(server_resp.public_key());
        let (ukey2_send, ukey2_recv) = Ukey2::new(init_raw, key, resp_raw, server_key, true);
        self.stream_handler.send(&finish).await;
        let _connection_response = self.stream_handler.next_offline().await?;
        let c_frame = get_conn_response();
        self.stream_handler.send(&c_frame).await;
        debug!("Recived message {:#?}", _connection_response);
        self.stream_handler
            .setup_ukey2(ukey2_send, ukey2_recv)
            .await;
        Ok(())
    }
    async fn handle_pairing(&mut self) -> RustdropResult<()> {
        let _server_resp = self.stream_handler.next_payload().await?;
        let p_frame = get_paired_frame();
        self.stream_handler.send_payload(&p_frame);
        let _server_resp = self.stream_handler.next_payload().await?;
        let p_res = get_paired_result();
        self.stream_handler.send_payload(&p_res);
        Ok(())
    }
    async fn run(mut self) -> RustdropResult<()> {
        let (init_raw, finish, key) = self.handle_init().await?;
        self.handle_ukey2_exchange(init_raw, finish, key).await?;
        self.handle_pairing().await?;
        let (intro, payload) = self.outgoing.get_frames();
        pin_mut!(payload);
        self.send
            .send_async(SenderEvent::AwaitingResponse())
            .await
            .unwrap();
        self.stream_handler.send_payload(&intro);
        let frame = self.stream_handler.next_payload().await?;
        let resp = process_transfer_response(frame);
        if resp {
            self.send.send_async(SenderEvent::Accepted()).await.unwrap();
            while let Some((id, data)) = payload.next().await {
                self.stream_handler.send_payload_raw(data, id)
            }
            self.send.send_async(SenderEvent::Finished()).await.unwrap();
        } else {
            self.send.send_async(SenderEvent::Rejected()).await.unwrap();
        }
        info!("Finished, disconnecting");
        self.stream_handler.send_disconnect();
        self.context.shutdown().await;
        Ok(())
    }
}
