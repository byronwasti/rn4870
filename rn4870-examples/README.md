# Examples

This is a separate rust crate containing examples of usage of the RN4870
driver. It is separate from the main driver crate to allow building for no_std
targets and still keep tests in the driver crate code.


## Bluepill-IRQ

- assumes bluepill board with RN4870 connected to Serial2 peripheral and reset to A3 (pin PA4).
- shows how to configure the module and then give rx,tx peripheral ownership back to work with it in IRQ handler
