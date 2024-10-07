use clap::Parser;
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};
use udp_stream::UdpStream;

use bluerobotics_ping::{
    common::Device as Common, device::PingDevice, error::PingError, message::ProtocolMessage,
};
use tokio_serial::{SerialPort, SerialPortBuilderExt};

#[tokio::main]
async fn main() -> Result<(), PingError> {
    println!("Parsing user provided values and creating port...");
    let port = create_port().await;

    println!("Creating your Ping 1D device");
    let common = match port {
        Port::Serial(port) => Common::new(port),
        Port::Udp(port) => Common::new(port),
    };

    // Creating a subscription channel which will receive 2 Protocol Messages, we'll print the device id!
    let mut subscribed = common.subscribe();
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
        tokio::try_join!(common.protocol_version(), common.device_information(),)
            .expect("Failed to join results");

    let version = format!(
        "{}.{}.{}",
        protocol_version_struct.version_major,
        protocol_version_struct.version_minor,
        protocol_version_struct.version_patch
    );

    println!("Protocol version is: {version}");
    println!("Device information: \n {:?}", device_information_struct);

    // Read the same 2 packages from previous requests, but from subscriber task, all above tasks have success, we did it!
    println!("Checking if subscriber returns with 2 same packages...");
    match rx.await {
        Ok(v) => println!("Received {} protocol messages", v.len()),
        Err(_) => println!("The oneshot sender dropped"),
    }

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
