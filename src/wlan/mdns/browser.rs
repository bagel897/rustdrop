use std::{
    any::Any,
    net::SocketAddr,
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
    service_name: String,
    ip_addrs: Option<Vec<SocketAddr>>,
}
pub fn get_dests() -> Vec<SocketAddr> {
    let mut browser = MdnsBrowser::new(ServiceType::new(TYPE, "tcp").unwrap());

    browser.set_service_discovered_callback(Box::new(on_service_discovered));
    let mut context: Arc<Mutex<Context>> = Arc::default();
    browser.set_context(Box::new(context.clone()));

    let event_loop = browser.browse_services().unwrap();

    loop {
        // calling `poll()` will keep this browser alive
        event_loop.poll(Duration::from_secs(0)).unwrap();
        if let Some(ips) = context.clone().lock().unwrap().ip_addrs.clone() {
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
    let un_parsed = format!("{}:{}", service.address(), service.port());
    let parsed = match SocketAddr::from_str(&un_parsed) {
        Ok(addr) => addr,
        Err(e) => {
            tracing::error!("{}", e);
            panic!("Original {} , error {}", un_parsed, e);
        }
    };
    ips.push(parsed);
    context.lock().unwrap().ip_addrs = Some(ips);
    // ...
}