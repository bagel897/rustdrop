use std::{
    any::Any,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use tracing::info;
use zeroconf::{
    browser::TMdnsBrowser, event_loop::TEventLoop, MdnsBrowser, ServiceDiscovery, ServiceType,
};

use super::constants::TYPE;

#[derive(Default, Debug)]
pub struct Context {
    ip_addrs: Option<Vec<SocketAddr>>,
}
pub fn get_dests() -> Vec<SocketAddr> {
    let mut browser = MdnsBrowser::new(ServiceType::new(TYPE, "tcp").unwrap());

    browser.set_service_discovered_callback(Box::new(on_service_discovered));
    let context: Arc<Mutex<Context>> = Arc::default();
    browser.set_context(Box::new(context.clone()));

    let event_loop = browser.browse_services().unwrap();

    loop {
        // calling `poll()` will keep this browser alive
        event_loop.poll(Duration::from_secs(0)).unwrap();
        if let Some(ips) = context.lock().unwrap().ip_addrs.clone() {
            return ips.to_vec();
        }
    }
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
    let mut ips = Vec::new();
    let parsed = match IpAddr::from_str(service.address()) {
        Ok(addr) => addr,
        Err(e) => {
            tracing::error!("{}", e);
            panic!("Original {} , error {}", service.address(), e);
        }
    };
    if parsed.is_ipv4() {
        ips.push(SocketAddr::new(parsed, *service.port()));
        context.lock().unwrap().ip_addrs = Some(ips);
    }
    // ...
}
