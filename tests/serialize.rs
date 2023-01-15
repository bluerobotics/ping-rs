use ping_rs::common::Messages as common_messages;
use ping_rs::{common, PingMessagePack};

#[test]
fn test_simple_serialization() {
    let general_request =
        common_messages::GeneralRequest(common::GeneralRequestStruct { requested_id: 5 });
    let message = PingMessagePack::from(&general_request);

    // From official ping protocol documentation
    assert_eq!(
        message.serialized(),
        [0x42, 0x52, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0x05, 0x00, 0xa1, 0x00]
    );
}
