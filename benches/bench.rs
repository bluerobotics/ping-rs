use bluerobotics_ping::{
    device::{Ping1D, Ping360, PingDevice},
    message::MessageInfo,
    ping1d::{self, ProfileStruct},
    Messages,
};
use criterion::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Instant;
use std::{net::SocketAddr, str::FromStr};
use tokio::runtime::Runtime;
use tokio_serial::{Error, SerialPort, SerialPortBuilderExt};
use udp_stream::UdpStream;

fn rt() -> Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .worker_threads(1)
        .thread_name("criterion-tokio-rt")
        .build()
        .unwrap()
}

async fn create_ping1d_usb() -> Ping1D {
    let port = tokio_serial::new("/dev/ttyUSB0".to_string(), 115200)
        .open_native_async()
        .map_err(|e| {
            eprintln!("Error opening serial port: {}", e);
            e
        })
        .unwrap();
    port.clear(tokio_serial::ClearBuffer::All).unwrap();

    Ping1D::new(port)
}

async fn create_ping360_udp() -> Ping360 {
    let socket_addr = SocketAddr::from_str(&format!("192.168.1.197:12345"))
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
    Ping360::new(port)
}

async fn receive_10_profiles(
    mut subscribed: tokio::sync::broadcast::Receiver<bluerobotics_ping::message::ProtocolMessage>,
) -> Result<(), Error> {
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
                        _ => panic!(),
                    }
                }
            }
            Err(_e) => {
                panic!()
            }
        }
    }
    Ok(())
}

fn ping1d_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Ping1D");
    group.sample_size(10); // Reduced sample size
    group.measurement_time(std::time::Duration::from_secs(60)); // Increased measurement time

    macro_rules! bench {
        ($bench_fn:ident($($arg:tt)*)) => {
            group.bench_function(stringify!($bench_fn), move |b| {
                b.to_async(rt()).iter_custom(|iters| async move {
                    let ping1d = create_ping1d_usb().await;

                    ping1d.continuous_stop(bluerobotics_ping::ping1d::ProfileStruct::id())
                        .await
                        .unwrap();

                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                    let mut total_duration = std::time::Duration::new(0, 0);
                    for _i in 0..iters {
                        let start = Instant::now();
                        let result = black_box(ping1d.$bench_fn($($arg)*).await);
                        total_duration += start.elapsed();
                        result.unwrap();
                    }
                    total_duration
                })
            });
        }
    }

    // Get Methods
    bench!(profile());
    // bench!(ping_interval());
    // bench!(transmit_duration());
    // bench!(range());
    // bench!(speed_of_sound());
    // bench!(firmware_version());
    // bench!(mode_auto());
    // bench!(distance_simple());
    // bench!(pcb_temperature());
    // bench!(ping_enable());
    // bench!(general_info());
    // bench!(distance());
    // bench!(processor_temperature());
    // bench!(voltage_5());
    // bench!(gain_setting());
    // bench!(device_id());

    // Custom - receive 10 profile packages
    group.bench_function("Receive 10 profiles", move |b| {
        b.to_async(rt()).iter_custom(|iters| async move {
            let ping1d = create_ping1d_usb().await;

            ping1d
                .continuous_start(bluerobotics_ping::ping1d::ProfileStruct::id())
                .await
                .unwrap();

            let mut total_duration = std::time::Duration::new(0, 0);
            for _i in 0..iters {
                let start = Instant::now();
                black_box(receive_10_profiles(ping1d.subscribe()).await.unwrap());
                total_duration += start.elapsed();
            }

            ping1d
                .continuous_stop(bluerobotics_ping::ping1d::ProfileStruct::id())
                .await
                .unwrap();

            total_duration
        })
    });

    group.finish();
}

fn ping360_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Ping360");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(60));

    macro_rules! bench360 {
        ($bench_fn:ident($($arg:tt)*)) => {
            group.bench_function(stringify!($bench_fn), move |b| {
                b.to_async(rt()).iter_custom(|iters| async move {
                    let ping360 = create_ping360_udp().await;
                    ping360.reset(0,0).await.unwrap();

                    let mut total_duration = std::time::Duration::new(0, 0);
                    for _i in 0..iters {
                        let start = Instant::now();
                        let result = black_box(ping360.$bench_fn($($arg)*).await);
                        total_duration += start.elapsed();
                        result.unwrap();
                    }
                    total_duration
                })
            });
        }
    }

    let mode: u8 = 1;
    let gain_setting: u8 = 0;
    let angle: u16 = 360;
    let transmit_duration: u16 = 7;
    let sample_period: u16 = 80;
    let transmit_frequency: u16 = 700;
    let number_of_samples: u16 = 1200;
    let transmit: u8 = 1;

    // Get Methods
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
    bench360!(motor_off());

    group.finish();
}

criterion_group!(benches, ping1d_benchmark, ping360_benchmark);
criterion_main!(benches);
