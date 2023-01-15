use std::convert::TryFrom;

use ping_rs::common::Messages as common_messages;
use ping_rs::{common, Messages};

#[test]
fn test_simple_deserialization() {
    let general_request =
        common_messages::GeneralRequest(common::GeneralRequestStruct { requested_id: 5 });

    let buffer: Vec<u8> = vec![
        0x42, 0x52, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0x05, 0x00, 0xa1, 0x00,
    ];
    let Messages::Common(parsed) = Messages::try_from(&buffer).unwrap() else {
        panic!("");
    };

    // From official ping protocol documentation
    assert_eq!(general_request, parsed);
}
