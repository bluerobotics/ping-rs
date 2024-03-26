use ping_rs::{common, message::ProtocolMessage, PingDevice};
use serialport::ClearBuffer;
use std::time::Duration;

fn main() {
    // See mod cli_args bellow.
    println!("Parsing user provided values...");
    let (port_name, baud_rate) = cli_args::get_cli_args();

    println!("Creating serial connection...");
    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Failed to open serial port");
    port.clear(ClearBuffer::All).expect("clear");

    let mut ping = PingDevice::new(port);

    println!(
        "Ping protocol request: \nResult: {:?}",
        ping.get_protocol_version()
    );
    println!(
        "Device information request: \nResult: {:?}",
        ping.get_device_information()
    );
    println!(
        "Set device_id request: \nResult: {:?}",
        ping.set_device_id(1)
    );

    let request =
        common::Messages::GeneralRequest(common::GeneralRequestStruct { requested_id: 5 });
    let mut package = ProtocolMessage::new();
    package.set_message(&request);
    println!("Manual request: \nResult: {:?}", ping.request(package));
}

mod cli_args {
    use clap::{Arg, Command};

    pub fn get_cli_args() -> (String, u32) {
        let matches = Command::new("Ping Common Example")
            .about("Execute generic requests from common module")
            .disable_version_flag(true)
            .arg(
                Arg::new("port")
                    .help("The device path to a serial port")
                    .required(true),
            )
            .arg(
                Arg::new("baud")
                    .help("The baud rate to connect at")
                    .use_value_delimiter(false)
                    .required(true)
                    .validator(valid_baud),
            )
            .get_matches();

        let port_name = matches.value_of("port").unwrap().to_string();
        let baud_rate = matches.value_of("baud").unwrap().parse::<u32>().unwrap();

        (port_name, baud_rate)
    }

    fn valid_baud(val: &str) -> Result<(), String> {
        val.parse::<u32>()
            .map(|_| ())
            .map_err(|_| "Invalid baud rate".into())
    }
}
