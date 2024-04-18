use crate::{common::NackStruct, decoder, message::ProtocolMessage};

#[derive(Debug)]
pub enum PingError {
    Io(std::io::Error),
    ParseError(decoder::ParseError),
    TokioBroadcastError(tokio::sync::broadcast::error::RecvError),
    TokioMpscError(tokio::sync::mpsc::error::SendError<ProtocolMessage>),
    JoinError,
    TimeoutError,
    TryFromError(ProtocolMessage),
    NackError(String),
}

impl From<std::io::Error> for PingError {
    fn from(err: std::io::Error) -> PingError {
        PingError::Io(err)
    }
}

impl From<tokio_serial::Error> for PingError {
    fn from(err: tokio_serial::Error) -> PingError {
        PingError::Io(std::io::Error::new(std::io::ErrorKind::Other, err))
    }
}
