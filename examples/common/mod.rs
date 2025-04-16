use clap::Parser;
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
    time::Duration,
};
use tokio::{net::TcpStream, time::timeout};
use tokio_serial::{SerialPort, SerialPortBuilderExt};
use tracing_subscriber;
use udp_stream::UdpStream;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, group = "source", conflicts_with_all = ["udp_address", "tcp_address"])]
    pub serial_port: Option<PathBuf>,
    #[arg(long, default_value_t = 115200)]
    pub serial_baud_rate: u32,
    #[arg(long, group = "source", conflicts_with_all = ["serial_port", "tcp_address"])]
    pub udp_address: Option<IpAddr>,
    #[arg(long, default_value_t = 8080)]
    pub udp_port: u32,
    #[arg(long, group = "source", conflicts_with_all = ["serial_port", "udp_address"])]
    pub tcp_address: Option<IpAddr>,
    #[arg(long, default_value_t = 8080)]
    pub tcp_port: u32,
}

pub enum Port {
    Serial(tokio_serial::SerialStream),
    Udp(udp_stream::UdpStream),
    Tcp(TcpStream),
}

pub fn configure_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_line_number(true)
        .with_file(true)
        .init();
}

pub async fn create_port() -> Port {
    let args = Args::parse();

    match (args.serial_port, args.udp_address, args.tcp_address) {
        (Some(serial_port), None, None) => {
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
        (None, Some(udp_address), None) => {
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
        (None, None, Some(tcp_address)) => {
            println!("Using TCP address: {}", tcp_address);
            let socket_addr = SocketAddr::from_str(&format!("{}:{}", tcp_address, args.tcp_port))
                .map_err(|e| {
                    eprintln!("Error parsing TCP address: {}", e);
                    e
                })
                .unwrap();

            match timeout(Duration::from_secs(10), TcpStream::connect(socket_addr)).await {
                Ok(connection_result) => match connection_result {
                    Ok(port) => {
                        println!("Successfully connected to TCP server");
                        Port::Tcp(port)
                    }
                    Err(e) => {
                        eprintln!("Error connecting to TCP socket: {}", e);
                        std::process::exit(1);
                    }
                },
                Err(_) => {
                    eprintln!("TCP connection timed out after 10 seconds");
                    std::process::exit(1);
                }
            }
        }
        (None, None, None) => {
            eprintln!("Error: either serial_port, udp_address, or tcp_address must be provided");
            std::process::exit(1);
        }
        _ => {
            eprintln!("Error: serial_port, udp_address, and tcp_address are mutually exclusive");
            std::process::exit(1);
        }
    }
}
