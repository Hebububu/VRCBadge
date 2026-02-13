use embedded_graphics_core::pixelcolor::raw::RawU16;
use embedded_graphics_core::pixelcolor::Rgb565;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin, PinDriver};
use esp_idf_hal::spi::config::{Config as SpiConfig, DriverConfig};
use esp_idf_hal::spi::{Dma, SpiDeviceDriver, SpiDriver, SPI2};
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7796;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{Builder, Display};
use slint::platform::software_renderer::{PhysicalRegion, Rgb565Pixel};

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

/// SPI/DMA buffer size for the mipidsi SPI interface.
/// With full-framebuffer rendering, each dirty rect can be thousands of bytes.
/// DMA excels at large bulk transfers — the setup/teardown overhead is amortized.
const SPI_BUFFER_SIZE: usize = 4096;

/// Total number of pixels in the framebuffer (480 × 320).
const FRAMEBUFFER_PIXELS: usize = DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize;

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
    // Configure SPI bus — 80MHz with write_only mode (no dummy bytes).
    // write_only sets SPI_DEVICE_NO_DUMMY, removing the dummy clock cycle normally
    // inserted for full-duplex reads, which unlocks 80MHz on the ESP32-S3.
    // ST7796S datasheet specifies 66MHz write max, but 80MHz works in practice.
    // If display shows glitches, reduce to 40_000_000.
    let spi_config = SpiConfig::new()
        .baudrate(80_000_000.into())
        .write_only(true);

    // Enable DMA for large bulk transfers. With full-framebuffer rendering,
    // each dirty rect sends thousands of bytes in a single set_pixels call.
    // DMA overhead is amortized over the large transfer size.
    let driver_config = DriverConfig::new().dma(Dma::Auto(SPI_BUFFER_SIZE));

    let spi_driver = SpiDriver::new(spi, sclk, mosi, None::<AnyIOPin>, &driver_config)?;

    let spi_device = SpiDeviceDriver::new(spi_driver, Some(cs), &spi_config)?;

    // D/C pin (data/command select)
    let dc_pin = PinDriver::output(dc)?;

    // Reset pin
    let rst_pin = PinDriver::output(rst)?;

    // Allocate SPI interface buffer in DMA-capable internal SRAM.
    // mipidsi copies pixels from the framebuffer (PSRAM) into this buffer (internal SRAM),
    // then DMA sends from internal SRAM to the SPI peripheral — no PSRAM bus contention
    // during the actual SPI transfer.
    let buffer: &'static mut [u8] = unsafe {
        let ptr = esp_idf_sys::heap_caps_malloc(
            SPI_BUFFER_SIZE,
            esp_idf_sys::MALLOC_CAP_DMA | esp_idf_sys::MALLOC_CAP_INTERNAL,
        ) as *mut u8;
        assert!(
            !ptr.is_null(),
            "Failed to allocate DMA buffer in internal SRAM"
        );
        core::ptr::write_bytes(ptr, 0, SPI_BUFFER_SIZE);
        core::slice::from_raw_parts_mut(ptr, SPI_BUFFER_SIZE)
    };

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

/// Allocate a full 480×320 RGB565 framebuffer in PSRAM.
///
/// Returns a leaked `'static` slice of `Rgb565Pixel` (307,200 bytes).
/// Uses `heap_caps_malloc` with `MALLOC_CAP_SPIRAM` to guarantee PSRAM placement,
/// preserving internal SRAM for stack, DMA buffers, and WiFi.
pub fn allocate_framebuffer() -> &'static mut [Rgb565Pixel] {
    let size_bytes = FRAMEBUFFER_PIXELS * core::mem::size_of::<Rgb565Pixel>();
    unsafe {
        let ptr = esp_idf_sys::heap_caps_malloc(size_bytes, esp_idf_sys::MALLOC_CAP_SPIRAM)
            as *mut Rgb565Pixel;
        assert!(
            !ptr.is_null(),
            "Failed to allocate {}KB framebuffer in PSRAM",
            size_bytes / 1024
        );
        core::ptr::write_bytes(ptr, 0, FRAMEBUFFER_PIXELS);
        log::info!(
            "Framebuffer allocated: {}KB in PSRAM ({} pixels)",
            size_bytes / 1024,
            FRAMEBUFFER_PIXELS
        );
        core::slice::from_raw_parts_mut(ptr, FRAMEBUFFER_PIXELS)
    }
}

/// Send only the dirty regions from the framebuffer to the display.
///
/// `PhysicalRegion` contains up to ~3-7 non-overlapping rectangles from Slint's
/// dirty tracking. For each rectangle, we send a single `set_pixels` call
/// (1× CASET + 1× RASET + 1× RAMWR + pixel data stream), which is far more
/// efficient than the 2,240 transactions needed for line-by-line rendering.
pub fn send_dirty_region(
    display: &mut BadgeDisplay,
    framebuffer: &[Rgb565Pixel],
    region: PhysicalRegion,
) {
    let stride = DISPLAY_WIDTH as usize;

    for (pos, size) in region.iter() {
        let x = pos.x as usize;
        let y = pos.y as usize;
        let w = size.width as usize;
        let h = size.height as usize;

        if w == 0 || h == 0 {
            continue;
        }

        // Extract pixels from the framebuffer in row-major order.
        // This is zero-copy — the iterator lazily reads from PSRAM and mipidsi
        // batches pixels into the 4KB internal SRAM SPI buffer before DMA-sending.
        let pixels = (y..(y + h)).flat_map(move |row| {
            framebuffer[row * stride + x..row * stride + x + w]
                .iter()
                .map(|p| Rgb565::from(RawU16::new(p.0)))
        });

        if let Err(e) = display.set_pixels(
            x as u16,
            y as u16,
            (x + w - 1) as u16,
            (y + h - 1) as u16,
            pixels,
        ) {
            log::error!(
                "Display SPI write failed for rect ({},{} {}x{}): {:?}",
                x,
                y,
                w,
                h,
                e
            );
        }
    }
}
