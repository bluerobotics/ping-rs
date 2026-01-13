use bluerobotics_ping::{
    device::{Ping1D, Ping360, PingDevice},
    message::MessageInfo,
    ping1d::{self, ProfileStruct},
    ping360::{self, AutoDeviceDataStruct},
    Messages,
};
use criterion::{criterion_group, criterion_main, Criterion};
use ping_viewer_next::device::manager::ManagerError;
use std::hint::black_box;
use std::time::Instant;
use std::{env, time::Duration};
use std::{net::SocketAddr, path::PathBuf, str::FromStr};
use tokio::{io::AsyncWriteExt, runtime::Runtime, time::timeout};
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialStream};
use udp_stream::UdpStream;

fn rt() -> Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name("criterion-tokio-rt")
        .build()
        .unwrap()
}

const SAMPLE_PERIOD_TICK_DURATION: f64 = 25e-9;
const FIRMWARE_MAX_TRANSMIT_DURATION: u16 = 1000;
const FIRMWARE_MIN_TRANSMIT_DURATION: u16 = 1;

fn calculate_sample_period(desired_range: f64, number_of_samples: u16, speed_of_sound: f64) -> u16 {
    const SAMPLE_PERIOD_TICK_DURATION: f64 = 25e-9;
    ((2.0 * desired_range)
        / (number_of_samples as f64 * speed_of_sound * SAMPLE_PERIOD_TICK_DURATION))
        .ceil() as u16
}

fn calculate_transmit_duration_max(sample_period: u16) -> u16 {
    let sample_based_max = sample_period as f64 * 64.0 * SAMPLE_PERIOD_TICK_DURATION * 1e6;
    sample_based_max.min(FIRMWARE_MAX_TRANSMIT_DURATION as f64) as u16
}

fn calculate_transmit_duration(range: f64, speed_of_sound: f64, sample_period: u16) -> u16 {
    // Calculate initial value
    let mut auto_duration = ((8000.0 * range) / speed_of_sound).round();

    // Ensure minimum based on sample period (convert to microseconds)
    let min_sample_based = (2.5 * sample_period as f64 * SAMPLE_PERIOD_TICK_DURATION * 1e6).ceil();
    auto_duration = auto_duration.max(min_sample_based).round();

    // Clamp between FIRMWARE_MIN_TRANSMIT_DURATION and max based on sample period
    let max_duration = calculate_transmit_duration_max(sample_period);
    auto_duration = auto_duration
        .max(FIRMWARE_MIN_TRANSMIT_DURATION as f64)
        .min(max_duration as f64);

    auto_duration.round() as u16
}

#[derive(Clone)]
enum ConnectionType {
    Serial(PathBuf, u32),
    Udp(IpAddr, u32),
}
struct BenchConfig {
    connection: ConnectionType,
    device_type: String,
    sample_size: usize,
    measurement_time: u64,
}

// Get configuration from environment variables
fn get_config() -> BenchConfig {
    let device_type = env::var("PING_DEVICE_TYPE").unwrap_or_else(|_| "ping1d".to_string());

    let sample_size = env::var("PING_SAMPLE_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    let measurement_time = env::var("PING_MEASUREMENT_TIME")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60);

    let connection = if let Ok(serial_port) = env::var("PING_SERIAL_PORT") {
        let baud_rate = env::var("PING_SERIAL_BAUD")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(115200);

        ConnectionType::Serial(PathBuf::from(serial_port), baud_rate)
    } else if let Ok(udp_address) = env::var("PING_UDP_ADDRESS") {
        let ip_addr = udp_address
            .parse::<IpAddr>()
            .expect("Invalid IP address format");
        let port = env::var("PING_UDP_PORT")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(12345);

        ConnectionType::Udp(ip_addr, port)
    } else {
        eprintln!("Warning: No connection method specified. Defaulting to /dev/ttyUSB0");
        ConnectionType::Serial(PathBuf::from("/dev/ttyUSB0"), 115200)
    };

    BenchConfig {
        connection,
        device_type,
        sample_size,
        measurement_time,
    }
}

async fn create_port(connection: &ConnectionType) -> Result<Port, Box<dyn std::error::Error>> {
    match connection {
        ConnectionType::Serial(path, baud_rate) => {
            let mut port =
                tokio_serial::new(path.to_string_lossy(), *baud_rate).open_native_async()?;
            set_baudrate_pre_routine(&mut port, *baud_rate)
                .await
                .unwrap();
            port.clear(tokio_serial::ClearBuffer::All)?;
            Ok(Port::Serial(port))
        }
        ConnectionType::Udp(addr, port) => {
            let socket_addr = SocketAddr::new(addr.clone().into(), *port as u16);
            let stream = UdpStream::connect(socket_addr).await?;
            Ok(Port::Udp(stream))
        }
    }
}

enum Port {
    Serial(tokio_serial::SerialStream),
    Udp(udp_stream::UdpStream),
}

async fn receive_10_profiles(
    mut subscribed: tokio::sync::broadcast::Receiver<bluerobotics_ping::message::ProtocolMessage>,
) -> Result<(), std::io::Error> {
    let mut profile_struct_vector: Vec<ProfileStruct> = Vec::new();
    for _i in 1..10 {
        let received = subscribed.recv().await;

        match received {
            Ok(msg) => {
                if msg.message_id == bluerobotics_ping::ping1d::ProfileStruct::id() {
                    match Messages::try_from(&msg) {
                        Ok(Messages::Ping1D(ping1d::Messages::Profile(answer))) => {
                            profile_struct_vector.push(answer)
                        }
                        _ => {}
                    }
                }
            }
            Err(_e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Receive error",
                ));
            }
        }
    }
    Ok(())
}

