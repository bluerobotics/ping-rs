pub const HEADER: [u8; 2] = ['B' as u8, 'R' as u8];

#[derive(Clone, Debug)]
pub struct ProtocolMessage {
    pub payload_length: u16,
    pub message_id: u16,
    pub src_device_id: u8,
    pub dst_device_id: u8,
    pub payload: Vec<u8>,
    pub checksum: u16,
}

impl ProtocolMessage {
    pub fn new() -> Self {
        ProtocolMessage {
            payload_length: 0,
            message_id: 0,
            src_device_id: 0,
            dst_device_id: 0,
            payload: Vec::new(),
            checksum: 0,
        }
    }
}

pub trait PingMessage
where
    Self: Sized + Serialize + Deserialize,
{
    fn message_id(&self) -> u16;
    fn message_name(&self) -> &'static str;

    fn message_id_from_name(name: &str) -> Result<u16, String>;
}

pub trait Serialize {
    fn serialize(&self, buffer: &mut [u8]) -> usize;
}

pub trait Deserialize
where
    Self: Sized,
{
    fn deserialize(buffer: &[u8]) -> Result<Self, &'static str>;
}
