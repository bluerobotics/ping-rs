# :crab: Ping Library :sound:

[SONARs](https://en.wikipedia.org/wiki/Sonar), or **So**und **N**avigation **A**nd **R**anging devices, transmit sound waves and measure their reflections to detect surrounding obstacles and objects.

[Sonars and other acoustic devices](https://bluerobotics.com/learn/a-smooth-operators-guide-to-underwater-sonars-and-acoustic-devices/) can be crucial for marine robotics (ROVs, AUVs, boats, etc), as they can provide obstacle detection and distance estimates when the visual field is limited.
Building robots with these capabilities enables precise navigation, target identification, and a considerably more efficient exploration of underwater environments.

The open source [Ping Protocol](https://docs.bluerobotics.com/ping-protocol) allows precise control of sonar devices like those from [Blue Robotics](https://bluerobotics.com/product-category/sonars/), enabling easy integration in your custom applications.

This library provides access to all capabilities of **ping** family devices, with all the benefits of the Rust development ecosystem!

Try **ping** today!

# 📖 Documentation:
* [Ping Echosounder](https://bluerobotics.com/store/sonars/echosounders/ping-sonar-r2-rp/)
    * :blue_book: [Operating Principles and Usage Guide](https://bluerobotics.com/learn/ping-sonar-technical-guide/)
    * :wrench: [BlueROV2 Installation Guide](https://bluerobotics.com/learn/ping-installation-guide-for-the-bluerov2/)
    * :wrench: [BlueBoat Integration Guide](https://bluerobotics.com/learn/ping2-integration-kit-for-blueboat-installation-guide/)
* [Ping360 Scanning Imaging Sonar](https://bluerobotics.com/store/sonars/imaging-sonars/ping360-sonar-r1-rp/)
    * :blue_book: [Understanding and Usage Guide](https://bluerobotics.com/learn/understanding-and-using-scanning-sonars/)
    * :wrench: [BlueROV2 Installation Guide](https://bluerobotics.com/learn/ping360-installation-guide-for-the-bluerov2/)
* :crab: Getting started with [Rust](https://doc.rust-lang.org/book/ch01-00-getting-started.html)
* :ocean: Getting started with [BlueOS](https://blueos.cloud/docs/blueos/latest/overview/)
* :books: **Ping** library [docs.](https://docs.bluerobotics.com/ping-rs/bluerobotics_ping/)
* :clipboard: Check the [examples](https://github.com/bluerobotics/ping-rs/tree/master/examples) folder for a quick start guide

# :whale: How to Use This Crate:

To harness the capabilities of a [Ping1D](https://docs.bluerobotics.com/ping-rs/bluerobotics_ping/ping1d/index.html) or [Ping360](https://docs.bluerobotics.com/ping-rs/bluerobotics_ping/ping360/index.html) type device, instantiate the corresponding object provided by this library.

Ping has the capability to work with any kind of layer that implements [asynchronous I/O](https://tokio.rs/tokio/tutorial/io) traits. The current examples are focused on serial and UDP, which are the connection methods with official support from [Blue Robotics](https://bluerobotics.com/).

Both device types have their own set of methods, as defined by the [Ping Protocol](https://docs.bluerobotics.com/ping-protocol/) specification.

Check the complete set of methods:
* [Ping1D](https://docs.bluerobotics.com/ping-rs/bluerobotics_ping/ping1d/struct.Device.html)
* [Ping360](https://docs.bluerobotics.com/ping-rs/bluerobotics_ping/ping360/struct.Device.html)

## :dolphin: To run examples use:

```shell
cargo run --example ping_1d -- --serial-port /dev/ttyUSB0
```

<details>
  <summary>Result</summary>

  ### Terminal output:

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
</details>


### :star: **Pro tip**
For external use via UDP, consider using [bridges](https://github.com/patrickelectric/bridges) to share your serial device to the network. Detailed instructions can be found [here](https://github.com/patrickelectric/bridges?tab=readme-ov-file#install-zap).
<details>
  <summary>Setting up a host and client</summary>

#### On the host :satellite: (Where ping device is connected):

```shell
bridges --port /dev/ttyUSB0:115200 -u 0.0.0.0:8080
```

#### On the client :computer::

```shell
cargo run --example ping_1d -- --udp-address 192.168.0.191 --udp-port 8080
```
</details>

Enjoy exploring with ping-rs! :ocean: