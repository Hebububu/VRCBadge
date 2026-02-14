use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;

/// SPA index page, embedded at compile time from firmware/static/index.html.
const INDEX_HTML: &str = include_str!("../../static/index.html");

/// Register page-serving routes.
pub fn register(server: &mut EspHttpServer<'static>) -> anyhow::Result<()> {
    server.fn_handler("/", Method::Get, |req| {
        let mut resp = req.into_response(
            200,
            Some("OK"),
            &[("Content-Type", "text/html; charset=utf-8")],
        )?;
        resp.write_all(INDEX_HTML.as_bytes()).map(|_| ())
    })?;

    Ok(())
}
