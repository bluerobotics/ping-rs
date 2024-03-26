include!(concat!(env!("OUT_DIR"), "/mod.rs"));

use message::ProtocolMessage;
use std::io::{Read, Write};

use crate::message::{DeserializeGenericMessage, HEADER};

use std::convert::TryFrom;

pub mod decoder;
pub mod message;

#[derive(Debug)]
pub enum PingError {
    Io(std::io::Error),
    MissingBytes,
    ParseError(decoder::ParseError),
    UnexpectedStructure,
}

pub struct PingDevice<D> {
    port: D,
    decoder: decoder::Decoder,
}

impl<D: Read + Write> PingDevice<D> {
    pub fn new(port: D) -> Self {
        Self {
            port,
            decoder: decoder::Decoder::new(),
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, PingError> {
        match self.port.read(buf) {
            Ok(value) => Ok(value),
            Err(_e) => Err(PingError::Io(_e)),
        }
    }

    pub fn write(&mut self, request: Vec<u8>) -> Result<(), PingError> {
        match self.port.write_all(&request) {
            Ok(_e) => {
                self.port.flush().unwrap();
                Ok(_e)
            }
            Err(_e) => Err(PingError::Io(_e)),
        }
    }

    pub fn request(
        &mut self,
        request: message::ProtocolMessage,
    ) -> Result<message::ProtocolMessage, PingError> {
        self.write(request.serialized()).unwrap();

        let mut serial_buf: Vec<u8> = vec![0; 20];

        self.port.read(serial_buf.as_mut_slice()).unwrap();

        for byte in &serial_buf {
            match self.decoder.parse_byte(byte.clone()) {
                decoder::DecoderResult::InProgress => continue,
                decoder::DecoderResult::Error(_e) => {
                    return Err(PingError::ParseError(_e));
                }
                decoder::DecoderResult::Success(_e) => {
                    return Ok(_e);
                }
            }
        }
        self.decoder.reset();
        Err(PingError::MissingBytes)
    }

    pub fn get_protocol_version(&mut self) -> Result<common::ProtocolVersionStruct, PingError> {
        let request =
            common::Messages::GeneralRequest(common::GeneralRequestStruct { requested_id: 5 });

        let mut package = message::ProtocolMessage::new();
        package.set_message(&request);

        let answer = self.request(package)?;

        match Messages::try_from(&answer) {
            Ok(Messages::Common(common::Messages::ProtocolVersion(answer))) => Ok(answer),
            _ => Err(PingError::UnexpectedStructure),
        }
    }

    pub fn get_device_information(&mut self) -> Result<common::DeviceInformationStruct, PingError> {
        let request =
            common::Messages::GeneralRequest(common::GeneralRequestStruct { requested_id: 4 });

        let mut package = message::ProtocolMessage::new();
        package.set_message(&request);

        let answer = self.request(package)?;

        match Messages::try_from(&answer) {
            Ok(Messages::Common(common::Messages::DeviceInformation(answer))) => Ok(answer),
            _ => Err(PingError::UnexpectedStructure),
        }
    }

    pub fn set_device_id(&mut self, device_id: u8) -> Result<common::AckStruct, PingError> {
        let request = common::Messages::SetDeviceId(common::SetDeviceIdStruct { device_id });

        let mut package = ProtocolMessage::new();
        package.set_message(&request);

        let answer = self.request(package)?;

        match Messages::try_from(&answer) {
            Ok(Messages::Common(common::Messages::Ack(answer))) => Ok(answer),
            _ => Err(PingError::UnexpectedStructure),
        }
    }
}

pub fn calculate_crc(pack_without_payload: &[u8]) -> u16 {
    return pack_without_payload
        .iter()
        .fold(0 as u16, |s, &v| s.wrapping_add(v as u16));
}
