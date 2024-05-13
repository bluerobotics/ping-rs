# ping-rs
https://docs.bluerobotics.com/ping-rs/bluerobotics_ping/

## Using example:

To run examples use:

```shell
cargo run --example ping_1d -- --serial-port /dev/ttyUSB0
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
Test set & get with a new speed of sound: 343.0 m/s
Test set & get with default speed of sound: 1500.0 m/s
Protocol version is: 1.0.0
Device id is: 1
Gain setting is: 6
Processor temperature is: 42.63 °C
Voltage at 5V lane is: 5.006 V
The distance to target is: 4538 mm
Waiting for 30 profiles...
Received 30 profiles
Turning-off the continuous messages stream from Ping1D
```
### **Pro tip** :grey_exclamation:
For external use via UDP, consider using [bridges](https://github.com/patrickelectric/bridges) to share your serial device to the network. Detailed instructions can be found [here](https://github.com/patrickelectric/bridges?tab=readme-ov-file#install-zap).

#### On the host :satellite: (Where ping device is connected):

```shell
bridges --port /dev/ttyUSB0:115200 -u 0.0.0.0:8080
```

#### On the client :computer::

```shell
cargo run --example ping_1d -- --udp-address 192.168.0.191 --udp-port 8080
```

Enjoy exploring with ping-rs! :ocean: