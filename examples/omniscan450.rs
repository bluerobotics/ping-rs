mod common;
use common::{configure_tracing, create_port, Port};

use bluerobotics_ping::{device::PingDevice, error::PingError, omniscan450::Device as OmniScan450};

#[tokio::main]
async fn main() -> Result<(), PingError> {
    configure_tracing();

    println!("Parsing user provided values and creating port...");
    let port = create_port().await;

    println!("Creating your Omniscan device");
    let omniscan450 = match port {
        Port::Serial(port) => OmniScan450::new(port),
        Port::Udp(port) => OmniScan450::new(port),
        Port::Tcp(port) => OmniScan450::new(port),
    };

    let device_information = omniscan450.device_information().await?;

    println!("Device information: {device_information:?}");

    Ok(())
}
