//! Driver for the RN4870 BLE module

#![no_std]

extern crate embedded_hal as hal;
#[macro_use(block)]
extern crate nb;

use hal::serial::{Read, Write};
use hal::digital::OutputPin;
use hal::blocking::delay::{DelayMs};

/// Error type
pub enum Error<ER, EW> {
    /// Serial read error
    Read(ER),

    /// Serial write error
    Write(EW),

    /// Invalid response from BLE module
    InvalidResponse,
}

/// Rn4870 Object
pub struct Rn4870<UART, NRST> {
    uart: UART,
    nrst: NRST,
}

impl<UART, NRST, EW, ER> Rn4870<UART, NRST>
where
    UART: Write<u8, Error = EW> + Read<u8, Error = ER>,
    NRST: OutputPin,
{
    /// Construct a new Rn4870 Object
    pub fn new(uart: UART, nrst: NRST) -> Self {
        Self {
            uart,
            nrst,
        }
    }

    /// Reset the RN4870 module
    ///
    /// Note that this must be done before
    /// the RN4870 will start responding to
    /// serial commands.
    pub fn reset<DELAY: DelayMs<u16>>(&mut self, delay: &mut DELAY) 
        -> Result<(), Error<ER, EW>> {

        self.nrst.set_low();
        delay.delay_ms(200u16);
        self.nrst.set_high();

        let mut buffer = [0; 8];
        let expected = [b'%',b'R',b'E',b'B',b'O',b'O',b'T',b'%'];

        self.blocking_read(&mut buffer[..]).map_err(|e| Error::Read(e))?;

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
    pub fn enter_cmd_mode(mut self) -> Result<(), Error<ER, EW>>{
        self.blocking_write(&[b'$', b'$', b'$'])
            .map_err(|e| Error::Write(e))?;

        let mut buffer = [0; 5];
        let expected = [b'C',b'M',b'D',b'>',b' '];

        self.blocking_read(&mut buffer[..]).map_err(|e| Error::Read(e))?;

        if buffer != expected {
            Err(Error::InvalidResponse)
        } else {
            Ok(())
        }
    }

    /// Enter Data Mode
    pub fn enter_data_mode(mut self) -> Result<(), Error<ER, EW>> {
        self.blocking_write(&[b'-', b'-', b'-', b'\r'])
            .map_err(|e| Error::Write(e))?;

        Ok(())
    }
}