// Receive 400 profiles, corresponding to a complete 360 degrees scanning
async fn ping360_full_scan(
    mut subscribed: tokio::sync::broadcast::Receiver<bluerobotics_ping::message::ProtocolMessage>,
) -> Result<(), std::io::Error> {
    let mut profile_struct_vector: Vec<AutoDeviceDataStruct> = Vec::new();
    for _i in 1..400 {
        let received = subscribed.recv().await;

        match received {
            Ok(msg) => {
                if msg.message_id == bluerobotics_ping::ping360::AutoDeviceDataStruct::id() {
                    match Messages::try_from(&msg) {
                        Ok(Messages::Ping360(ping360::Messages::AutoDeviceData(answer))) => {
                            profile_struct_vector.push(answer)
                        }
                        _ => {}
                    }
                }
            }
            Err(_e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Receive error",
                ));
            }
        }
    }
    Ok(())
}

// Macro that allows benchmarking of the Ping360 with ping360_full_scan for different ranges
fn helper_ping360_range_benchmark(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
    connection: ConnectionType,
    range_meters: u16,
    number_of_samples: u16,
    speed_of_sound: Option<f64>,
) {
    let speed_of_sound = speed_of_sound.unwrap_or(1500.0);
    let sample_period =
        calculate_sample_period(range_meters as f64, number_of_samples, speed_of_sound);
    let transmit_duration =
        calculate_transmit_duration(range_meters as f64, speed_of_sound, sample_period);

    let conn = connection.clone();
    group.bench_function(
        format!("Receive 400 profiles (Range:{range_meters}m, Samples: {number_of_samples}, Transmit Duration: {transmit_duration}ms), Sample Period: {sample_period}ms"),
        move |b| {
            let conn_clone = conn.clone();
            b.to_async(rt()).iter_custom(|iters| {
                let value = conn_clone.clone();
                async move {
                    let port = match create_port(&value).await {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to create port: {:?}", e);
                            return std::time::Duration::new(0, 0);
                        }
                    };

                    let ping360 = match port {
                        Port::Serial(port) => Ping360::new(port),
                        Port::Udp(port) => Ping360::new(port),
                    };

                    let mode: u8 = 1;
                    let gain_setting: u8 = 0;
                    let transmit_frequency: u16 = 740;
                    let start_angle = 0;
                    let stop_angle = 399;
                    let num_steps = 1;
                    let delay = 10;

                    // Start continuous mode
                    if let Err(e) = ping360
                        .auto_transmit(
                            mode,
                            gain_setting,
                            transmit_duration,
                            sample_period,
                            transmit_frequency,
                            number_of_samples,
                            start_angle,
                            stop_angle,
                            num_steps,
                            delay,
                        )
                        .await
                    {
                        eprintln!("Failed to start continuous mode: {:?}", e);
                        return std::time::Duration::new(0, 0);
                    };

                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                    let mut total_duration = std::time::Duration::new(0, 0);
                    for _i in 0..iters {
                        let start = Instant::now();
                        let _ = black_box(ping360_full_scan(ping360.subscribe()).await);
                        total_duration += start.elapsed();
                    }

                    let _ = ping360.motor_off().await;

                    total_duration
                }
            })
        },
    );
}

