use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;

/// Start the HTTP server and register API routes.
///
/// Returns the server handle — caller must hold it to keep the server alive.
pub fn init() -> anyhow::Result<EspHttpServer<'static>> {
    let config = Configuration {
        http_port: 80,
        stack_size: 10240,
        max_uri_handlers: 8,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&config)?;

    // Health check
    server.fn_handler("/api/health", Method::Get, |req| {
        req.into_ok_response()?.write_all(b"OK").map(|_| ())
    })?;

    log::info!("HTTP server started on port 80");

    Ok(server)
}
