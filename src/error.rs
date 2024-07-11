use crate::{decoder, message::ProtocolMessage};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PingError {
    Io(String),
    ParseError(decoder::ParseError),
    TokioBroadcastError(String),
    TokioMpscError(String),
    JoinError,
    TimeoutError,
    TryFromError(ProtocolMessage),
    NackError(String),
}

impl From<std::io::Error> for PingError {
    fn from(err: std::io::Error) -> PingError {
        PingError::Io(err.to_string())
    }
}

impl From<tokio::sync::mpsc::error::SendError<ProtocolMessage>> for PingError {
    fn from(err: tokio::sync::mpsc::error::SendError<ProtocolMessage>) -> PingError {
        PingError::TokioMpscError(err.to_string())
    }
}

impl From<tokio::sync::broadcast::error::RecvError> for PingError {
    fn from(err: tokio::sync::broadcast::error::RecvError) -> PingError {
        PingError::TokioBroadcastError(err.to_string())
    }
}
