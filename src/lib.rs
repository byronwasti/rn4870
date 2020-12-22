//! Driver for the RN4870 BLE module

#![no_std]

extern crate embedded_hal as hal;
#[macro_use(block)]
extern crate nb;

use hal::blocking::delay::DelayMs;
use hal::digital::v2::OutputPin;
use hal::serial::{Read, Write};

/// Error type
pub enum Error<ER, EW, GpioError> {
    /// Serial read error
    Read(ER),

    /// Serial write error
    Write(EW),

    /// Gpio Error,
    Gpio(GpioError),

    /// Invalid response from BLE module
    InvalidResponse,
}

/// Rn4870 Object
pub struct Rn4870<UART, NRST> {
    uart: UART,
    nrst: NRST,
}

impl<UART, NRST, EW, ER, GpioError> Rn4870<UART, NRST>
where
    UART: Write<u8, Error = EW> + Read<u8, Error = ER>,
    NRST: OutputPin<Error = GpioError>,
{
    /// Construct a new Rn4870 Object
    pub fn new(uart: UART, nrst: NRST) -> Self {
        Self { uart, nrst }
    }

    /// Reset the RN4870 module
    ///
    /// Note that this must be done before
    /// the RN4870 will start responding to
    /// serial commands.
    pub fn hard_reset<DELAY: DelayMs<u16>>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<(), Error<ER, EW, GpioError>> {
        self.nrst.set_low().map_err(Error::Gpio)?;
        delay.delay_ms(200u16);
        self.nrst.set_high().map_err(Error::Gpio)?;

        let mut buffer = [0; 8];
        let expected = [b'%', b'R', b'E', b'B', b'O', b'O', b'T', b'%'];

        self.blocking_read(&mut buffer[..])
            .map_err(|e| Error::Read(e))?;

        if buffer != expected {
            Err(Error::InvalidResponse)
        } else {
            Ok(())
        }
    }

    /// Internal blocking read
    ///
    /// TODO: Use `embedded_hal` traits for this in the future
    fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), ER> {
        for elem in buffer {
            *elem = block!(self.uart.read())?;
        }
        Ok(())
    }

    /// Internal blocking write
    ///
    /// TODO: Use `embedded_hal` traits for this in the future
    fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), EW> {
        for elem in buffer {
            block!(self.uart.write(*elem))?;
        }
        Ok(())
    }

    /// Escape hatch for handling hardware errors
    ///
    /// Until the `embedded_hal` traits include error handling there
    /// is no device-agnostic way to deal with hardware errors. This is
    /// an escape hatch to allow users to access the UART peripheral.
    pub fn handle_error<T: Fn(&mut UART) -> ()>(&mut self, func: T) {
        func(&mut self.uart);
    }

    /// Enter Command Mode
    pub fn enter_cmd_mode(&mut self) -> Result<(), Error<ER, EW, GpioError>> {
        self.blocking_write(&[b'$', b'$', b'$'])
            .map_err(|e| Error::Write(e))?;

        let mut buffer = [0; 5];
        let expected = [b'C', b'M', b'D', b'>', b' '];

        self.blocking_read(&mut buffer[..])
            .map_err(|e| Error::Read(e))?;

        if buffer != expected {
            Err(Error::InvalidResponse)
        } else {
            Ok(())
        }
    }

    /// Enter Data Mode
    pub fn enter_data_mode(&mut self) -> Result<(), Error<ER, EW, GpioError>> {
        self.blocking_write(&[b'-', b'-', b'-', b'\r'])
            .map_err(|e| Error::Write(e))?;

        Ok(())
    }

    fn send_command(
        &mut self,
        command: &str,
        argument: &str,
    ) -> Result<(), Error<ER, EW, GpioError>> {
        // Send command
        self.blocking_write(&command.as_bytes())
            .map_err(|e| Error::Write(e))?;

        self.blocking_write(&[b',']).map_err(|e| Error::Write(e))?;

        // Send argument
        self.blocking_write(&argument.as_bytes())
            .map_err(|e| Error::Write(e))?;

        // Send return carriage to end command
        self.blocking_write(&[b'\r']).map_err(|e| Error::Write(e))?;

        // Check for response
        let mut buffer = [0; 10];
        self.blocking_read(&mut buffer[..])
            .map_err(|e| Error::Read(e))?;

        // only if SR,<hex16> is set with 0x4000 (No prompt) then the prompt is not send
        if buffer == "AOK\r\nCMD> ".as_bytes() {
            Ok(())
        } else {
            Err(Error::InvalidResponse)
        }
    }

    /// Sets a serialized Bluetooth name for the device
    ///
    /// This function only works when in Command Mode.
    pub fn set_serialized_name(&mut self, name: &str) -> Result<(), Error<ER, EW, GpioError>> {
        // Name must be less than 15 characters
        if name.as_bytes().len() > 15 {
            panic!("Invalid name length");
        }

        self.send_command("S-", name)
    }
    ///
    /// Sets the device name
    ///
    /// This function only works when in Command Mode.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error<ER, EW, GpioError>> {
        // Name must be less than 20 characters
        if name.as_bytes().len() > 20 {
            panic!("Invalid name length");
        }

        self.send_command("SN", name)
    }

    /// Set default services
    pub fn set_default_services(&mut self, value: u8) -> Result<(), Error<ER, EW, GpioError>> {
        self.send_command("SS", "C0")
    }

    pub fn send_raw(&mut self, values: &[u8]) -> Result<(), EW> {
        for value in values {
            block!(self.uart.write(*value))?;
        }

        Ok(())
    }

    pub fn read_raw(&mut self) -> Result<u8, ER> {
        block!(self.uart.read())
    }
}
