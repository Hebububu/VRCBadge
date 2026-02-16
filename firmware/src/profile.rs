use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

/// Badge profile data — editable via the web interface.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub display_name: String,
    pub tagline: String,
    pub twitter_handle: String,
    pub discord_handle: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            display_name: "Hebu".into(),
            tagline: "Hello from VRCBadge!".into(),
            twitter_handle: "@Hebu_VRC".into(),
            discord_handle: "hebu".into(),
        }
    }
}

/// Shared current profile (read by GET /api/profile).
pub type CurrentProfile = Arc<Mutex<Profile>>;

/// Shared pending profile update (written by POST /api/profile, consumed by main loop).
pub type PendingProfile = Arc<Mutex<Option<Profile>>>;
