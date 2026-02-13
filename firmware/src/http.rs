use std::net::Ipv4Addr;

use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::{EspIOError, Write};

/// HTML page served at GET /
const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>VRCBadge</title>
<style>
body{background:#1a1a2e;color:#e0e0e0;font-family:sans-serif;margin:0;padding:20px;display:flex;flex-direction:column;align-items:center}
h1{color:#fff;margin-bottom:4px}
p.sub{color:#888;font-size:14px;margin-top:0}
</style>
</head>
<body>
<h1>VRCBadge</h1>
<p class="sub">Connected!</p>
</body>
</html>"#;

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

/// Start the HTTP server and register API routes.
///
/// `ap_ip` is the AP's actual IP address, used for captive portal redirects.
/// Returns the server handle — caller must hold it to keep the server alive.
pub fn init(ap_ip: Ipv4Addr) -> anyhow::Result<EspHttpServer<'static>> {
    let config = Configuration {
        http_port: 80,
        stack_size: 16384,
        max_uri_handlers: 16,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&config)?;

    let redirect_url = format!("http://{ap_ip}/");

    // Main page
    server.fn_handler("/", Method::Get, |req| {
        let mut resp = req.into_response(
            200,
            Some("OK"),
            &[("Content-Type", "text/html; charset=utf-8")],
        )?;
        resp.write_all(INDEX_HTML.as_bytes()).map(|_| ())
    })?;

    // Captive portal detection endpoints
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
        register_redirect(&mut server, path, redirect_url.clone())?;
    }

    // Health check
    server.fn_handler("/api/health", Method::Get, |req| {
        req.into_ok_response()?.write_all(b"OK").map(|_| ())
    })?;

    // Wildcard fallback — registered last so specific routes take priority
    register_redirect(&mut server, "/*", redirect_url)?;

    log::info!("HTTP server started on port 80");

    Ok(server)
}
