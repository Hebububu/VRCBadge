use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::http::Method;
use esp_idf_svc::io::{EspIOError, Write};

/// Register a GET handler that returns a 302 redirect to `url`.
fn register_redirect(
    server: &mut EspHttpServer<'static>,
    path: &str,
    url: String,
) -> anyhow::Result<()> {
    server.fn_handler(path, Method::Get, move |req| -> Result<(), EspIOError> {
        let mut resp = req.into_response(302, Some("Found"), &[("Location", &url)])?;
        resp.write_all(b"Redirecting...")?;
        Ok(())
    })?;
    Ok(())
}

/// Register captive portal detection endpoints and wildcard fallback.
///
/// Must be called LAST — the wildcard `/*` route should only match after
/// all specific routes have been registered.
pub fn register(server: &mut EspHttpServer<'static>, redirect_url: &str) -> anyhow::Result<()> {
    let captive_paths = [
        "/generate_204",        // Android / Chrome OS
        "/hotspot-detect.html", // Apple iOS / macOS
        "/connecttest.txt",     // Windows
        "/redirect",            // Windows secondary
        "/canonical.html",      // Firefox
        "/ncsi.txt",            // Windows NCSI
        "/success.txt",         // Various
    ];

    for path in captive_paths {
        register_redirect(server, path, redirect_url.to_string())?;
    }

    // Wildcard fallback — catches any unmatched GET request
    register_redirect(server, "/*", redirect_url.to_string())?;

    Ok(())
}
