#![no_std]

extern crate embedded_hal as hal;
#[macro_use(block)]
extern crate nb;

use hal::serial::{Read, Write};
use hal::digital::OutputPin;
use hal::blocking::delay::{DelayMs};


pub enum Error<ER, EW> {
    Read(ER),
    Write(EW),
    InvalidResponse,
}

pub struct CommandMode {}

pub struct DataMode {}

pub struct Rn4870<UART, NRST, S> {
    uart: UART,
    nrst: NRST,
    _state: S,
}

impl<UART, NRST, EW, ER> Rn4870<UART, NRST, DataMode>
where
    UART: Write<u8, Error = EW> + Read<u8, Error = ER>,
    NRST: OutputPin,
{
    pub fn new(uart: UART, nrst: NRST) -> Self {
        Self {
            uart,
            nrst,
            _state: DataMode {},
        }
    }

    pub fn enter_cmd_mode(mut self) -> Result<Rn4870<UART,NRST,CommandMode>, Error<ER, EW>>{
        self.blocking_write(&[b'$', b'$', b'$'])
            .map_err(|e| Error::Write(e))?;

        let mut buffer = [0; 5];
        let expected = [b'C',b'M',b'D',b'>',b' '];

        self.blocking_read(&mut buffer[..]).map_err(|e| Error::Read(e))?;

        if buffer != expected {
            Err(Error::InvalidResponse)
        } else {
            Ok(
                Rn4870 {
                    uart: self.uart,
                    nrst: self.nrst,
                    _state: CommandMode {},
                }
            )
        }
    }
}

impl<UART, NRST, EW, ER> Rn4870<UART, NRST, CommandMode>
where
    UART: Write<u8, Error = EW> + Read<u8, Error = ER>,
    NRST: OutputPin,
{
    pub fn enter_data_mode(mut self) -> Result<Rn4870<UART,NRST,DataMode>, Error<ER, EW>> {
        self.blocking_write(&[b'-', b'-', b'-', b'\r'])
            .map_err(|e| Error::Write(e))?;

        Ok(
            Rn4870 {
                uart: self.uart,
                nrst: self.nrst,
                _state: DataMode {},
            }
        )
    }

}

impl<UART, NRST, EW, ER, S> Rn4870<UART, NRST, S>
where
    UART: Write<u8, Error = EW> + Read<u8, Error = ER>,
    NRST: OutputPin,
{
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

    fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), EW> {
        for elem in buffer {
            block!(self.uart.write(*elem))?;
        }
        Ok(())
    }
    
    pub fn handle_error<T: Fn(&mut UART) -> ()>(&mut self, func: T) {
        func(&mut self.uart);
    }
}

