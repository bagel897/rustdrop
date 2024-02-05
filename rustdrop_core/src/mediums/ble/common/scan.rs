use std::error::Error;

use bluer::{
    monitor::{
        data_type::COMPLETE_LIST_128_BIT_SERVICE_CLASS_UUIDS, Monitor, MonitorHandle, Pattern,
        RssiSamplingPeriod,
    },
    Adapter,
};
use tracing::info;
use uuid::Uuid;

pub(crate) async fn scan_le(
    services: Vec<Uuid>,
) -> Result<(Adapter, MonitorHandle), Box<dyn Error>> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    let mm = adapter.monitor().await?;
    adapter.set_powered(true).await?;
    let pattern = services
        .into_iter()
        .map(|uuid| Pattern {
            data_type: COMPLETE_LIST_128_BIT_SERVICE_CLASS_UUIDS,
            start_position: 0x00,
            content: uuid.to_bytes_le().to_vec(),
        })
        .collect();
    info!("Scanning for {:?}", pattern);
    let monitor_handle = mm
        .register(Monitor {
            monitor_type: bluer::monitor::Type::OrPatterns,
            rssi_low_threshold: None,
            rssi_high_threshold: None,
            rssi_low_timeout: None,
            rssi_high_timeout: None,
            rssi_sampling_period: Some(RssiSamplingPeriod::First),
            patterns: Some(pattern),
            ..Default::default()
        })
        .await?;
    info!("Scanning BLE");
    Ok((adapter, monitor_handle))
}
