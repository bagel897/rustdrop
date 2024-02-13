use std::net::IpAddr;

use super::browser::parse_device;
use super::constants::TYPE;
use crate::{mediums::wlan::mdns::main::get_service_info, Context};
use mdns_sd::{IfKind, ServiceDaemon, ServiceEvent};
use tokio_stream::StreamExt;
use tracing::{debug, error, info};
pub(crate) struct Mdns {
    context: Context,
    daemon: ServiceDaemon,
}
impl Mdns {
    pub fn new(context: Context) -> Self {
        let daemon = ServiceDaemon::new().expect("Failed to create daemon");
        daemon.enable_interface(IfKind::All).unwrap();
        Self {
            context: context,
            daemon,
        }
    }

    pub async fn shutdown(&mut self) {
        info!("Shutting down");
        self.daemon.shutdown().unwrap();
    }
    pub(crate) async fn get_dests(&mut self) {
        let mut reciever = self.daemon.browse(TYPE).unwrap().into_stream();
        let child = self.context.clone();
        self.context.spawn(
            async move {
                while let Some(event) = reciever.next().await {
                    Self::on_service_discovered(event, &child).await;
                }
            },
            "mdns",
        );
    }
    async fn on_service_discovered(event: ServiceEvent, context: &Context) {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                debug!("Service discovered: {:?}", info);
                for addr in info.get_addresses() {
                    match parse_device(addr, &info) {
                        Ok(dest) => context.ui().await.discovered_device(dest).await,
                        Err(e) => error!("Error while parsing endpoint {:?}: {}", info, e),
                    };
                }
            }
            other_event => {
                info!("Received other event: {:?}", &other_event);
            }
        }
    }

    pub async fn advertise_mdns(&mut self, ips: Vec<IpAddr>) {
        // let token = self.context.child_token();
        let info = get_service_info(&self.context.config, ips);
        self.daemon.register(info).unwrap();
        // self.context.spawn(
        //     async move {
        //         info!("Started MDNS thread");
        //         token.cancelled().await;
        //     },
        //     "mdns",
        // );
    }
}
