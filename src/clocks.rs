use crate::pac::Syscon;

#[derive(Clone, Copy, Debug)]
pub struct Hertz(pub u32);

pub struct Clocks {
    pub cpu_frequency: Hertz,
    pub main_oscillator_frequency: Hertz,
    pub usb_frequency: Option<Hertz>,
}

pub struct ClockConfig {
    cpu_frequency: Hertz,
    enable_usb: bool,
}

impl ClockConfig {
    /// Create a new `ClockConfig` object
    pub fn new() -> Self {
        Self {
            cpu_frequency: Hertz(72_000_000),
            enable_usb: false,
        }
    }

    /// Enable USB. Must be done before `freeze`
    pub fn enable_usb(mut self) -> Self {
        self.enable_usb = true;
        self
    }

    /// Freeze the configuration of the clock
    ///
    /// This will result in a 72 MHz CPU clock, and 48 MHz USB clock if enabled
    ///
    /// # Arguments
    ///
    /// - `syscon` (`&mut Syscon`) - System control
    ///
    /// # Returns
    ///
    /// - `Clocks` - The configuration of the clocks of the system
    pub fn freeze(self, syscon: &mut Syscon) -> Clocks {
        // First, enable the crystal oscillator
        syscon.scs().write(|w| {
            w.oscrange()
                //  Select 1-20MHz
                .low()
                .oscen()
                // Enable the crystal
                .enabled()
        });
        while !syscon.scs().read().oscstat().is_ready() {}

        // Select the main oscillator as the clock source
        syscon
            .clksrcsel()
            .write(|w| w.clksrc().selects_the_main_osc());

        // Frequency of currently controller oscillator (FCCO)
        // FCCO = 2 * multiplier * input frequency / pre-divider
        // Registers use 0 based encoding to save space, so for '12', you write '11'
        unsafe {
            syscon
                // Configure the PLL
                .pll0cfg()
                // Write a new value. Multiplier is 12, pre-divider is 1
                .write(|w| w.msel0().bits(12 - 1).nsel0().bits(1 - 1));
        }
        // Now FCCO = 2 * 12 * 12 MHz / 1 = 288 MHz
        // Apply the changes
        self.feed_pll0(syscon);
        // Enable the PLL
        syscon.pll0con().write(|w| w.plle0().bit(true));
        // Apply changes
        self.feed_pll0(syscon);
        // Now we need to configure the CPU frequency to be 72 MHz, by dividing down the PLL.

        // Select the CPU clock config
        syscon.cclkcfg().
            // Write a new value 4.
            write(unsafe {|w| w.cclksel().bits(4 - 1)});
        if self.enable_usb {
            // If USB is enabled, set its frequency to 48 MHz (full speed 2.0)
            syscon
                .usbclkcfg()
                .write(unsafe { |w| w.usbsel().bits(6 - 1) });
        }
        // The CPU freqnecy is now at 288 / 7 = 72 MHz

        // The flash memory cannot keep up with 72 MHz. It's rated for 20 MHz. So, we
        // need to tell the CPU to wait a number of clock cycles before expecting a response.
        // To do this, wait 4 clock cycles, to effectivly access at a rate of 18 MHz.
        syscon
            .flashcfg()
            .modify(unsafe { |_, w| w.flashtim().bits(4 - 1) });
        // Wait for the PLL to stabilise by checking the lock. This lock is enabled
        // by the phase comparator when the 288 MHz and the 12 MHz input are perfectly
        // synced
        while !syscon.pll0stat().read().plock0().bit_is_set() {}

        // Connect the new 72 MHz clock to the CPU. We need to set both the enable and connect
        // bits because `write` zero-s the entire register. So this avoids connecting to a
        // disabled PLL
        syscon
            .pll0con()
            .write(|w| w.plle0().set_bit().pllc0().set_bit());
        // Apply changes
        self.feed_pll0(syscon);

        // The CPU is now running at 72 MHz.
        return Clocks {
            cpu_frequency: self.cpu_frequency,
            main_oscillator_frequency: Hertz(12_000_000),
            usb_frequency: if self.enable_usb {
                Some(Hertz(48_000_000))
            } else {
                None
            },
        };
    }

    /// Performs the PLL0 feed sequence to apply register changes.
    ///
    /// The PLL0 control and configuration registers are protected by a "lock"
    /// mechanism. Any changes to the PLL0 settings (like enabling or connecting)
    /// will not take effect until this specific two-word "feed" sequence is written
    /// to the `PLL0FEED` register.
    ///
    /// # Sequence Requirement
    ///
    /// The hardware requires exactly these two values in back-to-back cycles:
    /// 1. Write `0xAA` to `PLL0FEED`
    /// 2. Write `0x55` to `PLL0FEED`
    fn feed_pll0(&self, syscon: &mut Syscon) {
        syscon.pll0feed().write(unsafe { |w| w.bits(0xAA) });
        syscon.pll0feed().write(unsafe { |w| w.bits(0x55) });
    }
}
