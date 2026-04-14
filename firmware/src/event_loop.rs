//! Main event loop for the badge runtime.
//!
//! Runs forever after boot. Per tick:
//!
//! ```text
//! ┌──────────────────────────────────────────────┐
//! │ Slint timers/animations                      │
//! │ Touch poll → Slint events                    │
//! │ Deferred keyboard dismiss                    │
//! │ WiFiState.poll  (drain async wifi results)   │
//! │ Every 125 ticks (~2s):                       │
//! │   • AP client count                          │
//! │   • STA status (connect/disconnect)          │
//! │   • Toast auto-hide (5s)                     │
//! │   • About page sysinfo + log snapshot        │
//! │   • PendingState.poll_into_ui (web uploads)  │
//! │ Render into DMA framebuffer                  │
//! │ Sleep ~8ms                                   │
//! └──────────────────────────────────────────────┘
//! ```

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use slint::platform::software_renderer::{MinimalSoftwareWindow, Rgb565Pixel};

use crate::pending::PendingState;
use crate::touch::TouchController;
use crate::wifi_state::WiFiState;
use crate::{logger, platform, sysinfo, wifi, BadgeUI};

/// Everything the event loop needs to own.
pub struct LoopDeps {
    pub window: Rc<MinimalSoftwareWindow>,
    pub framebuffer: &'static mut [Rgb565Pixel],
    pub ui: BadgeUI,
    pub touch: Option<TouchController<'static>>,
    pub nvs: Rc<RefCell<EspNvs<NvsDefault>>>,
    pub wifi_handle: Arc<Mutex<BlockingWifi<EspWifi<'static>>>>,
    pub wifi_state: WiFiState,
    pub pending: PendingState,
    pub dismiss_keyboard: Arc<AtomicBool>,
    pub sta_connected: bool,
    pub boot_time: Instant,
}

/// Run the badge event loop forever.
pub fn run(mut deps: LoopDeps) -> ! {
    let mut loop_count: u32 = 0;
    let mut toast_shown_at: Option<Instant> = if deps.ui.get_toast_visible() {
        Some(Instant::now())
    } else {
        None
    };

    loop {
        // 1. Process Slint timers and animations
        slint::platform::update_timers_and_animations();

        // 2. Poll touch input → dispatch events to Slint
        if let Some(ref mut touch) = deps.touch {
            touch.poll(&deps.window);
        }

        // 2b. Dismiss virtual keyboard if requested (deferred from callback)
        if deps.dismiss_keyboard.swap(false, Ordering::Relaxed) {
            deps.window
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::platform::Key::Escape.into(),
                });
            deps.window
                .dispatch_event(slint::platform::WindowEvent::KeyReleased {
                    text: slint::platform::Key::Escape.into(),
                });
        }

        // 2c. Drain background WiFi op results
        deps.wifi_state.poll(
            &deps.ui,
            &mut deps.nvs.borrow_mut(),
            &deps.dismiss_keyboard,
            &mut deps.sta_connected,
        );

        // 3. Periodic poll (~every 2 seconds at 16ms sleep = 125 iterations)
        loop_count = loop_count.wrapping_add(1);
        if loop_count % 125 == 0 {
            // WiFi AP client count
            deps.ui.set_wifi_clients(wifi::connected_clients() as i32);

            // WiFi STA connection status
            if let Ok(wifi) = deps.wifi_handle.try_lock() {
                match wifi::sta_status(&wifi) {
                    wifi::StaStatus::Connected { ssid, ip } => {
                        if !deps.sta_connected {
                            log::info!("WiFi STA connected: {ssid} ({ip})");
                            deps.sta_connected = true;
                        }
                        deps.ui.set_sta_connected(true);
                        deps.ui.set_sta_ssid(ssid.into());
                        deps.ui.set_sta_ip(ip.to_string().into());
                    }
                    wifi::StaStatus::Disconnected => {
                        if deps.sta_connected {
                            log::warn!("WiFi STA disconnected");
                            deps.sta_connected = false;
                        }
                        deps.ui.set_sta_connected(false);
                    }
                }
            }

            // Toast auto-hide after 5 seconds
            if let Some(shown_at) = toast_shown_at {
                if shown_at.elapsed() >= Duration::from_secs(5) {
                    deps.ui.set_toast_visible(false);
                    deps.ui.set_toast_message("".into());
                    toast_shown_at = None;
                }
            }

            // About page: system info + logs
            deps.ui
                .set_about_uptime(sysinfo::uptime_string(&deps.boot_time).into());
            deps.ui
                .set_about_heap(format!("{} KB", sysinfo::free_heap_kb()).into());
            deps.ui
                .set_about_psram(format!("{} KB", sysinfo::free_psram_kb()).into());
            deps.ui.set_log_text(logger::snapshot().into());

            // Drain any pending profile/avatar/background updates from web
            deps.pending.poll_into_ui(&deps.ui, &deps.nvs);
        }

        // 4. Render directly into the DMA framebuffer.
        // The RGB panel hardware continuously DMA-refreshes from this buffer.
        let fb = &mut *deps.framebuffer;
        deps.window.draw_if_needed(|renderer| {
            renderer.render(fb, platform::DISPLAY_WIDTH as usize);
        });

        // 5. Sleep until next event needed
        if !deps.window.has_active_animations() {
            if let Some(duration) = slint::platform::duration_until_next_timer_update() {
                let sleep_ms = duration.as_millis().min(8) as u32;
                esp_idf_hal::delay::FreeRtos::delay_ms(sleep_ms);
            } else {
                esp_idf_hal::delay::FreeRtos::delay_ms(8);
            }
        }
    }
}
