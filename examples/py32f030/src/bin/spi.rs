#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use py32_hal::gpio::Pull;
use py32_hal::spi::{Spi, Config, mode::Slave};
use py32_hal::time::Hertz;
use py32_hal::{peripherals, dma};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("SPI Slave Demo Start");
    let p = py32_hal::init(Default::default());

    // --- SPI Configuration ---
    let mut config = Config::default();
    config.miso_pull = Pull::Up;
    config.frequency = Hertz(10000);

    let mut spi = Spi::new_slave(
        p.SPI1,
        p.PA2,
        Some(p.PA1),
        Some(p.PA0),
        Some(p.PA4),
        Some(p.DMA1_CH1),
        Some(p.DMA1_CH2),
        config,
    );

    info!("SPI Initialized in Slave Mode");

    // Buffers for transfer
    // Note: Ensure buffer length matches what the Master sends to avoid framing errors.
    let mut rx_buf = [0u8; 8];
    let tx_data = [0xCA, 0xFE, 0xBA, 0xBE, 0xCA, 0xFE, 0xBA, 0xBE];

    loop {
        // Wait for the Master to drive NSS low and clock out bytes.
        // `transfer` performs a simultaneous read/write using DMA.
        match spi.transfer(&mut rx_buf, &tx_data).await {
            Ok(()) => {
                info!("Transfer Complete");
                info!("Received: {=[u8]:#X}", rx_buf);
                info!("Sent:     {=[u8]:#X}", tx_data);
            },
            Err(e) => {
                error!("SPI Error: {:?}", e);
            }
        }

        rx_buf.fill(0);
    }
}