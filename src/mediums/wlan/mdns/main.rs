use std::{collections::HashMap, net::IpAddr};

use base64::{prelude::BASE64_URL_SAFE, Engine};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tracing::{debug, info};

use super::constants::{PCP, SERVICE_1, SERVICE_2, SERVICE_3};
use crate::{
    core::{protocol::get_endpoint_id, Config},
    mediums::wlan::mdns::constants::TYPE,
    runner::application::Application,
    UiHandle,
};
fn encode(data: &Vec<u8>) -> String {
    BASE64_URL_SAFE.encode(data)
}

fn get_txt(config: &Config) -> String {
    let data = get_endpoint_id(config);
    debug!("data {:#x?}", data);
    encode(&data)
}
fn name() -> Vec<u8> {
    let rng = thread_rng();
    let endpoint: Vec<u8> = rng.sample_iter(&Alphanumeric).take(4).collect();
    let data: Vec<u8> = vec![
        PCP,
        endpoint[0],
        endpoint[1],
        endpoint[2],
        endpoint[3],
        SERVICE_1,
        SERVICE_2,
        SERVICE_3,
        0x0,
        0x0,
    ];
    debug!("data {:#x?}, name: {:#x?}", data, endpoint);
    data
}
pub struct MDNSHandle {
    ips: Vec<IpAddr>,
}
impl MDNSHandle {
    pub(crate) fn new(ips: Vec<IpAddr>) -> Self {
        MDNSHandle { ips }
    }
    fn get_service_info<U: UiHandle>(&self, application: &Application<U>) -> ServiceInfo {
        let name_raw = name();
        let name = encode(&name_raw);
        let txt = get_txt(&application.config);
        let mut txt_record = HashMap::new();
        txt_record.insert("n".to_string(), txt);
        ServiceInfo::new(
            TYPE,
            &name,
            &name,
            self.ips.as_slice(),
            application.config.port,
            txt_record,
        )
        .unwrap()
    }
    pub async fn advertise_mdns<U: UiHandle>(self, application: &Application<U>) {
        let token = application.child_token();
        let mdns = ServiceDaemon::new().expect("Failed to create daemon");
        mdns.register(self.get_service_info(application)).unwrap();
        info!("Started MDNS thread");
        token.cancelled().await;
        info!("Shutting down");
        mdns.shutdown().unwrap();
    }
}

// fn on_service_registered(
//     result: zeroconf::Result<ServiceRegistration>,
//     context: Option<Arc<dyn Any>>,
// ) {
//     let service = result.unwrap();
//
//     info!("Service registered: {:?}", service);
//
//     let context = context
//         .as_ref()
//         .unwrap()
//         .downcast_ref::<Arc<Mutex<Context>>>()
//         .unwrap()
//         .clone();
//
//     context.lock().unwrap().service_name = service.name().clone();
//
//     info!("Context: {:?}", context);
//
//     // ...
// }
#[cfg(test)]
mod tests {

    use std::assert_eq;

    use tracing_test::traced_test;

    use super::*;
    use crate::{
        core::protocol::decode_endpoint_id,
        mediums::wlan::{mdns::browser::get_dests, wlan::get_ips},
        SimpleUI,
    };
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_mdns() {
        let app: Application<SimpleUI> = Application::default();
        app.spawn(
            async {
                let handle = MDNSHandle::new(get_ips());
                handle.advertise_mdns(&app).await;
            },
            "mdns",
        );
        let dests = get_dests().await;
        assert!(!dests.is_empty());
        assert!(dests.iter().any(|ip| ip.ip.port() == app.config.port));
        app.shutdown().await;
    }
    #[traced_test()]
    #[test]
    fn test_txt() {
        let config = Config::default();
        let endpoint_info_no_base64 = get_endpoint_id(&config);
        let endpoint_info = get_txt(&config);
        let decoded = BASE64_URL_SAFE.decode(endpoint_info).unwrap();
        assert_eq!(endpoint_info_no_base64.len(), decoded.len());
        info!("{:?}", decoded);
        let (devtype, name) = decode_endpoint_id(decoded.as_slice()).unwrap();
        assert_eq!(devtype, config.devtype);
        assert_eq!(name, config.name);
    }
}
