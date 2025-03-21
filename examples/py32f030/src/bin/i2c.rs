#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::*;
use embassy_executor::Spawner;
use py32_hal::i2c::{Error, I2c};
use py32_hal::time::Hertz;
use {defmt_rtt as _, panic_probe as _};

const ADDRESS: u8 = 0x42;
const WRITE_DATA: [u8; 2] = [0xC2, 0x10];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello world!");
    let p = py32_hal::init(Default::default());

    let mut i2c = I2c::new_blocking(p.I2C1, p.PA3, p.PA2, Hertz(100_000), Default::default());

    let mut data = [0u8; 1];

    match i2c.blocking_write_read(ADDRESS, &WRITE_DATA, &mut data) {
        Ok(()) => info!("Read data: {}", data[0]),
        Err(Error::Timeout) => error!("Operation timed out"),
        Err(e) => error!("I2c Error: {:?}", e),
    }
}
