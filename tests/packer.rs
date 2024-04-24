use ping_rs::common::{self, Messages as common_messages};
use ping_rs::message::{MessageInfo, PingMessage, ProtocolMessage};
use tracing_test::traced_test;

#[traced_test]
#[test]
fn test_same_packer() {
    let mut packer = ProtocolMessage::new();
    let mut general_request = common::GeneralRequestStruct { requested_id: 5 };
    packer.set_message(&common_messages::GeneralRequest(general_request.clone()));

    assert_eq!(
        packer.serialized(),
        [0x42, 0x52, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0x05, 0x00, 0xa1, 0x00]
    );

    packer.set_message(&common_messages::GeneralRequest(general_request.clone()));

    assert_eq!(
        packer.serialized(),
        [0x42, 0x52, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0x05, 0x00, 0xa1, 0x00]
    );

    packer.set_dst_device_id(1);
    assert_eq!(
        packer.serialized(),
        [0x42, 0x52, 0x02, 0x00, 0x06, 0x00, 0x00, 0x01, 0x05, 0x00, 0xa2, 0x00]
    );

    general_request.requested_id = 1211;
    packer.set_message(&common_messages::GeneralRequest(general_request.clone()));
    packer.set_dst_device_id(0);
    assert_eq!(
        packer.serialized(),
        [0x42, 0x52, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0xbb, 0x04, 0x5b, 0x01]
    );

    assert_eq!(
        common::GeneralRequestStruct::id(),
        common_messages::GeneralRequest(general_request).message_id()
    );
}
