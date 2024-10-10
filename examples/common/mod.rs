use clap::Parser;
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};
use tokio_serial::{SerialPort, SerialPortBuilderExt};
use udp_stream::UdpStream;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, group = "source", conflicts_with_all = ["udp_address"])]
    pub serial_port: Option<PathBuf>,
    #[arg(long, default_value_t = 115200)]
    pub serial_baud_rate: u32,
    #[arg(long, group = "source", conflicts_with_all = ["serial_port"])]
    pub udp_address: Option<IpAddr>,
    #[arg(long, default_value_t = 8080)]
    pub udp_port: u32,
}

pub enum Port {
    Serial(tokio_serial::SerialStream),
    Udp(udp_stream::UdpStream),
}

pub async fn create_port() -> Port {
    let args = Args::parse();

    match (args.serial_port, args.udp_address) {
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
    }
}
