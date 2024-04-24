use ping_rs::common;
use ping_rs::message::SerializePayload;
use tracing_test::traced_test;

#[traced_test]
#[test]
fn test_simple_serialization() {
    let general_request = common::GeneralRequestStruct { requested_id: 5 };

    assert_eq!(general_request.serialize(), [0x05, 0x00]);

    let nack = common::NackStruct {
        nacked_id: 0x666,
        nack_message: "BATATA".into(),
    };

    assert_eq!(
        nack.serialize(),
        [0x66, 0x6, 0x42, 0x41, 0x54, 0x41, 0x54, 0x41, 0x00]
    );
}
