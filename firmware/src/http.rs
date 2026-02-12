use std::sync::{Arc, Mutex};

use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;

use crate::platform::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

/// Expected size of a raw RGB888 background image (480 * 320 * 3 bytes).
const BACKGROUND_IMAGE_SIZE: usize = DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize * 3;

/// Shared state for passing image data from the HTTP thread to the main loop.
pub type SharedImageData = Arc<Mutex<Option<Vec<u8>>>>;

/// HTML upload page served at GET /
const UPLOAD_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>VRCBadge</title>
<style>
body{background:#1a1a2e;color:#e0e0e0;font-family:sans-serif;margin:0;padding:20px;display:flex;flex-direction:column;align-items:center}
h1{color:#fff;margin-bottom:4px}
p.sub{color:#888;font-size:14px;margin-top:0}
.upload-area{background:#2a2a4a;border:2px dashed #3a3a5a;border-radius:12px;padding:30px;text-align:center;width:90%;max-width:400px;margin-top:20px}
input[type=file]{margin:16px 0}
button{background:#1b4f72;color:#fff;border:none;border-radius:8px;padding:12px 32px;font-size:16px;cursor:pointer;margin-top:12px}
button:disabled{opacity:0.5;cursor:not-allowed}
#status{margin-top:16px;font-size:14px;min-height:20px}
.ok{color:#4caf50}.err{color:#f44336}
</style>
</head>
<body>
<h1>VRCBadge</h1>
<p class="sub">Background Image Upload</p>
<div class="upload-area">
<p>Select an image to set as badge background</p>
<p style="color:#888;font-size:12px">Image will be resized to 480x320</p>
<input type="file" id="file" accept="image/*">
<br>
<button id="btn" onclick="upload()" disabled>Upload</button>
</div>
<div id="status"></div>
<script>
const W=480,H=320;
const file=document.getElementById('file');
const btn=document.getElementById('btn');
const status=document.getElementById('status');
file.onchange=()=>{btn.disabled=!file.files.length};
function upload(){
  const f=file.files[0];
  if(!f)return;
  btn.disabled=true;
  status.textContent='Processing...';
  status.className='';
  const img=new Image();
  img.onload=()=>{
    const c=document.createElement('canvas');
    c.width=W;c.height=H;
    const ctx=c.getContext('2d');
    // Cover crop: scale to fill, center crop
    const s=Math.max(W/img.width,H/img.height);
    const sw=img.width*s,sh=img.height*s;
    ctx.drawImage(img,(W-sw)/2,(H-sh)/2,sw,sh);
    const rgba=ctx.getImageData(0,0,W,H).data;
    // Strip alpha: RGBA -> RGB
    const rgb=new Uint8Array(W*H*3);
    for(let i=0,j=0;i<rgba.length;i+=4,j+=3){
      rgb[j]=rgba[i];rgb[j+1]=rgba[i+1];rgb[j+2]=rgba[i+2];
    }
    status.textContent='Uploading ('+Math.round(rgb.length/1024)+' KB)...';
    fetch('/api/background',{method:'POST',body:rgb,headers:{'Content-Type':'application/octet-stream'}})
    .then(r=>{
      if(r.ok){status.textContent='Done!';status.className='ok';}
      else r.text().then(t=>{status.textContent='Error: '+t;status.className='err';});
    })
    .catch(e=>{status.textContent='Error: '+e;status.className='err';})
    .finally(()=>{btn.disabled=false;});
  };
  img.onerror=()=>{status.textContent='Failed to load image';status.className='err';btn.disabled=false;};
  img.src=URL.createObjectURL(f);
}
</script>
</body>
</html>"#;

/// Redirect handler for captive portal detection probes.
///
/// Returns a 302 redirect to `http://192.168.4.1/` so that the OS opens the
/// captive portal page automatically.
fn redirect_to_portal(
    req: esp_idf_svc::http::server::Request<&mut esp_idf_svc::http::server::EspHttpConnection>,
) -> Result<(), esp_idf_svc::io::EspIOError> {
    let mut resp = req.into_response(302, Some("Found"), &[("Location", "http://192.168.4.1/")])?;
    resp.write_all(b"Redirecting...")?;
    Ok(())
}

/// Start the HTTP server and register API routes.
///
/// Takes shared state for passing background image data to the main loop.
/// Returns the server handle — caller must hold it to keep the server alive.
pub fn init(pending_background: SharedImageData) -> anyhow::Result<EspHttpServer<'static>> {
    let config = Configuration {
        http_port: 80,
        stack_size: 16384,
        max_uri_handlers: 16,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&config)?;

    // Upload page
    server.fn_handler("/", Method::Get, |req| {
        let mut resp = req.into_response(
            200,
            Some("OK"),
            &[("Content-Type", "text/html; charset=utf-8")],
        )?;
        resp.write_all(UPLOAD_HTML.as_bytes()).map(|_| ())
    })?;

    // --- Captive portal detection endpoints ---
    // Android / Chrome OS
    server.fn_handler("/generate_204", Method::Get, redirect_to_portal)?;
    // Apple iOS / macOS
    server.fn_handler("/hotspot-detect.html", Method::Get, redirect_to_portal)?;
    // Windows
    server.fn_handler("/connecttest.txt", Method::Get, redirect_to_portal)?;
    server.fn_handler("/redirect", Method::Get, redirect_to_portal)?;
    // Firefox
    server.fn_handler("/canonical.html", Method::Get, redirect_to_portal)?;
    // Additional common captive portal probe paths
    server.fn_handler("/ncsi.txt", Method::Get, redirect_to_portal)?;
    server.fn_handler("/success.txt", Method::Get, redirect_to_portal)?;

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

    // Wildcard fallback — must be registered LAST so specific routes take priority.
    // Catches any unmatched GET request (e.g. captive portal probes we didn't list)
    // and redirects to the main page.
    server.fn_handler("/*", Method::Get, redirect_to_portal)?;

    log::info!("HTTP server started on port 80");

    Ok(server)
}
