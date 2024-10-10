mod common;
use common::{create_port, Port};

use bluerobotics_ping::{
    device::{Ping360, PingDevice},
    error::PingError,
};

#[tokio::main]
async fn main() -> Result<(), PingError> {
    println!("Parsing user provided values and creating port...");
    let port = create_port().await;

    println!("Creating your Ping 360 device");
    let ping360 = match port {
        Port::Serial(port) => Ping360::new(port),
        Port::Udp(port) => Ping360::new(port),
    };

    println!("Reading transducer data:");
    let p = ping360
        .transducer(1, 1, 0, 500, 20000, 700, 1000, 1, 0)
        .await?;
    println!("mode: {}", p.mode);
    println!("gain_setting: {}", p.gain_setting);
    println!("angle: {}", p.angle);
    println!("transmit_duration: {}", p.transmit_duration);
    println!("sample_period: {}", p.sample_period);
    println!("transmit_frequency: {}", p.transmit_frequency);
    println!("number_of_samples: {}", p.number_of_samples);
    println!("data_length: {}", p.data_length);
    println!("data: {:?}", p.data.len());

    // Creating futures to read different device Properties
    let (protocol_version_struct, device_information) =
        tokio::try_join!(ping360.protocol_version(), ping360.device_information())
            .expect("Failed to join results");

    let version = format!(
        "{}.{}.{}",
        protocol_version_struct.version_major,
        protocol_version_struct.version_minor,
        protocol_version_struct.version_patch
    );

    println!("Protocol version is: {version}");
    println!("Device information: {device_information:?}");

    Ok(())
}
