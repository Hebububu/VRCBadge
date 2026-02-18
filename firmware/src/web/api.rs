use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;

use crate::profile::{CurrentProfile, PendingProfile};
use crate::storage;

use super::SharedImageData;

/// Maximum body size for profile JSON (4 KB — plenty for a few short strings).
const MAX_PROFILE_BODY: usize = 4096;

/// Register API route handlers.
pub fn register(
    server: &mut EspHttpServer<'static>,
    pending_background: SharedImageData,
    pending_avatar: SharedImageData,
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

    // Avatar image upload (150x150 raw RGB888)
    server.fn_handler("/api/avatar", Method::Post, move |mut req| {
        let expected = storage::AVATAR_IMAGE_SIZE;
        let content_len = req
            .header("Content-Length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        if content_len != expected {
            let mut resp =
                req.into_response(400, Some("Bad Request"), &[("Content-Type", "text/plain")])?;
            let msg = format!("Expected {expected} bytes, got {content_len}");
            resp.write_all(msg.as_bytes()).map(|_| ())?;
            return Ok(());
        }

        let mut buf = vec![0u8; expected];
        let mut total_read = 0;
        while total_read < expected {
            let n = req.read(&mut buf[total_read..])?;
            if n == 0 {
                break;
            }
            total_read += n;
        }

        if total_read != expected {
            let mut resp =
                req.into_response(400, Some("Bad Request"), &[("Content-Type", "text/plain")])?;
            let msg = format!("Incomplete body: got {total_read} of {expected} bytes");
            resp.write_all(msg.as_bytes()).map(|_| ())?;
            return Ok(());
        }

        if let Ok(mut pending) = pending_avatar.lock() {
            *pending = Some(buf);
        }

        log::info!("Avatar image received ({expected} bytes)");
        req.into_ok_response()?.write_all(b"OK").map(|_| ())
    })?;

    // Clear background image (revert to solid color)
    let pending_bg_delete = pending_background.clone();
    server.fn_handler("/api/background", Method::Delete, move |req| {
        // Signal the main loop to clear the background by sending an empty vec.
        if let Ok(mut pending) = pending_bg_delete.lock() {
            *pending = Some(Vec::new());
        }
        log::info!("Background image clear requested");
        req.into_ok_response()?.write_all(b"OK").map(|_| ())
    })?;

    // Background image upload (480x320 raw RGB888)
    server.fn_handler("/api/background", Method::Post, move |mut req| {
        let expected = storage::BACKGROUND_IMAGE_SIZE;
        let content_len = req
            .header("Content-Length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        if content_len != expected {
            let mut resp =
                req.into_response(400, Some("Bad Request"), &[("Content-Type", "text/plain")])?;
            let msg = format!("Expected {expected} bytes, got {content_len}");
            resp.write_all(msg.as_bytes()).map(|_| ())?;
            return Ok(());
        }

        let mut buf = vec![0u8; expected];
        let mut total_read = 0;
        while total_read < expected {
            let n = req.read(&mut buf[total_read..])?;
            if n == 0 {
                break;
            }
            total_read += n;
        }

        if total_read != expected {
            let mut resp =
                req.into_response(400, Some("Bad Request"), &[("Content-Type", "text/plain")])?;
            let msg = format!("Incomplete body: got {total_read} of {expected} bytes");
            resp.write_all(msg.as_bytes()).map(|_| ())?;
            return Ok(());
        }

        if let Ok(mut pending) = pending_background.lock() {
            *pending = Some(buf);
        }

        log::info!("Background image received ({expected} bytes)");
        req.into_ok_response()?.write_all(b"OK").map(|_| ())
    })?;

    Ok(())
}
