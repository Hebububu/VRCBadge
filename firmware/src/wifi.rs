use std::net::Ipv4Addr;

use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, BlockingWifi, Configuration, EspWifi,
};
use esp_idf_sys::{esp, esp_wifi_ap_get_sta_list, wifi_sta_list_t};

/// Initialize WiFi in AP mode.
///
/// Returns the WiFi handle and the AP's actual IP address.
/// Caller must hold the handle to keep the AP alive.
pub fn init(
    modem: Modem,
    sys_loop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> anyhow::Result<(BlockingWifi<EspWifi<'static>>, Ipv4Addr)> {
    let mut wifi = BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, sys_loop)?;

    wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration {
        ssid: "VRCBadge".try_into().unwrap(),
        auth_method: AuthMethod::None,
        channel: 1,
        max_connections: 4,
        ..Default::default()
    }))?;

    wifi.start()?;
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().ap_netif().get_ip_info()?;
    let ip = Ipv4Addr::from(ip_info.ip.octets());

    log::info!("WiFi AP started — SSID: VRCBadge, IP: {ip}");

    Ok((wifi, ip))
}

/// Query the number of stations currently connected to the AP.
pub fn connected_clients() -> u8 {
    // SAFETY: wifi_sta_list_t is #[repr(C)] with only plain data fields (arrays, ints,
    // bitfields). Zero is a valid state for all fields.
    let mut sta_list: wifi_sta_list_t = unsafe { core::mem::zeroed() };

    // SAFETY: esp_wifi_ap_get_sta_list writes into the provided pointer. We pass a valid
    // mutable reference to a stack-allocated struct. The function is thread-safe (takes
    // an internal WiFi lock). Returns an error if WiFi is not started.
    match unsafe { esp!(esp_wifi_ap_get_sta_list(&mut sta_list)) } {
        Ok(()) => (sta_list.num.max(0) as u8).min(15),
        Err(_) => 0,
    }
}
