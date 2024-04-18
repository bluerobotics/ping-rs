use std::convert::TryFrom;

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::sync::{
    broadcast::{self, Sender},
    mpsc::{self, Receiver},
};
use tokio_serial::SerialStream;
use tokio_util::codec::{Decoder, Framed};

use crate::{
    codec::PingCodec,
    common::{self, ProtocolVersionStruct},
    error::PingError,
    message::{self, ProtocolMessage},
    ping1d::{self, DeviceIdStruct},
    Messages,
};

pub struct Common {
    tx: mpsc::Sender<ProtocolMessage>,
    rx: broadcast::Receiver<ProtocolMessage>,
}

impl Common {
    pub fn new(port: tokio_serial::SerialStream) -> Result<Self, PingError> {
        // Prepare Serial sink and stream modules
        let serial: Framed<tokio_serial::SerialStream, PingCodec> = PingCodec::new().framed(port);
        let (serial_sink, serial_stream) = serial.split();

        // Prepare Serial receiver broadcast and sender
        let (broadcast_tx, broadcast_rx) = broadcast::channel::<ProtocolMessage>(100);
        tokio::spawn(Self::stream(serial_stream, broadcast_tx));
        let (sender, sender_rx) = mpsc::channel::<ProtocolMessage>(100);
        tokio::spawn(Self::sink(serial_sink, sender_rx));

        Ok(Common {
            tx: sender,
            rx: broadcast_rx,
        })
    }

    async fn sink(
        mut sink: SplitSink<Framed<SerialStream, PingCodec>, ProtocolMessage>,
        mut sender_rx: Receiver<ProtocolMessage>,
    ) {
        while let Some(item) = sender_rx.recv().await {
            if let Err(_e) = sink.send(item).await {
                break;
            }
        }
    }

    async fn stream(
        mut serial_stream: SplitStream<Framed<SerialStream, PingCodec>>,
        broadcast_tx: Sender<ProtocolMessage>,
    ) {
        loop {
            while let Some(msg) = serial_stream.next().await {
                if let Ok(msg) = msg {
                    broadcast_tx.send(msg).unwrap();
                }
            }
        }
    }

    pub async fn send_message(&self, message: ProtocolMessage) -> Result<(), PingError> {
        match self.tx.send(message).await {
            Ok(msg) => Ok(msg),
            Err(err) => Err(PingError::TokioMpscError(err)),
        }
    }

    pub async fn receive_message(&mut self) -> Result<ProtocolMessage, PingError> {
        match self.rx.recv().await {
            Ok(msg) => Ok(msg),
            Err(err) => Err(PingError::TokioBroadcastError(err)),
        }
    }

    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ProtocolMessage> {
        self.rx.resubscribe()
    }
}

pub trait PingDevice {
    fn get_common(&self) -> &Common;

    fn get_mut_common(&mut self) -> &mut Common;

    fn subscribe(&mut self) -> tokio::sync::broadcast::Receiver<ProtocolMessage> {
        self.get_mut_common().subscribe()
    }

    async fn send_general_request(
        &mut self,
        request_number: common::GeneralRequestStruct,
    ) -> Result<(), PingError> {
        let request = common::Messages::GeneralRequest(request_number);
        let mut package = message::ProtocolMessage::new();
        package.set_message(&request);

        if let Err(e) = self.get_mut_common().send_message(package).await {
            return Err(e);
        };

        Ok(())
    }

    async fn get_firmware(&mut self) -> Result<ProtocolVersionStruct, PingError> {
        let mut receiver = self.subscribe();

        let result = tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(answer) => match Messages::try_from(&answer) {
                        Ok(Messages::Common(common::Messages::ProtocolVersion(answer))) => {
                            return Ok(answer)
                        }
                        _ => continue,
                    },
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(e) => return Err(PingError::TokioBroadcastError(e)),
                };
            }
        });

        if let Err(e) = self
            .send_general_request(common::GeneralRequestStruct { requested_id: 5 })
            .await
        {
            return Err(e);
        }

        match tokio::time::timeout(tokio::time::Duration::from_millis(10000), result).await {
            Ok(result) => match result {
                Ok(result) => result,
                Err(_) => Err(PingError::JoinError),
            },
            Err(_) => Err(PingError::TimeoutError),
        }
    }
}

pub struct Ping1D {
    common: Common,
}

impl Ping1D {
    pub fn new(port: tokio_serial::SerialStream) -> Self {
        Self {
            common: Common::new(port).unwrap(),
        }
    }

    pub async fn get_device_id(&mut self) -> Result<DeviceIdStruct, PingError> {
        let mut receiver = self.subscribe();

        let result = tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(answer) => match Messages::try_from(&answer) {
                        Ok(Messages::Ping1D(ping1d::Messages::DeviceId(answer))) => {
                            return Ok(answer)
                        }
                        _ => continue,
                    },
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(e) => return Err(PingError::TokioBroadcastError(e)),
                };
            }
        });

        if let Err(e) = self
            .send_general_request(common::GeneralRequestStruct { requested_id: 1201 })
            .await
        {
            return Err(e);
        }

        match tokio::time::timeout(tokio::time::Duration::from_millis(10000), result).await {
            Ok(result) => match result {
                Ok(result) => result,
                Err(_) => Err(PingError::JoinError),
            },
            Err(_) => Err(PingError::TimeoutError),
        }
    }
}

impl PingDevice for Ping1D {
    fn get_common(&self) -> &Common {
        &self.common
    }

    fn get_mut_common(&mut self) -> &mut Common {
        &mut self.common
    }
}
