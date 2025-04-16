use tracing::debug;

use crate::message::{ProtocolMessage, HEADER};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ParseError {
    InvalidStartByte,
    IncompleteData,
    ChecksumError(ProtocolMessage),
}

#[derive(Clone, Debug)]
pub enum DecoderState {
    AwaitingStart1,
    AwaitingStart2,
    ReadingHeader,
    ReadingPayload,
    ReadingChecksum,
}

#[derive(Debug)]
pub enum DecoderResult {
    Success(ProtocolMessage),
    InProgress(DecoderState),
    Error(ParseError),
}

pub struct Decoder {
    pub state: DecoderState,
    buffer: Vec<u8>,
    message: ProtocolMessage,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            state: DecoderState::AwaitingStart1,
            buffer: Vec::new(),
            message: ProtocolMessage::new(),
        }
    }

    pub fn parse_byte(&mut self, byte: u8) -> DecoderResult {
        debug!("Parsing byte: 0x{byte:02x} ({byte})");
        let state = &self.state;
        match state {
            DecoderState::AwaitingStart1 => {
                if byte == HEADER[0] {
                    self.state = DecoderState::AwaitingStart2;
                    return DecoderResult::InProgress(self.state.clone());
                }
                return DecoderResult::Error(ParseError::InvalidStartByte);
            }
            DecoderState::AwaitingStart2 => {
                if byte == HEADER[1] {
                    self.state = DecoderState::ReadingHeader;
                    self.buffer.clear();
                    return DecoderResult::InProgress(self.state.clone());
                }
                self.state = DecoderState::AwaitingStart1;
                return DecoderResult::Error(ParseError::InvalidStartByte);
            }
            DecoderState::ReadingHeader => {
                self.buffer.push(byte);
                // Basic information is available, moving to payload state
                if self.buffer.len() == 6 {
                    self.message.payload_length =
                        u16::from_le_bytes([self.buffer[0], self.buffer[1]]);
                    self.message.message_id = u16::from_le_bytes([self.buffer[2], self.buffer[3]]);
                    self.message.src_device_id = self.buffer[4];
                    self.message.dst_device_id = self.buffer[5];

                    if self.message.payload_length == 0 {
                        self.state = DecoderState::ReadingChecksum
                    } else {
                        self.state = DecoderState::ReadingPayload;
                    }
                    self.buffer.clear();
                }
                return DecoderResult::InProgress(self.state.clone());
            }
            DecoderState::ReadingPayload => {
                self.buffer.push(byte);
                debug!(
                    "DecoderState : ReadingPayload {:?} of {:?}",
                    self.buffer.len(),
                    self.message.payload_length
                );
                if self.buffer.len() == self.message.payload_length as usize {
                    self.message.payload = self.buffer.clone();
                    self.state = DecoderState::ReadingChecksum;
                    self.buffer.clear();
                }
                return DecoderResult::InProgress(self.state.clone());
            }
            DecoderState::ReadingChecksum => {
                self.buffer.push(byte);
                if self.buffer.len() == 2 {
                    self.message.checksum = u16::from_le_bytes([self.buffer[0], self.buffer[1]]);
                    self.reset();
                    let message = self.message.clone();
                    self.message = ProtocolMessage::new();
                    if !message.has_valid_crc() {
                        return DecoderResult::Error(ParseError::ChecksumError(message));
                    }
                    return DecoderResult::Success(message);
                }
                return DecoderResult::InProgress(self.state.clone());
            }
        }
    }

    fn reset(&mut self) {
        self.state = DecoderState::AwaitingStart1;
        self.buffer.clear();
    }
}
