# PCB Design (EasyEDA)

## Overview

<!-- TODO: Add EasyEDA schematic screenshot and PCB layout render -->
![PCB Design](../../images/pcb-design-placeholder.png)

*Image pending — will be added after EasyEDA design is complete.*

A custom PCB replaces both the carrier board and acrylic enclosure from the original CM4 design. The PCB serves as the structural backbone of the badge, mounting all components in a compact, wearable package.

## Design Tool

**EasyEDA** (https://easyeda.com) — free, browser-based PCB design tool with direct JLCPCB integration for ordering.

## PCB Requirements

### Board Specs

| Parameter      | Value                          |
| -------------- | ------------------------------ |
| Dimensions     | ~85 x 55mm (credit card size)  |
| Layers         | 2-layer (top + bottom)         |
| Thickness      | 1.6mm standard                 |
| Copper weight  | 1oz                            |
| Surface finish | HASL or ENIG                   |
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
│  [USB-C]  [TP4056]  [Boost]  [SW]  [LED] [LED]  │
│                                                   │
│  (M2)                                      (M2)  │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│                 BOTTOM SIDE                       │
│                                                   │
│  [JST Battery Connector]                         │
│  [Voltage divider (battery monitor)]             │
│  [Decoupling caps]                               │
│                                                   │
│  (M2)                                      (M2)  │
└─────────────────────────────────────────────────┘
```

## Schematic Blocks

### 1. USB-C Input

- USB Type-C 16-pin receptacle (power only, CC resistors for 5V)
- 5.1k resistors on CC1/CC2 for UFP identification
- ESD protection TVS diode on VBUS

### 2. LiPo Charging (TP4056)

- TP4056 linear charger IC
- Charge current set resistor (R_prog): 2k for ~500mA charge rate
- CHRG and STDBY status LEDs (red = charging, green = done)
- DW01A + FS8205 battery protection (over-discharge, over-charge, short circuit)

### 3. Boost Converter (3.7V to 5V)

- MT3608 or TPS61023 boost converter
- Input: 3.0-4.2V (LiPo range)
- Output: 5.0V stable
- Inductor: 22uH (per datasheet recommendation)
- Input/output capacitors: 22uF ceramic
- Feedback resistors for 5V output

### 4. Power Switch

- SPDT slide switch between boost output and Pi 5V input
- Alternatively: soft power via MOSFET + push button (for cleaner UX)

### 5. Battery Monitoring

- Voltage divider (100k / 100k) from battery to GPIO ADC pin
- Pi Zero 2 W has no hardware ADC — use MCP3008 SPI ADC or software-based GPIO measurement
- Alternative: fuel gauge IC (MAX17048) via I2C for accurate SoC reading

### 6. Pi Zero 2 W Header

- 2x20 pin (40-pin) through-hole header footprint
- Pi mounts on top side, components on bottom where possible
- Key GPIO allocations:
  - SPI0: Display (GPIO 7, 8, 10, 11, 18, 25, 27)
  - I2C1: Battery fuel gauge (GPIO 2, 3) — if using MAX17048
  - UART: RFID module (GPIO 14, 15) — future

### 7. Display Connector

- 26-pin or custom header matching the 3.5" SPI display pinout
- Display sits on top of the Pi Zero, connected via header pins
- Ensure correct pin mapping from display to Pi GPIO

### 8. RFID Header (Future)

- 4-pin header: VCC, GND, TX, RX (UART)
- Or USB breakout if Chameleon Tiny uses USB
- Positioned at board edge for easy access

### 9. Mounting Holes

- 4x M2 mounting holes at corners
- Top 2 holes double as lanyard attachment points
- Hole diameter: 2.2mm, pad diameter: 4mm

## EasyEDA Workflow

1. **Schematic**: Draw each block, connect nets, assign footprints
2. **PCB Layout**: Place components, route traces, define board outline
3. **Design Rules**: Min trace 0.2mm, min clearance 0.2mm, min via 0.3mm
4. **Generate Gerber**: Export for fabrication
5. **Order**: Direct JLCPCB integration from EasyEDA — upload Gerber, select options, order

## Fabrication

### Recommended: JLCPCB

- 5 pcs minimum order: ~$2-5 for the boards
- SMT assembly available: upload BOM + pick-and-place file
- Shipping: ~$5-15 (economy) or ~$15-25 (express)
- Turnaround: 3-5 days fabrication + shipping

### Assembly Options

| Option              | Cost     | Difficulty |
| ------------------- | -------- | ---------- |
| Hand solder all     | $0       | Medium     |
| JLCPCB SMT + hand   | ~$10-20  | Easy       |
| Full JLCPCB assembly | ~$20-40  | Easiest    |

Hand soldering is feasible — the TP4056, boost converter, and passives are all available in hand-solderable packages (SOT-23, 0805 passives). The USB-C connector requires careful soldering or hot air.

## Design Checklist

- [ ] USB-C with CC resistors for 5V detection
- [ ] TP4056 charging circuit with protection
- [ ] Boost converter with stable 5V output
- [ ] Power switch (slide or soft-power)
- [ ] Battery voltage monitoring circuit
- [ ] 40-pin Pi Zero header footprint
- [ ] SPI display header with correct pinout
- [ ] RFID UART header (future-proofing)
- [ ] Status LEDs (charge, power)
- [ ] M2 mounting holes with lanyard points
- [ ] Decoupling caps on all ICs
- [ ] Board outline matches display size (~85x55mm)
- [ ] Silkscreen labels for all connectors
