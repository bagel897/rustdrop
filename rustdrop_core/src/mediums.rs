use self::{bt::BluetoothDiscovery, wlan::WlanDiscovery};

pub mod bt;
mod generic;
pub mod wlan;
pub use generic::{Discovery, Medium};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Discover {
    Wlan(WlanDiscovery),
    Bluetooth(BluetoothDiscovery),
}
