mod advertisment;
mod bitfield;
mod bluetooth;
mod devtype;
mod endpoint;
mod mdns;
mod service;
pub use devtype::DeviceType;
pub(crate) use {
    bitfield::Bitfield, bluetooth::Name as BluetoothName, endpoint::EndpointInfo,
    mdns::Name as MdnsName,
};
