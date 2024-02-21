use bluer::Address;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::info;
use uuid::Uuid;

use super::consts::SERVICE_UUID_SHARING;
use crate::{
    core::{
        bits::{Bitfield, BleName},
        RustdropError,
    },
    mediums::{Discover, Discovery},
    Device, DeviceType,
};
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BluetoothDiscovery {
    addr: Address,
    service: Uuid,
}
impl Discovery for BluetoothDiscovery {
    async fn into_socket(
        self,
    ) -> Result<
        (
            impl AsyncRead + Send + Sync + Unpin,
            impl AsyncWrite + Send + Sync + Unpin,
        ),
        RustdropError,
    > {
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
pub async fn into_device(dev: bluer::Device, uuid: Uuid) -> Result<Device, RustdropError> {
    let mut name = dev.name().await?.unwrap_or(dev.alias().await?);
    let device_type = DeviceType::Unknown;
    if let Some(services) = dev.service_data().await? {
        if let Some(service) = services.get(&SERVICE_UUID_SHARING) {
            if let Ok(adv) = BleName::decode_base64(service) {
                name = adv.name;
            }
        }
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
