# Hardware Overview

## Platform: ESP32-S3-WROOM-1U

### Module Specs

- **CPU:** Dual-core Xtensa LX7 @ 240MHz
- **SRAM:** 512KB
- **PSRAM:** 8MB Octal-SPI
- **Flash:** 16MB
- **Wireless:** WiFi 802.11 b/g/n + BLE 5.0
- **Antenna:** U.FL connector (external antenna)
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
| MCU | ESP32-S3-WROOM-1U | Soldered directly to PCB, U.FL antenna |
| Display | Waveshare 3.5" RPi LCD (F) | Connected via GH1.25 13-pin cable |
| Charger | TP4056 (ESOP-8) | LiPo charging from USB-C 5V |
| Battery protection | DW01A + FS8205A | Over-discharge/over-current protection |
| LDO | ME6211C33M5 | 3.3V regulation from LiPo (3.0-4.2V), 500mA |
| Fuel gauge | MAX17048G+T10 | Battery voltage/percentage via I2C |
| Connector | USB-C (16-pin) | Charging + programming (built-in USB-JTAG) |
| Buttons | 2x tactile switch | BOOT (IO0) + RESET (EN) |
| LEDs | 2x LED (D1, D2) | Charge (CHG) and standby (STDBY) indicators |
| Battery | 3.7V 3000mAh LiPo pouch | |

### Power Path

```
USB-C 5V --> TP4056 --> LiPo 3.7V --> DW01A/FS8205A --> ME6211 --> 3.3V rail
                                                                      |
                                                                      +-- ESP32-S3
                                                                      +-- ST7796S display
                                                                      +-- GT911 touch
                                                                      +-- MAX17048
```

The ME6211C33M5 is a low-dropout regulator with 500mA output. Dropout voltage is ~100mV at light load, so it works down to ~3.4V LiPo voltage. Below that, the battery is near empty anyway (3.0V cutoff from DW01A).

**Current budget note:** ESP32-S3 peak draw during WiFi TX is ~500mA. ST7796S backlight adds ~40-80mA, GT911 ~10mA, MAX17048 ~50uA. Combined peak may reach or exceed 500mA — verify during prototyping. If insufficient, consider upgrading to a higher-current regulator.

### GPIO Pin Mapping

| Function | ESP32-S3 Pin | Interface | Notes |
|----------|-------------|-----------|-------|
| BOOT | IO0 | GPIO | Boot mode select (SW1) |
| I2C SDA | IO1 | I2C | Shared: GT911 touch + MAX17048 |
| I2C SCL | IO2 | I2C | Shared: GT911 touch + MAX17048 |
| Touch INT | IO3 | GPIO (input) | Interrupt, active low |
| Touch RST | IO4 | GPIO | Reset, low active |
| Display BL | IO7 | PWM | Backlight brightness control |
| Display RST | IO8 | GPIO | Reset, low active |
| Display DC | IO9 | GPIO | Data/command select |
| Display CS | IO10 | SPI | Chip select, low active |
| Display MOSI | IO11 | SPI | SPI data out |
| Display SCLK | IO12 | SPI | SPI clock |
| Display MISO | IO13 | SPI | SPI data in |
| USB D- | USB_D- | USB-JTAG | Built-in USB (pin 13) |
| USB D+ | USB_D+ | USB-JTAG | Built-in USB (pin 14) |
| EN | EN | — | R10 10k pull-up + C9 100nF RC + SW2 RESET |

**I2C bus sharing:** The GT911 touch controller (default address: 0x5D) and MAX17048 fuel gauge (address: 0x36) share the same I2C bus on IO1/IO2. No address conflict. Pull-ups: R8 4.7k (SCL), R9 4.7k (SDA).

**Unused pins:** IO5, IO6, IO14-IO18, IO21, IO35-IO48, TXD0, RXD0 are marked no-connect.

## BOM Estimate

| Category | Item | Cost |
|----------|------|------|
| MCU | ESP32-S3-WROOM-1U | $4-5 |
| Display | Waveshare 3.5" RPi LCD (F) | ~$20 |
| Battery | 3.7V 3000mAh LiPo pouch | $5-8 |
| Charger IC | TP4056 (ESOP-8) | $0.20 |
| Protection | DW01A + FS8205A | $0.30 |
| LDO | ME6211C33M5 | $0.20 |
| Fuel gauge | MAX17048G+T10 | $2-3 |
| Connector | USB-C (16-pin) | $0.30 |
| Buttons | 2x tactile switch (BOOT + RESET) | $0.20 |
| LEDs | 2x LED + resistors (CHG/STDBY) | $0.20 |
| Passives | Resistors, capacitors, etc. | $1-2 |
| PCB fab | JLCPCB 2-layer + SMT assembly | $10-30 |
| **Total** | | **~$45-70** |
