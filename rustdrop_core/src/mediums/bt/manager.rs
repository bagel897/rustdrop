use std::collections::HashSet;

use bluer::{rfcomm::Profile, Adapter, AdapterEvent, DiscoveryFilter, Session};
use tokio_stream::StreamExt;
use tracing::{info, trace};

use crate::{core::RustdropError, Application, UiHandle};

use super::{advertise_recv::get_name, consts::SERVICE_UUID};

pub(crate) struct Bluetooth<U: UiHandle> {
    session: Session,
    adapter: Adapter,
    app: Application<U>,
}
impl<U: UiHandle> Bluetooth<U> {
    pub async fn new(app: Application<U>) -> Result<Self, RustdropError> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        Ok(Self {
            session,
            adapter,
            app,
        })
    }
    async fn adv_profile(&mut self, profile: Profile, name: String) -> Result<(), RustdropError> {
        let mut handle = self.session.register_profile(profile).await?;
        let cancel = self.app.child_token();
        info!(
            "Advertising on Bluetooth adapter {} with name {}",
            self.adapter.name(),
            name
        );
        self.app.spawn(
            async move {
                while let Some(req) = handle.next().await {
                    info!("{:?}", req);
                }
                info!("No more requests");
                cancel.cancelled().await;
            },
            "BT Adv",
        );
        Ok(())
    }
    pub(crate) async fn adv_bt(&mut self) -> Result<(), RustdropError> {
        // self.discover_bt().await?;
        let name = get_name(&self.app.config);
        let profile = Profile {
            uuid: SERVICE_UUID,
            role: Some(bluer::rfcomm::Role::Server),
            // name: Some(name),
            require_authentication: Some(false),
            require_authorization: Some(false),
            channel: Some(0),
            psm: Some(0),
            auto_connect: Some(true),
            ..Default::default()
        };
        self.adv_profile(profile, name).await?;
        Ok(())
    }
    pub(crate) async fn discover_bt(&mut self) -> Result<(), RustdropError> {
        let filter = DiscoveryFilter {
            uuids: HashSet::from([SERVICE_UUID]),
            // transport: bluer::DiscoveryTransport::Auto,
            ..Default::default()
        };
        self.discover(filter).await?;
        Ok(())
    }
    async fn discover(&mut self, filter: DiscoveryFilter) -> Result<(), RustdropError> {
        self.adapter.set_discovery_filter(filter).await?;
        let mut discover = self.adapter.discover_devices().await?;
        while let Some(discovery) = discover.next().await {
            trace!("{:?}", discovery);
            if let AdapterEvent::DeviceAdded(addr) = discovery {
                let dev = self.adapter.device(addr)?;
                info!("{:?}", dev.all_properties().await?);
            }
        }
        Ok(())
    }
}
