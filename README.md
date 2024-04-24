# ping-rs
https://docs.bluerobotics.com/ping-rs/bluerobotics_ping/

## Using example:

To run examples use:

```shell
cargo run --example ping_1d -- --port-name /dev/ttyUSB0
```

Should output:
```shell
Parsing user provided values...
Creating your Ping 1D device
Testing set/get device id: 9
Testing set/get device id: 8
Testing set/get device id: 7
Testing set/get device id: 6
Testing set/get device id: 5
Testing set/get device id: 4
Testing set/get device id: 3
Testing set/get device id: 2
Testing set/get device id: 1
Set gain to auto: true
Protocol version is: 1.0.0
Device id is: 1
Waiting for 30 profiles...
Received 30 profiles
Turning-off the continuous messages stream from Ping1D
```