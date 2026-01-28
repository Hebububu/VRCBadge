# ESP32-S3 Migration Plan

## Why Migrate from Pi Zero 2W

The current design mounts a Raspberry Pi Zero 2W onto a custom PCB via a 40-pin header. Since we already design a custom PCB, a direct MCU approach removes unnecessary overhead:

| Metric | Pi Zero 2W | ESP32-S3 |
|--------|-----------|----------|
| Boot time | 15-30s (Linux) | 1-3s (bare metal / RTOS) |
| Battery life (3000mAh) | 4-6h | 12-20h |
| Module cost | ~$15 + SD card | ~$4-5 |
| PCB complexity | 40-pin header, boost converter, microSD | Module soldered directly, LDO only |
| Board size | Constrained by Pi footprint (65x30mm) | Module is 18x25mm |
| Programming | SSH over WiFi | USB-C (built-in USB-JTAG) |

The Pi Zero 2W's Linux environment provides no benefit here — we don't need a filesystem, package manager, or multi-process OS. The badge runs a single Rust binary.

## MCU Selection: ESP32-S3-WROOM-1-N16R8

### Why ESP32-S3

| Candidate | Verdict |
|-----------|---------|
| **ESP32-S3 (PSRAM)** | **Selected** — 8MB PSRAM, built-in WiFi, Slint officially supported, Tokio+Axum working in `std` mode via esp-idf |
| ESP32-C6 | No PSRAM — 512KB total RAM insufficient for 480x320 display + Slint + WiFi + HTTP |
| STM32H7 + ESP32 | Two MCUs, 4-layer PCB, complex firmware split, no cost benefit |
| RP2350 (Pico 2W) | WiFi Rust support immature — `cyw43` driver for Pico 2W still WIP |

### Module Specs

- **CPU:** Dual-core Xtensa LX7 @ 240MHz
- **SRAM:** 512KB
- **PSRAM:** 8MB Octal-SPI
- **Flash:** 16MB
- **Wireless:** WiFi 802.11 b/g/n + BLE 5.0
- **Antenna:** Integrated PCB antenna
- **Module size:** ~18x25mm
- **Unit cost:** ~$4-5

The 8MB PSRAM is the deciding factor. Slint's software renderer with a 480x320 RGB565 framebuffer requires ~300KB for the runtime plus line buffers. The remaining PSRAM is available for WiFi buffers, HTTP request handling, and image storage in RAM.

## Software Architecture

### Two Viable Approaches

#### Option A: `std` mode (recommended — preserves current architecture)

```
Tokio (async runtime)          — works on ESP32-S3 via esp-idf
Axum (HTTP server)             — works with mio_unsupported_force_poll_poll flag
Slint (UI framework)           — officially supported, line-by-line rendering
esp-idf-hal (GPIO/SPI/I2C)    — replaces rppal
esp-idf-svc (WiFi, NVS)       — WiFi and storage management
SPIFFS/LittleFS                — replaces filesystem for image storage
```

This approach lets us keep the existing Tokio + Axum + Slint architecture largely intact. The main changes are hardware abstraction (rppal → esp-idf-hal) and storage (filesystem → flash partition).

#### Option B: `no_std` mode (leaner, if Option A hits RAM or stability issues)

```
Embassy (async runtime)        — replaces Tokio
picoserve (HTTP server)        — Axum-inspired API, designed for embedded
Slint (UI framework)           — same, line-by-line rendering
esp-hal (GPIO/SPI/I2C)         — bare-metal HAL
```

This avoids the esp-idf C runtime entirely. Lower RAM overhead, but requires rewriting the HTTP layer and async patterns.

**Decision:** Deferred to prototyping. Prototype both on a dev board, commit to whichever proves more stable and performant.

### Slint on MCU

From the Slint embedded documentation, the key configuration:

**Cargo features needed:**
- `compat-1-2`
- `unsafe-single-threaded`
- `libm`
- `renderer-software`

**Rendering strategy — line-by-line (`render_by_line()`):**
- Renders one horizontal line at a time instead of a full framebuffer
- Memory per line: ~960 bytes (480 pixels x 2 bytes RGB565)
- Use dual line buffers with DMA for async SPI writes
- Slint runtime fits in <300KB RAM
- Build script must set `EmbedResourcesKind::EmbedForSoftwareRenderer`

### Display Pipeline Change

```
Current:  Slint → /dev/fb1 → fbtft kernel driver → SPI → ILI9486
New:      Slint → render_by_line() → DMA → SPI → ILI9486
```

No Linux kernel driver needed. The MCU writes directly to the display over SPI.

### Storage Change

```
Current:  Avatar images saved to Pi filesystem (SD card)
New:      Images stored in flash partition (SPIFFS or LittleFS)
```

The 16MB flash is partitioned:
- ~4MB for firmware
- ~12MB for data (SPIFFS/LittleFS) — avatar images, profile config

For `std` mode, esp-idf provides a VFS layer that maps flash partitions to file paths, so `std::fs` calls work transparently.

