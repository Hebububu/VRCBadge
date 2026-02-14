mod api;
mod captive;
mod pages;

use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};

use esp_idf_svc::http::server::{Configuration, EspHttpServer};

/// Shared state for passing image data from the HTTP thread to the main loop.
pub type SharedImageData = Arc<Mutex<Option<Vec<u8>>>>;

/// Start the HTTP server and register all routes.
///
/// `ap_ip` is the AP's actual IP address, used for captive portal redirects.
/// Takes shared state for passing background image data to the main loop.
/// Returns the server handle — caller must hold it to keep the server alive.
pub fn init(
    ap_ip: Ipv4Addr,
    pending_background: SharedImageData,
) -> anyhow::Result<EspHttpServer<'static>> {
    let config = Configuration {
        http_port: 80,
        stack_size: 16384,
        max_uri_handlers: 16,
        uri_match_wildcard: true,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&config)?;
    let redirect_url = format!("http://{ap_ip}/");

    // Registration order matters: specific routes first, wildcard fallback last.
    pages::register(&mut server)?;
    api::register(&mut server, pending_background)?;
    captive::register(&mut server, &redirect_url)?;

    log::info!("HTTP server started on port 80");

    Ok(server)
}
