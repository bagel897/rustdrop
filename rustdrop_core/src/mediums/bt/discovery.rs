use std::{
    any,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use bluer::{Adapter, Address, AddressType, DeviceEvent::PropertyChanged, DeviceProperty, UuidExt};
use futures::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{debug, event, info, span, Level};
use uuid::Uuid;

use super::consts::{SERVICE_UUID, SERVICE_UUID_NEW, SERVICE_UUID_SHARING};
use crate::{
    core::bits::{Bitfield, BleName, BluetoothName},
    mediums::{
        bt::consts::{BLE_CHAR, SERVICE_UUID_RECIEVING},
        Discover, Discovery,
    },
    runner::DiscoveringHandle,
    Context, Device, DeviceType, RustdropResult,
};
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BluetoothDiscovery {
    addr: Address,
    service: Uuid,
}
impl Discovery for BluetoothDiscovery {
    async fn into_socket(
        self,
    ) -> RustdropResult<(
        impl AsyncRead + Send + Sync + Unpin,
        impl AsyncWrite + Send + Sync + Unpin,
    )> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;
        let dev = adapter.device(self.addr)?;
        dev.connect_profile(&self.service).await?;
        let services = dev.services().await?;
        for service in services {
            info!("Service {:?}", service);
            if service.uuid().await? == self.service {
                for char in service.characteristics().await? {
                    if char.uuid().await? == self.service {
                        return Ok((char.notify_io().await?, char.write_io().await?));
                    }
                }
            }
        }

        todo!();
    }
}
struct DiscoveringBluetooth {
    dev: bluer::Device,
    addr: Address,
    name: String,
    send: DiscoveringHandle,
    to_connect: HashSet<Uuid>,
}
impl DiscoveringBluetooth {
    async fn init(&mut self) -> RustdropResult<()> {
        info!("INIT");
        if self.dev.is_services_resolved().await? {
            if let Some(name) = self.handle_receiving_ble().await? {
                todo!()
            }
        }
        Ok(())
    }
    async fn connect_profiles(&mut self, services: HashSet<Uuid>) -> RustdropResult<bool> {
        let mut any_profiles: bool = false;
        for profile in self.to_connect.iter() {
            if services.contains(profile) {
                info!("Found {}", profile);
                any_profiles = true;
                // info!("{:?}", self.dev.connect_profile(profile).await);
            }
        }
        if any_profiles && !self.dev.is_connected().await? {
            info!("{:?}", self.dev.connect().await);
        }
        Ok(any_profiles)
    }
    async fn handle_receiving_ble(&mut self) -> RustdropResult<Option<BleName>> {
        if let Ok(service) = self
            .dev
            .service(SERVICE_UUID_RECIEVING.as_u16().unwrap())
            .await
        {
            info!("Found receiving service: {:?}", service.uuid().await);
            for char in service.characteristics().await? {
                info!("Char {:?}", char);
                if char.uuid().await? == BLE_CHAR {
                    info!("Char :{:?}", char);
                }
            }
        }
        Ok(None)
    }
    async fn try_process_service_data(&mut self) -> RustdropResult<()> {
        if let Some(services) = self.dev.service_data().await? {
            self.process_service_data(services).await?;
        }
        Ok(())
    }

    async fn process_service_data(
        &mut self,
        service_data: HashMap<Uuid, Vec<u8>>,
    ) -> RustdropResult<()> {
        info!("Services: {:?}", service_data);
        for (id, service) in service_data {
            info!("{}: {:?}", id, BleName::decode_base64(&service));
            info!("{}: {:?}", id, BleName::decode(&service));
        }
        Ok(())
    }
    async fn handle_recv_bt(&mut self) {}
    #[tracing::instrument(fields(addr=%self.addr, name=self.name), skip_all)]

    pub async fn handle_events(mut self) -> RustdropResult<()> {
        if let Some(services) = self.dev.uuids().await? {
            if !self.connect_profiles(services.clone()).await? {
                return Ok(());
            }
        }
        self.try_process_service_data().await?;
        self.init().await?;
        let mut events = self.dev.events().await?;
        while let Some(event) = events.next().await {
            match event {
                PropertyChanged(DeviceProperty::ServiceData(data)) => {
                    self.process_service_data(data).await?;
                }
                PropertyChanged(DeviceProperty::Uuids(uuids)) => {
                    if !self.connect_profiles(uuids).await? {
                        return Ok(());
                    }
                }
                PropertyChanged(DeviceProperty::Connected(true)) => {
                    info!("Connected!");
                    self.init().await?
                }
                PropertyChanged(DeviceProperty::ServicesResolved(true)) => self.init().await?,
                PropertyChanged(DeviceProperty::Rssi(_)) => {}
                event => info!("{:?}", event),
            };
        }
        Ok(())
    }
}
pub async fn into_device(dev: bluer::Device, uuid: Uuid) -> RustdropResult<Device> {
    let mut name = dev.name().await?.unwrap_or(dev.alias().await?);
    let decoded = BluetoothName::decode_base64(name.as_bytes());
    info!("Name {}: {:?}", name, decoded);
    let device_type = DeviceType::Unknown;
    if let Some(services) = dev.service_data().await? {
        info!("Services: {:?}", services);
        if let Some(service) = services.get(&SERVICE_UUID_SHARING) {
            if let Ok(adv) = BleName::decode_base64(service) {
                name = adv.name;
            }
        } else if let Some(service) = services.get(&SERVICE_UUID) {
            if let Ok(adv) = BluetoothName::decode_base64(service) {
                name = adv.name;
            }
        } else if let Some(service) = services.get(&SERVICE_UUID_NEW) {
            if let Ok(adv) = BluetoothName::decode_base64(service) {
                name = adv.name;
            }
        }
    }
    if let Ok(services) = dev.services().await {
        for service in services {
            if service.uuid().await? == SERVICE_UUID_RECIEVING {
                info!("Discovered");
                // TODO
            }
            for char in service.characteristics().await? {
                info!("Char :{:?}", char);
            }
        }
    }
    if let Ok(service) = dev.service(SERVICE_UUID_RECIEVING.as_u16().unwrap()).await {
        for char in service.characteristics().await? {
            if char.uuid().await? == BLE_CHAR {
                info!("Char :{:?}", char);
            }
        }
    } else {
    }
    let discovery = BluetoothDiscovery {
        addr: dev.address(),
        service: uuid,
    };
    Ok(Device {
        device_name: name,
        device_type,
        discovery: Discover::Bluetooth(discovery),
    })
}
pub async fn handle_dev(
    addr: Address,
    adapter: &mut Adapter,
    context: &Context,
    send: &DiscoveringHandle,
) -> RustdropResult<()> {
    debug!("Handling {}", addr);
    let dev = adapter.device(addr).unwrap();
    let handle = DiscoveringBluetooth {
        name: dev.name().await?.unwrap_or_default(),
        dev,
        addr,
        send: send.clone(),
        to_connect: [SERVICE_UUID_RECIEVING, SERVICE_UUID].into(),
    };
    context.spawn(async move {
        let e = handle.handle_events().await;
        info!("{:?}", e);
    });
    Ok(())
}
