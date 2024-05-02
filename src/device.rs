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
use tracing::error;

use crate::{
    codec::PingCodec,
    common,
    error::PingError,
    message::{self, MessageInfo, ProtocolMessage},
    Messages,
};

// Make devices available, each device uses Common and PingDevice.
pub use crate::ping1d::Device as Ping1D;
pub use crate::ping360::Device as Ping360;

pub struct Common {
    tx: mpsc::Sender<ProtocolMessage>,
    rx: broadcast::Receiver<ProtocolMessage>,
}

impl Common {
    pub fn new(port: tokio_serial::SerialStream) -> Self {
        // Prepare Serial sink and stream modules
        let serial: Framed<tokio_serial::SerialStream, PingCodec> = PingCodec::new().framed(port);
        let (serial_sink, serial_stream) = serial.split();

        // Prepare Serial receiver broadcast and sender
        let (broadcast_tx, broadcast_rx) = broadcast::channel::<ProtocolMessage>(100);
        tokio::spawn(Self::stream(serial_stream, broadcast_tx));
        let (sender, sender_rx) = mpsc::channel::<ProtocolMessage>(100);
        tokio::spawn(Self::sink(serial_sink, sender_rx));

        Common {
            tx: sender,
            rx: broadcast_rx,
        }
    }

    async fn sink(
        mut sink: SplitSink<Framed<SerialStream, PingCodec>, ProtocolMessage>,
        mut sender_rx: Receiver<ProtocolMessage>,
    ) {
        while let Some(item) = sender_rx.recv().await {
            if let Err(e) = sink.send(item).await {
                error!("{e:?}");
            }
        }
    }

    async fn stream(
        mut serial_stream: SplitStream<Framed<SerialStream, PingCodec>>,
        broadcast_tx: Sender<ProtocolMessage>,
    ) {
        'outside_loop: loop {
            while let Some(msg) = serial_stream.next().await {
                match msg {
                    Ok(msg) => {
                        if let Err(e) = broadcast_tx.send(msg) {
                            error!("{e:?}");
                            break 'outside_loop;
                        };
                    }
                    Err(e) => {
                        error!("{e:?}");
                    }
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

    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ProtocolMessage> {
        self.rx.resubscribe()
    }
}

pub trait PingDevice {
    fn new(port: tokio_serial::SerialStream) -> Self;

    fn get_common(&self) -> &Common;

    fn get_mut_common(&mut self) -> &mut Common;

    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ProtocolMessage> {
        self.get_common().subscribe()
    }

    async fn send_general_request(&self, requested_id: u16) -> Result<(), PingError> {
        let request =
            common::Messages::GeneralRequest(common::GeneralRequestStruct { requested_id });
        let mut package = message::ProtocolMessage::new();
        package.set_message(&request);

        if let Err(e) = self.get_common().send_message(package).await {
            return Err(e);
        };

        Ok(())
    }

    async fn wait_for_message<T: 'static>(
        &self,
        mut receiver: tokio::sync::broadcast::Receiver<ProtocolMessage>,
    ) -> Result<T, PingError>
    where
        T: crate::message::MessageInfo + std::marker::Sync + Clone + std::marker::Send,
    {
        let future = async move {
            loop {
                match receiver.recv().await {
                    Ok(answer) => {
                        if T::id() != answer.message_id {
                            continue;
                        };
                        let message = Messages::try_from(&answer)
                            .map_err(|_e| PingError::TryFromError(answer))?;
                        return Ok(message.inner::<T>().unwrap().clone());
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(e) => return Err(PingError::TokioBroadcastError(e)),
                };
            }
        };

        match tokio::time::timeout(tokio::time::Duration::from_secs(15), future).await {
            Ok(result) => result,
            Err(_) => Err(PingError::TimeoutError),
        }
    }

    async fn wait_for_ack(
        &self,
        mut receiver: tokio::sync::broadcast::Receiver<ProtocolMessage>,
        message_id: u16,
    ) -> Result<(), PingError> {
        let future = async move {
            loop {
                match receiver.recv().await {
                    Ok(answer) => {
                        if common::AckStruct::id() != answer.message_id
                            && common::NackStruct::id() != answer.message_id
                        {
                            continue;
                        }
                        match Messages::try_from(&answer) {
                            Ok(Messages::Common(common::Messages::Ack(answer))) => {
                                if answer.acked_id != message_id {
                                    continue;
                                };
                                return Ok(());
                            }
                            Ok(Messages::Common(common::Messages::Nack(answer))) => {
                                if answer.nacked_id != message_id {
                                    continue;
                                };
                                return Err(PingError::NackError(answer.nack_message));
                            }
                            _ => return Err(PingError::TryFromError(answer)), // Almost unreachable, but raises error ProtocolMessage
                        };
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(e) => return Err(PingError::TokioBroadcastError(e)),
                };
            }
        };

        match tokio::time::timeout(tokio::time::Duration::from_secs(15), future).await {
            Ok(result) => result,
            Err(_) => Err(PingError::TimeoutError),
        }
    }

    async fn request<T: 'static>(&self) -> Result<T, PingError>
    where
        T: crate::message::MessageInfo + std::marker::Sync + Clone + std::marker::Send,
    {
        let receiver = self.subscribe();

        self.send_general_request(T::id()).await?;

        self.wait_for_message(receiver).await
    }

    async fn set_device_id(&self, device_id: u8) -> Result<(), PingError> {
        let request = common::Messages::SetDeviceId(common::SetDeviceIdStruct { device_id });
        let mut package = ProtocolMessage::new();
        package.set_message(&request);

        let receiver = self.subscribe();

        self.get_common().send_message(package).await?;

        self.wait_for_ack(receiver, common::SetDeviceIdStruct::id())
            .await
    }

    async fn get_protocol_version(
        &self,
    ) -> Result<crate::common::ProtocolVersionStruct, PingError> {
        self.request().await
    }
}
