use std::collections::HashSet;

use bluer::{
    monitor::{MonitorEvent, MonitorManager},
    rfcomm::Profile,
    Adapter, AdapterEvent, Device, DeviceEvent, DiscoveryFilter, Session, Uuid,
};
use bytes::Bytes;
use flume::Sender;
use tokio::{
    select,
    sync::mpsc::{self, UnboundedReceiver},
};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::{info, trace};

use super::{
    advertise_recv::get_name,
    consts::{
        SERVICE_DATA, SERVICE_ID_BLE, SERVICE_UUID, SERVICE_UUID_RECIEVING, SERVICE_UUID_SHARING,
    },
    BluetoothDiscovery,
};
use crate::{
    core::RustdropError,
    mediums::{
        bt::{
            ble::{get_advertisment, get_monitor, process_device},
            consts::SERVICE_UUID_NEW,
            discovery::into_device,
        },
        Medium,
    },
    runner::DiscoveringHandle,
    Context, ReceiveEvent,
};

pub(crate) struct Bluetooth {
    session: Session,
    adapter: Adapter,
    context: Context,
    monitor_manager: MonitorManager,
}
impl Bluetooth {
    pub async fn new(context: Context) -> Result<Self, RustdropError> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;
        let mm = adapter.monitor().await?;
        Ok(Self {
            session,
            adapter,
            context,
            monitor_manager: mm,
        })
    }
    async fn adv_profile(
        &mut self,
        profile: Profile,
        name: String,
        send: Sender<ReceiveEvent>,
    ) -> Result<(), RustdropError> {
        self.adapter.set_discoverable(true).await?;
        let mut handle = self.session.register_profile(profile).await?;
        info!(
            "Advertising on Bluetooth adapter {} with name {}",
            self.adapter.name(),
            name
        );
        let mut context = self.context.clone();
        self.context.spawn(
            async move {
                while let Some(req) = handle.next().await {
                    info!("Received BLE request{:?}", req);
                    let child = context.clone();
                    let (rx, tx) = req.accept().unwrap().into_split();
                    let send = send.clone();
                    context.spawn(
                        async move {
                            Self::recieve(rx, tx, child, send).await;
                        },
                        "receiving",
                    );
                }
                info!("No more requests");
            },
            "BT Adv",
        );
        Ok(())
    }
    pub(crate) async fn adv_bt(&mut self, send: Sender<ReceiveEvent>) -> Result<(), RustdropError> {
        // self.discover_bt().await?;
        let name = get_name(&self.context.config);
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
        self.adv_profile(profile, name, send).await?;
        Ok(())
    }
    pub(crate) async fn discover_bt_send(
        &mut self,
        send: DiscoveringHandle,
    ) -> Result<(), RustdropError> {
        let ids = [SERVICE_UUID_SHARING];
        let filter = DiscoveryFilter {
            uuids: HashSet::from(ids),
            transport: bluer::DiscoveryTransport::Auto,
            ..Default::default()
        };
        self.discover(filter, send, ids.into()).await?;
        Ok(())
    }
    pub(crate) async fn discover_bt_recv(
        &mut self,
        send: DiscoveringHandle,
    ) -> Result<(), RustdropError> {
        // When sharing, find devices which are receiving;
        let ids = [SERVICE_UUID_RECIEVING, SERVICE_UUID_NEW, SERVICE_UUID];
        let filter = DiscoveryFilter {
            uuids: HashSet::from(ids),
            transport: bluer::DiscoveryTransport::Auto,
            ..Default::default()
        };
        self.discover(filter, send, ids.into()).await?;
        Ok(())
    }
    async fn discover(
        &mut self,
        filter: DiscoveryFilter,
        send: DiscoveringHandle,
        allowed_ids: HashSet<Uuid>,
    ) -> Result<(), RustdropError> {
        self.adapter.set_discovery_filter(filter).await?;
        let mut discover = self.adapter.discover_devices().await?;
        let adapter = self.adapter.clone();
        self.context.spawn(
            async move {
                while let Some(discovery) = discover.next().await {
                    trace!("{:?}", discovery);
                    match discovery {
                        AdapterEvent::DeviceAdded(addr) => {
                            let dev = adapter.device(addr).unwrap();
                            info!("Discovered {:?}", dev.all_properties().await.unwrap());
                            for uuid in dev.uuids().await.unwrap().unwrap() {
                                if allowed_ids.contains(&uuid) {
                                    let device = into_device(dev.clone(), uuid).await.unwrap();
                                    send.discovered(device.into()).await
                                }
                            }
                        }
                        AdapterEvent::DeviceRemoved(_) => {
                            // send.send_async(DiscoveryEvent::Removed()).await.unwrap();
                        }
                        _ => (),
                    }
                }
            },
            "discover_bt",
        );
        Ok(())
    }
    async fn advertise(
        &mut self,
        service_id: String,
        service_uuid: Uuid,
        adv_data: Bytes,
    ) -> Result<CancellationToken, RustdropError> {
        info!(
            "Advertising on Bluetooth adapter {} with address {}",
            self.adapter.name(),
            self.adapter.address().await?
        );
        let le_advertisement = get_advertisment(service_id, service_uuid, adv_data);
        let cancel = CancellationToken::new();
        let c2 = cancel.child_token();
        info!("{:?}", &le_advertisement);
        let handle = self.adapter.advertise(le_advertisement).await?;
        self.context.spawn(
            async move {
                c2.cancelled().await;
                info!("Removing advertisement");
                drop(handle);
            },
            "ble advertise",
        );
        Ok(cancel)
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
        self.context.spawn(
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
        self.context.spawn(
            async move {
                loop {
                    select! {
                        Some(dev) = devices.recv() => {
                            info!("{:?}", dev)
                        }
                        Some(event) = events.recv() => {
                            info!("{:?}", event)
                        }
                        else => break
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
        self.context.spawn(
            async move {
                loop {
                    select! {
                        Some(dev) = devices.recv() => {
                            info!("{:?}", dev)
                        }
                        Some(event) = events.recv() => {
                            info!("{:?}", event)
                        }
                        else => break
                    }
                }
            },
            "discovery_process",
        );
        Ok(())
    }
}
impl Medium for Bluetooth {
    type Discovery = BluetoothDiscovery;
    async fn start_recieving(&mut self, send: Sender<ReceiveEvent>) -> Result<(), RustdropError> {
        self.scan_for_incoming().await?;
        self.adv_bt(send).await?;
        Ok(())
    }
    async fn discover(&mut self, send: DiscoveringHandle) -> Result<(), RustdropError> {
        self.trigger_reciever().await?;
        self.discover_bt_recv(send).await?;
        Ok(())
    }
}
