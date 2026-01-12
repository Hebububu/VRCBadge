# Digital Conference Badge Documentation

A wearable digital badge with dynamic display, VRChat avatar sync, and future RFID door key emulation.

## Documentation

### [Hardware](./hardware/)
- [Overview](./hardware/overview.md) - CM4 selection, carrier boards, platform decisions
- [Shopping List](./hardware/shopping-list.md) - Components and estimated prices
- [Assembly](./hardware/assembly.md) - Hardware stack diagram, weight/thermal/battery considerations

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
- [CM4 Datasheet](https://www.raspberrypi.com/documentation/computers/compute-module.html)
- [Chameleon Tiny Wiki](https://github.com/RfidResearchGroup/ChameleonUltra)
