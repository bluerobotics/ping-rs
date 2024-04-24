use std::convert::TryFrom;

use bluerobotics_ping::common::Messages as common_messages;
use bluerobotics_ping::{common, Messages};

#[test]
fn test_inner() {
    let general_request_struct = common::GeneralRequestStruct { requested_id: 5 };
    let general_request = common_messages::GeneralRequest(general_request_struct.clone());
    let inner_struct: &common::GeneralRequestStruct = general_request
        .inner()
        .expect("Failed to fetch inner struct");

    assert_eq!(*inner_struct, general_request_struct);

    // From official ping protocol documentation
    let buffer: Vec<u8> = vec![
        0x42, 0x52, 0x02, 0x00, // payload length
        0x06, 0x00, // message id
        0x00, 0x00, // src and dst id
        0x05, 0x00, // payload
        0xa1, 0x00, // crc
    ];
    let decoded_message = Messages::try_from(&buffer).expect("Failed to parse common message.");
    let inner_struct: &common::GeneralRequestStruct = decoded_message
        .inner()
        .expect("Failed to fetch inner struct");
    assert_eq!(*inner_struct, general_request_struct);
}
