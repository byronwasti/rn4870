# Examples

This is a separate rust crate containing examples of usage of the RN4870
driver. It is separate from the main driver crate to allow building for no_std
targets and still keep tests in the driver crate code.


## Bluepill-IRQ

- assumes bluepill board with RN4870 connected to Serial2 peripheral and reset to A3 (pin PA4).
- shows how to configure the module and then give rx,tx peripheral ownership back to work with it in IRQ handler
- with `[cargo-embed](https://github.com/probe-rs/cargo-embed)` you can easily flash it to bluepill using command
  `cargo embed --bin bluepill-irq --chip STM32F103C8 --release --target thumbv7m-none-eabi`
- example expect you data to be `\0` delimited, so when you send eg. `0xdeadbeef00` you should see something like this in RTT console

```
init
BLE radio configured
status: %CONNECT,1,000000000000%
status: %STREAM_OPEN%
status: %CONN_PARAM,0018,0000,0200%
data: [de, ad, be, ef]
status: %DISCONNECT%
```