## Hardware / PCB Changes

### Component Changes

| Component | Current (Pi Zero 2W) | New (ESP32-S3) |
|-----------|----------------------|-----------------|
| Processor | Pi Zero 2W on 40-pin header | ESP32-S3-WROOM-1-N16R8 soldered to PCB |
| Power rail | 5V (TPS61023 boost from 3.7V LiPo) | 3.3V (LDO from LiPo) |
| Storage | microSD card | Internal 16MB flash (SPIFFS/LittleFS) |
| Programming | SSH over WiFi | USB-C via built-in USB-JTAG (or UART) |

### What Stays

| Component | Notes |
|-----------|-------|
| TP4056 charger | Unchanged — charges LiPo from USB-C 5V |
| DW01A + FS8205 protection | Unchanged — battery protection circuit |
| TPS22918 soft switch | Keep, adjust enable threshold for 3.3V rail |
| MAX17048 fuel gauge | Keep — route I2C to ESP32-S3 GPIO pins |
| USB-C connector | Keep — charging + programming |
| RFID UART header | Keep — route to ESP32-S3 UART pins |
| 3.5" ILI9486 display | Keep — route SPI to ESP32-S3 SPI pins |
| LiPo 3000mAh cell | Keep |
| Tactile power button | Keep |

### What Gets Removed

| Component | Reason |
|-----------|--------|
| **TPS61023 boost converter** | No 5V rail needed — ESP32-S3 runs at 3.3V |
| **40-pin header** | No Pi to mount |
| **microSD card slot** | Flash storage replaces SD |
| Associated passives (inductor, capacitors for boost) | No boost converter |

### New Components

| Component | Purpose | Approx. Cost |
|-----------|---------|-------------|
| ESP32-S3-WROOM-1-N16R8 | MCU module | ~$4-5 |
| AP2112K-3.3 (or similar LDO) | 3.3V regulation from LiPo (3.0-4.2V) | ~$0.30 |
| 2x 10µF decoupling caps | LDO input/output | ~$0.10 |
| 2x 0.1µF decoupling caps | ESP32 power pins | ~$0.10 |

The AP2112K-3.3 is a low-dropout regulator with 600mA output, sufficient for ESP32-S3 peak current (~500mA during WiFi TX). Dropout voltage is ~250mV, so it works down to ~3.55V LiPo voltage. Below that, the battery is near empty anyway (3.0V cutoff from DW01A).

### Power Path (New)

```
USB-C 5V ──→ TP4056 ──→ LiPo 3.7V ──→ DW01A/FS8205 ──→ TPS22918 ──→ AP2112K ──→ 3.3V rail
                                                                         │
                                                                         ├── ESP32-S3
                                                                         ├── ILI9486 display
                                                                         └── MAX17048
```

Compared to the current design, the boost converter stage is eliminated entirely. The 3.3V LDO is simpler, cheaper, and more efficient.

### GPIO Pin Mapping (ESP32-S3)

| Function | ESP32-S3 Pin | Notes |
|----------|-------------|-------|
| SPI MOSI (display) | GPIO 11 | FSPI default |
| SPI SCLK (display) | GPIO 12 | FSPI default |
| SPI CS (display) | GPIO 10 | FSPI default |
| Display DC | GPIO 9 | Data/command select |
| Display RST | GPIO 8 | Reset |
| Display BL | GPIO 7 | Backlight PWM |
| I2C SDA (fuel gauge) | GPIO 1 | Any GPIO works |
| I2C SCL (fuel gauge) | GPIO 2 | Any GPIO works |
| UART TX (RFID) | GPIO 17 | UART1 |
| UART RX (RFID) | GPIO 18 | UART1 |
| Power button sense | GPIO 3 | Input with interrupt |
| USB D+ | GPIO 20 | Built-in USB-JTAG |
| USB D- | GPIO 19 | Built-in USB-JTAG |

Pin assignments are preliminary and subject to change during PCB layout to optimize routing.

## Updated BOM Estimate

| Category | Item | Cost |
|----------|------|------|
| MCU | ESP32-S3-WROOM-1-N16R8 | $4-5 |
| Display | 3.5" ILI9486 SPI | $8-12 |
| Battery | 3.7V 3000mAh LiPo pouch | $5-8 |
| Charger IC | TP4056 | $0.20 |
| Protection | DW01A + FS8205 | $0.30 |
| LDO | AP2112K-3.3 | $0.30 |
| Soft switch | TPS22918 | $0.80 |
| Fuel gauge | MAX17048 | $2-3 |
| Connector | USB-C (16-pin) | $0.30 |
| Button | Tactile switch | $0.10 |
| Passives | Resistors, capacitors, etc. | $1-2 |
| PCB fab | JLCPCB 2-layer + SMT assembly | $10-30 |
| **Total (without RFID)** | | **~$35-65** |
| RFID (future) | Chameleon Tiny module | $35 |
| **Total (with RFID)** | | **~$70-100** |

