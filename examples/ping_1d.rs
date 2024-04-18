use ping_rs::device::{Ping1D, PingDevice};
use ping_rs::error::PingError;
use tokio_serial::SerialPortBuilderExt;

#[tokio::main]
async fn main() -> Result<(), PingError> {
    let mut port = tokio_serial::new("/dev/ttyUSB1", 115200).open_native_async()?;
    #[cfg(unix)]
    port.set_exclusive(false)?;

    let mut ping1d = Ping1D::new(port);

    let mut subscribed = ping1d.subscribe();
    tokio::spawn(async move {
        loop {
            let received = subscribed.recv().await;
            match received {
                Ok(msg) => println!("Subscribed channel received: \n Start: \n {msg:?} \n Ending"),
                Err(_e) => break,
            }
        }
    });

    match ping1d.get_firmware().await {
        Ok(version) => {
            println!("Firmware version: {:?}", version);
        }
        Err(err) => {
            eprintln!("Error getting firmware version: {:?}", err);
        }
    }

    match ping1d.get_device_id().await {
        Ok(version) => {
            println!("Device id: {:?}", version);
        }
        Err(err) => {
            eprintln!("Error getting firmware version: {:?}", err);
        }
    }

    Ok(())
}
