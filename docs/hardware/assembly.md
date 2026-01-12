# Hardware Assembly

## Complete Hardware Stack

```mermaid
flowchart TB
    subgraph Badge["Badge Assembly"]
        Display["Display (4-5\" DSI touchscreen)"]
        subgraph Carrier["Carrier Board (USB-C)"]
            CM4["CM4 (Wireless variant)"]
        end
        UPS["UPS Module + 18650 Battery"]
        RFID["Chameleon Tiny (RFID emulator)"]
        Enclosure["Self-designed acrylic enclosure + Lanyard attachment"]
    end

    Display -->|DSI| Carrier
    Carrier -->|GPIO| UPS
    UPS -->|"USB (future)"| RFID
```

## Considerations

### Weight Warning

7" display = 150-250g. Combined with battery and boards = 350-450g on neck.

**Recommendation:** Use 4-5" display for all-day comfort.

### Thermal

CM4 runs warm. Include heatsink. Ensure enclosure has ventilation.

### Battery Life Estimate

- 5" display + CM4 + WiFi active ≈ 3-5 hours on single 18650
- With aggressive power management ≈ 6-8 hours
- Consider dual 18650 if space allows

### Display Connection Priority

DSI > HDMI > SPI

- DSI: cleanest, single ribbon cable, touch integrated
- HDMI: bulky adapter needed
- SPI: slow refresh, CPU overhead
