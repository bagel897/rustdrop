use crate::{
    core::{
        protocol::{decode_endpoint_id, get_paired_frame, get_paired_result},
        ukey2::{get_public, get_public_private, Ukey2},
        util::get_random,
    },
    protobuf::{
        location::nearby::connections::{OfflineFrame, PairedKeyEncryptionFrame},
        securegcm::{
            ukey2_message::Type, Ukey2ClientFinished, Ukey2ClientInit, Ukey2HandshakeCipher,
            Ukey2ServerInit,
        },
        sharing::nearby::PairedKeyResultFrame,
    },
    wlan::wlan_common::{get_conn_response, StreamHandler},
};
use bytes::Bytes;
use prost::Message;

use tokio::net::TcpStream;
use tracing::{info, span, Level};
use x25519_dalek::{PublicKey, StaticSecret};

enum StateMachine {
    Init,
    Request,
    UkeyInit {
        init: Ukey2ClientInit,
        resp: Ukey2ServerInit,
        keypair: StaticSecret,
    },
    UkeyFinish,
    PairedKeyBegin,
    PairedKeyFinish,
}
pub struct WlanReader {
    stream_handler: StreamHandler,
    state: StateMachine,
}

impl WlanReader {
    pub async fn new(stream: TcpStream) -> Self {
        let stream_handler = StreamHandler::new(stream);
        WlanReader {
            stream_handler,
            state: StateMachine::Init,
        }
    }
    fn handle_con_request(&mut self, message: OfflineFrame) {
        info!("{:?}", message);
        self.state = StateMachine::Request;
        let submessage = message.v1.unwrap().connection_request.unwrap();
        let endpoint_id = submessage.endpoint_info();
        let (devtype, name) = decode_endpoint_id(endpoint_id);
        info!("Connection request from {} {:?}", name, devtype)
    }
    async fn handle_ukey2_client_init(&mut self, message: Ukey2ClientInit) {
        info!("{:?}", message);
        assert_eq!(message.version(), 1);
        assert_eq!(message.random().len(), 32);
        let mut resp = Ukey2ServerInit::default();
        let keypair = get_public_private();
        resp.version = Some(1);
        resp.random = Some(get_random(32));
        resp.handshake_cipher = Some(Ukey2HandshakeCipher::P256Sha512.into());
        resp.public_key = Some(PublicKey::from(&keypair).as_bytes().to_vec());
        info!("{:?}", resp);
        self.stream_handler
            .send_ukey2(&resp, Type::ServerInit)
            .await;
        self.state = StateMachine::UkeyInit {
            init: message,
            resp,
            keypair,
        };
    }

    async fn handle_ukey2_client_finish(
        &mut self,
        message: Ukey2ClientFinished,
        keypair: &StaticSecret,
        init: &Ukey2ClientInit,
        resp: &Ukey2ServerInit,
    ) {
        info!("{:?}", message);

        let client_pub_key = get_public(message.public_key());
        let ukey2 = Ukey2::new(
            Bytes::from(init.encode_to_vec()),
            keypair.clone(),
            Bytes::from(resp.encode_to_vec()),
            client_pub_key,
            false,
        );
        self.state = StateMachine::UkeyFinish;
        self.stream_handler.send(&get_conn_response()).await;
        self.stream_handler.setup_ukey2(ukey2);
        let p_key = get_paired_frame();
        self.stream_handler.send_securemessage(&p_key).await;
        // let payload:
        // self.send_encrypted()
    }