fn ping1d_benchmark(c: &mut Criterion, config: &BenchConfig) {
    let mut group = c.benchmark_group("Ping1D");
    group.sample_size(config.sample_size);
    group.measurement_time(std::time::Duration::from_secs(config.measurement_time));

    let connection = config.connection.clone();

    macro_rules! bench {
        ($bench_fn:ident($($arg:tt)*)) => {
            let conn = connection.clone();
            group.bench_function(stringify!($bench_fn), move |b| {
                let conn_clone = conn.clone();
                b.to_async(rt()).iter_custom(|iters| {
                let value = conn_clone.clone();
                async move {
                    let port = match create_port(&value).await {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to create port: {:?}", e);
                            return std::time::Duration::new(0, 0);
                        }
                    };

                    let ping1d = match port {
                        Port::Serial(port) => Ping1D::new(port),
                        Port::Udp(port) => Ping1D::new(port),
                    };

                    // Try to stop any ongoing continuous streams
                    let _ = ping1d.continuous_stop(bluerobotics_ping::ping1d::ProfileStruct::id()).await;

                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                    let mut total_duration = std::time::Duration::new(0, 0);
                    for _i in 0..iters {
                        let start = Instant::now();
                        let result = black_box(ping1d.$bench_fn($($arg)*).await);
                        total_duration += start.elapsed();
                        if let Err(e) = result {
                            eprintln!("Error in {}: {:?}", stringify!($bench_fn), e);
                        }
                    }
                    total_duration
                }
                })
            });
        }
    }

    // Get Methods
    bench!(profile());
    bench!(ping_interval());
    bench!(transmit_duration());
    bench!(range());
    bench!(speed_of_sound());
    bench!(firmware_version());
    bench!(mode_auto());
    bench!(distance_simple());
    bench!(pcb_temperature());
    bench!(ping_enable());
    bench!(general_info());
    bench!(distance());
    bench!(processor_temperature());
    bench!(voltage_5());
    bench!(gain_setting());
    bench!(device_id());

    // Custom - receive 10 profile packages
    let conn = connection.clone();
    group.bench_function("Receive 10 profiles", move |b| {
        let conn_clone = conn.clone();
        b.to_async(rt()).iter_custom(|iters| {
            let value = conn_clone.clone();
            async move {
                let port = match create_port(&value).await {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Failed to create port: {:?}", e);
                        return std::time::Duration::new(0, 0);
                    }
                };

                let ping1d = match port {
                    Port::Serial(port) => Ping1D::new(port),
                    Port::Udp(port) => Ping1D::new(port),
                };

                // Start continuous mode
                if let Err(e) = ping1d
                    .continuous_start(bluerobotics_ping::ping1d::ProfileStruct::id())
                    .await
                {
                    eprintln!("Failed to start continuous mode: {:?}", e);
                    return std::time::Duration::new(0, 0);
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                let mut total_duration = std::time::Duration::new(0, 0);
                for _i in 0..iters {
                    let start = Instant::now();
                    let _ = black_box(receive_10_profiles(ping1d.subscribe()).await);
                    total_duration += start.elapsed();
                }

                // Stop continuous mode
                let _ = ping1d
                    .continuous_stop(bluerobotics_ping::ping1d::ProfileStruct::id())
                    .await;

                total_duration
            }
        })
    });

    group.finish();
}

fn ping360_benchmark(c: &mut Criterion, config: &BenchConfig) {
    let mut group = c.benchmark_group("Ping360");
    group.sample_size(config.sample_size);
    group.measurement_time(std::time::Duration::from_secs(config.measurement_time));

    let connection = config.connection.clone();

    macro_rules! bench360 {
        ($bench_fn:ident($($arg:tt)*)) => {
            let conn = connection.clone();
            group.bench_function(stringify!($bench_fn), move |b| {
                let conn_clone = conn.clone();
                b.to_async(rt()).iter_custom(|iters| {
                let value = conn_clone.clone();
                async move {
                    // Create a ping360 device for each benchmark run
                    let port = match create_port(&value).await {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to create port: {:?}", e);
                            return std::time::Duration::new(0, 0);
                        }
                    };

                    let ping360 = match port {
                        Port::Serial(port) => Ping360::new(port),
                        Port::Udp(port) => Ping360::new(port),
                    };

                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                    let mut total_duration = std::time::Duration::new(0, 0);
                    for _i in 0..iters {
                        let start = Instant::now();
                        let result = black_box(ping360.$bench_fn($($arg)*).await);
                        total_duration += start.elapsed();
                        if let Err(e) = result {
                            eprintln!("Error in {}: {:?}", stringify!($bench_fn), e);
                        }
                    }
                    total_duration
                }
                })
            });
        }
    }

    let mode: u8 = 1;
    let gain_setting: u8 = 0;
    let angle: u16 = 0;
    let transmit_duration: u16 = 53;
    let sample_period: u16 = 445;
    let transmit_frequency: u16 = 740;
    let number_of_samples: u16 = 1200;
    let transmit: u8 = 1;

    bench360!(motor_off());
    bench360!(transducer(
        mode,
        gain_setting,
        angle,
        transmit_duration,
        sample_period,
        transmit_frequency,
        number_of_samples,
        transmit,
        0,
    ));

    helper_ping360_range_benchmark(&mut group, connection.clone(), 2, 1200, None);

    helper_ping360_range_benchmark(&mut group, connection.clone(), 5, 1200, None);

    helper_ping360_range_benchmark(&mut group, connection.clone(), 10, 1200, None);

    helper_ping360_range_benchmark(&mut group, connection.clone(), 20, 1200, None);

    helper_ping360_range_benchmark(&mut group, connection.clone(), 2, 2000, None);

    helper_ping360_range_benchmark(&mut group, connection.clone(), 5, 5000, None);

    helper_ping360_range_benchmark(&mut group, connection.clone(), 10, 5000, None);

    helper_ping360_range_benchmark(&mut group, connection.clone(), 20, 5000, None);

    group.finish();
}

