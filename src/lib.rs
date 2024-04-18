include!(concat!(env!("OUT_DIR"), "/mod.rs"));

use message::ProtocolMessage;

use crate::message::{DeserializeGenericMessage, HEADER};

use std::convert::TryFrom;

pub mod codec;
pub mod decoder;
pub mod device;
pub mod error;
pub mod message;

pub fn calculate_crc(pack_without_payload: &[u8]) -> u16 {
    return pack_without_payload
        .iter()
        .fold(0 as u16, |s, &v| s.wrapping_add(v as u16));
}
