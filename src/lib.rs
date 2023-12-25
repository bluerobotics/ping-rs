include!(concat!(env!("OUT_DIR"), "/mod.rs"));

use message::ProtocolMessage;

use crate::message::{DeserializeGenericMessage, HEADER};

use std::convert::TryFrom;

pub mod decoder;
pub mod message;

impl TryFrom<&Vec<u8>> for Messages {
    type Error = String; // TODO: define error types for each kind of failure

    fn try_from(buffer: &Vec<u8>) -> Result<Self, Self::Error> {
        // Parse start1 and start2
        if !((buffer[0] == HEADER[0]) && (buffer[1] == HEADER[1])) {
            return Err(format!("Message should start with \"BR\" ASCII sequence, received: [{0}({:0x}), {1}({:0x})]", buffer[0], buffer[1]));
        }

        let payload_length = u16::from_le_bytes([buffer[2], buffer[3]]);
        let protocol_message = ProtocolMessage {
            payload_length,
            message_id: u16::from_le_bytes([buffer[4], buffer[5]]),
            src_device_id: buffer[6],
            dst_device_id: buffer[7],
            payload: buffer[8..(8 + payload_length) as usize].into(),
            checksum: u16::from_le_bytes([
                buffer[(8 + payload_length) as usize],
                buffer[(8 + payload_length + 1) as usize],
            ]),
        };

        if !protocol_message.has_valid_crc() {
            return Err(format!(
                "Missmatch crc, expected: 0x{:04x}, received: 0x{:04x}",
                protocol_message.calculate_crc(),
                protocol_message.checksum
            ));
        }

        // Try to parse with each module
        if let Ok(message) =
            bluebps::Messages::deserialize(protocol_message.message_id, &protocol_message.payload)
        {
            return Ok(Messages::Bluebps(message));
        }
        if let Ok(message) =
            common::Messages::deserialize(protocol_message.message_id, &protocol_message.payload)
        {
            return Ok(Messages::Common(message));
        }
        if let Ok(message) =
            ping1d::Messages::deserialize(protocol_message.message_id, &protocol_message.payload)
        {
            return Ok(Messages::Ping1d(message));
        }
        if let Ok(message) =
            ping360::Messages::deserialize(protocol_message.message_id, &protocol_message.payload)
        {
            return Ok(Messages::Ping360(message));
        }

        Err("Unknown message".into())
    }
}

pub enum Messages {
    Bluebps(bluebps::Messages),
    Common(common::Messages),
    Ping1d(ping1d::Messages),
    Ping360(ping360::Messages),
}

pub fn calculate_crc(pack_without_payload: &[u8]) -> u16 {
    return pack_without_payload
        .iter()
        .fold(0 as u16, |s, &v| s.wrapping_add(v as u16));
}

impl TryFrom<&ProtocolMessage> for Messages {
    type Error = String; // TODO: define error types for each kind of failure

    fn try_from(message: &ProtocolMessage) -> Result<Self, Self::Error> {
        if !message.has_valid_crc() {
            return Err(format!(
                "Missmatch crc, expected: 0x{:04x}, received: 0x{:04x}",
                message.calculate_crc(),
                message.checksum
            ));
        }

        // Try to parse with each module
        if let Ok(message) = bluebps::Messages::deserialize(message.message_id, &message.payload) {
            return Ok(Messages::Bluebps(message));
        }
        if let Ok(message) = common::Messages::deserialize(message.message_id, &message.payload) {
            return Ok(Messages::Common(message));
        }
        if let Ok(message) = ping1d::Messages::deserialize(message.message_id, &message.payload) {
            return Ok(Messages::Ping1d(message));
        }
        if let Ok(message) = ping360::Messages::deserialize(message.message_id, &message.payload) {
            return Ok(Messages::Ping360(message));
        }

        Err("Unknown message".into())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
