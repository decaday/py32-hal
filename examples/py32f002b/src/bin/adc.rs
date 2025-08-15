#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use py32_hal::adc::{Adc, SampleTime};
use py32_hal::peripherals::ADC1;
use py32_hal::{adc, bind_interrupts};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC_COMP => adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = py32_hal::init(Default::default());
    info!("Hello World!");
    
    // PA2 and PB6 are SWD pins, so reusing them may lock you out of programming.
    // Refer to the `unsafe-reuse-swd-pins` feature's comments in py32-hal/Cargo.toml.

    let mut adc = Adc::new(p.ADC1, Irqs);
    // adc.set_ckmode(adc::vals::Ckmode::PCLK_DIV4);
    adc.set_sample_time(SampleTime::CYCLES71_5);
    let mut pin = p.PA6;
    let mut vrefint = adc.enable_vref();

    loop {
        let vrefint_sample = adc.read(&mut vrefint).await;
        if vrefint_sample == 0 {
            warn!("vrefint_sample is 0, skipping measurement to avoid division by zero");
            Timer::after_millis(100).await;
            continue;
        }
        let convert_to_millivolts = |sample| {
            const VREFINT_MV: u32 = 1200; // mV

            (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
        };

        let v = adc.read(&mut pin).await;
        info!("vrefint_sample: {}", vrefint_sample);
        info!("--> {} - {} mV", v, convert_to_millivolts(v));
        Timer::after_millis(500).await;
    }
}
