use std::io::Write;

pub const HEADER: [u8; 2] = ['B' as u8, 'R' as u8];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProtocolMessage {
    pub payload_length: u16,
    pub message_id: u16,
    pub src_device_id: u8,
    pub dst_device_id: u8,
    pub payload: Vec<u8>,
    pub checksum: u16,
}

impl ProtocolMessage {
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

    pub fn new() -> Self {
        Default::default()
    }

    // Assuming PingMessage is a trait that your code defines
    pub fn set_message(&mut self, message: &impl PingMessage) {
        self.message_id = message.message_id();
        self.payload = message.serialize(); // Assuming serialize returns Vec<u8>
        self.payload_length = self.payload.len() as u16;
        self.update_checksum();
    }

    #[inline]
    pub fn set_src_device_id(&mut self, src_device_id: u8) {
        self.src_device_id = src_device_id;
        self.update_checksum();
    }

    #[inline]
    pub fn dst_device_id(&self) -> u8 {
        self.dst_device_id
    }

    #[inline]
    pub fn set_dst_device_id(&mut self, dst_device_id: u8) {
        self.dst_device_id = dst_device_id;
        self.update_checksum();
    }

    #[inline]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    #[inline]
    pub fn checksum(&self) -> u16 {
        self.checksum
    }

    #[inline]
    pub fn update_checksum(&mut self) {
        self.checksum = self.calculate_crc();
    }

    pub fn calculate_crc(&self) -> u16 {
        let mut checksum: u16 = 0;
        checksum += HEADER[0] as u16;
        checksum += HEADER[1] as u16;
        self.payload_length
            .to_le_bytes()
            .iter()
            .for_each(|byte| checksum += *byte as u16);
        self.message_id
            .to_le_bytes()
            .iter()
            .for_each(|byte| checksum += *byte as u16);
        checksum += self.src_device_id as u16;
        checksum += self.dst_device_id as u16;
        for &byte in &self.payload {
            checksum += byte as u16;
        }
        checksum
    }

    pub fn has_valid_crc(&self) -> bool {
        self.checksum == self.calculate_crc()
    }

    pub fn length(&self) -> usize {
        HEADER.len() + 2 + 2 + 1 + 1 + self.payload_length as usize + 2
    }

    pub fn write(&self, writer: &mut dyn Write) -> std::io::Result<usize> {
        let data = self.serialized();
        writer.write_all(&data)?;
        Ok(data.len())
    }

    pub fn serialized(&self) -> Vec<u8> {
        let mut serialized_data = Vec::with_capacity(self.length());
        serialized_data.extend_from_slice(&HEADER);
        serialized_data.extend_from_slice(&self.payload_length.to_le_bytes());
        serialized_data.extend_from_slice(&self.message_id.to_le_bytes());
        serialized_data.push(self.src_device_id);
        serialized_data.push(self.dst_device_id);
        serialized_data.extend_from_slice(&self.payload);
        serialized_data.extend_from_slice(&self.checksum.to_le_bytes());
        serialized_data
    }
}

// This information is only related to the message itself,
// not the entire package with header, dst, src and etc.
pub trait PingMessage
where
    Self: Sized + SerializePayload + SerializePayload,
{
    fn message_id(&self) -> u16;
    fn message_name(&self) -> &'static str;

    fn message_id_from_name(name: &str) -> Result<u16, String>;
}

pub trait SerializePayload {
    fn serialize(&self) -> Vec<u8>;
}

pub trait DeserializePayload {
    fn deserialize(payload: &[u8]) -> Self;
}

pub trait DeserializeGenericMessage
where
    Self: Sized,
{
    fn deserialize(message_id: u16, payload: &[u8]) -> Result<Self, &'static str>;
}
