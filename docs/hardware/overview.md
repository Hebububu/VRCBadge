# Hardware Overview

<!-- TODO: Add photo of Pi Zero 2 W + display setup once assembled -->
![Hardware Overview](../../images/hardware-overview-placeholder.png)

*Image pending — will be added after hardware is assembled.*

## Chosen Platform: Raspberry Pi Zero 2 W

### Why Pi Zero 2 W?

- Ultra-compact form factor (65mm x 30mm) — ideal for wearable badge
- Built-in WiFi + Bluetooth
- Quad-core ARM Cortex-A53 @ 1GHz (64-bit)
- 512MB RAM — sufficient for badge UI and API server
- Low power draw (~1.5-2W) — longer battery life than CM4
- Same Pi ecosystem, community support, and `rppal` compatibility
- 40-pin GPIO header — direct SPI for display, GPIO for power monitoring
- No carrier board needed — solders directly onto custom PCB
- Significantly cheaper than CM4 + carrier board

### Specs

| Spec       | Value                                |
| ---------- | ------------------------------------ |
| SoC        | BCM2710A1 (quad-core Cortex-A53)     |
| RAM        | 512MB LPDDR2                         |
| WiFi       | 2.4GHz 802.11 b/g/n                  |
| Bluetooth  | BLE 4.2                              |
| GPIO       | 40-pin header                        |
| Video Out  | Mini HDMI (not used for badge)       |
| Storage    | microSD slot                         |
| Power      | 5V via micro USB (or custom via PCB) |
| Dimensions | 65mm x 30mm                          |

### Tradeoffs vs CM4

| Aspect          | Zero 2 W           | CM4                  |
| --------------- | ------------------- | -------------------- |
| RAM             | 512MB (fixed)       | 1-8GB (selectable)   |
| Size            | 65x30mm             | 55x40mm + carrier    |
| Display         | SPI only (no DSI)   | DSI + HDMI           |
| Power draw      | ~1.5-2W             | ~3-5W                |
| Cost            | ~$15                | ~$65 + carrier       |
| Carrier needed? | No (direct on PCB)  | Yes                  |
| Form factor     | Better for wearable | Better for dev board |

The lower RAM and SPI-only display are acceptable tradeoffs for the massive size, cost, and power savings.

## Display: 3.5" SPI Touchscreen

### Why SPI?

The Pi Zero 2 W has no DSI connector. SPI displays connect via the GPIO header, which works well with our custom PCB design.

### Display Requirements

- 3.5" IPS panel (~480x320 resolution)
- SPI interface (connects to GPIO header)
- Touch input (resistive or capacitive)
- Wide viewing angle for badge readability

### Recommended Displays

| Display                    | Resolution | Touch       | Notes                        |
| -------------------------- | ---------- | ----------- | ---------------------------- |
| Waveshare 3.5" SPI (Rev C) | 480x320    | Resistive   | Well-documented, Pi-specific |
| Pimoroni HyperPixel 4.0    | 480x800    | Capacitive  | Higher res, uses DPI not SPI |
| Generic ILI9488 3.5"       | 480x320    | Resistive   | Cheap, wide availability     |

### SPI Display Driver

The display requires a kernel driver (`fbtft` / `fb_ili9486`) or userspace framebuffer driver. This is handled in the software setup phase.

### Display Connection (SPI Pinout)

| Pin  | Function | GPIO   |
| ---- | -------- | ------ |
| VCC  | 3.3V     | Pin 1  |
| GND  | Ground   | Pin 6  |
| MOSI | SPI Data | GPIO10 |
| SCLK | SPI Clock| GPIO11 |
| CS   | Chip Sel | GPIO8  |
| DC   | Data/Cmd | GPIO25 |
| RST  | Reset    | GPIO27 |
| BL   | Backlight| GPIO18 |
| T_CS | Touch CS | GPIO7  |
