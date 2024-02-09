use std::collections::HashSet;

use bluer::{
    monitor::{MonitorEvent, MonitorManager},
    rfcomm::Profile,
    Adapter, AdapterEvent, Device, DeviceEvent, DiscoveryFilter, Session, Uuid,
};
use bytes::Bytes;
use tokio::{
    select,
    sync::mpsc::{self, UnboundedReceiver},
};
use tokio_stream::StreamExt;
use tracing::{info, trace};

use crate::{
    core::RustdropError,
    mediums::bt::ble::{get_advertisment, get_monitor, process_device},
    Application, UiHandle,
};

use super::{
    advertise_recv::get_name,
    consts::{
        SERVICE_DATA, SERVICE_ID_BLE, SERVICE_UUID, SERVICE_UUID_RECIEVING, SERVICE_UUID_SHARING,
    },
};

pub(crate) struct Bluetooth<U: UiHandle> {
    session: Session,
    adapter: Adapter,
    app: Application<U>,
    monitor_manager: MonitorManager,
}
impl<U: UiHandle> Bluetooth<U> {
    pub async fn new(app: Application<U>) -> Result<Self, RustdropError> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;
        let mm = adapter.monitor().await?;
        Ok(Self {
            session,
            adapter,
            app,
            monitor_manager: mm,
        })
    }
    async fn adv_profile(&mut self, profile: Profile, name: String) -> Result<(), RustdropError> {
        self.adapter.set_discoverable(true).await?;
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
            // role: Some(bluer::rfcomm::Role::Server),
            name: Some(name.clone()),
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
            uuids: HashSet::from([SERVICE_UUID, SERVICE_UUID_SHARING, SERVICE_UUID_RECIEVING]),
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
                info!("Discovered {:?}", dev.all_properties().await?);
            }
        }
        Ok(())
    }
    async fn advertise(
        &mut self,
        service_id: String,
        service_uuid: Uuid,
        adv_data: Bytes,
    ) -> Result<(), RustdropError> {
        info!(
            "Advertising on Bluetooth adapter {} with address {}",
            self.adapter.name(),
            self.adapter.address().await?
        );
        let le_advertisement = get_advertisment(service_id, service_uuid, adv_data);
        let cancel = self.app.child_token();
        info!("{:?}", &le_advertisement);
        let handle = self.adapter.advertise(le_advertisement).await.unwrap();
        self.app.spawn(
            async move {
                cancel.cancelled().await;
                info!("Removing advertisement");
                drop(handle);
            },
            "ble advertise",
        );
        Ok(())
    }
    async fn scan_le(
        &mut self,
        services: Vec<Uuid>,
    ) -> Result<(UnboundedReceiver<Device>, UnboundedReceiver<DeviceEvent>), RustdropError> {
        let mut monitor_handle = self.monitor_manager.register(get_monitor(services)).await?;
        info!("Scanning BLE");
        let (devices_tx, devices_rx) = mpsc::unbounded_channel();
        let (events_tx, events_rx) = mpsc::unbounded_channel();
        let adapter = self.adapter.clone();
        self.app.spawn(
            async move {
                while let Some(mevt) = monitor_handle.next().await {
                    if let MonitorEvent::DeviceFound(devid) = mevt {
                        info!("Discovered device {:?}", devid);
                        let dev = adapter.device(devid.device).unwrap();
                        process_device(dev, &devices_tx, &events_tx).await;
                    }
                }
                info!("Closing BLE scan");
            },
            "ble_advertise",
        );
        Ok((devices_rx, events_rx))
    }
    pub async fn scan_for_incoming(&mut self) -> Result<(), RustdropError> {
        self.advertise(SERVICE_ID_BLE.into(), SERVICE_UUID_RECIEVING, SERVICE_DATA)
            .await?;
        let (mut devices, mut events) = self.scan_le(vec![SERVICE_UUID_SHARING]).await?;
        self.app.spawn(
            async move {
                loop {
                    select! {
                        dev = devices.recv() => {
                            info!("{:?}", dev)
                        }
                        event = events.recv() => {
                            info!("{:?}", event)
                        }
                    }
                }
            },
            "discovery_process",
        );
        Ok(())
    }
    pub async fn trigger_reciever(&mut self) -> Result<(), RustdropError> {
        self.advertise(SERVICE_ID_BLE.into(), SERVICE_UUID_SHARING, SERVICE_DATA)
            .await?;
        let (mut devices, mut events) = self.scan_le(vec![SERVICE_UUID_RECIEVING]).await?;
        self.app.spawn(
            async move {
                loop {
                    select! {
                        dev = devices.recv() => {
                            info!("{:?}", dev)
                        }
                        event = events.recv() => {
                            info!("{:?}", event)
                        }
                    }
                }
            },
            "discovery_process",
        );
        Ok(())
    }
}
