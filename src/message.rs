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
