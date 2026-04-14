//! Pending updates from the HTTP server, drained by the main loop.
//!
//! The web handlers run on the HTTP server thread and write incoming
//! profile/image uploads into shared `Mutex<Option<T>>` slots. The main
//! loop polls those slots once per ~2-second tick, applies the update to
//! the Slint UI, and persists it to NVS / SPIFFS.
//!
//! Three slots live here (profile, avatar, background) plus the always-live
//! `current_profile` snapshot used by `GET /api/profile`.

use std::cell::RefCell;
use std::rc::Rc;

use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use slint::{Image, Rgb8Pixel, SharedPixelBuffer};

use crate::ui_helpers::{apply_profile_colors, apply_rounded_corners};
use crate::{platform, profile, storage, web, BadgeUI};

/// All shared slots used to hand updates from the HTTP server to the main loop.
///
/// Cheaply cloneable: each field is an `Arc`. `web::init` clones the inner
/// fields it needs.
#[derive(Clone)]
pub struct PendingState {
    pub avatar: web::SharedImageData,
    pub background: web::SharedImageData,
    pub profile: profile::PendingProfile,
    /// Always-live snapshot of the currently displayed profile. Used by the
    /// HTTP server to serve `GET /api/profile`.
    pub current_profile: profile::CurrentProfile,
}

impl PendingState {
    pub fn new(initial_profile: profile::Profile) -> Self {
        Self {
            avatar: std::sync::Arc::new(std::sync::Mutex::new(None)),
            background: std::sync::Arc::new(std::sync::Mutex::new(None)),
            profile: std::sync::Arc::new(std::sync::Mutex::new(None)),
            current_profile: std::sync::Arc::new(std::sync::Mutex::new(initial_profile)),
        }
    }

    /// Drain any pending profile / avatar / background updates and apply
    /// them to the UI + persistent storage. Must be called from the main
    /// thread (NVS is `!Send`).
    pub fn poll_into_ui(&self, ui: &BadgeUI, nvs: &Rc<RefCell<EspNvs<NvsDefault>>>) {
        // Profile update
        if let Ok(mut pending) = self.profile.try_lock() {
            if let Some(new_profile) = pending.take() {
                ui.set_display_name(new_profile.display_name.clone().into());
                ui.set_tagline(new_profile.tagline.clone().into());
                ui.set_twitter_handle(new_profile.twitter_handle.clone().into());
                ui.set_discord_handle(new_profile.discord_handle.clone().into());
                apply_profile_colors(ui, &new_profile);
                // Update current profile snapshot for future GET /api/profile
                if let Ok(mut current) = self.current_profile.try_lock() {
                    *current = new_profile.clone();
                }
                storage::save_profile(&mut nvs.borrow_mut(), &new_profile);
                log::info!("Badge profile updated");
            }
        }

        // Avatar image upload
        if let Ok(mut pending) = self.avatar.try_lock() {
            if let Some(mut rgb_data) = pending.take() {
                // Save raw image first (without rounded corners) so reload
                // works if border-radius changes in the future.
                storage::save_image("avatar", &rgb_data);
                apply_rounded_corners(
                    &mut rgb_data,
                    storage::AVATAR_WIDTH,
                    storage::AVATAR_HEIGHT,
                    20,
                    [0x2a, 0x2a, 0x4a],
                );
                let buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                    &rgb_data,
                    storage::AVATAR_WIDTH,
                    storage::AVATAR_HEIGHT,
                );
                ui.set_avatar_image(Image::from_rgb8(buffer));
                log::info!("Avatar image updated");
            }
        }

        // Background image upload. An empty vec signals "clear".
        if let Ok(mut pending) = self.background.try_lock() {
            if let Some(rgb_data) = pending.take() {
                if rgb_data.is_empty() {
                    ui.set_background_image(Image::default());
                    storage::delete_image("background");
                    log::info!("Background image cleared");
                } else {
                    let buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                        &rgb_data,
                        platform::DISPLAY_WIDTH,
                        platform::DISPLAY_HEIGHT,
                    );
                    ui.set_background_image(Image::from_rgb8(buffer));
                    storage::save_image("background", &rgb_data);
                    log::info!("Background image updated");
                }
            }
        }
    }
}