    async fn handle_message(&mut self) -> Result<(), ()> {
        match &self.state {
            StateMachine::Init => {
                let message = self.stream_handler.next_message().await?;
                self.handle_con_request(message)
            }
            StateMachine::Request => {
                let message = self.stream_handler.next_ukey_message().await?;
                self.handle_ukey2_client_init(message).await
            }
            StateMachine::UkeyInit {
                init,
                resp,
                keypair,
            } => {
                let message = self.stream_handler.next_ukey_message().await?;
                self.handle_ukey2_client_finish(
                    message,
                    &keypair.clone(),
                    &init.clone(),
                    &resp.clone(),
                )
                .await
            }
            StateMachine::UkeyFinish => {
                let _p_key: PairedKeyEncryptionFrame =
                    self.stream_handler.next_decrypted().await.unwrap();
                self.state = StateMachine::PairedKeyBegin;
                let resp = get_paired_result();
                self.stream_handler.send_securemessage(&resp).await;
            }
            StateMachine::PairedKeyBegin => {
                self.state = StateMachine::PairedKeyFinish;
                let _p_key: PairedKeyResultFrame =
                    self.stream_handler.next_decrypted().await.unwrap();
            }
            StateMachine::PairedKeyFinish => {
                // TODO
                return Err(());
            }
        }
        info!("Handled message");
        Ok(())
    }
    pub async fn run(&mut self) {
        let span = span!(Level::TRACE, "Handling connection");
        let _enter = span.enter();
        while let Ok(()) = self.handle_message().await {}
        self.stream_handler.shutdown().await;
    }
}
#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use tokio::{io::AsyncWriteExt, net::TcpListener};
    use tracing_test::traced_test;

    use crate::{
        core::Config, protobuf::location::nearby::connections::OfflineFrame, wlan::wlan::get_ips,
    };

    use super::*;
    const SAMPLE_DATA: &'static str = "000001F8080112F303080112EE030A04494232441225225A413775C4BC5A0D310E68EFBFBDEFBFBD7536EFBFBD0D456C6C656E27732070686F6E6520B1F6FCD0FDFFFFFFFF012805280828032809280A280228042807321F225A413775C4BC5A0D310E68DED77536FD0D456C6C656E27732070686F6E653A82030801121130303A35663A36373A65653A36303A62661A04C0A8037A2001280130F8283A340A32F828D028AD2DF12CE428FD2D992DBC28C12DD52D852DE92DF1128A1385139413F6129913A313EC128F138013A8139E13FB12422E0A2CE428BC28F828C12D852DAD2DD028992DF12C8F138A13F612F11280139413A81399139E138513A313FB12EC124A00522A0A28E428BC28F828D028AD2DF12C852DC12D992DF112851394139E138F1399138A13FB128013F612EC125AC9010AC601B335F734E7369334FB31D733B737F332CB32C730BB349736F72FA72FA3328331DB358F37D3318B3083368B35E731D72EE32FA734FF339731FF2ED336DF32BF36BF31A3379F30FB36EB33B330AB36C32EEF35AF33932FAB318733DB308F32CF34EB2ECF2FB732C735C3339B339F35BB2FE334EF30CC2BE428C42C9C2CE92DE02BFD2DB82BF12CF42B882C8C29AD2DFC2AD52DD028B429992DD82CC829852DC12DBC28A42BA029F828902BB02C8F13F112991385138A139413A8139E13A313F612EC12FB128013";

    const SAMPLE_DATA2: &'static str = "00000200080112FB03080112F6030A04534C4E5A122D22EFBFBDEFBFBD670B58191E3CEFBFBD44D5ABEFBFBDEFBFBDEFBFBDEFBFBD0D456C6C656E27732070686F6E6520EF98BF9AFCFFFFFFFF012805280828032809280A280228042807321F22ADF1670B58191E3CF744D5ABC0ADB8ED0D456C6C656E27732070686F6E653A82030801121130303A35663A36373A65653A36303A61661A04C0A8037A2001280130F8283A340A32FD2DE428AD2DE92DF828852DBC28C12D992DD52DF12CD0289413A8139E138A13A3138513F612FB12EC128013F11299138F13422E0A2CC12DF828992DD028F12CBC28AD2DE428852D9913851380138F138A139E13A813F112F612EC12A3139413FB124A00522A0A28852DF12CAD2DF828C12D992DE428BC28D0288513EC128F1380138A1394139E13F612F1129913FB125AC9010AC601C333BB2F9F35DB30FF2EBF31DF32F734B335FF33D72E8733A72F9B33FB31C7309F30D733E736EF30CB32F332DB35BF36C32E8336B7378331A332EB2EAB31E32F8B35E334932FB3308F37CF2F9334EF35D336D331B732E731BB34F72FEB33C735A7348B309736AF3397318F32AB36A337FB36CF34B02C8C29C42CCC2BF12CF828AD2DE92DD52D882CC12D902BB82B9C2CBC28852D992DD82CA029A42BFC2AE428D028B429FD2DF42BC829E02BEC12A81399139413F6128F139E1385138A13F112FB128013A31300000088080212830108011220A88B390D5F4D3C13010CE8888F5328FE72D2A4C3AACAD3DA03962AFBD61832221A4408641240592E791CD01D11AFEA971A7E2D5845ED92CCA08F988FFF93C6E3556E11DB4CE76B9FB0AFE848D76D4FDAB3E13E690E789373D9014E6709D3F6592855F44EA85822174145535F3235365F4342432D484D41435F534841323536";
    async fn get_streams(config: &Config) -> (TcpStream, TcpStream) {
        let ips = get_ips();
        let ip = ips.first().unwrap();
        let addr = SocketAddr::new(*ip, config.port);
        let server_listening = TcpListener::bind(addr).await.unwrap();
        let client_stream = TcpStream::connect(addr).await.unwrap();
        let server_stream = server_listening.accept().await.unwrap().0;
        (client_stream, server_stream)
    }
    #[traced_test]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_with_sample() {
        let config = Config::default();
        let (mut client_stream, server_stream) = get_streams(&config).await;
        let decoded_data = hex::decode(SAMPLE_DATA).unwrap();
        client_stream
            .write_buf(&mut decoded_data.as_slice())
            .await
            .unwrap();
        client_stream.shutdown().await.unwrap();
        let mut server = WlanReader::new(server_stream).await;
        server.run().await;
    }
    #[traced_test]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_with_sample2() {
        let config = Config::default();
        let (mut client_stream, server_stream) = get_streams(&config).await;
        let decoded_data = hex::decode(SAMPLE_DATA2).unwrap();
        client_stream
            .write_buf(&mut decoded_data.as_slice())
            .await
            .unwrap();
        client_stream.shutdown().await.unwrap();
        let mut server = WlanReader::new(server_stream).await;
        server.run().await;
    }
    #[test]
    fn test_decode() {
        let decoded_data = hex::decode(SAMPLE_DATA).unwrap();
        OfflineFrame::decode(&decoded_data[4..]).unwrap();
    }
}
