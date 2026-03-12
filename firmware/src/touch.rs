use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin, PinDriver};
use esp_idf_hal::i2c::config::Config as I2cConfig;
use esp_idf_hal::i2c::{I2cDriver, I2C0};
use gt911::Gt911Blocking;
use slint::platform::software_renderer::MinimalSoftwareWindow;
use slint::platform::{PointerEventButton, WindowEvent};
use slint::LogicalPosition;

use crate::platform::DISPLAY_HEIGHT;
use crate::platform::DISPLAY_WIDTH;

/// GT911 default I2C address on the JC8048W550 board.
const GT911_ADDR: u8 = 0x5D;

/// I2C bus speed — use 100kHz (standard mode) for reliability with internal pull-ups.
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
    /// On the JC8048W550 board:
    /// - SDA = GPIO 19, SCL = GPIO 20
    /// - RST = GPIO 38 (active-low reset)
    /// - INT = not connected (polled mode)
    ///
    /// The INT pin is unused on this board, so we skip the address-selection
    /// sequence and always use address 0x5D.
    pub fn new(i2c: I2C0, sda: AnyIOPin, scl: AnyIOPin, rst: AnyOutputPin) -> anyhow::Result<Self> {
        // Reset the GT911: pull RST low for 20ms, then release high.
        let mut rst_pin = PinDriver::output(rst)?;
        rst_pin.set_low()?;
        esp_idf_hal::delay::FreeRtos::delay_ms(20);
        rst_pin.set_high()?;

        // Keep RST pin driven HIGH — GT911 needs RST held high for normal operation.
        core::mem::forget(rst_pin);

        // Wait for GT911 to boot
        esp_idf_hal::delay::FreeRtos::delay_ms(100);

        // Configure I2C
        let i2c_config = I2cConfig::new().baudrate(I2C_FREQ.into());
        let mut i2c_driver = I2cDriver::new(i2c, sda, scl, &i2c_config)?;

        // Initialize GT911 at default address 0x5D
        let driver = Gt911Blocking::new(GT911_ADDR);
        driver
            .init(&mut i2c_driver)
            .map_err(|e| anyhow::anyhow!("GT911 init failed at 0x{:02X}: {:?}", GT911_ADDR, e))?;

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
    /// On the JC8048W550, the GT911 reports coordinates directly in the display's
    /// native 800x480 orientation (0,0 = top-left, no inversion needed).
    pub fn poll(&mut self, window: &MinimalSoftwareWindow) {
        self.poll_count = self.poll_count.wrapping_add(1);

        // Periodic debug stats every ~5 seconds (5000ms / 16ms = 312 polls)
        if self.poll_count % 312 == 0 {
            log::info!(
                "Touch stats: polls={}, touches={}, not_ready={}, errors={}",
                self.poll_count,
                self.touch_count,
                self.not_ready_count,
                self.error_count,
            );
        }

        match self.driver.get_touch(&mut self.i2c) {
            Ok(Some(point)) => {
                self.touch_count = self.touch_count.wrapping_add(1);

                // GT911 on the JC8048W550 reports coordinates directly in
                // display orientation (0,0 = top-left, 799,479 = bottom-right).
                // No inversion needed.
                let x = (point.x as f32).clamp(0.0, DISPLAY_WIDTH as f32 - 1.0);
                let y = (point.y as f32).clamp(0.0, DISPLAY_HEIGHT as f32 - 1.0);
                let position = LogicalPosition::new(x, y);
                self.last_position = position;

                // Log first 20 touches for debugging
                if self.touch_count <= 20 {
                    log::info!("Touch #{}: ({:.0}, {:.0})", self.touch_count, x, y,);
                }

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
            }
            Err(e) => {
                self.error_count = self.error_count.wrapping_add(1);
                log::warn!("Touch read error #{}: {:?}", self.error_count, e);
            }
        }
    }
}
