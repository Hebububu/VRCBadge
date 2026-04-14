//! UI callback wiring.
//!
//! All Slint callbacks live here. Each closure captures the shared state it
//! needs (cheap `Arc`/`Rc` clones); ownership of `backlight` is consumed by
//! the brightness slider closure.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use esp_idf_hal::ledc::LedcDriver;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use slint::ComponentHandle;

use crate::wifi_state::WiFiState;
use crate::{storage, wifi, BadgeUI, VirtualKeyboardHandler};

/// Wire all UI callbacks: brightness slider, virtual keyboard, WiFi controls.
pub fn wire(
    ui: &BadgeUI,
    wifi_handle: Arc<Mutex<BlockingWifi<EspWifi<'static>>>>,
    wifi_state: WiFiState,
    nvs: Rc<RefCell<EspNvs<NvsDefault>>>,
    backlight: LedcDriver<'static>,
    max_duty: u32,
) {
    // Brightness slider → LEDC PWM (debounced to avoid flicker).
    // Converts 10-100% from the UI slider to 0-255 LEDC duty cycle.
    {
        let backlight = RefCell::new(backlight);
        let last_brightness = RefCell::new(50.0_f32);
        ui.on_brightness_changed(move |percent| {
            let clamped = percent.clamp(10.0, 100.0);
            let mut last = last_brightness.borrow_mut();
            if (clamped - *last).abs() >= 2.0 {
                *last = clamped;
                let duty = (clamped / 100.0 * max_duty as f32) as u32;
                let _ = backlight.borrow_mut().set_duty(duty);
            }
        });
    }

    // Virtual keyboard: dispatch tapped key as KeyPressed + KeyReleased so
    // Slint routes it to the focused TextInput.
    {
        let weak = ui.as_weak();
        ui.global::<VirtualKeyboardHandler>()
            .on_key_pressed(move |key: slint::SharedString| {
                if let Some(ui) = weak.upgrade() {
                    ui.window()
                        .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                            text: key.clone(),
                        });
                    ui.window()
                        .dispatch_event(slint::platform::WindowEvent::KeyReleased { text: key });
                }
            });
    }

    // WiFi: scan for nearby networks (background thread)
    {
        let weak = ui.as_weak();
        let wifi = wifi_handle.clone();
        let state = wifi_state.clone();
        ui.on_wifi_scan(move || state.spawn_scan(wifi.clone(), weak.clone()));
    }

    // WiFi: connect to a network (background thread)
    {
        let weak = ui.as_weak();
        let wifi = wifi_handle.clone();
        let state = wifi_state;
        ui.on_wifi_connect(move |ssid, password| {
            state.spawn_connect(
                wifi.clone(),
                weak.clone(),
                ssid.to_string(),
                password.to_string(),
            );
        });
    }

    // WiFi: disconnect station mode
    {
        let weak = ui.as_weak();
        let wifi = wifi_handle.clone();
        ui.on_wifi_disconnect(move || {
            let Some(ui) = weak.upgrade() else { return };
            if let Ok(mut wifi) = wifi.lock() {
                let _ = wifi::disconnect_sta(&mut wifi);
            }
            ui.set_sta_connected(false);
            ui.set_sta_ssid("".into());
            ui.set_sta_ip("".into());
            ui.set_wifi_connect_status("Disconnected".into());
        });
    }

    // WiFi: forget saved credentials and disconnect
    {
        let weak = ui.as_weak();
        let wifi = wifi_handle;
        ui.on_wifi_forget(move || {
            let Some(ui) = weak.upgrade() else { return };
            if let Ok(mut wifi) = wifi.lock() {
                let _ = wifi::disconnect_sta(&mut wifi);
            }
            storage::delete_wifi_credentials(&mut nvs.borrow_mut());
            ui.set_sta_connected(false);
            ui.set_sta_ssid("".into());
            ui.set_sta_ip("".into());
            ui.set_has_wifi_credentials(false);
            ui.set_wifi_connect_status("Credentials forgotten".into());
        });
    }
}
