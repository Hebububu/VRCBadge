//! Pure UI helpers and pixel manipulation routines used by the main loop.

use crate::profile;
use crate::BadgeUI;

/// Paint pixels outside a rounded rectangle with a background color.
///
/// Slint's software renderer does not clip children to `border-radius`
/// (upstream issue #4176), so we bake the rounded corners directly into
/// the RGB888 pixel buffer before handing it to Slint.
///
/// The radius is specified in pixels and applies to all four corners.
pub fn apply_rounded_corners(data: &mut [u8], w: u32, h: u32, radius: u32, bg: [u8; 3]) {
    let r = radius.min(w / 2).min(h / 2);
    let r_sq = (r * r) as i64;

    for y in 0..h {
        for x in 0..w {
            // Determine which corner (if any) this pixel is near
            let (dx, dy) = if x < r && y < r {
                // Top-left corner
                (r - 1 - x, r - 1 - y)
            } else if x >= w - r && y < r {
                // Top-right corner
                (x - (w - r), r - 1 - y)
            } else if x < r && y >= h - r {
                // Bottom-left corner
                (r - 1 - x, y - (h - r))
            } else if x >= w - r && y >= h - r {
                // Bottom-right corner
                (x - (w - r), y - (h - r))
            } else {
                continue; // Not in a corner region
            };

            let dist_sq = (dx as i64) * (dx as i64) + (dy as i64) * (dy as i64);
            if dist_sq > r_sq {
                let idx = ((y * w + x) * 3) as usize;
                data[idx] = bg[0];
                data[idx + 1] = bg[1];
                data[idx + 2] = bg[2];
            }
        }
    }
}

/// Apply color properties from a profile to the Slint UI.
pub fn apply_profile_colors(ui: &BadgeUI, p: &profile::Profile) {
    if let Some(c) = profile::parse_hex_color(&p.background_color) {
        ui.set_badge_background_color(c);
    }
    if let Some(c) = profile::parse_hex_color(&p.tagline_color) {
        ui.set_tagline_color(c);
    }
    if let Some(c) = profile::parse_hex_color(&p.tagline_background_color) {
        ui.set_tagline_background_color(c);
    }
}
