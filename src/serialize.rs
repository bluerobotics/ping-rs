pub trait PingMessage
where
    Self: Sized,
{
    fn message_id(&self) -> u16;
    fn message_name(&self) -> &'static str;

    fn message_id_from_name(name: &str) -> Result<u16, &'static str>;
}