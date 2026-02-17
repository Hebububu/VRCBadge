//! Persistent storage: NVS for profile data, SPIFFS for images.
//!
//! SPIFFS is mounted at `/storage` via the raw ESP-IDF C API (with
//! `format_if_mount_failed: true` so the first boot auto-formats).
//! NVS uses the default `nvs` partition with namespace `"badge"`.

use std::ffi::CString;

use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use esp_idf_sys::{esp, esp_vfs_spiffs_conf_t, esp_vfs_spiffs_register};

use crate::platform::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::profile::Profile;

/// Avatar image dimensions (must match the Slint UI container: 150x150).
pub const AVATAR_WIDTH: u32 = 150;
pub const AVATAR_HEIGHT: u32 = 150;
pub const AVATAR_IMAGE_SIZE: usize = (AVATAR_WIDTH * AVATAR_HEIGHT * 3) as usize;

/// Background image dimensions (full display: 480x320).
pub const BACKGROUND_IMAGE_SIZE: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT * 3) as usize;

/// NVS namespace for badge settings (max 15 chars).
const NVS_NAMESPACE: &str = "badge";

/// NVS key for the JSON-serialized profile (max 15 chars).
const NVS_KEY_PROFILE: &str = "profile";

/// SPIFFS mount path.
const SPIFFS_MOUNT: &str = "/storage";

/// SPIFFS partition label (must match `partitions.csv`).
const SPIFFS_LABEL: &str = "storage";

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

/// Mount the SPIFFS `storage` partition at `/storage`.
///
/// On first boot the partition is unformatted, so we set
/// `format_if_mount_failed = true`.  Formatting a 12 MB partition takes
/// roughly 5-10 seconds — this only happens once.
pub fn init_spiffs() -> anyhow::Result<()> {
    let base_path = CString::new(SPIFFS_MOUNT)?;
    let label = CString::new(SPIFFS_LABEL)?;

    esp!(unsafe {
        esp_vfs_spiffs_register(&esp_vfs_spiffs_conf_t {
            base_path: base_path.as_ptr(),
            partition_label: label.as_ptr(),
            max_files: 5,
            format_if_mount_failed: true,
        })
    })
    .map_err(|e| anyhow::anyhow!("Failed to mount SPIFFS partition '{}': {e}", SPIFFS_LABEL))?;

    log::info!("SPIFFS mounted at {SPIFFS_MOUNT}");
    Ok(())
}

/// Open an NVS read-write handle for the `"badge"` namespace.
pub fn init_nvs(partition: EspDefaultNvsPartition) -> anyhow::Result<EspNvs<NvsDefault>> {
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, true)?;
    log::info!("NVS namespace '{NVS_NAMESPACE}' opened");
    Ok(nvs)
}

// ---------------------------------------------------------------------------
// Profile (NVS)
// ---------------------------------------------------------------------------

/// Load the saved profile from NVS, returning `None` on any error or if no
/// profile has been saved yet.
pub fn load_profile(nvs: &EspNvs<NvsDefault>) -> Option<Profile> {
    // Query the stored string length (includes NUL terminator).
    let len = match nvs.str_len(NVS_KEY_PROFILE) {
        Ok(Some(len)) => len,
        Ok(None) => return None, // key does not exist
        Err(e) => {
            log::warn!("NVS str_len('{NVS_KEY_PROFILE}') error: {e}");
            return None;
        }
    };

    let mut buf = vec![0u8; len];
    let json = match nvs.get_str(NVS_KEY_PROFILE, &mut buf) {
        Ok(Some(s)) => s,
        Ok(None) => return None,
        Err(e) => {
            log::warn!("NVS get_str('{NVS_KEY_PROFILE}') error: {e}");
            return None;
        }
    };

    match serde_json::from_str::<Profile>(json) {
        Ok(profile) => {
            log::info!("Loaded profile from NVS");
            Some(profile)
        }
        Err(e) => {
            log::warn!("Failed to parse stored profile JSON: {e}");
            None
        }
    }
}

/// Save the profile to NVS as a JSON string.
pub fn save_profile(nvs: &mut EspNvs<NvsDefault>, profile: &Profile) {
    let json = match serde_json::to_string(profile) {
        Ok(j) => j,
        Err(e) => {
            log::error!("Failed to serialize profile: {e}");
            return;
        }
    };

    if let Err(e) = nvs.set_str(NVS_KEY_PROFILE, &json) {
        log::error!("Failed to write profile to NVS: {e}");
    } else {
        log::info!("Profile saved to NVS ({} bytes)", json.len());
    }
}

// ---------------------------------------------------------------------------
// Images (SPIFFS)
// ---------------------------------------------------------------------------

/// Load a raw RGB888 image file from SPIFFS.
///
/// Returns `None` if the file does not exist, cannot be read, or has an
/// unexpected size.  `expected_size` is the exact byte count we expect
/// (e.g. `AVATAR_IMAGE_SIZE` or `BACKGROUND_IMAGE_SIZE`).
pub fn load_image(name: &str, expected_size: usize) -> Option<Vec<u8>> {
    let path = format!("{SPIFFS_MOUNT}/{name}.rgb");
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            // `NotFound` is expected on first boot — don't warn for it.
            if e.kind() != std::io::ErrorKind::NotFound {
                log::warn!("Failed to read {path}: {e}");
            }
            return None;
        }
    };

    if data.len() != expected_size {
        log::warn!(
            "{path}: unexpected size {} (expected {expected_size}), ignoring",
            data.len()
        );
        return None;
    }

    log::info!("Loaded {path} ({} KB)", data.len() / 1024);
    Some(data)
}

/// Save a raw RGB888 image to SPIFFS.  Logs errors but never panics.
pub fn save_image(name: &str, data: &[u8]) {
    let path = format!("{SPIFFS_MOUNT}/{name}.rgb");
    if let Err(e) = std::fs::write(&path, data) {
        log::error!("Failed to write {path}: {e}");
    } else {
        log::info!("Saved {path} ({} KB)", data.len() / 1024);
    }
}
