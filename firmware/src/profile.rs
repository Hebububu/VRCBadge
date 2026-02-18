use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

/// Badge profile data — editable via the web interface.
///
/// Colors are stored as CSS hex strings (e.g. `"#1a1a2e"`).
/// Missing/invalid colors fall back to defaults in the UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub display_name: String,
    pub tagline: String,
    pub twitter_handle: String,
    pub discord_handle: String,
    /// Background solid color (CSS hex, e.g. "#1a1a2e").
    /// Used when no background image is uploaded.
    #[serde(default = "default_background_color")]
    pub background_color: String,
    /// Tagline text color (CSS hex, e.g. "#e0e8f0").
    #[serde(default = "default_tagline_color")]
    pub tagline_color: String,
    /// Tagline bar background color (CSS hex, e.g. "#1b4f72").
    #[serde(default = "default_tagline_background_color")]
    pub tagline_background_color: String,
}

fn default_background_color() -> String {
    "#1a1a2e".into()
}

fn default_tagline_color() -> String {
    "#e0e8f0".into()
}

fn default_tagline_background_color() -> String {
    "#1b4f72".into()
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            display_name: "Hebu".into(),
            tagline: "Hello from VRCBadge!".into(),
            twitter_handle: "@Hebu_VRC".into(),
            discord_handle: "hebu".into(),
            background_color: default_background_color(),
            tagline_color: default_tagline_color(),
            tagline_background_color: default_tagline_background_color(),
        }
    }
}

/// Parse a CSS hex color string (e.g. "#1a1a2e") into an `slint::Color`.
/// Returns `None` if the string is malformed.
pub fn parse_hex_color(hex: &str) -> Option<slint::Color> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(slint::Color::from_rgb_u8(r, g, b))
}

/// Shared current profile (read by GET /api/profile).
pub type CurrentProfile = Arc<Mutex<Profile>>;

/// Shared pending profile update (written by POST /api/profile, consumed by main loop).
pub type PendingProfile = Arc<Mutex<Option<Profile>>>;
