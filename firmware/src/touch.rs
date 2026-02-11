use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin, Pin, PinDriver, Pull};
use esp_idf_hal::i2c::config::Config as I2cConfig;
use esp_idf_hal::i2c::{I2cDriver, I2C0};
use gt911::Gt911Blocking;
use slint::platform::software_renderer::MinimalSoftwareWindow;
use slint::platform::{PointerEventButton, WindowEvent};
use slint::LogicalPosition;

use crate::platform::DISPLAY_HEIGHT;
use crate::platform::DISPLAY_WIDTH;

/// GT911 default I2C address.
const GT911_ADDR: u8 = 0x5D;

/// I2C bus speed — GT911 supports up to 400kHz.
const I2C_FREQ: u32 = 400_000;

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

        // 2. Pull RST low for >= 1ms (we use 10ms)
        let mut rst_pin = PinDriver::output(rst)?;
        rst_pin.set_low()?;
        esp_idf_hal::delay::FreeRtos::delay_ms(10);

        // 3. Release RST high — GT911 latches INT state as address select
        rst_pin.set_high()?;
        esp_idf_hal::delay::FreeRtos::delay_ms(5);

        // 4. Reconfigure INT as input with pull-down after address is latched.
        //    The GT911 drives INT as an open-drain output to signal data-ready.
        //    We must NOT use output mode (fights GT911) or input_output_od
        //    (some GPIOs have alternate SPI functions, OD mode interferes with SPI display).
        //    Plain input mode lets GT911 freely drive the line.
        //    Pull-down keeps the line low when GT911 isn't driving it.
        //
        //    SAFETY: We reclaim the same GPIO pin after dropping the output driver.
        //    No other code uses this pin.
        drop(int_pin);
        let mut int_input = PinDriver::input(unsafe { AnyIOPin::new(int_pin_num) })?;
        int_input.set_pull(Pull::Down)?;
        core::mem::forget(int_input);

        // 5. Wait for GT911 to fully boot
        esp_idf_hal::delay::FreeRtos::delay_ms(50);

        // Configure I2C
        let i2c_config = I2cConfig::new().baudrate(I2C_FREQ.into());
        let mut i2c_driver = I2cDriver::new(i2c, sda, scl, &i2c_config)?;

        // Initialize GT911
        let driver = Gt911Blocking::new(GT911_ADDR);
        driver
            .init(&mut i2c_driver)
            .map_err(|e| anyhow::anyhow!("GT911 init failed: {:?}", e))?;

        log::info!(
            "GT911 touch controller initialized (addr: 0x{:02X})",
            GT911_ADDR
        );

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

    /// Poll touch input and dispatch events to the Slint window.
    ///
    /// Call this once per iteration of the main event loop.
    /// Maps GT911 touch coordinates to Slint `WindowEvent`s following the
    /// Pressed -> Moved -> Released pattern.
    pub fn poll(&mut self, window: &MinimalSoftwareWindow) {
        self.poll_count = self.poll_count.wrapping_add(1);

        // Log stats every ~500 polls (~8 seconds at 60fps)
        if self.poll_count % 500 == 0 {
            log::info!(
                "Touch stats: polls={}, not_ready={}, touches={}, errors={}",
                self.poll_count,
                self.not_ready_count,
                self.touch_count,
                self.error_count
            );
        }

        match self.driver.get_touch(&mut self.i2c) {
            Ok(Some(point)) => {
                self.touch_count = self.touch_count.wrapping_add(1);

                log::info!(
                    "TOUCH: raw x={}, y={}, area={} | display={}x{}",
                    point.x,
                    point.y,
                    point.area,
                    DISPLAY_WIDTH,
                    DISPLAY_HEIGHT
                );

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
                    log::info!(
                        "TOUCH: released at ({}, {})",
                        self.last_position.x,
                        self.last_position.y
                    );
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
