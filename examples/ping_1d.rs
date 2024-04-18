use clap::Parser;
use std::{convert::TryFrom, path::PathBuf};

use bluerobotics_ping::{
    device::{Ping1D, PingDevice},
    error::PingError,
    message::MessageInfo,
    ping1d::{self, ProfileStruct},
    Messages,
};
use tokio_serial::{SerialPort, SerialPortBuilderExt};

#[tokio::main]
async fn main() -> Result<(), PingError> {
    println!("Parsing user provided values...");
    let args = Args::parse();

    let port =
        tokio_serial::new(args.port_name.to_string_lossy(), args.baud_rate).open_native_async()?;
    port.clear(tokio_serial::ClearBuffer::All)?;

    println!("Creating your Ping 1D device");
    let ping1d = Ping1D::new(port);

    // Creating a subscription channel which will receive 30 Profile measurements, we'll check this after the next methods!
    let mut subscribed = ping1d.subscribe();
    let (tx, rx) = tokio::sync::oneshot::channel::<Vec<ProfileStruct>>();
    ping1d
        .continuous_start(bluerobotics_ping::ping1d::ProfileStruct::id())
        .await?;

    tokio::spawn(async move {
        let mut profile_struct_vector: Vec<ProfileStruct> = Vec::new();
        loop {
            let received = subscribed.recv().await;
            match received {
                Ok(msg) => {
                    if msg.message_id == bluerobotics_ping::ping1d::ProfileStruct::id() {
                        match Messages::try_from(&msg) {
                            Ok(Messages::Ping1D(ping1d::Messages::Profile(answer))) => {
                                profile_struct_vector.push(answer)
                            }
                            _ => continue,
                        }
                    }
                }
                Err(_e) => break,
            }
            if profile_struct_vector.len() >= 30 {
                tx.send(profile_struct_vector).unwrap();
                break;
            };
        }
    });

    for n in (1..10).rev() {
        println!("Testing set/get device id: {n}");
        ping1d.set_device_id(n).await?;
        assert_eq!(n, ping1d.get_device_id().await.unwrap().device_id);
    }

    // Testing set command, all set commands check for their Ack message, Error and NAck error are possible
    println!(
        "Set gain to auto: {:?}",
        ping1d.set_mode_auto(1).await.is_ok()
    );

    // Creating two futures to read Protocol Version and Device ID
    let res1 = async { ping1d.get_protocol_version().await };
    let res2 = async { ping1d.get_device_id().await };
    let (protocol_version_struct, device_id_struct) =
        tokio::try_join!(res1, res2).expect("Failed to join results");

    let version = format!(
        "{}.{}.{}",
        protocol_version_struct.version_major,
        protocol_version_struct.version_minor,
        protocol_version_struct.version_patch
    );

    println!("Protocol version is: {version}");
    println!("Device id is: {:?}", device_id_struct.device_id);

    // Read the 30 packages we are waiting since the start of this example, all above tasks have success, we did it!
    println!("Waiting for 30 profiles...");
    match rx.await {
        Ok(v) => println!("Received {} profiles", v.len()),
        Err(_) => println!("The oneshot sender dropped"),
    }

    println!("Turning-off the continuous messages stream from Ping1D");
    ping1d
        .continuous_stop(bluerobotics_ping::ping1d::ProfileStruct::id())
        .await?;

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    port_name: PathBuf,
    #[arg(short, long, default_value_t = 115200)]
    baud_rate: u32,
}
