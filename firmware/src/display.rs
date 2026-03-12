//! Display driver for the JC8048W550C — ST7262 RGB parallel panel.
//!
//! The ST7262 uses the ESP32-S3's built-in LCD_CAM peripheral in 16-bit RGB565
//! parallel mode. The hardware continuously DMA-refreshes the display from a
//! framebuffer in PSRAM at the pixel clock rate (~16MHz, ~39 FPS).
//!
//! Unlike SPI displays, there are no commands to send — we just write pixels
//! to the framebuffer and the hardware handles the rest.
//!
//! **Bounce buffers**: The framebuffer lives in PSRAM, but DMA reads from two
//! internal SRAM bounce buffers instead of PSRAM directly. An ISR copies data
//! from PSRAM to the bounce buffers, avoiding EDMA bandwidth contention with
//! the CPU, I2C, WiFi, and flash operations that share the PSRAM bus. Without
//! bounce buffers, the display drifts/shifts when any of these compete for
//! PSRAM bandwidth.
//!
//! The `esp_lcd_rgb_panel` API is not exposed by `esp-idf-sys` bindgen,
//! so we declare the FFI manually. The C implementation is linked via
//! `esp_lcd_panel_rgb.c.obj` in the ESP-IDF build.

use slint::platform::software_renderer::Rgb565Pixel;

use crate::platform::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

// ---------------------------------------------------------------------------
// Raw FFI declarations for esp_lcd_rgb_panel (esp_lcd_panel_rgb.h)
// ---------------------------------------------------------------------------

/// `lcd_clock_source_t` = `SOC_MOD_CLK_PLL_F160M` = 6 in ESP-IDF v5.3
/// (soc_module_clk_t enum starts at 1: CPU=1, RTC_FAST=2, RTC_SLOW=3, APB=4, PLL_F80M=5, PLL_F160M=6)
type LcdClockSourceT = i32;
const LCD_CLK_SRC_PLL160M: LcdClockSourceT = 6;

/// Mirror of `esp_lcd_rgb_timing_t` from esp_lcd_panel_rgb.h
#[repr(C)]
#[derive(Debug, Default)]
struct EspLcdRgbTimingT {
    pclk_hz: u32,
    h_res: u32,
    v_res: u32,
    hsync_pulse_width: u32,
    hsync_back_porch: u32,
    hsync_front_porch: u32,
    vsync_pulse_width: u32,
    vsync_back_porch: u32,
    vsync_front_porch: u32,
    /// Bitfield flags: hsync_idle_low:1, vsync_idle_low:1, de_idle_high:1,
    ///                  pclk_active_neg:1, pclk_idle_high:1
    flags: u32,
}

/// Mirror of `esp_lcd_rgb_panel_config_t` from esp_lcd_panel_rgb.h.
///
/// Uses `SOC_LCD_RGB_DATA_WIDTH = 16` for the data_gpio_nums array.
#[repr(C)]
struct EspLcdRgbPanelConfigT {
    clk_src: LcdClockSourceT,
    timings: EspLcdRgbTimingT,
    data_width: usize,
    bits_per_pixel: usize,
    num_fbs: usize,
    bounce_buffer_size_px: usize,
    sram_trans_align: usize, // deprecated but must be present for struct layout
    psram_trans_align: usize, // union with dma_burst_size
    hsync_gpio_num: i32,
    vsync_gpio_num: i32,
    de_gpio_num: i32,
    pclk_gpio_num: i32,
    disp_gpio_num: i32,
    data_gpio_nums: [i32; 16], // SOC_LCD_RGB_DATA_WIDTH = 16
    /// Bitfield flags: disp_active_low:1, refresh_on_demand:1, fb_in_psram:1,
    ///                  double_fb:1, no_fb:1, bb_invalidate_cache:1
    flags: u32,
}

