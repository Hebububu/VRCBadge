use std::collections::VecDeque;
use std::ffi::CString;
use std::sync::Mutex;

/// Maximum number of log lines kept in the ring buffer.
const MAX_LINES: usize = 50;

static RING_BUFFER: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

/// Dual logger: writes to ESP-IDF serial console AND stores in a ring buffer
/// for display on the About page.
struct DualLogger;

impl log::Log for DualLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Forward to ESP-IDF serial console via esp_log_write
        let level = match record.level() {
            log::Level::Error => esp_idf_sys::esp_log_level_t_ESP_LOG_ERROR,
            log::Level::Warn => esp_idf_sys::esp_log_level_t_ESP_LOG_WARN,
            log::Level::Info => esp_idf_sys::esp_log_level_t_ESP_LOG_INFO,
            log::Level::Debug => esp_idf_sys::esp_log_level_t_ESP_LOG_DEBUG,
            log::Level::Trace => esp_idf_sys::esp_log_level_t_ESP_LOG_VERBOSE,
        };

        let target = record.target();
        let message = format!("{}\n", record.args());
        if let (Ok(tag), Ok(msg)) = (CString::new(target), CString::new(message)) {
            unsafe {
                esp_idf_sys::esp_log_write(level, tag.as_ptr(), msg.as_ptr());
            }
        }

        // Store in ring buffer for About page display
        let level_char = match record.level() {
            log::Level::Error => 'E',
            log::Level::Warn => 'W',
            log::Level::Info => 'I',
            log::Level::Debug => 'D',
            log::Level::Trace => 'T',
        };

        // Strip the crate prefix for cleaner display
        let short_target = target
            .strip_prefix("vrcbadge_firmware::")
            .or_else(|| target.strip_prefix("vrcbadge_firmware"))
            .unwrap_or(target);

        let line = if short_target.is_empty() {
            format!("{} {}", level_char, record.args())
        } else {
            format!("{} [{}] {}", level_char, short_target, record.args())
        };

        if let Ok(mut buf) = RING_BUFFER.lock() {
            if buf.len() >= MAX_LINES {
                buf.pop_front();
            }
            buf.push_back(line);
        }
    }

    fn flush(&self) {}
}

static DUAL_LOGGER: DualLogger = DualLogger;

/// Initialize the dual logger (serial + ring buffer).
/// Must be called once at startup, before any log macros.
pub fn init() {
    // Set our dual logger as the global logger
    log::set_logger(&DUAL_LOGGER).ok();
    log::set_max_level(log::LevelFilter::Info);
}

/// Return all buffered log lines as a single newline-joined string (snapshot).
pub fn snapshot() -> String {
    match RING_BUFFER.lock() {
        Ok(buf) => {
            let mut result = String::new();
            for (i, line) in buf.iter().enumerate() {
                if i > 0 {
                    result.push('\n');
                }
                result.push_str(line);
            }
            result
        }
        Err(_) => String::from("(log buffer locked)"),
    }
}
