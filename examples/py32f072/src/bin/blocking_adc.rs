#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use py32_hal::adc::{Adc, SampleTime};
use py32_hal::rcc::{HsiFs, Pll, PllMul, PllSource, Sysclk};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg: py32_hal::Config = Default::default();
    cfg.rcc.hsi = Some(HsiFs::HSI_24MHZ);
    cfg.rcc.pll = Some(Pll {
        src: PllSource::HSI,
        mul: PllMul::MUL3,
    });
    cfg.rcc.sys = Sysclk::PLL;
    let p = py32_hal::init(cfg);

    info!("Hello World!");

    // Automatically calculate the minimum prescaler using PCLK.
    let mut adc = Adc::new(p.ADC1);

    // use py32_hal::adc::Prescaler;
    // let mut adc = Adc::new_with_prediv(p.ADC1, Prescaler::Div4);

    // The minimum conversion time for each resolution is as follows (sampling time + conversion time):
    // 12-bit: 3.5 + 12.5 = 16 ADCCLK cycles
    // 10-bit: 3.5 + 10.5 = 14 ADCCLK cycles
    // 8-bit:  3.5 + 8.5  = 12 ADCCLK cycles
    // 6-bit:  3.5 + 6.5  = 10 ADCCLK cycles
    adc.set_sample_time(SampleTime::CYCLES71_5);
    let mut pin = p.PA7;

    let mut vrefint = adc.enable_vrefint();

    loop {
        let vrefint_sample = adc.blocking_read(&mut vrefint);
        let v = adc.blocking_read(&mut pin);
        info!("value: {}", v);
        info!("vrefint_sample: {}", vrefint_sample);
        info!(
            "--> {} - {} mV",
            v,
            convert_to_millivolts(v, vrefint_sample)
        );
        Timer::after_millis(100).await;
    }
}

pub fn convert_to_millivolts(sample: u16, vrefint: u16) -> u16 {
    const VREFINT_MV: u32 = 1200; // mV

    (u32::from(sample) * VREFINT_MV / u32::from(vrefint)) as u16
}