extern "C" {
    fn esp_lcd_new_rgb_panel(
        rgb_panel_config: *const EspLcdRgbPanelConfigT,
        ret_panel: *mut esp_idf_sys::esp_lcd_panel_handle_t,
    ) -> esp_idf_sys::esp_err_t;

    fn esp_lcd_rgb_panel_get_frame_buffer(
        panel: esp_idf_sys::esp_lcd_panel_handle_t,
        fb_num: u32,
        fb0: *mut *mut core::ffi::c_void,
        ...
    ) -> esp_idf_sys::esp_err_t;
}

// ---------------------------------------------------------------------------
// Pin assignments (fixed on JC8048W550 board)
// ---------------------------------------------------------------------------

/// Data Enable
const PIN_DE: i32 = 40;
/// Vertical Sync
const PIN_VSYNC: i32 = 41;
/// Horizontal Sync
const PIN_HSYNC: i32 = 39;
/// Pixel Clock
const PIN_PCLK: i32 = 42;

/// 16-bit RGB565 data bus — order: B[0:4], G[0:5], R[0:4]
/// This matches the ESP-IDF `data_gpio_nums` array layout for little-endian RGB565.
const DATA_PINS: [i32; 16] = [
    8, 3, 46, 9, 1, // B0..B4
    5, 6, 7, 15, 16, 4, // G0..G5
    45, 48, 47, 21, 14, // R0..R4
];

// ---------------------------------------------------------------------------
// Timing parameters (from vendor Arduino demos)
// ---------------------------------------------------------------------------

/// Pixel clock frequency in Hz.
/// 16MHz gives ~39 FPS with the standard porch values.
const PIXEL_CLOCK_HZ: u32 = 16_000_000;

/// Horizontal/Vertical sync timing (symmetric 8/4/8 for this panel).
const H_FRONT_PORCH: u32 = 8;
const H_PULSE_WIDTH: u32 = 4;
const H_BACK_PORCH: u32 = 8;
const V_FRONT_PORCH: u32 = 8;
const V_PULSE_WIDTH: u32 = 4;
const V_BACK_PORCH: u32 = 8;

/// Total number of pixels in the framebuffer (800 x 480).
const FRAMEBUFFER_PIXELS: usize = DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize;

/// Opaque handle to the ESP-IDF RGB panel.
/// We keep this alive so the DMA refresh continues.
pub struct RgbDisplay {
    _panel: esp_idf_sys::esp_lcd_panel_handle_t,
}

