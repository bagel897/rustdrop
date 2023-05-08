use std::time::Instant;
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
    ip_addrs: Vec<SocketAddr>,
}
pub fn get_dests() -> Vec<SocketAddr> {
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
    return context.lock().unwrap().ip_addrs.clone();
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
        let addr = SocketAddr::new(parsed, *service.port());
        context.lock().unwrap().ip_addrs.push(addr);
    }
    // ...
}
