include!(concat!(env!("OUT_DIR"), "/mod.rs"));

use crate::message::{Deserialize, PingMessage};

const PAYLOAD_SIZE: usize = 255;

use std::fmt;
use std::{convert::TryFrom, io::Write};

pub mod message;

pub const HEADER: [u8; 2] = ['B' as u8, 'R' as u8];

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PingMessagePack([u8; 1 + Self::HEADER_SIZE + PAYLOAD_SIZE + 2]);

impl Default for PingMessagePack {
    fn default() -> Self {
        let mut new = Self([0; 1 + Self::HEADER_SIZE + PAYLOAD_SIZE + 2]);
        new.0[0] = HEADER[0];
        new.0[1] = HEADER[1];
        new
    }
}

impl<T: PingMessage> From<&T> for PingMessagePack {
    fn from(message: &T) -> Self {
        let mut new: Self = Default::default();
        new.set_message(message);
        new
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

impl TryFrom<&Vec<u8>> for Messages {
    type Error = String; // TODO: define error types for each kind of failure

    fn try_from(buffer: &Vec<u8>) -> Result<Self, Self::Error> {
        // Parse start1 and start2
        if !((buffer[0] == HEADER[0]) && (buffer[1] == HEADER[1]))
        {
            return Err(format!("Message should start with \"BR\" ASCII sequence, received: [{0}({:0x}), {1}({:0x})]", buffer[0], buffer[1]));
        }

        // Get the package data
        let payload_length = u16::from_le_bytes([buffer[2], buffer[3]]);
        let _message_id = u16::from_le_bytes([buffer[4], buffer[5]]);
        let _src_device_id = buffer[6];
        let _dst_device_id = buffer[7];
        let _payload = &buffer[8..(8 + payload_length) as usize];
        let crc = u16::from_le_bytes([
            buffer[(8 + payload_length) as usize],
            buffer[(8 + payload_length + 1) as usize],
        ]);
        let calculated_crc = calculate_crc(&buffer[..buffer.len() - 2]);
        if crc != calculated_crc {
            return Err(format!(
                "Missmatch crc, expected: 0x{calculated_crc:04x}, received: 0x{crc:04x}"
            ));
        }

        // Try to parse with each module
        if let Ok(message) = bluebps::Messages::deserialize(buffer) {
            return Ok(Messages::Bluebps(message));
        }
        if let Ok(message) = common::Messages::deserialize(buffer) {
            return Ok(Messages::Common(message));
        }
        if let Ok(message) = ping1d::Messages::deserialize(buffer) {
            return Ok(Messages::Ping1d(message));
        }
        if let Ok(message) = ping360::Messages::deserialize(buffer) {
            return Ok(Messages::Ping360(message));
        }

        Err("Unknown message".into())
    }
}

impl PingMessagePack {
    /**
     * Message Format
     *
     * Each message consists of a header, optional payload, and checksum. The binary format is specified as follows:
     *
     * | Byte        | Type | Name           | Description                                                                                               |
     * |-------------|------|----------------|-----------------------------------------------------------------------------------------------------------|
     * | 0           | u8   | start1         | Start frame identifier, ASCII 'B'                                                                         |
     * | 1           | u8   | start2         | Start frame identifier, ASCII 'R'                                                                         |
     * | 2-3         | u16  | payload_length | Number of bytes in payload.                                                                               |
     * | 4-5         | u16  | message_id     | The message id.                                                                                           |
     * | 6           | u8   | src_device_id  | The device ID of the device sending the message.                                                          |
     * | 7           | u8   | dst_device_id  | The device ID of the intended recipient of the message.                                                   |
     * | 8-n         | u8[] | payload        | The message payload.                                                                                      |
     * | (n+1)-(n+2) | u16  | checksum       | The message checksum. The checksum is calculated as the sum of all the non-checksum bytes in the message. |
     */

    const HEADER_SIZE: usize = 8;

    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_message(&mut self, message: &impl PingMessage) {
        let message_id = message.message_id();
        let (left, right) = self.0.split_at_mut(Self::HEADER_SIZE);
        let length = message.serialize(right) as u16;

        // Set payload_length
        left[2..=3].copy_from_slice(&length.to_le_bytes());

        // Set message_id
        left[4..=5].copy_from_slice(&message_id.to_le_bytes());

        self.update_checksum();
    }

    #[inline]
    pub fn payload_length(&self) -> u16 {
        u16::from_le_bytes([self.0[2], self.0[3]])
    }

    #[inline]
    pub fn message_id(&self) -> u16 {
        u16::from_le_bytes([self.0[4], self.0[5]])
    }

    #[inline]
    pub fn src_device_id(&self) -> u8 {
        self.0[6]
    }

    #[inline]
    pub fn set_src_device_id(&mut self, src_device_id: u8) {
        self.0[6] = src_device_id;
        self.update_checksum();
    }

    #[inline]
    pub fn dst_device_id(&self) -> u8 {
        self.0[7]
    }

    #[inline]
    pub fn set_dst_device_id(&mut self, dst_device_id: u8) {
        self.0[7] = dst_device_id;
        self.update_checksum();
    }

    pub fn payload(&self) -> &[u8] {
        let payload_length: usize = self.payload_length().into();
        &self.0[Self::HEADER_SIZE..(Self::HEADER_SIZE + payload_length)]
    }

    pub fn checksum(&self) -> u16 {
        let payload_length: usize = self.payload_length().into();
        let index_start_checksum = Self::HEADER_SIZE + payload_length;
        u16::from_le_bytes([
            self.0[index_start_checksum],
            self.0[index_start_checksum + 1],
        ])
    }

    pub fn update_checksum(&mut self) {
        let payload_length: usize = self.payload_length().into();
        let index_start_checksum = Self::HEADER_SIZE + payload_length;
        let checksum = self.calculate_crc();
        self.0[index_start_checksum..=(index_start_checksum + 1)]
            .copy_from_slice(&checksum.to_le_bytes());
    }

    pub fn calculate_crc(&self) -> u16 {
        let payload_length: usize = self.payload_length().into();
        let array = &self.0[0..(Self::HEADER_SIZE + payload_length)];
        return calculate_crc(array);
    }

    pub fn has_valid_crc(&self) -> bool {
        self.checksum() == self.calculate_crc()
    }

    pub fn length(&self) -> usize {
        Self::HEADER_SIZE + self.payload_length() as usize + 2
    }

    pub fn write(&self, writer: &mut dyn Write) -> std::io::Result<usize> {
        let length = self.length();
        writer.write_all(&self.0[..length])?;
        Ok(length)
    }

    pub fn serialized(&self) -> &[u8] {
        return &self.0[0..self.length()];
    }
}

impl fmt::Debug for PingMessagePack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PingMessagePack")
            .field("start1", &self.0[0])
            .field("start2", &self.0[1])
            .field("payload_length", &self.payload_length())
            .field("message_id", &self.message_id())
            .field("src_device_id", &self.src_device_id())
            .field("dst_device_id", &self.dst_device_id())
            .field("payload", &self.payload())
            .field("checksum", &self.checksum())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
