<div align="center">

# VRCBadge

<img src="./images/rfid-namecard.png" alt="RFID Namecard">

I made RFID namecard before (for friends and myself)<br>
and you know, all VRC players change avatar quite often.

After making those namecards 3 times (dumb),<br>
I decided to make a digital badge for myself. (simple + fun toy project!)

It's gonna be a wearable digital badge powered by ESP32-S3<br>
Namecard looking UI (Yes, changeable image ofc)<br>
editable profile information (twitter, discord etc..), and future RFID door key emulation.

This project is inspired by Github Conference Badge video, posted by Wes Bos. [Ref Video](https://youtu.be/_jPm_zN95FE?si=vHjaHDR8lpRF7iKP)

### Working Prototype

<img src="./images/guiton_jc8048w550.jpg" alt="Badge running on Guition JC8048W550" width="720">

Badge UI running on a Guition JC8048W550 dev board (ESP32-S3 + 4.8" RGB parallel LCD with capacitive touch). Built with [Slint](https://slint.dev).

### Enclosure (WIP)

<img src="./images/enclosure_wip.jpg" alt="Enclosure WIP" width="720">
<img src="./images/hardware/front.png" alt="Enclosure front render" width="720">
<img src="./images/hardware/back.png" alt="Enclosure back render" width="720">

Enclosure drawn with Autodesk Fusion 360.

### Schematic (Future)

<img src="./images/schematic.png" alt="Schematic" width="720">

Custom 2-layer PCB designed for JLCPCB fabrication.
USB-C input with TP4056 LiPo charging, DW01A+FS8205A battery protection, and ME6211 3.3V LDO.
ESP32-S3-WROOM-1U drives a 3.5" SPI LCD with capacitive touch, while a MAX17048 fuel gauge tracks battery level over I2C.

</div>

## Documentation

See [/docs](./docs/README.md) for full documentation:

- [Hardware](./docs/hardware/) - ESP32-S3, custom PCB, display, BOM
- [Software](./docs/software/) - Architecture, API
- [RFID](./docs/rfid/) - Technical guide (future phase)
