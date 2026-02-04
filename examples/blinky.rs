#![no_std]
#![no_main]

use cortex_m_rt::entry;
use lpc176x5x_hal::gpio::GpioExt;
use lpc176x5x_hal::pac;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let pins = dp.gpio.split();

    let mut usr_led = pins.p1_1.into_output();
    let mut rx_led = pins.p1_4.into_output();
    let mut tx_led = pins.p1_8.into_output();
    let mut cc1v8 = pins.p1_9.into_output();
    let mut usb_led = pins.p1_18.into_output();

    loop {
        usr_led.set_high();
        delay(800_000);

        rx_led.set_high();
        delay(800_000);

        tx_led.set_high();
        delay(800_000);

        cc1v8.set_high();
        delay(800_000);

        // The USB LED is inverted for some reason
        usb_led.set_low();
        delay(800_000);

        usr_led.set_low();
        rx_led.set_low();
        tx_led.set_low();
        cc1v8.set_low();
        usb_led.set_high();

        delay(800_000);
    }
}

fn delay(count: u32) {
    for _ in 0..count {
        cortex_m::asm::nop();
    }
}
