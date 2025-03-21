#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_time::Timer;
use py32_hal::gpio::{Level, Output, Speed};
use py32_hal::rcc::{HsiFs, Pll, PllSource, Sysclk};

use cortex_m::Peripherals;
use defmt::*;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let cp = Peripherals::take().unwrap();
    let systick = cp.SYST;

    let mut cfg: py32_hal::Config = Default::default();
    cfg.rcc.hsi = Some(HsiFs::HSI_24MHZ);
    cfg.rcc.pll = Some(Pll {
        src: PllSource::HSI,
    });
    cfg.rcc.sys = Sysclk::PLL;
    let p = py32_hal::init(cfg, systick);

    info!("Hello World!");

    let mut led = Output::new(p.PB1, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(1000).await;

        info!("low");
        led.set_low();
        Timer::after_millis(1000).await;
    }
}
