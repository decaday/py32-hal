#![cfg_attr(not(test), no_std)]
#![allow(async_fn_in_trait)]

// This must go FIRST so that all the other modules see its macros.
mod fmt;
include!(concat!(env!("OUT_DIR"), "/_macros.rs"));

mod macros;

pub use py32_metapac as pac;

// This must go last, so that it sees all the impl_foo! macros defined earlier.
pub(crate) mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]
    #![allow(missing_docs)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

// pub use crate::_generated::interrupt;

pub use _generated::{peripherals, Peripherals};
pub use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

pub mod time;
pub mod rcc;
pub mod gpio;

// pub fn init(config: Config) -> Peripherals {
// }

pub fn init() -> Peripherals {
    critical_section::with(|cs| {
        let p = Peripherals::take_with_cs(cs);
        unsafe {
            gpio::init(cs);
        };
        p
    })
}