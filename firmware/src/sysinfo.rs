use std::time::Instant;

/// Returns free internal heap memory in KB.
pub fn free_heap_kb() -> u32 {
    let bytes = unsafe { esp_idf_sys::esp_get_free_heap_size() };
    bytes / 1024
}

/// Returns free PSRAM (SPI RAM) in KB.
pub fn free_psram_kb() -> u32 {
    let bytes =
        unsafe { esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_SPIRAM as u32) };
    (bytes / 1024) as u32
}

/// Format uptime as "Xh Ym Zs" from a start instant.
pub fn uptime_string(start: &Instant) -> String {
    let secs = start.elapsed().as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}

/// Returns the firmware version from Cargo.toml.
pub fn firmware_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
