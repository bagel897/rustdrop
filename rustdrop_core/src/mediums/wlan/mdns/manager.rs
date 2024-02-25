use std::net::IpAddr;

use futures::StreamExt;
use mdns_sd::{IfKind, ServiceDaemon, ServiceEvent};
use tracing::{debug, error, info};

use super::{browser::parse_device, constants::TYPE};
use crate::{mediums::wlan::mdns::main::get_service_info, runner::DiscoveringHandle, Context};
pub(crate) struct Mdns {
    context: Context,
    daemon: ServiceDaemon,
}
impl Mdns {
    pub fn new(context: Context) -> Self {
        let daemon = ServiceDaemon::new().expect("Failed to create daemon");
        daemon.enable_interface(IfKind::All).unwrap();
        Self { context, daemon }
    }

    pub fn shutdown(&mut self) {
        info!("Shutting down");
        self.daemon.shutdown().unwrap();
    }
    pub(crate) async fn get_dests(&mut self, mut sender: DiscoveringHandle) {
        let mut reciever = self.daemon.browse(TYPE).unwrap().into_stream();
        self.context.spawn(async move {
            while let Some(event) = reciever.next().await {
                Self::on_service_discovered(event, &mut sender).await;
            }
        });
    }
    async fn on_service_discovered(event: ServiceEvent, sender: &mut DiscoveringHandle) {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                debug!("Service discovered: {:?}", info);
                if let Err(e) = parse_device(&info, sender).await {
                    error!("Error while parsing endpoint {:?}: {}", info, e)
                };
            }
            ServiceEvent::SearchStarted(_) => {}
            other_event => {
                info!("Received other event: {:?}", &other_event);
            }
        }
    }

    pub async fn advertise_mdns(&self, ips: Vec<IpAddr>, port: u16) {
        // let token = self.context.child_token();
        let info = get_service_info(
            &self.context.config,
            self.context.endpoint_info.clone(),
            ips,
            port,
        );
        info!("Started MDNS thread {:?}", info);
        self.daemon.register(info).unwrap();
        // self.context.spawn(
        //     async move {
        //         token.cancelled().await;
        //     },
        //     "mdns",
        // );
    }
}
impl Drop for Mdns {
    fn drop(&mut self) {
        self.shutdown();
    }
}
// #[cfg(test)]
// mod tests {
//
//     use std::assert_eq;
//
//     use base64::engine::general_purpose::URL_SAFE;
//     use tracing_test::traced_test;
//
//     use super::*;
//     use crate::{
//         core::protocol::{decode_endpoint_id, get_endpoint_info},
//         Config,
//     };
//     #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
//     async fn test_mdns() {
//         let context: Context = Context::default();
//         context.spawn(
//             async {
//                 let handle = Mdns::new(context);
//                 handle.advertise_mdns(&context).await;
//             },
//             "mdns",
//         );
//         let dests = get_dests().await;
//         assert!(!dests.is_empty());
//         assert!(dests.iter().any(|ip| ip.ip.port() == context.config.port));
//         context.shutdown().await;
//     }
//     #[traced_test()]
//     #[test]
//     fn test_txt() {
//         let config = Config::default();
//         let endpoint_info_no_base64 = get_endpoint_info(&config);
//         let decoded = BASE64_URL_SAFE.decode(endpoint_info_no_base64).unwrap();
//         assert_eq!(endpoint_info_no_base64.len(), decoded.len());
//         info!("{:?}", decoded);
//         let (devtype, name) = decode_endpoint_id(decoded.as_slice()).unwrap();
//         assert_eq!(devtype, config.devtype);
//         assert_eq!(name, config.name);
//     }
// }
