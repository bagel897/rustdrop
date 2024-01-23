use std::error::Error;

use bluer::monitor::{
    data_type::{
        COMPLETE_LIST_128_BIT_SERVICE_CLASS_UUIDS, COMPLETE_LIST_16_BIT_SERVICE_CLASS_UUIDS,
        INCOMPLETE_LIST_128_BIT_SERVICE_CLASS_UUIDS, INCOMPLETE_LIST_16_BIT_SERVICE_CLASS_UUIDS,
    },
    Monitor, MonitorEvent, Pattern, RssiSamplingPeriod,
};
use futures::StreamExt;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::ble::consts::SERVICE_UUID16;

use super::consts::SERVICE_UUID;
pub(crate) async fn scan_for_ble(cancel: CancellationToken) -> Result<(), Box<dyn Error>> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    let mm = adapter.monitor().await?;
    adapter.set_powered(true).await?;
    let pattern = Pattern {
        data_type: COMPLETE_LIST_16_BIT_SERVICE_CLASS_UUIDS,
        start_position: 0x00,
        content: SERVICE_UUID16.to_vec(),
    };
    info!("Scanning for {:?}", pattern);
    let mut monitor_handle = mm
        .register(Monitor {
            monitor_type: bluer::monitor::Type::OrPatterns,
            rssi_low_threshold: None,
            rssi_high_threshold: None,
            rssi_low_timeout: None,
            rssi_high_timeout: None,
            rssi_sampling_period: Some(RssiSamplingPeriod::First),
            // patterns: Some(vec![Pattern {
            //     data_type: INCOMPLETE_LIST_128_BIT_SERVICE_CLASS_UUIDS,
            //     start_position: 0x00,
            //     content: SERVICE_UUID.into_bytes().to_vec(),
            // }]),
            patterns: Some(vec![pattern]),
            ..Default::default()
        })
        .await?
        .fuse();
    info!("Scanning BLE");
    loop {
        select! {
            _ = cancel.cancelled() => {
            break;
            }
            mevt = monitor_handle.select_next_some() => {
        if let MonitorEvent::DeviceFound(devid) = mevt {
            info!("Discovered device {:?}", devid);
            let dev = adapter.device(devid.device)?;
            tokio::spawn(async move {
                let mut events = dev.events().await.unwrap();
                while let Some(ev) = events.next().await {
                    info!("On device {:?}, received event {:?}", dev, ev);
                }
            });
        }


            }
        }
    }
    info!("Closing BLE scan");
    Ok(())
}
