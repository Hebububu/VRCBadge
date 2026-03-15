//! WiFi driver: AP + optional STA (station) mode.
//!
//! The badge always runs an access point (`VRCBadge`, open, 192.168.71.x)
//! for the configuration web portal. Optionally, it also connects to a
//! nearby WiFi network in station mode (for future OTA updates, etc.).
//!
//! When both modes are active, ESP-IDF runs them simultaneously using
//! `Configuration::Mixed`.

use std::net::Ipv4Addr;

use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi,
};
use esp_idf_sys::{esp, esp_wifi_ap_get_sta_list, wifi_sta_list_t};

/// Result of scanning a nearby access point.
#[derive(Debug, Clone)]
pub struct ScannedAp {
    pub ssid: String,
    /// RSSI in dBm (e.g. -40 = strong, -80 = weak).
    pub rssi: i8,
    /// Whether the network requires authentication.
    pub auth_required: bool,
}

/// Station connection status.
#[derive(Debug, Clone)]
pub enum StaStatus {
    Disconnected,
    Connected { ssid: String, ip: Ipv4Addr },
}

/// AP configuration used for all modes.
fn ap_config() -> AccessPointConfiguration {
    AccessPointConfiguration {
        ssid: "VRCBadge".try_into().unwrap(),
        auth_method: AuthMethod::None,
        channel: 1,
        max_connections: 4,
        ..Default::default()
    }
}

/// Initialize WiFi in AP-only mode.
///
/// Returns the WiFi handle and the AP's IP address.
/// The handle must be kept alive for the AP to stay running.
pub fn init(
    modem: Modem,
    sys_loop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> anyhow::Result<(BlockingWifi<EspWifi<'static>>, Ipv4Addr)> {
    let mut wifi = BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, sys_loop)?;

    wifi.set_configuration(&Configuration::AccessPoint(ap_config()))?;
    wifi.start()?;
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().ap_netif().get_ip_info()?;
    let ip = Ipv4Addr::from(ip_info.ip.octets());

    log::info!("WiFi AP started -- SSID: VRCBadge, IP: {ip}");

    Ok((wifi, ip))
}

/// Scan for nearby access points.
///
/// Scanning requires the STA interface to be active. If the WiFi is in
/// AP-only mode, this temporarily switches to Mixed mode for the scan
/// and reverts afterwards. If already in Mixed mode (STA connected or
/// previously configured), the scan runs directly.
///
/// Returns up to 20 APs sorted by signal strength (strongest first).
pub fn scan(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<Vec<ScannedAp>> {
    log::info!("Starting WiFi scan...");

    // Scanning requires the STA interface. If we're in AP-only mode,
    // temporarily switch to Mixed mode with a dummy STA config.
    let was_ap_only = matches!(wifi.get_configuration()?, Configuration::AccessPoint(_));

    if was_ap_only {
        let dummy_sta = ClientConfiguration::default();
        wifi.set_configuration(&Configuration::Mixed(dummy_sta, ap_config()))?;
        wifi.stop()?;
        wifi.start()?;
    }

    let scan_result = wifi.scan();

    // Revert to AP-only if we switched (even if scan failed)
    if was_ap_only {
        let _ = revert_to_ap_only(wifi);
    }

    let scan_result = scan_result?;

    let mut aps: Vec<ScannedAp> = scan_result
        .into_iter()
        .filter(|ap| !ap.ssid.is_empty())
        .map(|ap| ScannedAp {
            ssid: ap.ssid.to_string(),
            rssi: ap.signal_strength,
            auth_required: ap.auth_method != Some(AuthMethod::None),
        })
        .collect();

    // Sort by signal strength (strongest first)
    aps.sort_by(|a, b| b.rssi.cmp(&a.rssi));
    aps.truncate(20);

    log::info!("WiFi scan found {} APs", aps.len());
    Ok(aps)
}

/// Connect to an external WiFi network (station mode) while keeping the AP running.
///
/// Switches from AP-only to Mixed (AP+STA) mode, then attempts to connect.
/// Uses a 10-second timeout. On failure, reverts to AP-only mode.
pub fn connect_sta(
    wifi: &mut BlockingWifi<EspWifi<'static>>,
    ssid: &str,
    password: &str,
) -> anyhow::Result<Ipv4Addr> {
    log::info!("Connecting to WiFi network: {ssid}");

    let auth = if password.is_empty() {
        AuthMethod::None
    } else {
        AuthMethod::WPA2Personal
    };

    let client_config = ClientConfiguration {
        ssid: ssid
            .try_into()
            .map_err(|_| anyhow::anyhow!("SSID too long"))?,
        password: password
            .try_into()
            .map_err(|_| anyhow::anyhow!("Password too long"))?,
        auth_method: auth,
        ..Default::default()
    };

    // Switch to Mixed mode (AP stays running, STA connects)
    wifi.set_configuration(&Configuration::Mixed(client_config, ap_config()))?;
    wifi.stop()?;
    wifi.start()?;

    // Attempt to connect with timeout
    match wifi.connect() {
        Ok(()) => {}
        Err(e) => {
            log::warn!("WiFi STA connect failed: {e}, reverting to AP-only");
            revert_to_ap_only(wifi)?;
            return Err(anyhow::anyhow!("WiFi connect failed: {e}"));
        }
    }

    // Wait for IP with timeout
    match wifi.wait_netif_up() {
        Ok(()) => {}
        Err(e) => {
            log::warn!("WiFi STA netif up failed: {e}, reverting to AP-only");
            revert_to_ap_only(wifi)?;
            return Err(anyhow::anyhow!("WiFi netif up failed: {e}"));
        }
    }

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    let ip = Ipv4Addr::from(ip_info.ip.octets());

    log::info!("WiFi STA connected to {ssid}, IP: {ip}");
    Ok(ip)
}

/// Disconnect from the external WiFi network and revert to AP-only mode.
pub fn disconnect_sta(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    log::info!("Disconnecting WiFi STA");
    let _ = wifi.disconnect();
    revert_to_ap_only(wifi)?;
    log::info!("Reverted to AP-only mode");
    Ok(())
}

/// Query station connection status (connected AP name + IP, or disconnected).
pub fn sta_status(wifi: &BlockingWifi<EspWifi<'static>>) -> StaStatus {
    if !wifi.is_connected().unwrap_or(false) {
        return StaStatus::Disconnected;
    }

    let ip_info = match wifi.wifi().sta_netif().get_ip_info() {
        Ok(info) => info,
        Err(_) => return StaStatus::Disconnected,
    };

    let ip = Ipv4Addr::from(ip_info.ip.octets());

    // Get the connected SSID from the current configuration
    let ssid = match wifi.get_configuration() {
        Ok(Configuration::Mixed(client, _)) => client.ssid.to_string(),
        _ => String::new(),
    };

    if ip == Ipv4Addr::UNSPECIFIED {
        StaStatus::Disconnected
    } else {
        StaStatus::Connected { ssid, ip }
    }
}

/// Query the number of stations currently connected to the AP.
pub fn connected_clients() -> u8 {
    let mut sta_list: wifi_sta_list_t = unsafe { core::mem::zeroed() };
    match unsafe { esp!(esp_wifi_ap_get_sta_list(&mut sta_list)) } {
        Ok(()) => (sta_list.num.max(0) as u8).min(15),
        Err(_) => 0,
    }
}

/// Revert WiFi from Mixed mode back to AP-only.
fn revert_to_ap_only(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    wifi.set_configuration(&Configuration::AccessPoint(ap_config()))?;
    wifi.stop()?;
    wifi.start()?;
    wifi.wait_netif_up()?;
    Ok(())
}
