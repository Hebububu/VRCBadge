# PCB Design (EasyEDA)

## Overview

<!-- TODO: Add EasyEDA schematic screenshot and PCB layout render -->
![PCB Design](../../images/pcb-design-placeholder.png)

*Image pending — will be added after EasyEDA design is complete.*

A custom PCB replaces both the carrier board and acrylic enclosure from the original CM4 design. The PCB serves as the structural backbone of the badge, mounting all components in a compact, wearable package.

> For a step-by-step walkthrough of the design process, see [PCB Design Guideline](./pcb-guideline.md).

## Design Tool

**EasyEDA Pro** (desktop app) — https://easyeda.com/page/download

Free PCB design tool with direct JLCPCB integration for ordering and LCSC component library built-in.

## Assembly Method

**JLCPCB SMT assembly** for all SMD components. Through-hole parts (headers, connectors, button) hand-soldered after delivery.

## PCB Requirements

### Board Specs

| Parameter      | Value                          |
| -------------- | ------------------------------ |
| Dimensions     | ~85 x 55mm (credit card size)  |
| Layers         | 2-layer (top + bottom)         |
| Thickness      | 1.6mm standard                 |
| Copper weight  | 1oz                            |
| Surface finish | HASL (lead-free)               |
| Solder mask    | Black (badge aesthetic)        |
| Silkscreen     | White                          |

### Functional Blocks

```
┌─────────────────────────────────────────────────┐
│                  TOP SIDE                         │
│                                                   │
│  ┌─────────────────────────────────┐              │
│  │   Pi Zero 2 W Footprint        │  [RFID HDR]  │
│  │   (2x20 pin header)            │              │
│  └─────────────────────────────────┘              │
│                                                   │
│  [USB-C]  [TP4056]  [Boost]  [BTN]  [LED] [LED] │
│                                                   │
│  (M2)                                      (M2)  │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│                 BOTTOM SIDE                       │
│                                                   │
│  [JST Battery Connector]                         │
│  [MAX17048 fuel gauge]                           │
│  [Decoupling caps]                               │
│                                                   │
│  (M2)                                      (M2)  │
└─────────────────────────────────────────────────┘
```

## Component Selection

| Block              | Component  | LCSC Part | Package   | Assembly |
| ------------------ | ---------- | --------- | --------- | -------- |
| LiPo Charger       | TP4056     | C16581    | SOP-8     | SMT      |
| Battery Protection | DW01A      | C351410   | SOT-23-6  | SMT      |
| Protection MOSFET  | FS8205     | C32254    | SOT-23-6  | SMT      |
| Boost Converter    | TPS61023   | C84773    | SOT-23-6  | SMT      |
| Load Switch        | TPS22918   | C130340   | SOT-23-5  | SMT      |
| Battery Gauge      | MAX17048   | C2682025  | DFN-8     | SMT      |
| USB-C Connector    | 16P SMD    | (search)  | SMD       | SMT      |
| JST Battery        | PH 2.0 2P | (search)  | THT       | Hand     |
| Pi Zero Header     | 2x20 2.54mm| —        | THT       | Hand     |
| Display Header     | Custom     | —         | THT       | Hand     |
| RFID Header        | 1x4 2.54mm| —         | THT       | Hand     |
| Push Button        | 6mm tactile| (search)  | THT       | Hand     |

## Schematic Blocks

### 1. USB-C Input

- USB Type-C 16-pin receptacle (power only)
- 5.1k resistors on CC1/CC2 for UFP identification (required for 5V)
- ESD protection TVS diode on VBUS

### 2. LiPo Charging (TP4056 + DW01A/FS8205)

- TP4056 linear charger IC — 500mA charge rate (2k PROG resistor)
- CHRG and STDBY status LEDs (red = charging, green = done)
- DW01A + FS8205 battery protection (over-discharge, over-charge, short circuit)

### 3. Boost Converter (TPS61023 — 3.7V to 5V)

- TPS61023 boost converter — high efficiency, clean output
- Input: 3.0-4.2V (LiPo range)
- Output: 5.0V stable
- 4.7uH shielded inductor (per datasheet)
- 10uF ceramic input/output capacitors
- Feedback resistor divider sets output voltage

### 4. Soft Power Switch (TPS22918)

- TPS22918 load switch between boost output and Pi 5V input
- Tactile push button for power on
- Pi GPIO holds power on after boot, releases for shutdown
- Enables clean software-controlled shutdown

### 5. Battery Monitoring (MAX17048)

- MAX17048 I2C fuel gauge — reports battery state-of-charge (%) directly
- Connected to Pi via I2C1 (GPIO2 SDA, GPIO3 SCL)
- No external ADC needed — simpler than voltage divider + MCP3008
- Optional ALRT pin for low-battery GPIO interrupt

### 6. Pi Zero 2 W Header

- 2x20 pin (40-pin) through-hole female header
- Pi mounts on top side
- Key GPIO allocations:
  - SPI0: Display (GPIO 7, 8, 10, 11, 18, 25, 27)
  - I2C1: Battery fuel gauge (GPIO 2, 3)
  - UART: RFID module (GPIO 14, 15) — future

### 7. Display Connector

- Header matching 3.5" SPI display pinout
- Display sits on top of the Pi Zero, connected via header pins
- Pin mapping depends on specific display model

### 8. RFID Header (Future)

- 4-pin header: VCC, GND, TX (GPIO14), RX (GPIO15)
- Positioned at board edge for easy access
- Not populated in v1

### 9. Mounting Holes

- 4x M2 mounting holes at corners
- Top 2 holes double as lanyard attachment points
- Hole diameter: 2.2mm, pad diameter: 4mm

## Fabrication & Ordering

### JLCPCB Order Settings

| Option          | Value                |
| --------------- | -------------------- |
| Layers          | 2                    |
| Quantity        | 5 (minimum)          |
| Thickness       | 1.6mm                |
| Solder mask     | Black                |
| Silkscreen      | White                |
| Surface finish  | HASL (lead-free)     |
| SMT Assembly    | Economic, top side   |

### Estimated Cost

| Item                | Cost       |
| ------------------- | ---------- |
| PCB fabrication (5) | ~$2-5      |
| SMT assembly        | ~$10-20    |
| Components (LCSC)   | ~$5-10     |
| Shipping            | ~$5-25     |
| **Total**           | **~$25-50** |

## Design Checklist

- [ ] USB-C with CC resistors for 5V detection
- [ ] TP4056 charging circuit with DW01A/FS8205 protection
- [ ] TPS61023 boost converter with stable 5V output
- [ ] TPS22918 soft power switch with push button
- [ ] MAX17048 battery fuel gauge on I2C
- [ ] 40-pin Pi Zero female header footprint
- [ ] SPI display header with correct pinout
- [ ] RFID UART header (future-proofing)
- [ ] Status LEDs (charge red, done green)
- [ ] M2 mounting holes with lanyard points
- [ ] Decoupling caps on all ICs
- [ ] I2C pull-up resistors (4.7k)
- [ ] Ground copper pour on bottom layer
- [ ] Board outline ~85x55mm
- [ ] Silkscreen labels for all connectors
- [ ] ERC passed (no errors)
- [ ] DRC passed (no errors)
