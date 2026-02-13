use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin, Pin, PinDriver, Pull};
use esp_idf_hal::i2c::config::Config as I2cConfig;
use esp_idf_hal::i2c::{I2cDriver, I2C0};
use gt911::Gt911Blocking;
use slint::platform::software_renderer::MinimalSoftwareWindow;
use slint::platform::{PointerEventButton, WindowEvent};
use slint::LogicalPosition;

use crate::platform::DISPLAY_HEIGHT;
use crate::platform::DISPLAY_WIDTH;

/// GT911 I2C addresses — address depends on INT pin state during reset.
const GT911_ADDR_LOW: u8 = 0x5D; // INT low during reset
const GT911_ADDR_HIGH: u8 = 0x14; // INT high during reset

/// I2C bus speed — use 100kHz (standard mode) for reliability with internal pull-ups.
/// The ESP32-S3's internal pull-ups (~45kΩ) are too weak for 400kHz fast mode.
const I2C_FREQ: u32 = 100_000;

/// Touch state machine — tracks whether a finger is currently down.
#[derive(Debug, Clone, Copy, PartialEq)]
enum TouchState {
    Released,
    Pressed,
}

/// Touch controller wrapper that handles GT911 polling and Slint event dispatch.
pub struct TouchController<'a> {
    driver: Gt911Blocking<I2cDriver<'a>>,
    i2c: I2cDriver<'a>,
    state: TouchState,
    last_position: LogicalPosition,
    poll_count: u32,
    not_ready_count: u32,
    touch_count: u32,
    error_count: u32,
}

impl<'a> TouchController<'a> {
    /// Initialize the GT911 touch controller.
    ///
    /// Sets up I2C bus and performs GT911 init sequence.
    /// The INT pin must be driven LOW during reset to select I2C address 0x5D,
    /// then set to floating (open-drain high) so the GT911 can drive it as data-ready.
    ///
    /// Because GPIO 3 (INT) is an ESP32-S3 strapping pin with a default pull-up,
    /// the GT911 may latch address 0x14 if it powers up before the firmware runs.
    /// This function tries both addresses.
    pub fn new(
        i2c: I2C0,
        sda: AnyIOPin,
        scl: AnyIOPin,
        rst: AnyOutputPin,
        int: AnyIOPin,
    ) -> anyhow::Result<Self> {
        // GT911 reset sequence (datasheet section 5.2):
        // We need to drive INT low during reset, then switch to input mode.
        // Use esp-idf GPIO API directly to avoid pin ownership issues.
        let int_pin_num = int.pin();

        // 1. Drive INT low to select I2C address 0x5D
        let mut int_pin = PinDriver::output(int)?;
        int_pin.set_low()?;

        // 2. Pull RST low for >= 10ms (we use 20ms for reliability)
        let mut rst_pin = PinDriver::output(rst)?;
        rst_pin.set_low()?;
        esp_idf_hal::delay::FreeRtos::delay_ms(20);

        // 3. Release RST high — GT911 latches INT state as address select
        rst_pin.set_high()?;
        esp_idf_hal::delay::FreeRtos::delay_ms(5);

        // Keep RST pin driven HIGH — GT911 needs RST held high for normal operation.
        // Without this, the PinDriver drop would disable GPIO 4, causing it to float.
        core::mem::forget(rst_pin);

        // 4. Reconfigure INT as input with pull-down after address is latched.
        //    The GT911 drives INT as an open-drain output to signal data-ready.
        //    Plain input mode lets GT911 freely drive the line.
        //    Pull-down keeps the line low when GT911 isn't driving it.
        //
        //    SAFETY: We reclaim the same GPIO pin after dropping the output driver.
        //    No other code uses this pin.
        drop(int_pin);
        let mut int_input = PinDriver::input(unsafe { AnyIOPin::new(int_pin_num) })?;
        int_input.set_pull(Pull::Down)?;
        core::mem::forget(int_input);

        // 5. Wait for GT911 to fully boot (100ms for extra margin)
        esp_idf_hal::delay::FreeRtos::delay_ms(100);

        // Configure I2C with explicit pull-ups enabled
        let i2c_config = I2cConfig::new().baudrate(I2C_FREQ.into());
        let mut i2c_driver = I2cDriver::new(i2c, sda, scl, &i2c_config)?;

        // Try address 0x5D first (INT was low during reset), then fall back to 0x14
        // (GT911 may have latched 0x14 if it powered up while GPIO 3 was high during
        // ESP32-S3 boot — GPIO 3 is a strapping pin with a default pull-up).
        let (driver, addr) = Self::init_with_address_scan(&mut i2c_driver)?;

        log::info!("GT911 touch controller initialized (addr: 0x{:02X})", addr);

        Ok(Self {
            driver,
            i2c: i2c_driver,
            state: TouchState::Released,
            last_position: LogicalPosition::new(0.0, 0.0),
            poll_count: 0,
            not_ready_count: 0,
            touch_count: 0,
            error_count: 0,
        })
    }

