use std::collections::HashSet;

use bluer::{
    monitor::{MonitorEvent, MonitorManager},
    rfcomm::Profile,
    Adapter, AdapterEvent, Device, DeviceEvent, DiscoveryFilter, Session, Uuid,
};
use bytes::Bytes;
use flume::Sender;
use futures::StreamExt;
use tokio::{
    select,
    sync::mpsc::{self, UnboundedReceiver},
};
use tokio_util::sync::CancellationToken;
use tracing::{info, trace};

use super::{
    consts::{
        SERVICE_DATA, SERVICE_ID_BLE, SERVICE_UUID, SERVICE_UUID_RECIEVING, SERVICE_UUID_SHARING,
    },
    BluetoothDiscovery,
};
use crate::{
    core::bits::{Bitfield, BluetoothName},
    mediums::{
        bt::{
            ble::{get_advertisment, get_monitor, process_device},
            consts::SERVICE_UUID_NEW,
            discovery::handle_dev,
        },
        Medium,
    },
    runner::DiscoveringHandle,
    Context, ReceiveEvent, RustdropResult,
};

pub(crate) struct Bluetooth {
    session: Session,
    adapter: Adapter,
    context: Context,
    monitor_manager: MonitorManager,
}
impl Bluetooth {
    pub async fn new(context: Context) -> RustdropResult<Self> {
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
        &self,
        profile: Profile,
        name: String,
        send: Sender<ReceiveEvent>,
    ) -> RustdropResult<()> {
        // self.adapter.set_discoverable(true).await?;
        let mut handle = self.session.register_profile(profile).await?;
        info!(
            "Advertising on Bluetooth adapter {} with name {}",
            self.adapter.name(),
            name
        );
        let context = self.context.clone();
        self.context.spawn(async move {
            while let Some(req) = handle.next().await {
                info!("Received BLE request{:?}", req);
                let child = context.clone();
                let (rx, tx) = req.accept().unwrap().into_split();
                let send = send.clone();
                context.spawn(async move {
                    Self::recieve(rx, tx, child, send).await;
                });
            }
            info!("No more requests");
        });
        Ok(())
    }
    pub(crate) async fn adv_bt(&mut self, send: Sender<ReceiveEvent>) -> RustdropResult<()> {
        // self.discover_bt().await?;
        let name = BluetoothName::new(&self.context.config, self.context.endpoint_info.clone())
            .to_base64();
        let profile = Profile {
            uuid: SERVICE_UUID,
            role: Some(bluer::rfcomm::Role::Server),
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
    pub(crate) async fn discover_bt_send(&mut self, send: DiscoveringHandle) -> RustdropResult<()> {
        let ids = [SERVICE_UUID_SHARING];
        let filter = DiscoveryFilter {
            uuids: HashSet::from(ids),
            transport: bluer::DiscoveryTransport::Auto,
            ..Default::default()
        };
        self.discover(filter, send, ids.into()).await?;
        Ok(())
    }
    pub(crate) async fn discover_bt_recv(&mut self, send: DiscoveringHandle) -> RustdropResult<()> {
        // When sharing, find devices which are receiving;
        let ids = [
            SERVICE_UUID_RECIEVING,
            SERVICE_UUID_NEW,
            SERVICE_UUID,
            SERVICE_UUID_SHARING,
        ];
        let filter = DiscoveryFilter {
            uuids: HashSet::from(ids),
            transport: bluer::DiscoveryTransport::Auto,
            discoverable: true,
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
    ) -> RustdropResult<()> {
        self.adapter.set_discovery_filter(filter).await?;
        let mut discover = self.adapter.discover_devices_with_changes().await?;
        let mut adapter: Adapter = self.adapter.clone();
        for addr in self.adapter.device_addresses().await? {
            handle_dev(addr, &mut self.adapter, &self.context, &send).await?;
        }
        let child = self.context.clone();
        self.context.spawn(async move {
            while let Some(discovery) = discover.next().await {
                trace!("{:?}", discovery);
                match discovery {
                    AdapterEvent::DeviceAdded(addr) => {
                        handle_dev(addr, &mut adapter, &child, &send).await.unwrap();
                    }
                    AdapterEvent::DeviceRemoved(_) => {
                        // send.send_async(DiscoveryEvent::Removed()).await.unwrap();
                    }
                    _ => (),
                }
            }
        });
        Ok(())
    }
    async fn advertise(
        &mut self,
        service_id: String,
        service_uuid: Uuid,
        adv_data: Bytes,
    ) -> RustdropResult<CancellationToken> {
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
        self.context.spawn(async move {
            c2.cancelled().await;
            info!("Removing advertisement");
            drop(handle);
        });
        Ok(cancel)
    }
    async fn scan_le(
        &mut self,
        services: Vec<Uuid>,
    ) -> RustdropResult<(UnboundedReceiver<Device>, UnboundedReceiver<DeviceEvent>)> {
        let mut monitor_handle = self.monitor_manager.register(get_monitor(services)).await?;
        info!("Scanning BLE");
        let (devices_tx, devices_rx) = mpsc::unbounded_channel();
        let (events_tx, events_rx) = mpsc::unbounded_channel();
        let adapter = self.adapter.clone();
        self.context.spawn(async move {
            while let Some(mevt) = monitor_handle.next().await {
                if let MonitorEvent::DeviceFound(devid) = mevt {
                    info!("Discovered device {:?}", devid);
                    let dev = adapter.device(devid.device).unwrap();
                    process_device(dev, &devices_tx, &events_tx).await;
                }
            }
            info!("Closing BLE scan");
        });
        Ok((devices_rx, events_rx))
    }
    pub async fn scan_for_incoming(&mut self) -> RustdropResult<()> {
        self.advertise(SERVICE_ID_BLE.into(), SERVICE_UUID_RECIEVING, SERVICE_DATA)
            .await?;
        let (mut devices, mut events) = self.scan_le(vec![SERVICE_UUID_SHARING]).await?;
        self.context.spawn(async move {
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
        });
        Ok(())
    }
    pub async fn trigger_reciever(&mut self) -> RustdropResult<()> {
        self.advertise(SERVICE_ID_BLE.into(), SERVICE_UUID_SHARING, SERVICE_DATA)
            .await?;
        let (mut devices, mut events) = self.scan_le(vec![SERVICE_UUID_RECIEVING]).await?;
        self.context.spawn(async move {
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
        });
        Ok(())
    }
}
impl Medium for Bluetooth {
    type Discovery = BluetoothDiscovery;
    async fn start_recieving(&mut self, send: Sender<ReceiveEvent>) -> RustdropResult<()> {
        self.scan_for_incoming().await?;
        self.adv_bt(send).await?;
        Ok(())
    }
    async fn discover(&mut self, send: DiscoveringHandle) -> RustdropResult<()> {
        self.trigger_reciever().await?;
        // self.adv_bt_recv().await?;
        self.discover_bt_recv(send).await?;
        Ok(())
    }
}
