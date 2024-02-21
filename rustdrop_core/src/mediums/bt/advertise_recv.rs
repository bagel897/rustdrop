use std::collections::HashMap;

use base64::{prelude::BASE64_URL_SAFE, Engine};
use bytes::{BufMut, BytesMut};
use tracing::info;

use crate::{core::protocol::get_endpoint_info, mediums::bt::consts::SERVICE_ID, Config};

use super::consts::PCP;
pub(super) fn get_name(config: &Config) -> String {
    let mut result = BytesMut::new();
    result.put_u8(PCP);
    result.extend_from_slice(config.endpoint_id.as_bytes());
    result.extend_from_slice(&SERVICE_ID);
    result.put_u8(0x0);
    result.extend_from_slice(&BytesMut::zeroed(6));
    let endpoint_info = get_endpoint_info(config);
    info!("{:?}", endpoint_info);
    result.put_u8(endpoint_info.len().try_into().unwrap());
    result.extend_from_slice(&endpoint_info);
    result.put_u8((result.len() + 1).try_into().unwrap());
    let e: HashMap<usize, u8> = result.to_vec().iter().cloned().enumerate().collect();
    info!("{:#X?}", result.to_vec());
    BASE64_URL_SAFE.encode(result)
}
