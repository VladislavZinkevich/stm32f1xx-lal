#![no_std]

#[cfg(feature = "stm32f100")]
pub use stm32f1::stm32f100 as pac;

#[cfg(feature = "stm32f101")]
pub use stm32f1::stm32f101 as pac;

#[cfg(feature = "stm32f103")]
pub use stm32f1::stm32f103 as pac;

#[cfg(feature = "stm32f105")]
pub use stm32f1::stm32f107 as pac;

#[cfg(feature = "stm32f107")]
pub use stm32f1::stm32f107 as pac;

pub mod systime;
pub mod prelude;
pub mod gpio;