#[cfg(feature = "json_schema")]
mod schema_tests {
    use bluerobotics_ping::common::Messages as CommonMessages;
    use bluerobotics_ping::common::{
        AckStruct, AsciiTextStruct, DeviceInformationStruct, GeneralRequestStruct, NackStruct,
        ProtocolVersionStruct,
    };
    use bluerobotics_ping::Messages;
    use jsonschema::Validator;
    use schemars::schema_for;
    use serde_json::json;
    use std::fs::{create_dir_all, File};
    use std::io::{Read, Write};
    use std::path::Path;

    fn generate_validator() -> Validator {
        let schema = schema_for!(Messages);
        jsonschema::validator_for(&json!(&schema)).unwrap()
    }

    fn save_schema(filename: &str) {
        let schema = schema_for!(Messages);
        let schema_json = serde_json::to_string_pretty(&schema).unwrap();
        let tmp_dir = Path::new("tests/tmp");
        create_dir_all(tmp_dir).unwrap();
        let file_path = tmp_dir.join(filename);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", schema_json).unwrap();
    }

    fn load_schema(filename: &str) -> Validator {
        let file_path = Path::new("tests/tmp").join(filename);
        let mut file = File::open(&file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let schema: serde_json::Value = serde_json::from_str(&contents).unwrap();
        jsonschema::validator_for(&schema).unwrap()
    }

    fn create_test_messages() -> Vec<Messages> {
        vec![
            Messages::Common(CommonMessages::Ack(AckStruct { acked_id: 1 })),
            Messages::Common(CommonMessages::AsciiText(AsciiTextStruct {
                ascii_message: "Test message".to_string(),
            })),
            Messages::Common(CommonMessages::DeviceInformation(DeviceInformationStruct {
                device_type: 1,
                device_revision: 2,
                firmware_version_major: 3,
                firmware_version_minor: 4,
                firmware_version_patch: 5,
                reserved: 0,
            })),
            Messages::Common(CommonMessages::GeneralRequest(GeneralRequestStruct {
                requested_id: 5,
            })),
            Messages::Common(CommonMessages::ProtocolVersion(ProtocolVersionStruct {
                version_major: 1,
                version_minor: 0,
                version_patch: 0,
                reserved: 0,
            })),
            Messages::Common(CommonMessages::Nack(NackStruct {
                nacked_id: 1,
                nack_message: "Error message".to_string(),
            })),
        ]
    }

    #[test]
    fn test_valid_messages() {
        let validator = generate_validator();
        let messages = create_test_messages();

        for (i, message) in messages.iter().enumerate() {
            let message_json = json!(&message);
            assert!(
                validator.is_valid(&message_json),
                "Schema validation failed for message {}: {:?}",
                i,
                message
            );
        }
    }

    #[test]
    fn test_invalid_messages() {
        let validator = generate_validator();

        // Test with an invalid message structure
        let invalid_json = json!({
            "invalid_field": "invalid_value"
        });
        assert!(
            !validator.is_valid(&invalid_json),
            "Schema validation should fail for invalid message structure"
        );

        // Test with a malformed message structure
        let malformed_json = json!({
            "GeneralRequest": {
                "requested_id": "not_a_number" // Should be a number, not a string
            }
        });
        assert!(
            !validator.is_valid(&malformed_json),
            "Schema validation should fail for malformed message structure"
        );
    }

    #[test]
    fn test_schema_save_load() {
        let filename = "message_schema.json";

        save_schema(filename);
        println!("Successfully saved schema to tests/tmp/{}", filename);

        let validator = load_schema(filename);

        let test_message = Messages::Common(CommonMessages::Ack(AckStruct { acked_id: 1 }));
        let message_json = json!(&test_message);
        assert!(
            validator.is_valid(&message_json),
            "Schema validation failed for loaded schema"
        );
    }
}
