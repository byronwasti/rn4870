#![no_std]

extern crate embedded_hal as hal;
#[macro_use(block)]
extern crate nb;

use hal::serial::{Read};
use hal::blocking::serial::{Write};
use hal::digital::OutputPin;
use hal::blocking::delay::{DelayMs};


pub enum Error<ER, EW> {
    Read(ER),
    Write(EW),
    InvalidResponse,
}

struct Rn4870<UART, NRST> {
    pub uart: UART,
    nrst: NRST,
}

impl<UART, NRST, EW, ER> Rn4870<UART, NRST>
where
    UART: Write<u8, Error = EW> + Read<u8, Error = ER>,
    NRST: OutputPin,
{
    pub fn new(uart: UART, nrst: NRST) -> Self {
        Self {
            uart,
            nrst,
        }
    }

    pub fn reset<DELAY: DelayMs<u16>>(&mut self, delay: &mut DELAY) -> Result<(), Error<ER, EW>> {
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

    fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), ER> {
        for elem in buffer {
            *elem = block!(self.uart.read())?;
        }
        Ok(())
    }
}

pub struct DataMode<UART, NRST> {
    rn4870: Rn4870<UART, NRST>,
}

impl<UART, NRST, EW, ER> DataMode<UART, NRST>
where
    UART: Write<u8, Error = EW> + Read<u8, Error = ER>,
    NRST: OutputPin,
{
    pub fn new(uart: UART, nrst: NRST) -> DataMode<UART, NRST> {
        DataMode {
            rn4870: Rn4870::new(uart, nrst),
        }
    }

    pub fn reset<T: DelayMs<u16>>(&mut self, delay: &mut T) -> Result<(), Error<ER, EW>> {
        self.rn4870.reset(delay)
    }

    pub fn enter_cmd_mode(mut self) -> Result<CommandMode<UART, NRST>, Error<ER, EW>>{
        self.rn4870.uart
            .bwrite_all(&[b'$', b'$', b'$'])
            .map_err(|e| Error::Write(e))?;

        let mut buffer = [0; 5];
        let expected = [b'C',b'M',b'D',b'>',b' '];

        self.rn4870.blocking_read(&mut buffer[..]).map_err(|e| Error::Read(e))?;

        if buffer != expected {
            Err(Error::InvalidResponse)
        } else {
            Ok(CommandMode {
                rn4870: self.rn4870,
            })
        }
    }

    pub fn handle_error<T: Fn(&mut UART) -> ()>(&mut self, func: T) {
        func(&mut self.rn4870.uart);
    }
}

pub struct CommandMode<UART, NRST> {
    rn4870: Rn4870<UART, NRST>,
}

impl<UART, NRST, EW, ER> CommandMode<UART, NRST>
where
    UART: Write<u8, Error = EW> + Read<u8, Error = ER>,
    NRST: OutputPin,
{
    pub fn enter_data_mode(mut self) -> Result<DataMode<UART, NRST>, Error<ER, EW>> {
        self.rn4870.uart
            .bwrite_all(&[b'-', b'-', b'-', b'\r'])
            .map_err(|e| Error::Write(e))?;

        Ok(DataMode {
            rn4870: self.rn4870,
        })
    }

    pub fn reset<T: DelayMs<u16>>(&mut self, delay: &mut T) -> Result<(), Error<ER, EW>> {
        self.rn4870.reset(delay)
    }
}

