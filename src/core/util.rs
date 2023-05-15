use rand_new::{thread_rng, RngCore};

use crate::protobuf::sharing::nearby::PairedKeyEncryptionFrame;

pub fn get_random(bytes: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut resp_buf = vec![0u8; bytes];
    rng.fill_bytes(&mut resp_buf);
    return resp_buf;
}
pub fn get_iv() -> [u8; 16] {
    let mut rng = thread_rng();
    let mut resp_buf = [0u8; 16];
    rng.fill_bytes(&mut resp_buf);
    return resp_buf;
}
pub fn iv_from_vec(vec: Vec<u8>) -> [u8; 16] {
    let mut buf = [0u8; 16];
    for i in 0..16 {
        buf[i] = *vec.get(i).unwrap();
    }
    return buf;
}
pub fn get_paired_frame() -> PairedKeyEncryptionFrame {
    let mut p_key = PairedKeyEncryptionFrame::default();
    p_key.secret_id_hash = Some(get_random(6));
    p_key.signed_data = Some(get_random(72));
    p_key
}
