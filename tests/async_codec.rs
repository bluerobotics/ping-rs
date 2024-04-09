use futures::{SinkExt, StreamExt};
use ping_rs::codec::PingCodec;
use ping_rs::{common, message::ProtocolMessage, ping1d, Messages};
use std::convert::TryFrom;
use tokio::time::{sleep, Duration};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::{Decoder, Framed};

async fn run_ping_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut port = tokio_serial::new("/dev/ttyUSB0", 115_200).open_native_async()?;
    #[cfg(unix)]
    port.set_exclusive(false)?;

    let stream: Framed<tokio_serial::SerialStream, PingCodec> = PingCodec::new().framed(port);
    let (mut tx, mut rx) = stream.split();

    let request_id_list = [
        1200, 1201, 1202, 1203, 1204, 1205, 1206, 1207, 1208, 1210, 1211, 1212, 1213, 1214, 1215,
        1300,
    ];

    tokio::spawn(async move {
        for request_id in request_id_list {
            let request = common::Messages::GeneralRequest(common::GeneralRequestStruct {
                requested_id: request_id,
            });
            let mut package = ProtocolMessage::new();
            package.set_message(&request);

            tx.send(package).await.unwrap();
        }
    });

    for _n in 0..request_id_list.len() {
        let item = rx
            .next()
            .await
            .expect("Error awaiting future in RX stream.")
            .expect("Reading stream resulted in an error");

        assert!(item.has_valid_crc());

        match Messages::try_from(&item) {
            Ok(Messages::Ping1D(ping1d::Messages::FirmwareVersion(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::DeviceId(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::Voltage5(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::SpeedOfSound(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::Range(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::ModeAuto(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::PingInterval(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::GainSetting(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::TransmitDuration(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::GeneralInfo(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::DistanceSimple(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::Distance(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::ProcessorTemperature(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::PcbTemperature(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::PingEnable(answer))) => {
                println!("{answer:?}")
            }
            Ok(Messages::Ping1D(ping1d::Messages::Profile(answer))) => {
                println!("{answer:?}")
            }
            Ok(_) => {
                panic!("Unexpected package. {:#?}", &item)
            }
            Err(e) => {
                panic!("Error on decoder: {:?}", e)
            }
        }
    }

    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "local_runner"), ignore)]
async fn test_run_ping_test() {
    tokio::try_join!(run_ping_test()).unwrap();
}
