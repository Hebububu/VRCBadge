use std::ops::Range;

use embedded_graphics_core::pixelcolor::raw::RawU16;
use embedded_graphics_core::pixelcolor::Rgb565;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin, PinDriver};
use esp_idf_hal::spi::config::Config as SpiConfig;
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver, SPI2};
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7796;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{Builder, Display};
use slint::platform::software_renderer::{LineBufferProvider, Rgb565Pixel};

use crate::platform::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

/// Type alias for our concrete display.
///
/// Generic parameters:
///   DI = SpiInterface<'static, SpiDeviceDriver, PinDriver (DC)>
///   MODEL = ST7796
///   RST = PinDriver (RST)
pub type BadgeDisplay = Display<
    SpiInterface<
        'static,
        SpiDeviceDriver<'static, SpiDriver<'static>>,
        PinDriver<'static, AnyOutputPin, esp_idf_hal::gpio::Output>,
    >,
    ST7796,
    PinDriver<'static, AnyOutputPin, esp_idf_hal::gpio::Output>,
>;

/// SPI transfer buffer — leaked to get 'static lifetime for the display interface.
const SPI_BUFFER_SIZE: usize = 512;

/// Initialize the ST7796S display over SPI.
///
/// Returns the initialized display in landscape orientation (480x320).
pub fn init(
    spi: SPI2,
    sclk: AnyOutputPin,
    mosi: AnyOutputPin,
    cs: AnyOutputPin,
    dc: AnyOutputPin,
    rst: AnyOutputPin,
) -> anyhow::Result<BadgeDisplay> {
    // Configure SPI bus — ST7796S supports up to 80MHz, start at 40MHz for reliability
    let spi_config = SpiConfig::new().baudrate(40_000_000.into());

    // MISO is not used (display is write-only), pass None with AnyIOPin type
    let spi_driver = SpiDriver::new(spi, sclk, mosi, None::<AnyIOPin>, &Default::default())?;

    let spi_device = SpiDeviceDriver::new(spi_driver, Some(cs), &spi_config)?;

    // D/C pin (data/command select)
    let dc_pin = PinDriver::output(dc)?;

    // Reset pin
    let rst_pin = PinDriver::output(rst)?;

    // Leak a buffer to get 'static lifetime — needed because Display holds a reference to it
    let buffer: &'static mut [u8] = Box::leak(Box::new([0u8; SPI_BUFFER_SIZE]));

    // Build display interface
    let di = SpiInterface::new(spi_device, dc_pin, buffer);

    // Initialize display
    let display = Builder::new(ST7796, di)
        .reset_pin(rst_pin)
        .color_order(ColorOrder::Bgr)
        .invert_colors(ColorInversion::Inverted)
        .display_size(320, 480) // Native portrait resolution
        .orientation(Orientation::new().rotate(Rotation::Deg90).flip_vertical()) // Landscape, un-mirror
        .init(&mut FreeRtos)
        .map_err(|e| anyhow::anyhow!("Display init failed: {:?}", e))?;

    log::info!(
        "ST7796S display initialized ({}x{} landscape)",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT
    );

    Ok(display)
}

/// Line buffer provider that renders each Slint line directly to the ST7796S via SPI.
///
/// Implements `LineBufferProvider` for Slint's `render_by_line()` strategy.
/// Each line uses ~960 bytes (480 pixels x 2 bytes RGB565).
pub struct DisplayLineBuffer<'a> {
    display: &'a mut BadgeDisplay,
    line_buffer: [Rgb565Pixel; DISPLAY_WIDTH as usize],
}

impl<'a> DisplayLineBuffer<'a> {
    pub fn new(display: &'a mut BadgeDisplay) -> Self {
        Self {
            display,
            line_buffer: [Rgb565Pixel(0); DISPLAY_WIDTH as usize],
        }
    }
}

impl LineBufferProvider for DisplayLineBuffer<'_> {
    type TargetPixel = Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: Range<usize>,
        render_fn: impl FnOnce(&mut [Rgb565Pixel]),
    ) {
        // Let Slint render into our buffer
        render_fn(&mut self.line_buffer[range.clone()]);

        // Convert Rgb565Pixel to mipidsi-compatible colors and send to display
        let pixels = self.line_buffer[range.clone()]
            .iter()
            .map(|p| Rgb565::from(RawU16::new(p.0)));

        // set_pixels takes inclusive start/end coordinates
        let _ = self.display.set_pixels(
            range.start as u16,
            line as u16,
            range.end.saturating_sub(1) as u16,
            line as u16,
            pixels,
        );
    }
}
