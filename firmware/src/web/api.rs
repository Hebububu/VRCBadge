use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;

use crate::platform::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::profile::{CurrentProfile, PendingProfile};

use super::SharedImageData;

/// Expected size of a raw RGB888 background image (480 * 320 * 3 bytes).
const BACKGROUND_IMAGE_SIZE: usize = DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize * 3;

/// Maximum body size for profile JSON (4 KB — plenty for a few short strings).
const MAX_PROFILE_BODY: usize = 4096;

/// Register API route handlers.
pub fn register(
    server: &mut EspHttpServer<'static>,
    pending_background: SharedImageData,
    current_profile: CurrentProfile,
    pending_profile: PendingProfile,
) -> anyhow::Result<()> {
    // Health check
    server.fn_handler("/api/health", Method::Get, |req| {
        req.into_ok_response()?.write_all(b"OK").map(|_| ())
    })?;

    // Get current profile as JSON
    server.fn_handler("/api/profile", Method::Get, move |req| {
        let json = match current_profile.lock() {
            Ok(profile) => serde_json::to_string(&*profile).unwrap_or_default(),
            Err(_) => "{}".into(),
        };
        let mut resp = req.into_response(
            200,
            Some("OK"),
            &[("Content-Type", "application/json; charset=utf-8")],
        )?;
        resp.write_all(json.as_bytes()).map(|_| ())
    })?;

    // Update profile from JSON
    server.fn_handler("/api/profile", Method::Post, move |mut req| {
        let content_len = req
            .header("Content-Length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        if content_len == 0 || content_len > MAX_PROFILE_BODY {
            let mut resp =
                req.into_response(400, Some("Bad Request"), &[("Content-Type", "text/plain")])?;
            resp.write_all(b"Invalid content length")?;
            return Ok(());
        }

        // Read body
        let mut buf = vec![0u8; content_len];
        let mut total_read = 0;
        while total_read < content_len {
            let n = req.read(&mut buf[total_read..])?;
            if n == 0 {
                break;
            }
            total_read += n;
        }

        // Parse JSON
        let profile = match serde_json::from_slice(&buf[..total_read]) {
            Ok(p) => p,
            Err(e) => {
                let mut resp =
                    req.into_response(400, Some("Bad Request"), &[("Content-Type", "text/plain")])?;
                let msg = format!("Invalid JSON: {e}");
                resp.write_all(msg.as_bytes())?;
                return Ok(());
            }
        };

        // Store for the main loop to pick up
        if let Ok(mut pending) = pending_profile.lock() {
            *pending = Some(profile);
        }

        log::info!("Profile updated via web");
        req.into_ok_response()?.write_all(b"OK").map(|_| ())
    })?;

    // Background image upload
    server.fn_handler("/api/background", Method::Post, move |mut req| {
        // Validate content length
        let content_len = req
            .header("Content-Length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        if content_len != BACKGROUND_IMAGE_SIZE {
            let mut resp =
                req.into_response(400, Some("Bad Request"), &[("Content-Type", "text/plain")])?;
            let msg = format!(
                "Expected {} bytes, got {}",
                BACKGROUND_IMAGE_SIZE, content_len
            );
            resp.write_all(msg.as_bytes()).map(|_| ())?;
            return Ok(());
        }

        // Read body into buffer
        let mut buf = vec![0u8; BACKGROUND_IMAGE_SIZE];
        let mut total_read = 0;
        while total_read < BACKGROUND_IMAGE_SIZE {
            let n = req.read(&mut buf[total_read..])?;
            if n == 0 {
                break;
            }
            total_read += n;
        }

        if total_read != BACKGROUND_IMAGE_SIZE {
            let mut resp =
                req.into_response(400, Some("Bad Request"), &[("Content-Type", "text/plain")])?;
            let msg = format!(
                "Incomplete body: got {} of {} bytes",
                total_read, BACKGROUND_IMAGE_SIZE
            );
            resp.write_all(msg.as_bytes()).map(|_| ())?;
            return Ok(());
        }

        // Store image data for the main loop to pick up
        if let Ok(mut pending) = pending_background.lock() {
            *pending = Some(buf);
        }

        log::info!(
            "Background image received ({} bytes)",
            BACKGROUND_IMAGE_SIZE
        );
        req.into_ok_response()?.write_all(b"OK").map(|_| ())
    })?;

    Ok(())
}
