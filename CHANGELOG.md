# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v0.3.0] - 2021-05-26

### Changed

- [breaking-change] `Rn4870` constructor now expects split `Rx` and `Tx` of the serial peripheral structure
- Added `Rn4870.release` method to give back ownership of `Rx` and `Tx` (see example)
- Added example of driver usage on bluepill board
- [breaking-change] `send_command`'s argument is now `Option<&str>` - there are commands which do not take variable argument
- Added more set command wrappers
  - `set_firmware_revision`,
  - `set_hardware_revision`
  - `set_software_revision`
  - `set_model_name`
  - `set_manufacturer_name`
  - `set_serial_number`

### Fixed

-

## [v0.2.2] - 2020-12-30

### Changed

- Added `Services` struct to represent services bit flags for
- Added `soft_reset` method to allow appying configuration
- Added `set_serialized_name` (wraps `S-` set command)
- [breaking-change] `set_default_services` replaced with `set_services` which takes `Services` variant as argument

### Fixed

- also expect `CMD>` string in reply of `send_command`
