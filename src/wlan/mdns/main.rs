use std::{
    any::Any,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    core::{Config, DeviceType},
    wlan::mdns::constants::TYPE,
};
use base64::{engine::general_purpose, Engine};
use general_purpose::URL_SAFE;

use prost::Message;
use rand_new::{distributions::Alphanumeric, thread_rng, Rng};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use zeroconf::{prelude::*, MdnsService, ServiceRegistration, ServiceType, TxtRecord};
#[derive(Default, Debug)]
pub struct Context {
    service_name: String,
}
use super::constants::{DOMAIN, PCP, SERVICE_1, SERVICE_2, SERVICE_3};
fn get_devtype_bit(devtype: DeviceType) -> u8 {
    match devtype {
        DeviceType::UNKNOWN => 0,
        DeviceType::PHONE => 1,
        DeviceType::TABLET => 2,
        DeviceType::LAPTOP => 3,
    }
}
fn get_bitfield(devtype: DeviceType) -> u8 {
    return get_devtype_bit(devtype) << 1;
}
fn encode(data: &Vec<u8>) -> String {
    return URL_SAFE.encode(data);
}

fn get_txt(config: &Config) -> String {
    let mut data: Vec<u8> = thread_rng().sample_iter(&Alphanumeric).take(17).collect();
    data[0] = get_bitfield(config.devtype);
    let mut encoded = config.name.encode_to_vec();
    data.push(encoded.len() as u8);
    data.append(&mut encoded);
    debug!("data {:#x?}", data);
    return encode(&data);
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
    return data;
}
pub struct MDNSHandle {
    token: CancellationToken,
    config: Config,
}
impl MDNSHandle {
    pub(crate) fn new(config: &Config, token: CancellationToken) -> Self {
        MDNSHandle {
            token,
            config: config.clone(),
        }
    }
    pub fn run(&mut self) {
        self.advertise_mdns();
    }
    fn advertise_mdns(&self) {
        info!("Started MDNS thread");
        let name_raw = name();
        let name = encode(&name_raw);
        let txt = get_txt(&self.config);
        let service_type = ServiceType::new(TYPE, "tcp").unwrap();
        debug!("Service Type {:?}", service_type);
        let mut service = MdnsService::new(service_type, self.config.port);
        service.set_name(&name);
        service.set_network_interface(zeroconf::NetworkInterface::Unspec);
        service.set_domain(DOMAIN);
        let mut txt_record = TxtRecord::new();
        debug!("Txt: {}", txt);
        txt_record.insert("n", &txt).unwrap();
        let context: Arc<Mutex<Context>> = Arc::default();
        service.set_registered_callback(Box::new(on_service_registered));
        service.set_context(Box::new(context));
        service.set_txt_record(txt_record);
        let event_loop = service.register().unwrap();

        loop {
            // calling `poll()` will keep this service alive
            event_loop.poll(Duration::from_secs(1)).unwrap();
            if self.token.is_cancelled() {
                info!("Shutting down");
                return;
            }
        }
    }
}

fn on_service_registered(
    result: zeroconf::Result<ServiceRegistration>,
    context: Option<Arc<dyn Any>>,
) {
    let service = result.unwrap();

    info!("Service registered: {:?}", service);

    let context = context
        .as_ref()
        .unwrap()
        .downcast_ref::<Arc<Mutex<Context>>>()
        .unwrap()
        .clone();

    context.lock().unwrap().service_name = service.name().clone();

    info!("Context: {:?}", context);

    // ...
}
#[cfg(test)]
mod tests {

    use std::thread;

    use crate::wlan::mdns::browser::get_dests;

    use super::*;
    #[test]
    fn test_mdns() {
        let config = Config::default();
        let token = CancellationToken::new();
        let clone = token.clone();
        let mut handle = MDNSHandle::new(&config, clone);
        let run_handle = thread::spawn(move || handle.run());
        let ips = get_dests();
        assert!(!ips.is_empty());
        assert!(ips.iter().any(|ip| ip.port() == config.port));
        token.cancel();
        run_handle.join().unwrap();
    }
}
