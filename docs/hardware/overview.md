# Hardware Overview

## Platform: ESP32-S3-WROOM-1-N16R8

### Module Specs

- **CPU:** Dual-core Xtensa LX7 @ 240MHz
- **SRAM:** 512KB
- **PSRAM:** 8MB Octal-SPI
- **Flash:** 16MB
- **Wireless:** WiFi 802.11 b/g/n + BLE 5.0
- **Antenna:** Integrated PCB antenna
- **Module size:** ~18x25mm
- **Unit cost:** ~$4-5

## Display: Waveshare 3.5" RPi LCD (F)

**Product:** [Waveshare 3.5inch RPi LCD (F)](https://www.waveshare.com/3.5inch-rpi-lcd-f.htm) (SKU: 30896)

| Spec | Value |
|------|-------|
| Resolution | 320 x 480 (portrait default) |
| Display driver | ST7796S |
| Display interface | 4-wire SPI |
| Display panel | IPS |
| Viewing angle | 170° |
| Color depth | 262K RGB |
| Touch driver | GT911 |
| Touch interface | I2C |
| Touch type | 5-point capacitive |
| Touch panel | Toughened glass |
| Operating voltage | 3.3V - 5V (onboard regulator, powered from 3.3V rail in this design) |
| Logic level | 3.3V |
| Display area | 49.36 x 73.84 mm |
| Panel size | 61.00 x 92.44 mm |
| Refresh rate | 60Hz |
| Connector | GH1.25 13-pin cable or Pigo pin header |

### Pin Interface (13-pin)

| Pin # | Name | Description |
|-------|------|-------------|
| 1 | TP_RST | Touch panel reset, low active |
| 2 | TP_INT | Touch panel interrupt |
| 3 | TP_SCL | Touch panel I2C clock |
| 4 | TP_SDA | Touch panel I2C data |
| 5 | LCD_BL | LCD backlight control |
| 6 | LCD_RST | LCD reset, low active |
| 7 | LCD_DC | LCD data/command select (high=data, low=command) |
| 8 | LCD_CS | LCD chip select, low active |
| 9 | SCLK | SPI clock |
| 10 | MOSI | SPI MOSI |
| 11 | MISO | SPI MISO |
| 12 | GND | Ground |
| 13 | VCC | Power input (3.3V - 5V) |

### Resources

- [ST7796S Datasheet](https://files.waveshare.com/wiki/common/ST7796S_Datasheet.pdf)
- [GT911 Datasheet](https://files.waveshare.com/wiki/common/GT911_Datasheet.pdf)
- [Display Schematic](https://files.waveshare.com/wiki/3.5inch%20RPi%20LCD%20(F)/3.5inch_RPi_LCD_(F).pdf)
- [Product Wiki](https://www.waveshare.com/wiki/3.5inch_RPi_LCD_(F))
- [ESP32-S3 Demo (Arduino)](https://files.waveshare.com/wiki/3.5inch_RPi_LCD_F/3inch5_RPI_LCD_F_ESP32S3.zip)

## PCB Design

### Component Summary

| Component | Part | Notes |
|-----------|------|-------|
| MCU | ESP32-S3-WROOM-1-N16R8 | Soldered directly to PCB |
| Display | Waveshare 3.5" RPi LCD (F) | Connected via GH1.25 13-pin cable |
| Charger | TP4056 | LiPo charging from USB-C 5V |
| Battery protection | DW01A + FS8205 | Over-discharge/over-current protection |
| Soft switch | TPS22918 | Load switch for 3.3V rail |
| LDO | AP2112K-3.3 | 3.3V regulation from LiPo (3.0-4.2V), 600mA |
| Fuel gauge | MAX17048 | Battery voltage/percentage via I2C |
| Connector | USB-C (16-pin) | Charging + programming (built-in USB-JTAG) |
| Button | Tactile switch | Power on/off |
| RFID header | UART pin header | Future Chameleon Tiny module |
| Battery | 3.7V 3000mAh LiPo pouch | |

### Power Path

```
USB-C 5V --> TP4056 --> LiPo 3.7V --> DW01A/FS8205 --> TPS22918 --> AP2112K --> 3.3V rail
                                                                                 |
                                                                                 +-- ESP32-S3
                                                                                 +-- ST7796S display
                                                                                 +-- GT911 touch
                                                                                 +-- MAX17048
```

The AP2112K-3.3 is a low-dropout regulator with 600mA output. Dropout voltage is ~250mV, so it works down to ~3.55V LiPo voltage. Below that, the battery is near empty anyway (3.0V cutoff from DW01A).

**Current budget note:** ESP32-S3 peak draw during WiFi TX is ~500mA. ST7796S backlight adds ~40-80mA, GT911 ~10mA, MAX17048 ~50uA. Combined peak may approach or exceed 600mA — verify during prototyping. If insufficient, consider upgrading to a higher-current regulator (e.g., AP2114H-3.3, 1A output).

### GPIO Pin Mapping

| Function | ESP32-S3 Pin | Interface | Notes |
|----------|-------------|-----------|-------|
| Display MOSI | GPIO 11 | SPI (FSPI) | FSPI default |
| Display SCLK | GPIO 12 | SPI (FSPI) | FSPI default |
| Display CS | GPIO 10 | SPI (FSPI) | FSPI default |
| Display DC | GPIO 9 | GPIO | Data/command select |
| Display RST | GPIO 8 | GPIO | Reset, low active |
| Display BL | GPIO 7 | PWM | Backlight brightness control |
| Touch SDA | GPIO 1 | I2C | Shared bus with MAX17048 |
| Touch SCL | GPIO 2 | I2C | Shared bus with MAX17048 |
| Touch INT | GPIO 3 | GPIO (input) | Interrupt, active low |
| Touch RST | GPIO 4 | GPIO | Reset, low active |
| Fuel gauge SDA | GPIO 1 | I2C | Shared bus with GT911 (addr: 0x36) |
| Fuel gauge SCL | GPIO 2 | I2C | Shared bus with GT911 (addr: 0x36) |
| UART TX (RFID) | GPIO 17 | UART1 | Future Chameleon Tiny |
| UART RX (RFID) | GPIO 18 | UART1 | Future Chameleon Tiny |
| Power button | GPIO 5 | GPIO (input) | Interrupt for wake/sleep |
| USB D+ | GPIO 20 | USB-JTAG | Built-in |
| USB D- | GPIO 19 | USB-JTAG | Built-in |

**I2C bus sharing:** The GT911 touch controller (default address: 0x5D) and MAX17048 fuel gauge (address: 0x36) share the same I2C bus on GPIO 1/2. No address conflict.

Pin assignments are preliminary and subject to change during PCB layout to optimize routing.

## BOM Estimate

| Category | Item | Cost |
|----------|------|------|
| MCU | ESP32-S3-WROOM-1-N16R8 | $4-5 |
| Display | Waveshare 3.5" RPi LCD (F) | ~$20 |
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
| **Total (without RFID)** | | **~$45-70** |
| RFID (future) | Chameleon Tiny module | $35 |
| **Total (with RFID)** | | **~$80-105** |
