# Shopping List

## Core Components

| Component          | Specific Model                          | Est. Price |
| ------------------ | --------------------------------------- | ---------- |
| Compute            | Raspberry Pi Zero 2 W                   | ~$15       |
| Display            | Waveshare 3.5" SPI Touch (ILI9486)      | ~$15-25    |
| Storage            | 32GB microSD (A2 rated)                 | ~$10       |
| Battery            | 3.7V LiPo pouch cell (3000mAh)         | ~$8-12     |
| Future: RFID       | Chameleon Tiny                          | ~$35       |

## PCB Components (on custom board)

| Component          | Part / Value                            | Est. Price |
| ------------------ | --------------------------------------- | ---------- |
| Charging IC        | TP4056 (USB-C LiPo charger module)      | ~$1        |
| Boost converter    | MT3608 / TPS61023 (3.7V -> 5V)         | ~$1-2      |
| USB-C connector    | USB Type-C 16-pin receptacle            | ~$0.50     |
| Power switch       | SPDT slide switch                       | ~$0.20     |
| Battery connector  | JST PH 2.0mm 2-pin                     | ~$0.20     |
| SPI display header | 2.54mm female header (26-pin or custom) | ~$0.50     |
| Pi Zero header     | 2x20 pin header (40-pin)               | ~$1        |
| RFID header        | 4-pin header (UART/USB breakout)        | ~$0.30     |
| Protection         | Battery protection MOSFET + fuse        | ~$0.50     |
| Capacitors         | Decoupling caps (100nF, 10uF)           | ~$0.30     |
| LEDs               | Charging status indicator (red/green)   | ~$0.20     |
| Resistors          | Assorted (voltage divider, LED limit)   | ~$0.20     |

## PCB Fabrication

| Item               | Details                                 | Est. Price |
| ------------------ | --------------------------------------- | ---------- |
| PCB fabrication    | JLCPCB / PCBWay (5 pcs minimum)        | ~$5-10     |
| SMD assembly       | Optional JLCPCB SMT service             | ~$10-20    |
| Stencil            | Optional solder paste stencil           | ~$5        |

## Cost Summary

| Category                  | Estimated Total |
| ------------------------- | --------------- |
| Core components           | ~$48-62         |
| PCB components            | ~$5-7           |
| PCB fabrication            | ~$10-30         |
| **Total (without RFID)**  | **~$65-100**    |
| **Total (with RFID)**     | **~$100-135**   |

Significant cost reduction from the CM4 design (~$180-220) due to:
- Pi Zero 2 W ($15) vs CM4 + carrier ($85-95)
- No separate UPS HAT ($15-25 saved)
- No acrylic enclosure ($20 saved)
- SPI display cheaper than DSI display
