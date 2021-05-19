#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate nb;
extern crate panic_rtt_target;
extern crate rtt_target;
extern crate stm32f1xx_hal;

use core::mem::MaybeUninit;

use cortex_m_rt::entry;
use nb::block;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    delay::Delay,
    pac::{self, interrupt},
    prelude::*,
    serial::{Config, Event, Serial},
};

const STATUS_DELIMITER: u8 = b'%';
static mut RX: MaybeUninit<stm32f1xx_hal::serial::Rx2> = MaybeUninit::uninit();
static mut BUF: heapless::Vec<u8, 256> = heapless::Vec::new();

#[pac::interrupt]
fn USART2() {
    #[allow(unsafe_code)]
    let rx = unsafe { &mut *RX.as_mut_ptr() };
    #[allow(unsafe_code)]
    let result = unsafe { &mut BUF };

    match block!(rx.read()) {
        Ok(byte) => {
            if byte == STATUS_DELIMITER && result.starts_with(&[STATUS_DELIMITER]) {
                result.push(byte).unwrap();
                rprintln!("status: {}", core::str::from_utf8(&result[..]).unwrap());
                result.clear();
            } else if byte == STATUS_DELIMITER && result.len() > 0 {
                rprintln!("data: {:02x?}", result);
                result.clear();
                result.push(byte).unwrap();
            } else {
                result.push(byte).unwrap();
            }
        }
        Err(e) => {
            rprintln!("error = {:?}", e);
        }
    };
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    // Get access to the device specific peripherals from the peripheral access crate
    let p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(64.mhz())
        .hclk(64.mhz())
        .pclk1(24.mhz())
        .pclk2(64.mhz())
        .freeze(&mut flash.acr);

    // Prepare the alternate function I/O registers
    let mut afio = p.AFIO.constrain(&mut rcc.apb2);

    // Prepare the GPIOA peripheral
    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);

    // Module peripherals
    let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rx = gpioa.pa3;
    let reset = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    let mut delay = Delay::new(cp.SYST, clocks);

    // Set up the usart device. Taks ownership over the USART register and tx/rx pins. The rest of
    // the registers are used to enable and configure the device.
    let mut serial = Serial::usart2(
        p.USART2,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(115_200.bps()),
        clocks,
        &mut rcc.apb1,
    );

    serial.listen(Event::Rxne);
    let (tx, rx) = serial.split();
    let mut ble_radio = rn4870::Rn4870::new(rx, tx, reset);

    rprintln!("init");
    ble_radio.hard_reset(&mut delay).unwrap();
    ble_radio.enter_cmd_mode().unwrap();
    ble_radio
        .set_services(
            rn4870::Services::DEVICE_INFORMATION
                | rn4870::Services::UART_TRANSPARENT
                | rn4870::Services::DEVICE_INFORMATION,
        )
        .unwrap();
    ble_radio.set_serialized_name("Serialized").unwrap();
    ble_radio.set_manufacturer_name("Manufacturer").unwrap();
    ble_radio.set_model_name("Model1").unwrap();
    ble_radio.set_name("Name").unwrap();
    ble_radio.soft_reset().unwrap();
    let (_tx, rx) = ble_radio.release();
    rprintln!("BLE radio configured");

    #[allow(unsafe_code)]
    {
        let rx_static = unsafe { &mut *RX.as_mut_ptr() };
        *rx_static = rx;
        unsafe {
            pac::NVIC::unmask(pac::Interrupt::USART2);
        }
    }

    loop {
        // cortex_m::asm::wfi();
        core::hint::spin_loop();
    }
}
