use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;

use crate::platform::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

use super::SharedImageData;

/// Expected size of a raw RGB888 background image (480 * 320 * 3 bytes).
const BACKGROUND_IMAGE_SIZE: usize = DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize * 3;

/// Register API route handlers.
pub fn register(
    server: &mut EspHttpServer<'static>,
    pending_background: SharedImageData,
) -> anyhow::Result<()> {
    // Health check
    server.fn_handler("/api/health", Method::Get, |req| {
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
