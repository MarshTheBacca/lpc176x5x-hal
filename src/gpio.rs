use crate::pac::Gpio;
use core::marker::PhantomData;

const GPIO_BASE: usize = 0x2009_C000;

pub struct Input;
pub struct Output;

pub trait GpioExt {
    fn split(self) -> Parts;
}

macro_rules! define_pins {
    ($(
        $struct_name:ident: ($port:expr, $pin:expr)),+ $(,)?
    ) => {
        $(
            pub struct $struct_name<MODE> {
                _mode: PhantomData<MODE>,
            }

            impl<MODE> $struct_name<MODE> {
                pub fn into_output(self) -> $struct_name<Output> {
                    let dir_reg = (GPIO_BASE + ($port * 0x20)) as *mut u32;
                    unsafe {
                        let current = core::ptr::read_volatile(dir_reg);
                        core::ptr::write_volatile(dir_reg, current | (1 << $pin));
                    }
                    $struct_name { _mode: PhantomData }
                }
            }

            impl $struct_name<Output> {
                pub fn set_high(&mut self) {
                    let set_reg = (GPIO_BASE + ($port * 0x20) + 0x18) as *mut u32;
                    unsafe { core::ptr::write_volatile(set_reg, 1 << $pin); }
                }

                pub fn set_low(&mut self) {
                    let clr_reg = (GPIO_BASE + ($port * 0x20) + 0x1C) as *mut u32;
                    unsafe { core::ptr::write_volatile(clr_reg, 1 << $pin); }
                }
            }
        )+

        pub struct Parts {
            $( pub $struct_name: $struct_name<Input>, )+
        }

        impl GpioExt for Gpio {
            fn split(self) -> Parts {
                Parts {
                    $( $struct_name: $struct_name { _mode: PhantomData }, )+
                }
            }
        }
    };
}

define_pins! {
    // Port 1
    p0_0:  (0, 0),
    p0_1:  (0, 1),
    p0_2:  (0, 2),
    p0_3:  (0, 3),

    // Port 1
    p1_1:  (1, 1), // USR LED
    p1_4:  (1, 4), // RX LED
    p1_8:  (1, 8), // TX LED
    p1_9:  (1, 9), // 1v8 LED
    p1_18: (1, 18),  // USB LED
    p1_20: (1, 20),
    p1_21: (1, 21),
    p1_22: (1, 22),
    p1_23: (1, 23),
    p1_25: (1, 25),
}