    /// Try initializing the GT911 at address 0x5D, then 0x14 if that fails.
    fn init_with_address_scan(
        i2c: &mut I2cDriver<'a>,
    ) -> anyhow::Result<(Gt911Blocking<I2cDriver<'a>>, u8)> {
        for &addr in &[GT911_ADDR_LOW, GT911_ADDR_HIGH] {
            log::info!("Trying GT911 at address 0x{:02X}...", addr);
            let driver = Gt911Blocking::new(addr);
            match driver.init(i2c) {
                Ok(()) => return Ok((driver, addr)),
                Err(e) => {
                    log::warn!("GT911 at 0x{:02X} failed: {:?}", addr, e);
                }
            }
        }

        Err(anyhow::anyhow!(
            "GT911 not found at 0x5D or 0x14 — check I2C wiring (SDA=GPIO1, SCL=GPIO2)"
        ))
    }

    /// Poll touch input and dispatch events to the Slint window.
    ///
    /// Call this once per iteration of the main event loop.
    /// Maps GT911 touch coordinates to Slint `WindowEvent`s following the
    /// Pressed -> Moved -> Released pattern.
    pub fn poll(&mut self, window: &MinimalSoftwareWindow) {
        self.poll_count = self.poll_count.wrapping_add(1);

        match self.driver.get_touch(&mut self.i2c) {
            Ok(Some(point)) => {
                self.touch_count = self.touch_count.wrapping_add(1);

                // GT911 reports in portrait (x=0..319, y=0..479).
                // Display is landscape (480x320). Transform coordinates:
                //   display_x = gt911_y  (swap)
                //   display_y = 319 - gt911_x  (swap + invert)
                let x = (point.y as f32).min(DISPLAY_WIDTH as f32 - 1.0);
                let y = ((DISPLAY_HEIGHT as u16)
                    .saturating_sub(1)
                    .saturating_sub(point.x) as f32)
                    .max(0.0);
                let position = LogicalPosition::new(x, y);
                self.last_position = position;

                match self.state {
                    TouchState::Released => {
                        // New touch — send PointerPressed
                        window.dispatch_event(WindowEvent::PointerPressed {
                            position,
                            button: PointerEventButton::Left,
                        });
                        self.state = TouchState::Pressed;
                    }
                    TouchState::Pressed => {
                        // Finger still down — send PointerMoved
                        window.dispatch_event(WindowEvent::PointerMoved { position });
                    }
                }
            }
            Ok(None) => {
                // Finger lifted
                if self.state == TouchState::Pressed {
                    window.dispatch_event(WindowEvent::PointerReleased {
                        position: self.last_position,
                        button: PointerEventButton::Left,
                    });
                    window.dispatch_event(WindowEvent::PointerExited);
                    self.state = TouchState::Released;
                }
            }
            Err(gt911::Error::NotReady) => {
                self.not_ready_count = self.not_ready_count.wrapping_add(1);
                // No new data — poll again next iteration (silent)
            }
            Err(e) => {
                self.error_count = self.error_count.wrapping_add(1);
                log::warn!("Touch read error: {:?}", e);
            }
        }
    }
}
