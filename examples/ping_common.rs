mod common;
use common::{configure_tracing, create_port, Port};

use bluerobotics_ping::{
    common::Device, device::PingDevice, error::PingError, message::ProtocolMessage,
};

#[tokio::main]
async fn main() -> Result<(), PingError> {
    configure_tracing();

    println!("Parsing user provided values and creating port...");
    let port = create_port().await;

    println!("Creating your Ping device");
    let ping = match port {
        Port::Serial(port) => Device::new(port),
        Port::Udp(port) => Device::new(port),
        Port::Tcp(port) => Device::new(port),
    };

    // Creating a subscription channel which will receive 2 Protocol Messages, we'll print the device id!
    let mut subscribed = ping.subscribe();
    let (tx, rx) = tokio::sync::oneshot::channel::<Vec<ProtocolMessage>>();

    tokio::spawn(async move {
        let mut profile_vector: Vec<ProtocolMessage> = Vec::new();
        loop {
            let received = subscribed.recv().await;
            match received {
                Ok(msg) => {
                    println!(
                        "Received a message from device with id {}",
                        msg.src_device_id
                    );
                    profile_vector.push(msg);
                }
                Err(_e) => break,
            }
            if profile_vector.len() >= 2 {
                tx.send(profile_vector).unwrap();
                break;
            };
        }
    });

    // Creating futures to read different device Properties
    let (protocol_version_struct, device_information_struct) =
        tokio::try_join!(ping.protocol_version(), ping.device_information(),)
            .expect("Failed to join results");

    let version = format!(
        "{}.{}.{}",
        protocol_version_struct.version_major,
        protocol_version_struct.version_minor,
        protocol_version_struct.version_patch
    );

    println!("Protocol version is: {version}");
    println!("Device information: \n {device_information_struct:#?}");

    // Read the same 2 packages from previous requests, but from subscriber task, all above tasks have success, we did it!
    println!("Checking if subscriber returns with 2 same packages...");
    match rx.await {
        Ok(v) => println!("Received {} protocol messages", v.len()),
        Err(_) => println!("The oneshot sender dropped"),
    };

    Ok(())
}
