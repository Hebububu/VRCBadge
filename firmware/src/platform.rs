use std::rc::Rc;
use std::time::{Duration, Instant};

use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, PlatformError, WindowAdapter};
use slint::PhysicalSize;

/// Display dimensions (800 wide x 480 tall, native landscape).
pub const DISPLAY_WIDTH: u32 = 800;
pub const DISPLAY_HEIGHT: u32 = 480;

/// Custom Slint platform for ESP32-S3 with ST7262 RGB display.
///
/// Uses `MinimalSoftwareWindow` for full framebuffer rendering and
/// `std::time::Instant` for timekeeping (available in esp-idf std mode).
pub struct Esp32Platform {
    window: Rc<MinimalSoftwareWindow>,
    start: Instant,
}

impl Esp32Platform {
    pub fn new() -> Self {
        let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
        window.set_size(PhysicalSize::new(DISPLAY_WIDTH, DISPLAY_HEIGHT));

        Self {
            window,
            start: Instant::now(),
        }
    }

    /// Get a clone of the window handle (needed by main loop for event dispatch and rendering).
    pub fn window(&self) -> Rc<MinimalSoftwareWindow> {
        self.window.clone()
    }
}

impl Platform for Esp32Platform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        self.start.elapsed()
    }
}
