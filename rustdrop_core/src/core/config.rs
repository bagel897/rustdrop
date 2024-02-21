use std::path::PathBuf;

use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::DeviceType;

#[derive(Clone, Debug)]
pub struct Config {
    pub devtype: DeviceType,
    pub name: String,
    pub dest: PathBuf,
    pub(crate) endpoint_id: u32,
}
impl Default for Config {
    fn default() -> Self {
        let mut rng = thread_rng();
        let mut endpoint: [u8; 4] = rng
            .sample_iter(Alphanumeric)
            .take(4)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();
        Config {
            devtype: DeviceType::Laptop,
            name: hostname::get().unwrap().to_str().unwrap().into(),
            dest: dirs::download_dir()
                .expect("Set an XDG download directory, see isue #3")
                .join("nearby"),
            endpoint_id: u32::from_be_bytes(endpoint),
        }
    }
}
