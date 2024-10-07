use clap::Parser;
use std::{
    convert::TryFrom,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};
use udp_stream::UdpStream;

use bluerobotics_ping::{
    device::{Ping360, PingDevice},
    error::PingError,
    message::MessageInfo,
    ping360::{self},
    Messages,
};
use tokio_serial::{SerialPort, SerialPortBuilderExt};

#[tokio::main]
async fn main() -> Result<(), PingError> {
    println!("Parsing user provided values and creating port...");
    let port = create_port().await;

    println!("Creating your Ping 1D device");
    let ping360 = match port {
        Port::Serial(port) => Ping360::new(port),
        Port::Udp(port) => Ping360::new(port),
    };

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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, group = "source",
    conflicts_with_all = ["udp_address"])]
    serial_port: Option<PathBuf>,
    #[arg(long, default_value_t = 115200)]
    serial_baud_rate: u32,
    #[arg(long, group = "source",
    conflicts_with_all = ["serial_port"])]
    udp_address: Option<IpAddr>,
    #[arg(long, default_value_t = 8080)]
    udp_port: u32,
}

enum Port {
    Serial(tokio_serial::SerialStream),
    Udp(udp_stream::UdpStream),
}

async fn create_port() -> Port {
    let args = Args::parse();

    let port = match (args.serial_port, args.udp_address) {
        (Some(serial_port), None) => {
            println!("Using serial port: {:?}", serial_port);
            let port = tokio_serial::new(serial_port.to_string_lossy(), args.serial_baud_rate)
                .open_native_async()
                .map_err(|e| {
                    eprintln!("Error opening serial port: {}", e);
                    e
                })
                .unwrap();
            port.clear(tokio_serial::ClearBuffer::All).unwrap();
            Port::Serial(port)
        }
        (None, Some(udp_address)) => {
            println!("Using UDP address: {}", udp_address);
            let socket_addr = SocketAddr::from_str(&format!("{}:{}", udp_address, args.udp_port))
                .map_err(|e| {
                    eprintln!("Error parsing UDP address: {}", e);
                    e
                })
                .unwrap();
            let port = UdpStream::connect(socket_addr)
                .await
                .map_err(|e| {
                    eprintln!("Error connecting to UDP socket: {}", e);
                    e
                })
                .unwrap();
            Port::Udp(port)
        }
        (None, None) => {
            eprintln!("Error: either serial_port_name or udp_address must be provided");
            std::process::exit(1);
        }
        (Some(_), Some(_)) => {
            eprintln!("Error: serial_port_name and udp_address are mutually exclusive");
            std::process::exit(1);
        }
    };
    port
}