/// Initialize the ST7262 RGB parallel display via ESP-IDF's lcd_rgb_panel driver.
///
/// Returns the display handle (must be kept alive) and a `'static` reference to the
/// framebuffer in PSRAM. The framebuffer is allocated by the ESP-IDF driver and is
/// continuously DMA-scanned to the display.
pub fn init() -> anyhow::Result<(RgbDisplay, &'static mut [Rgb565Pixel])> {
    // Build the timing flags bitfield (matches Arduino_GFX library config):
    // bit 0: hsync_idle_low = 1 (hsync polarity = 0 -> idle low)
    // bit 1: vsync_idle_low = 1 (vsync polarity = 0 -> idle low)
    // bit 2: de_idle_high = 0
    // bit 3: pclk_active_neg = 1 (data latched on falling edge)
    // bit 4: pclk_idle_high = 0
    let timing_flags: u32 = (1 << 0) | (1 << 1) | (1 << 3);

    let timings = EspLcdRgbTimingT {
        pclk_hz: PIXEL_CLOCK_HZ,
        h_res: DISPLAY_WIDTH,
        v_res: DISPLAY_HEIGHT,
        hsync_pulse_width: H_PULSE_WIDTH,
        hsync_back_porch: H_BACK_PORCH,
        hsync_front_porch: H_FRONT_PORCH,
        vsync_pulse_width: V_PULSE_WIDTH,
        vsync_back_porch: V_BACK_PORCH,
        vsync_front_porch: V_FRONT_PORCH,
        flags: timing_flags,
    };

    // Panel config flags bitfield:
    // bit 0: disp_active_low = 0
    // bit 1: refresh_on_demand = 0
    // bit 2: fb_in_psram = 1
    // bit 3: double_fb = 0
    // bit 4: no_fb = 0
    // bit 5: bb_invalidate_cache = 1 (free cache lines after bounce buffer copy)
    let panel_flags: u32 = (1 << 2) | (1 << 5); // fb_in_psram + bb_invalidate_cache

    // Bounce buffer: 10 scanlines = 8000 pixels.
    // Two internal SRAM buffers (16KB each, 32KB total) are allocated by the driver.
    // DMA reads from these instead of PSRAM directly, eliminating bandwidth
    // contention with CPU/I2C/WiFi/flash that causes pixel drifting.
    // 800*480 = 384,000 px / 8,000 = 48 (exact multiple, required by driver).
    const BOUNCE_BUFFER_LINES: usize = 10;
    let bounce_buffer_size_px = BOUNCE_BUFFER_LINES * DISPLAY_WIDTH as usize;

    let panel_config = EspLcdRgbPanelConfigT {
        clk_src: LCD_CLK_SRC_PLL160M,
        timings,
        data_width: 16,
        bits_per_pixel: 0, // 0 = same as data_width (16-bit RGB565)
        num_fbs: 1,
        bounce_buffer_size_px,
        sram_trans_align: 8,
        psram_trans_align: 64,
        hsync_gpio_num: PIN_HSYNC,
        vsync_gpio_num: PIN_VSYNC,
        de_gpio_num: PIN_DE,
        pclk_gpio_num: PIN_PCLK,
        disp_gpio_num: -1, // not used
        data_gpio_nums: DATA_PINS,
        flags: panel_flags,
    };

    // Create the panel
    let mut panel: esp_idf_sys::esp_lcd_panel_handle_t = core::ptr::null_mut();
    esp_idf_sys::esp!(unsafe { esp_lcd_new_rgb_panel(&panel_config, &mut panel) })
        .map_err(|e| anyhow::anyhow!("esp_lcd_new_rgb_panel failed: {e}"))?;

    // Reset and init via the standard panel API (these ARE in esp-idf-sys bindings)
    esp_idf_sys::esp!(unsafe { esp_idf_sys::esp_lcd_panel_reset(panel) })
        .map_err(|e| anyhow::anyhow!("esp_lcd_panel_reset failed: {e}"))?;

    esp_idf_sys::esp!(unsafe { esp_idf_sys::esp_lcd_panel_init(panel) })
        .map_err(|e| anyhow::anyhow!("esp_lcd_panel_init failed: {e}"))?;

    // Get the framebuffer pointer allocated by the driver
    let mut fb_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
    esp_idf_sys::esp!(unsafe { esp_lcd_rgb_panel_get_frame_buffer(panel, 1, &mut fb_ptr) })
        .map_err(|e| anyhow::anyhow!("esp_lcd_rgb_panel_get_frame_buffer failed: {e}"))?;

    assert!(!fb_ptr.is_null(), "RGB panel framebuffer pointer is null");

    // Wrap the framebuffer as a Rgb565Pixel slice.
    // The ESP-IDF RGB panel driver owns this memory and continuously DMA-reads it.
    let framebuffer =
        unsafe { core::slice::from_raw_parts_mut(fb_ptr as *mut Rgb565Pixel, FRAMEBUFFER_PIXELS) };

    log::info!(
        "ST7262 RGB display initialized ({}x{}, {}KB framebuffer in PSRAM)",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        FRAMEBUFFER_PIXELS * 2 / 1024,
    );

    Ok((RgbDisplay { _panel: panel }, framebuffer))
}

/// No-op flush — bounce buffers handle cache coherency.
///
/// With bounce buffers enabled, the ESP-IDF RGB panel driver copies data from
/// the PSRAM framebuffer to internal SRAM bounce buffers via an ISR, handling
/// all cache synchronization internally. We no longer need manual
/// `Cache_WriteBack_Addr` calls.
///
/// This function exists to keep the call site in main.rs unchanged, making it
/// easy to swap back to manual flushing if we ever disable bounce buffers.
pub fn flush_dirty_region(
    _framebuffer: &[Rgb565Pixel],
    _region: slint::platform::software_renderer::PhysicalRegion,
) {
    // Bounce buffer ISR handles PSRAM -> SRAM copy and cache sync automatically.
}
