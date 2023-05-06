use crate::core::Config;

use super::mdns::advertise_mdns;

pub(crate) fn init(config: Config) {
    advertise_mdns(config);
}
