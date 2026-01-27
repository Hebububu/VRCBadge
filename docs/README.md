# Digital Conference Badge Documentation

A wearable digital badge with dynamic display, VRChat avatar sync, and future RFID door key emulation.

## Documentation

### [Hardware](./hardware/)
- [Overview](./hardware/overview.md) - Pi Zero 2 W platform, 3.5" SPI display
- [Shopping List](./hardware/shopping-list.md) - Components and estimated prices
- [Assembly](./hardware/assembly.md) - Hardware stack diagram, weight/thermal/battery considerations
- [PCB Design](./hardware/pcb-design.md) - Custom PCB via EasyEDA, schematic blocks, fabrication
- [PCB Guideline](./hardware/pcb-guideline.md) - Step-by-step PCB design walkthrough for beginners

### [Software](./software/)
- [Architecture](./software/architecture.md) - Tech stack, system architecture diagram
- [API](./software/api.md) - HTTP endpoints documentation

### [Development](./development/)
- [Setup](./development/setup.md) - Cross-compilation setup
- [Deployment](./development/deployment.md) - Deploy script, systemd service
- [Roadmap](./development/roadmap.md) - Development phases and milestones

### [RFID](./rfid/)
- [Technical Guide](./rfid/technical-guide.md) - RFID frequencies, reader vs emulator, Chameleon

## Resources

- [Slint Documentation](https://slint.dev/docs)
- [rppal GPIO Library](https://github.com/golemparts/rppal)
- [Axum Web Framework](https://github.com/tokio-rs/axum)
- [Pi Zero 2 W Datasheet](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#raspberry-pi-zero-2-w)
- [EasyEDA PCB Design](https://easyeda.com)
- [JLCPCB Fabrication](https://jlcpcb.com)
- [Chameleon Tiny Wiki](https://github.com/RfidResearchGroup/ChameleonUltra)
