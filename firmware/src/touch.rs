use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin, PinDriver};
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
}

impl<'a> TouchController<'a> {
    /// Initialize the GT911 touch controller.
    ///
    /// Sets up I2C bus and performs GT911 init sequence.
    /// The touch reset pin is toggled to ensure the GT911 is in a known state.
    pub fn new(i2c: I2C0, sda: AnyIOPin, scl: AnyIOPin, rst: AnyOutputPin) -> anyhow::Result<Self> {
        // Toggle touch reset pin: hold low 10ms, then release
        let mut rst_pin = PinDriver::output(rst)?;
        rst_pin.set_low()?;
        esp_idf_hal::delay::FreeRtos::delay_ms(10);
        rst_pin.set_high()?;
        esp_idf_hal::delay::FreeRtos::delay_ms(50); // GT911 needs ~50ms after reset

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
        })
    }

    /// Poll touch input and dispatch events to the Slint window.
    ///
    /// Call this once per iteration of the main event loop.
    /// Maps GT911 touch coordinates to Slint `WindowEvent`s following the
    /// Pressed -> Moved -> Released pattern.
    pub fn poll(&mut self, window: &MinimalSoftwareWindow) {
        match self.driver.get_touch(&mut self.i2c) {
            Ok(Some(point)) => {
                // Clamp coordinates to display bounds
                let x = (point.x as f32).min(DISPLAY_WIDTH as f32 - 1.0);
                let y = (point.y as f32).min(DISPLAY_HEIGHT as f32 - 1.0);
                let position = LogicalPosition::new(x, y);

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
                    // Send release at last known position (0,0 is fine — Slint
                    // uses the last PointerMoved position for release handling)
                    window.dispatch_event(WindowEvent::PointerReleased {
                        position: LogicalPosition::new(0.0, 0.0),
                        button: PointerEventButton::Left,
                    });
                    window.dispatch_event(WindowEvent::PointerExited);
                    self.state = TouchState::Released;
                }
            }
            Err(gt911::Error::NotReady) => {
                // No new data — poll again next iteration
            }
            Err(e) => {
                log::warn!("Touch read error: {:?}", e);
            }
        }
    }
}
