use bluerobotics_ping::{message::ProtocolMessage, Messages};
use rand::rngs::StdRng;
use tokio::runtime::Runtime;

pub fn rt() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("criterion-tokio-rt")
        .build()
        .unwrap()
}

#[cfg(feature = "arbitrary")]
pub fn create_random_messages(mut rng: &mut StdRng, count: usize) -> Vec<Messages> {
    use arbitrary::Arbitrary;

    let mut messages = Vec::with_capacity(count);

    for _ in 0..count {
        let mut buf = [0u8; 1024]; // plenty of bytes available
        rand::RngCore::fill_bytes(&mut rng, &mut buf);

        let mut u = arbitrary::Unstructured::new(&buf);

        if let Ok(msg) = Messages::arbitrary(&mut u) {
            messages.push(msg);
        }
    }

    messages
}
#[cfg(not(feature = "arbitrary"))]
pub fn create_random_messages(_rng: &mut StdRng, _count: usize) -> Vec<Messages> {
    panic!("Missing 'arbitrary' feature. Re-run it with `--features=arbitrary`")
}

pub fn create_random_protocol_messages(rng: &mut StdRng, count: usize) -> Vec<ProtocolMessage> {
    create_random_messages(rng, count)
        .iter()
        .map(protocol_message_from_messages)
        .collect()
}

#[inline(always)]
pub fn protocol_message_from_messages(message: &Messages) -> ProtocolMessage {
    let mut protocol_message = ProtocolMessage::new();
    match message {
        Messages::Ping360(message) => protocol_message.set_message(message),
        Messages::Omniscan450(message) => protocol_message.set_message(message),
        Messages::Bluebps(message) => protocol_message.set_message(message),
        Messages::Ping1D(message) => protocol_message.set_message(message),
        Messages::Common(message) => protocol_message.set_message(message),
    }
    protocol_message
}
