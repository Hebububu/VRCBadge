//! State machine for background WiFi operations.
//!
//! WiFi scan and connect are blocking I/O that must run on a worker thread
//! (the main thread is busy with Slint rendering and touch polling). This
//! module owns the shared state used to communicate results back to the main
//! loop, where they are drained and applied to the UI.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};

use crate::{storage, wifi, BadgeUI, ScanResult};

/// Result of a background WiFi operation, polled by the main loop.
pub enum WiFiOpResult {
    /// No pending result.
    Idle,
    /// Scan completed with a list of APs.
    ScanDone(Vec<wifi::ScannedAp>),
    /// Connect succeeded: SSID, password (for NVS save), and IP.
    ConnectOk {
        ssid: String,
        password: String,
        ip: std::net::Ipv4Addr,
    },
    /// Connect failed with an error message.
    ConnectFailed(String),
}

/// Owns the shared `WiFiOpResult` slot and provides spawn + poll helpers.
///
/// Cheap to clone: the struct only holds an `Arc`. Callbacks clone it before
/// being moved into Slint closures.
#[derive(Clone)]
pub struct WiFiState {
    result: Arc<Mutex<WiFiOpResult>>,
}

impl WiFiState {
    pub fn new() -> Self {
        Self {
            result: Arc::new(Mutex::new(WiFiOpResult::Idle)),
        }
    }

    /// Spawn a background scan. The UI is updated immediately on the main
    /// thread to show "scanning"; results land via [`Self::poll`] on the
    /// next main-loop tick.
    pub fn spawn_scan(
        &self,
        wifi: Arc<Mutex<BlockingWifi<EspWifi<'static>>>>,
        ui: slint::Weak<BadgeUI>,
    ) {
        if let Some(ui) = ui.upgrade() {
            ui.set_wifi_scanning(true);
            ui.set_wifi_connect_status("".into());
        }
        let result = self.result.clone();
        std::thread::Builder::new()
            .name("wifi-scan".into())
            .stack_size(4096)
            .spawn(move || {
                let aps = if let Ok(mut wifi) = wifi.lock() {
                    wifi::scan(&mut wifi).unwrap_or_default()
                } else {
                    Vec::new()
                };
                if let Ok(mut op) = result.lock() {
                    *op = WiFiOpResult::ScanDone(aps);
                }
            })
            .ok();
    }

    /// Spawn a background connect attempt. The UI is updated immediately to
    /// show "Connecting..."; the result lands via [`Self::poll`] later.
    pub fn spawn_connect(
        &self,
        wifi: Arc<Mutex<BlockingWifi<EspWifi<'static>>>>,
        ui: slint::Weak<BadgeUI>,
        ssid: String,
        password: String,
    ) {
        if let Some(ui) = ui.upgrade() {
            ui.set_wifi_connect_status("Connecting...".into());
        }
        let result = self.result.clone();
        std::thread::Builder::new()
            .name("wifi-conn".into())
            .stack_size(4096)
            .spawn(move || {
                let outcome = if let Ok(mut wifi) = wifi.lock() {
                    wifi::connect_sta(&mut wifi, &ssid, &password)
                } else {
                    Err(anyhow::anyhow!("WiFi lock failed"))
                };
                if let Ok(mut op) = result.lock() {
                    *op = match outcome {
                        Ok(ip) => WiFiOpResult::ConnectOk { ssid, password, ip },
                        Err(e) => WiFiOpResult::ConnectFailed(format!("{e}")),
                    };
                }
            })
            .ok();
    }

    /// Drain any pending result and apply it to the UI / NVS / shared flags.
    /// Must be called from the main thread.
    pub fn poll(
        &self,
        ui: &BadgeUI,
        nvs: &mut EspNvs<NvsDefault>,
        dismiss_keyboard: &Arc<AtomicBool>,
        sta_connected: &mut bool,
    ) {
        let Ok(mut op) = self.result.try_lock() else {
            return;
        };
        match std::mem::replace(&mut *op, WiFiOpResult::Idle) {
            WiFiOpResult::Idle => {}
            WiFiOpResult::ScanDone(results) => {
                let model: Vec<ScanResult> = results
                    .iter()
                    .map(|ap| ScanResult {
                        ssid: ap.ssid.clone().into(),
                        rssi: ap.rssi as i32,
                        secure: ap.auth_required,
                    })
                    .collect();
                let model_rc = std::rc::Rc::new(slint::VecModel::from(model));
                ui.set_wifi_scan_results(model_rc.into());
                ui.set_wifi_scanning(false);
            }
            WiFiOpResult::ConnectOk { ssid, password, ip } => {
                ui.set_sta_connected(true);
                ui.set_sta_ssid(ssid.clone().into());
                ui.set_sta_ip(ip.to_string().into());
                ui.set_has_wifi_credentials(true);
                ui.set_wifi_connect_status("Connected".into());
                storage::save_wifi_credentials(nvs, &ssid, &password);
                dismiss_keyboard.store(true, Ordering::Relaxed);
                *sta_connected = true;
            }
            WiFiOpResult::ConnectFailed(msg) => {
                ui.set_sta_connected(false);
                ui.set_wifi_connect_status(format!("Failed: {msg}").into());
            }
        }
    }
}
