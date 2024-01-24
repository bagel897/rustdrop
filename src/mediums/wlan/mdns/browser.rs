use std::time::Instant;
use std::{
    any::Any,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use tracing::info;
use zeroconf::txt_record::TTxtRecord;
use zeroconf::{
    browser::TMdnsBrowser, event_loop::TEventLoop, MdnsBrowser, ServiceDiscovery, ServiceType,
};

use crate::core::protocol::{decode_endpoint_id, Device};

use super::constants::TYPE;

#[derive(Default, Debug)]
pub struct Context {
    dests: Vec<Device>,
}
pub(crate) fn get_dests() -> Vec<Device> {
    let mut browser = MdnsBrowser::new(ServiceType::new(TYPE, "tcp").unwrap());

    browser.set_service_discovered_callback(Box::new(on_service_discovered));
    let context: Arc<Mutex<Context>> = Arc::default();
    browser.set_context(Box::new(context.clone()));

    let event_loop = browser.browse_services().unwrap();
    let start = Instant::now();
    loop {
        // calling `poll()` will keep this browser alive
        event_loop.poll(Duration::from_secs(0)).unwrap();
        if start.elapsed().as_secs() > 1 {
            break;
        }
    }
    return context.lock().unwrap().dests.clone();
}

fn on_service_discovered(
    result: zeroconf::Result<ServiceDiscovery>,
    context: Option<Arc<dyn Any>>,
) {
    let service = result.unwrap();
    info!("Service discovered: {:?}", service);
    let context = context
        .as_ref()
        .unwrap()
        .downcast_ref::<Arc<Mutex<Context>>>()
        .unwrap()
        .clone();
    let parsed = match IpAddr::from_str(service.address()) {
        Ok(addr) => addr,
        Err(e) => {
            tracing::error!("{}", e);
            panic!("Original {} , error {}", service.address(), e);
        }
    };
    if parsed.is_ipv4() {
        let endpoint_info = service.txt().as_ref().unwrap().get("n").unwrap();
        let addr = SocketAddr::new(parsed, *service.port());
        let (device_type, name) = decode_endpoint_id(&URL_SAFE.decode(endpoint_info).unwrap());
        let dest = Device {
            device_type,
            device_name: name,
            ip: addr,
        };

        context.lock().unwrap().dests.push(dest);
    }
    // ...
}
