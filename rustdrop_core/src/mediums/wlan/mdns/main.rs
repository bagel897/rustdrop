use std::{collections::HashMap, net::IpAddr};

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use mdns_sd::ServiceInfo;
use tracing::debug;

use super::constants::{PCP, SERVICE_1, SERVICE_2, SERVICE_3};
use crate::{
    core::{protocol::get_endpoint_info, Config},
    mediums::wlan::mdns::constants::TYPE,
};
fn encode(data: &Vec<u8>) -> String {
    BASE64_URL_SAFE_NO_PAD.encode(data)
}

fn get_txt(config: &Config) -> String {
    let data = get_endpoint_info(config);
    debug!("data {:#x?}", data);
    encode(&data)
}
fn name(config: &Config) -> Vec<u8> {
    let endpoint = config.endpoint_id.as_bytes();
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
pub fn get_service_info(config: &Config, ips: Vec<IpAddr>) -> ServiceInfo {
    let name_raw = name(config);
    let name = encode(&name_raw);
    let txt = get_txt(config);
    let mut txt_record = HashMap::new();
    txt_record.insert("n".to_string(), txt);
    let service = ServiceInfo::new(TYPE, &name, &name, &*ips, config.port, txt_record).unwrap();
    service.enable_addr_auto()
}
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
        let context: context<SimpleUI> = context::default();
        context.spawn(
            async {
                let handle = MDNSHandle::new(get_ips());
                handle.advertise_mdns(&context).await;
            },
            "mdns",
        );
        let dests = get_dests().await;
        assert!(!dests.is_empty());
        assert!(dests.iter().any(|ip| ip.ip.port() == context.config.port));
        context.shutdown().await;
    }
    #[traced_test()]
    #[test]
    fn test_txt() {
        let config = Config::default();
        let endpoint_info_no_base64 = get_endpoint_info(&config);
        let endpoint_info = get_txt(&config);
        let decoded = BASE64_URL_SAFE.decode(endpoint_info).unwrap();
        assert_eq!(endpoint_info_no_base64.len(), decoded.len());
        info!("{:?}", decoded);
        let (devtype, name) = decode_endpoint_id(decoded.as_slice()).unwrap();
        assert_eq!(devtype, config.devtype);
        assert_eq!(name, config.name);
    }
}
