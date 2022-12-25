include!(concat!(env!("OUT_DIR"), "/mod.rs"));

use crate::serialize::PingMessage;

const PAYLOAD_SIZE: usize = 255;

use std::io::Write;

use crc_any::CRCu16;

pub mod serialize;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PingMessagePack([u8; 1 + Self::HEADER_SIZE + PAYLOAD_SIZE + 2]);

impl Default for PingMessagePack {
    fn default() -> Self {
        let mut new = Self::new();
        new.0[0] = 'B' as u8;
        new.0[1] = 'R' as u8;
        new
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

    pub const fn new() -> Self {
        Self([0; 1 + Self::HEADER_SIZE + PAYLOAD_SIZE + 2])
    }

    pub fn from(message: impl PingMessage) -> Self {
        let mut new: Self = Default::default();
        new.set_message(message);
        new
    }

    pub fn set_message(&mut self, message: impl PingMessage) {
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
    }

    #[inline]
    pub fn dst_device_id(&self) -> u8 {
        self.0[7]
    }

    #[inline]
    pub fn set_dst_device_id(&mut self, dst_device_id: u8) {
        self.0[7] = dst_device_id;
    }

    pub fn payload(&self) -> &[u8] {
        let payload_length: usize = self.payload_length().into();
        &self.0[Self::HEADER_SIZE..=(Self::HEADER_SIZE + payload_length)]
    }

    pub fn checksum(&self) -> u16 {
        let payload_length: usize = self.payload_length().into();
        let index_start_checksum = 1 + Self::HEADER_SIZE + payload_length;
        u16::from_le_bytes([
            self.0[index_start_checksum],
            self.0[index_start_checksum + 1],
        ])
    }

    pub fn update_checksum(&mut self) {
        let payload_length: usize = self.payload_length().into();
        let index_start_checksum = 1 + Self::HEADER_SIZE + payload_length;
        let checksum = self.calculate_crc();
        self.0[index_start_checksum..=(index_start_checksum + 1)]
            .copy_from_slice(&checksum.to_le_bytes());
    }

    pub fn calculate_crc(&self) -> u16 {
        let payload_length: usize = self.payload_length().into();
        let mut crc_calculator = CRCu16::crc16mcrf4cc();
        crc_calculator.digest(&self.0[0..=(Self::HEADER_SIZE + payload_length)]);
        crc_calculator.get_crc()
    }

    pub fn has_valid_crc(&self) -> bool {
        self.checksum() == self.calculate_crc()
    }

    pub fn length(&self) -> usize {
        Self::HEADER_SIZE + self.payload_length() as usize + 2
    }

    pub fn write(&self, writer: &mut Write) -> std::io::Result<usize> {
        let length = self.length();
        writer.write_all(&self.0[..length])?;
        Ok(length)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
