# Hardware Overview

## Chosen Platform: Raspberry Pi Compute Module 4 (CM4)

### Why CM4?

- Same Pi ecosystem and community support
- Modular design — choose carrier board with USB-C
- More powerful than Zero 2 W
- Full Linux compatibility
- WiFi + Bluetooth available

### CM4 Naming Convention

```
CM4 [RAM] [Wireless] [Storage]

Second digit:
  0 = No wireless
  1 = WiFi + Bluetooth ← You want this

Third section:
  000 = Lite (SD card)
  008/016/032 = eMMC size in GB

Examples:
  CM4102000 → 2GB RAM, Wireless, Lite (SD)  ← Budget pick
  CM4104000 → 4GB RAM, Wireless, Lite (SD)  ← Recommended
  CM4108032 → 8GB RAM, Wireless, 32GB eMMC  ← Overkill
```

### Recommended Purchase

**CM4102000** or **CM4104000** (Wireless + Lite variant)

- Lite = SD card slot, easier for development
- 2GB or 4GB RAM is sufficient for badge UI

## CM4 Requires a Carrier Board

The CM4 is just a compute module — no ports, no GPIO access. You need a carrier board.

### Requirements for Carrier Board

- USB-C power input
- GPIO breakout (for UPS, future RFID)
- DSI connector (for display)
- Compact form factor

### Recommended Carriers

| Board                   | USB-C | Size   | Notes                   |
| ----------------------- | ----- | ------ | ----------------------- |
| Waveshare CM4-Nano      | ✓     | Tiny   | Minimal, badge-friendly |
| Waveshare CM4-IO-BASE-B | ✓     | Medium | More GPIO, good for dev |
| Pimoroni CM4 IO         | ✓     | Medium | Quality build           |
