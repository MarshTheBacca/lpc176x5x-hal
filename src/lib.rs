#![no_std]
pub use lpc176x5x_pac as pac;

pub mod clocks;
pub mod gpio;

pub use crate::clocks::{ClockConfig, Clocks, Hertz};
pub use crate::gpio::GpioExt;
