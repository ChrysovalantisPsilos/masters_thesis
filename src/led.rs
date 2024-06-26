use esp_idf_svc::hal::{gpio::OutputPin, peripheral::Peripheral, rmt::{config::TransmitConfig, FixedLengthSignal, PinState, Pulse, RmtChannel, TxRmtDriver}};
use anyhow::{Context, Error};
pub use rgb::RGB8;
use core::time::Duration;


/// Implementation of WS2812RMT LED.
pub struct AddressableLed<'a> {
    tx_rtm_driver: TxRmtDriver<'a>,
}

// WS2812B timing constants. These are defined in the datasheet.
const T0H: Duration =Duration::from_nanos(350);
const T0L: Duration = Duration::from_nanos(800);
const T1H: Duration = Duration::from_nanos(700);
const T1L: Duration = Duration::from_nanos(600);

impl<'d> AddressableLed<'d> {
    /// Creates a new instance of our Led.
    /// # Arguments
    /// * `led` - The LED peripheral.
    /// * `channel` - The RMT channel peripheral.
    /// # Returns
    /// The new led instance
    /// # Errors
    /// Returns an error if the LED or RMT channel cannot be initialized.
    pub fn new(led: impl Peripheral<P = impl OutputPin> + 'd, channel: impl Peripheral<P = impl RmtChannel> + 'd,) -> Result<Self, Error> {
        let config = TransmitConfig::new().clock_divider(2);
        let tx = TxRmtDriver::new(channel, led, &config).context("Failed to create the RMT driver")?;
        Ok(Self { tx_rtm_driver: tx })
    }

    /// Sets the color of the LED.
    /// # Arguments
    /// * `rgb` - The color to set the LED to.
    /// # Returns
    /// Ok if the operation was successful.
    /// # Errors
    /// Returns an error if the LED cannot be set.
    fn set_pixel(&mut self, rgb: RGB8) -> Result<(),Error> {
        let color: u32 = ((rgb.g as u32) << 16) | ((rgb.r as u32) << 8) | rgb.b as u32;
        let ticks_hz = self.tx_rtm_driver.counter_clock()?;
        let t0h = Pulse::new_with_duration(ticks_hz, PinState::High, &T0H)?;
        let t0l = Pulse::new_with_duration(ticks_hz, PinState::Low, &T0L)?;
        let t1h = Pulse::new_with_duration(ticks_hz, PinState::High, &T1H)?;
        let t1l = Pulse::new_with_duration(ticks_hz, PinState::Low, &T1L)?;
        let mut signal = FixedLengthSignal::<24>::new();
        for i in (0..24).rev() {
            let p = 2_u32.pow(i);
            let bit = p & color != 0;
            let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
            signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
        }
        self.tx_rtm_driver.start_blocking(&signal)?;

        Ok(())
    }
}

/// Sets the LED to green.
/// # Arguments
/// * `led` - The LED peripheral.
pub fn led_green(led: &mut AddressableLed) -> Result<(),Error> {
    led.set_pixel(RGB8::new(0, 10, 0))
}

/// Sets the LED to red.
/// # Arguments
/// * `led` - The LED peripheral.
pub fn led_red(led: &mut AddressableLed) -> Result<(),Error> {
    led.set_pixel(RGB8::new(10, 0, 0))
}

/// Sets the LED to off.
/// # Arguments
/// * `led` - The LED peripheral.
pub fn led_off(led: &mut AddressableLed) -> Result<(),Error> {
    led.set_pixel(RGB8::new(0, 0, 0))
}