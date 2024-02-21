pub struct EndpointInfo {}
struct Advertisment {
    // endpoint_id: i32,
    pub name: String,
    mac: String,
}
impl Advertisment {
    pub fn parse_bytes(raw: &[u8]) -> Self {
        let name_size = raw[35] as usize;
        Self {
            // endpoint_id: raw[13..16].get_i32(),
            name: String::from_utf8(raw[36..(36 + name_size)].into()).unwrap(),
            mac: String::from_utf8(raw[41..46].into()).unwrap(),
        }
    }
}
