use rand::{thread_rng, RngCore};

use crate::protobuf::securegcm::Ukey2Alert;

pub fn get_random(bytes: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut resp_buf = vec![0u8; bytes];
    rng.fill_bytes(&mut resp_buf);
    resp_buf
}
pub fn get_iv() -> [u8; 16] {
    let mut rng = thread_rng();
    let mut resp_buf = [0u8; 16];
    rng.fill_bytes(&mut resp_buf);
    resp_buf
}
pub fn iv_from_vec(vec: Vec<u8>) -> [u8; 16] {
    let mut buf = [0u8; 16];
    for i in 0..16 {
        buf[i] = *vec.get(i).unwrap();
    }
    buf
}
pub fn ukey_alert_to_str(alert: Ukey2Alert) -> String {
    format!(
        "Ukey2Alert: type: {}, error_message: {}",
        alert.r#type().as_str_name(),
        String::from_utf8(alert.error_message().as_bytes().to_vec()).unwrap()
    )
}