fn run_benches(c: &mut Criterion) {
    println!("Loading configuration from environment variables...");
    let config = get_config();

    println!("Running benchmarks for device type: {}", config.device_type);
    println!(
        "Sample size: {}, Measurement time: {}s",
        config.sample_size, config.measurement_time
    );

    match config.device_type.as_str() {
        "ping1d" => ping1d_benchmark(c, &config),
        "ping360" => ping360_benchmark(c, &config),
        _ => {
            eprintln!(
                "Invalid device type: {}. Using ping1d as default.",
                config.device_type
            );
            ping1d_benchmark(c, &config);
        }
    }
}

criterion_group!(benches, run_benches);
criterion_main!(benches);

// Helper needed for macro expansion
#[derive(Clone)]
enum IpAddr {
    V4(std::net::Ipv4Addr),
    V6(std::net::Ipv6Addr),
}

impl FromStr for IpAddr {
    type Err = std::net::AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let std_ip = s.parse::<std::net::IpAddr>()?;
        match std_ip {
            std::net::IpAddr::V4(ip) => Ok(IpAddr::V4(ip)),
            std::net::IpAddr::V6(ip) => Ok(IpAddr::V6(ip)),
        }
    }
}

impl std::fmt::Display for IpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpAddr::V4(ip) => write!(f, "{}", ip),
            IpAddr::V6(ip) => write!(f, "{}", ip),
        }
    }
}

impl From<IpAddr> for std::net::IpAddr {
    fn from(ip: IpAddr) -> Self {
        match ip {
            IpAddr::V4(ip) => std::net::IpAddr::V4(ip),
            IpAddr::V6(ip) => std::net::IpAddr::V6(ip),
        }
    }
}

pub async fn set_baudrate_pre_routine(
    port: &mut SerialStream,
    baud_rate: u32,
) -> Result<(), ManagerError> {
    timeout(Duration::from_millis(100), async {
        port.set_baud_rate(baud_rate).map_err(|e| {
            ManagerError::Other(format!("Failed to set baud rate {}: {}", baud_rate, e))
        })?;
        tokio::time::sleep(Duration::from_millis(10)).await;

        port.set_break()
            .map_err(|e| ManagerError::Other(format!("Failed to set BREAK: {}", e)))?;
        tokio::time::sleep(Duration::from_millis(10)).await;

        port.clear_break()
            .map_err(|e| ManagerError::Other(format!("Failed to clear BREAK: {}", e)))?;
        tokio::time::sleep(Duration::from_millis(10)).await;

        port.write_all(b"U")
            .await
            .map_err(|e| ManagerError::Other(format!("Failed to write 'U': {}", e)))?;
        tokio::time::sleep(Duration::from_millis(10)).await;

        port.flush()
            .await
            .map_err(|e| ManagerError::Other(format!("Failed to flush: {}", e)))?;
        tokio::time::sleep(Duration::from_millis(10)).await;

        port.clear(tokio_serial::ClearBuffer::All)
            .map_err(|err| ManagerError::DeviceSourceError(err.to_string()))?;

        Ok(())
    })
    .await
    .map_err(|_| ManagerError::Other("set_baudrate_pre_routine: Operation timed out".to_string()))?
}
