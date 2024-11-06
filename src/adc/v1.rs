use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::into_ref;
use py32_metapac::rcc::Rcc;
use py32_metapac::adc::vals::Ckmode;

use super::blocking_delay_us;
use crate::adc::{Adc, AdcChannel, Instance, Resolution, SampleTime};
use crate::interrupt::typelevel::Interrupt;
use crate::peripherals::ADC as ADC1;
use crate::{interrupt, rcc, Peripheral};

pub const VDDA_CALIB_MV: u32 = 3300;
pub const VREF_INT: u32 = 1230;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().isr().read().eoc() {
            T::regs().ier().modify(|w| w.set_eocie(false));
        } else {
            return;
        }

        T::state().waker.wake();
    }
}

// pub struct Vbat;

// impl AdcChannel<ADC1> for Vbat {}

// impl super::SealedAdcChannel<ADC1> for Vbat {
//     fn channel(&self) -> u8 {
//         18
//     }
// }

pub struct Vref;
impl AdcChannel<ADC1> for Vref {}
impl super::SealedAdcChannel<ADC1> for Vref {
    fn channel(&self) -> u8 {
        12
    }
}

pub struct Temperature;
impl AdcChannel<ADC1> for Temperature {}
impl super::SealedAdcChannel<ADC1> for Temperature {
    fn channel(&self) -> u8 {
        11
    }
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(
        adc: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        into_ref!(adc);
        rcc::enable_and_reset::<T>();

        // Delay 1μs when using HSI14 as the ADC clock.
        //
        // Table 57. ADC characteristics
        // tstab = 14 * 1/fadc
        blocking_delay_us(1);

        // A.7.1 ADC calibration code example
        T::regs().cfgr1().modify(|reg| reg.set_dmaen(false));
        T::regs().cr().modify(|reg| reg.set_adcal(true));

        while T::regs().cr().read().adcal() {}

        // A.7.2 ADC enable sequence code example
        // if T::regs().isr().read().adrdy() {
        //     T::regs().isr().modify(|reg| reg.set_adrdy(true));
        // }
        T::regs().cr().modify(|reg| reg.set_aden(true));

        // Delay for ADC stabilization time, parameter tSTAB, ADC_STAB_DELAY_US
        // Compute number of CPU cycles to wait for
        blocking_delay_us(1);

        // py32 adc has no adrdy
        // while !T::regs().isr().read().adrdy() 

        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Self {
            adc,
            sample_time: SampleTime::from_bits(0),
        }
    }

    // pub fn enable_vbat(&self) -> Vbat {
    //     // SMP must be ≥ 56 ADC clock cycles when using HSI14.
    //     //
    //     // 6.3.20 Vbat monitoring characteristics
    //     // ts_vbat ≥ 4μs
    //     T::regs().ccr().modify(|reg| reg.set_vbaten(true));
    //     Vbat
    // }

    pub fn enable_vref(&self) -> Vref {
        // Table 28. Embedded internal reference voltage
        // tstart = 10μs
        T::regs().ccr().modify(|reg| reg.set_vrefen(true));
        blocking_delay_us(10);
        Vref
    }

    pub fn enable_temperature(&self) -> Temperature {
        // SMP must be ≥ 56 ADC clock cycles when using HSI14.
        //
        // 6.3.19 Temperature sensor characteristics
        // tstart ≤ 10μs (parameter tSTART)
        // ts_temp ≥ 4μs
        T::regs().ccr().modify(|reg| reg.set_tsen(true));
        blocking_delay_us(10);
        Temperature
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().cfgr1().modify(|reg| reg.set_res(resolution.into()));
    }

    // pub fn set_ckmode(&mut self, ckmode: Ckmode) {
    //     // set ADC clock mode
    //     T::regs().cfgr2().modify(|reg| reg.set_ckmode(ckmode));
    // }

    pub async fn read(&mut self, channel: &mut impl AdcChannel<T>) -> u16 {
        let ch_num = channel.channel();
        channel.setup();

        // A.7.5 Single conversion sequence code example - Software trigger
        T::regs().chselr().write(|reg| reg.set_chselx(ch_num as usize, true));

        self.convert().await
    }

    async fn convert(&mut self) -> u16 {
        T::regs().isr().modify(|reg| {
            reg.set_eoc(true);
            reg.set_eosmp(true);
        });

        T::regs().smpr().modify(|reg| reg.set_smp(self.sample_time.into()));
        T::regs().ier().modify(|w| w.set_eocie(true));

        blocking_delay_us(1);
        T::regs().cr().modify(|reg| reg.set_adstart(true));

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            if T::regs().isr().read().eoc() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        T::regs().dr().read().data()
    }
}

impl<'d, T: Instance> Drop for Adc<'d, T> {
    fn drop(&mut self) {
        // A.7.3 ADC disable code example
        T::regs().cr().modify(|reg| reg.set_adstp(true));
        while T::regs().cr().read().adstp() {}

        // py32 ADC cant be disable by software 
        // T::regs().cr().modify(|reg| reg.set_());
        // while T::regs().cr().read().aden() {}

        rcc::disable::<T>();
    }
}