Compared to the Pi Zero 2W design (~$65-100 without RFID), the ESP32-S3 version saves ~$30 by eliminating the Pi, SD card, and boost converter.

## Files That Need Updating

After prototyping confirms the ESP32-S3 approach works, the following project files need changes:

| File | Change |
|------|--------|
| `Cargo.toml` | Replace rppal → esp-idf-hal/esp-hal, add esp-idf-svc, update slint features |
| `src/main.rs` | New entry point for ESP-IDF or bare-metal |
| `docs/hardware/overview.md` | Platform rationale (Pi Zero → ESP32-S3) |
| `docs/hardware/pcb-design.md` | Rearchitect: ESP32-S3 module, remove boost, add LDO |
| `docs/hardware/pcb-guideline.md` | Updated schematic/layout walkthrough |
| `docs/hardware/shopping-list.md` | Updated BOM |
| `docs/hardware/assembly.md` | Updated physical stack (no Pi, thinner) |
| `docs/software/architecture.md` | New software arch (std vs no_std, display pipeline, storage) |
| `docs/development/setup.md` | ESP32-S3 toolchain (espup, cargo-espflash) |
| `docs/development/deployment.md` | Flash via USB instead of SSH/SCP |
| `docs/development/roadmap.md` | Restructured phases |

## Migration Roadmap

### Phase 1: ESP32-S3 Dev Board Prototype

**Goal:** Validate the platform before committing to a custom PCB.

Buy an ESP32-S3-DevKitC-1 (N16R8 variant, ~$10) and wire up the ILI9486 display via breadboard.

**Success criteria:**
- ESP-IDF Rust toolchain builds and flashes
- WiFi connects and serves HTTP
- SPI display shows a test pattern

### Phase 2: Display Driver

**Goal:** Slint renders the badge UI to the ILI9486 display.

- Implement ILI9486 SPI driver (or adapt existing `mipidsi` crate)
- Integrate Slint with `render_by_line()` and DMA
- Verify frame rate is acceptable for static badge content

**Success criteria:**
- Slint `.slint` files render correctly on the physical display
- No visible tearing or artifacts

### Phase 3: HTTP API

**Goal:** REST endpoints for avatar upload and profile management.

- Tokio + Axum (std mode) or Embassy + picoserve (no_std mode)
- Image upload stored to SPIFFS/LittleFS
- Profile data stored to NVS (non-volatile storage)

**Success criteria:**
- `POST /api/avatar` accepts an image and stores it
- `POST /api/profile` updates displayed name/tagline
- `GET /api/status` returns battery %, WiFi RSSI

### Phase 4: Integration

**Goal:** Full badge functionality on dev board.

- Avatar display from flash storage
- Battery monitoring via MAX17048 (wire to dev board I2C)
- Power management (WiFi sleep, display dimming)

**Success criteria:**
- End-to-end flow: upload avatar via phone → badge displays it
- Battery percentage reads correctly
- Power draw measured and documented

### Phase 5: Custom PCB

**Goal:** Design and order the ESP32-S3 PCB.

- EasyEDA schematic with ESP32-S3 module, AP2112K LDO, existing charging circuit
- Optimized layout (potentially smaller than current 85x55mm)
- Order from JLCPCB with SMT assembly

**Success criteria:**
- PCB arrives and works on first revision (or with minor bodge wires)
- All peripherals functional: display, WiFi, I2C, UART header, USB-C

### Phase 6: RFID Integration

**Goal:** Chameleon Tiny module via UART (unchanged from original plan).

- Wire RFID module to UART header
- Port serial communication code
- Card slot management UI

**Success criteria:**
- Read/write NFC cards through the badge
- UI shows current active card slot

## Verification Checklist

Before committing to a custom PCB, verify all of the following on the ESP32-S3-DevKitC-1:

- [ ] Slint renders correctly to ILI9486 over SPI
- [ ] Axum (or picoserve) HTTP server accepts image uploads over WiFi
- [ ] MAX17048 reads battery voltage/percentage over I2C
- [ ] Total RAM usage stays under 8MB PSRAM
- [ ] WiFi + display active power draw measured
- [ ] Boot time from power-on to UI displayed < 3 seconds
- [ ] Flash storage read/write works (SPIFFS or LittleFS)
- [ ] USB-JTAG programming works reliably

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Axum too heavy for ESP32-S3 in std mode | Medium | High | Fall back to picoserve (no_std) |
| PSRAM bandwidth bottleneck with WiFi + display | Low | Medium | Use line-by-line rendering (no full framebuffer) |
| ILI9486 3.3V logic compatibility | Low | Low | Display modules typically accept 3.3V SPI; verify datasheet |
| AP2112K dropout at low battery | Low | Low | Battery cutoff at 3.0V (DW01A) is well above 3.55V dropout threshold |
| esp-idf Rust bindings unstable | Medium | Medium | Pin known-good versions; consider no_std as fallback |
