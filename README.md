# ping-rs
https://docs.bluerobotics.com/ping-rs/ping_rs/

## Using example:

To run examples use:

```shell
cargo run --example ping_common /dev/ttyUSB0 115200
```

Should output:
```shell
Parsing user provided values...
Creating serial connection...
Ping protocol request:
Result: Ok(ProtocolVersionStruct { version_major: 1, version_minor: 0, version_patch: 0, reserved: 0 })
Device information request:
Result: Ok(DeviceInformationStruct { device_type: 1, device_revision: 1, firmware_version_major: 3, firmware_version_minor: 29, firmware_version_patch: 0, reserved: 0 })
Set device_id request:
Result: Ok(AckStruct { acked_id: 100 })
Manual request:
Result: Ok(ProtocolMessage { payload_length: 4, message_id: 5, src_device_id: 1, dst_device_id: 0, payload: [1, 0, 0, 0], checksum: 159 })
```