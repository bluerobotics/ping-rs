use crate::decoder;

#[derive(Debug)]
pub enum PingError {
    Io(std::io::Error),
    ParseError(decoder::ParseError),
}

impl From<std::io::Error> for PingError {
    fn from(err: std::io::Error) -> PingError {
        PingError::Io(err)
    }
}
