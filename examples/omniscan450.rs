mod common;
use common::{create_port, Port};

use bluerobotics_ping::{device::PingDevice, error::PingError, omniscan450::Device as OmniScan450};

#[tokio::main]
async fn main() -> Result<(), PingError> {
    println!("Parsing user provided values and creating port...");
    let port = create_port().await;

    println!("Creating your Omniscan device");
    let omniscan450 = match port {
        Port::Serial(port) => OmniScan450::new(port),
        Port::Udp(port) => OmniScan450::new(port),
    };

    println!("Reading mono profile:");
    let p = omniscan450.os_mono_profile().await?;
    println!("ping_number: {}", p.ping_number);
    println!("start_mm: {}", p.start_mm);
    println!("length_mm: {}", p.length_mm);
    println!("timestamp_ms: {}", p.timestamp_ms);
    println!("ping_hz: {}", p.ping_hz);
    println!("gain_index: {}", p.gain_index);
    println!("num_results: {}", p.num_results);
    println!("sos_dmps: {}", p.sos_dmps);
    println!("channel_number: {}", p.channel_number);
    println!("reserved: {}", p.reserved);
    println!("pulse_duration_sec: {}", p.pulse_duration_sec);
    println!("analog_gain: {}", p.analog_gain);
    println!("max_pwr_db: {}", p.max_pwr_db);
    println!("min_pwr_db: {}", p.min_pwr_db);
    println!("transducer_heading_deg: {}", p.transducer_heading_deg);
    println!("vehicle_heading_deg: {}", p.vehicle_heading_deg);
    println!("pwr_results: {}", p.pwr_results);
    // Creating futures to read different device Properties
    let (protocol_version_struct, device_information) = tokio::try_join!(
        omniscan450.protocol_version(),
        omniscan450.device_information()
    )
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
