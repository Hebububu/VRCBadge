# Software Architecture

## Tech Stack

| Layer         | Choice     | Rationale                            |
| ------------- | ---------- | ------------------------------------ |
| Language      | Rust       | Developer preference, performance    |
| Async Runtime | Tokio      | Industry standard                    |
| UI Framework  | Slint      | Designed for embedded, touch support |
| HTTP Server   | Axum       | Lightweight, async, Rust-native      |
| GPIO/SPI      | rppal      | Best Pi GPIO library for Rust        |
| Serial (RFID) | serialport | For Chameleon communication          |

## System Architecture

```mermaid
flowchart TB
    subgraph Software["Badge Software"]
        subgraph UI["Slint UI Layer"]
            Avatar["Avatar display"]
            Profile["Profile info"]
            Touch["Touch interactions"]
            Status["Battery/WiFi status"]
        end

        subgraph Runtime["Tokio Async Runtime"]
            Axum["Axum API :3000"]
            RFID["RFID Svc (future)"]
            Monitor["System Monitor (battery etc)"]
        end
    end

    UI <--> Runtime
```
