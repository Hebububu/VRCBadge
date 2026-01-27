# PCB Design Guideline

A step-by-step guide for designing the VRCBadge custom PCB in EasyEDA Pro, from schematic to JLCPCB order.

> **Target audience:** First-time PCB designer.
> **Tool:** EasyEDA Pro (desktop app).
> **Fabrication:** JLCPCB with SMT assembly for SMD parts, hand-solder through-hole parts.

## Prerequisites

1. **Install EasyEDA Pro** — https://easyeda.com/page/download
2. **Create a JLCPCB account** — https://jlcpcb.com (same login works for EasyEDA and LCSC)
3. **No need to buy components separately** — JLCPCB/LCSC parts library is built into EasyEDA Pro. Pick parts from their catalog and they source + solder them.

### Recommended Learning Resources

Before starting your design, spend 1-2 hours on these:

- Official EasyEDA Pro documentation: https://docs.easyeda.com/en/
- Search YouTube for "EasyEDA Pro tutorial beginner" — many good walkthroughs
- Search YouTube for "TP4056 PCB design" — the charging circuit is well-documented
- TI WEBENCH for boost converter design: https://www.ti.com/design-resources/design-tools-simulation/webench-power-designer.html

---

## Finalized Component Decisions

| Block             | Component      | LCSC Part   | Rationale                                          |
| ----------------- | -------------- | ----------- | -------------------------------------------------- |
| LiPo Charger      | TP4056         | C16581      | Ubiquitous, well-documented, cheap                 |
| Battery Protection | DW01A         | C351410     | Standard LiPo protection IC                        |
| Protection MOSFET | FS8205         | C32254      | Dual N-MOSFET pair for DW01A                       |
| Boost Converter   | TPS61023       | C84773      | Clean 5V output, high efficiency, good for display |
| Load Switch       | TPS22918       | C130340     | Soft power on/off, clean shutdown support          |
| Battery Gauge     | MAX17048       | C2682025    | I2C fuel gauge, no ADC needed, reports % directly  |
| USB-C Connector   | USB-C 16P SMD  | (search)    | Search LCSC for "USB Type-C 16P SMD"              |
| Battery Connector | JST PH 2.0 2P | (search)    | Standard LiPo battery connector                   |

> **Note:** LCSC part numbers can change. Always verify availability on https://www.lcsc.com before finalizing your design. Prefer "Basic" or "Extended" parts — JLCPCB stocks these and charges less for assembly.

---

## Phase 1: Schematic Design

Create a new project in EasyEDA Pro, then draw the schematic block by block. Each block below includes the circuit description, key parts, and tips.

### General Schematic Tips

- **Use net labels** instead of drawing wires everywhere. Label a wire "5V", "BAT+", or "GND" and EasyEDA connects them automatically across the sheet.
- **Add power flags** to VCC and GND nets to avoid ERC warnings.
- **One block at a time** — draw, verify, then move to the next.
- **Copy reference designs** — every IC datasheet has a "Typical Application" schematic. Use it exactly unless you have a reason to deviate.

---

### Block 1: USB-C Power Input

Provides 5V power from any USB-C charger.

```
USB-C Receptacle
  VBUS (pin A4/B4) ──> 5V_USB net
  GND  (pin A1/B1) ──> GND net
  CC1  (pin A5)     ──> 5.1k resistor ──> GND
  CC2  (pin B5)     ──> 5.1k resistor ──> GND
  Shield            ──> GND
```

**Parts:**
- USB Type-C 16-pin SMD receptacle (search LCSC: "USB-C 16P")
- 2x 5.1k resistors (0805 package)
- Optional: TVS diode on VBUS for ESD protection

**Why CC resistors?**
The 5.1k resistors on CC1/CC2 identify the device as a "USB sink" (power consumer). Without them, USB-C chargers will not provide power.

**Tips:**
- Only VBUS, GND, CC1, CC2, and Shield are needed. Leave data pins (D+/D-) unconnected — this is power-only.
- USB-C receptacles have pins on both sides (A and B). Connect both VBUS pins together, both GND pins together.

---

### Block 2: LiPo Charging (TP4056 + DW01A Protection)

Safely charges the LiPo battery from 5V USB input.

```
5V_USB ──> TP4056 (VCC pin)
             │
             ├── PROG pin ──> 2k resistor ──> GND     (sets 500mA charge current)
             ├── CHRG pin ──> 1k resistor ──> LED red ──> VCC   (charging indicator)
             ├── STDBY pin ──> 1k resistor ──> LED green ──> VCC (fully charged)
             ├── BAT pin  ──> BAT+ net (to protection circuit)
             ├── CE pin   ──> VCC (chip enable, always on)
             ├── TEMP pin ──> 10k NTC or tie to GND via 10k (disable temp sense)
             └── GND pin  ──> GND

BAT+ ──> DW01A + FS8205 protection circuit ──> B+ / B- (to JST battery connector)
```

**Parts:**
- TP4056 (LCSC: C16581) — SOP-8 package
- DW01A (LCSC: C351410) — SOT-23-6 package
- FS8205 (LCSC: C32254) — SOT-23-6 dual MOSFET
- 2k resistor (0805) — sets 500mA charge current via PROG pin
- 1k resistors (0805) x2 — for LED current limiting
- Red + green LEDs (0805)
- 10k resistor (0805) — for TEMP pin (disable temperature sensing)
- 1uF capacitor (0805) — input decoupling on VCC
- 1uF capacitor (0805) — output decoupling on BAT

**Charge current reference (PROG resistor):**

| R_PROG | Charge Current |
| ------ | -------------- |
| 10k    | 100mA          |
| 5k     | 200mA          |
| 2k     | 500mA          |
| 1.2k   | 1000mA         |

500mA is recommended — safe for most 3000mAh LiPo cells and within USB-C 5V/500mA default.

**DW01A + FS8205 protection circuit:**

This is a standard circuit — search "DW01A FS8205 schematic" for reference. It protects against:
- Over-charge (>4.3V)
- Over-discharge (<2.4V)
- Short circuit

**Tips:**
- The TP4056 reference circuit is one of the most replicated designs in hobby electronics. Search for "TP4056 schematic" and you will find hundreds of verified examples.
- The CHRG and STDBY LEDs are connected to VCC (not GND) because these pins sink current when active.

---

### Block 3: Boost Converter (TPS61023 — 3.7V to 5V)

Steps up the 3.7V battery voltage to stable 5V for the Pi.

```
BAT+ (3.0-4.2V) ──> TPS61023
                       │
                       ├── VIN pin   ──> BAT+ (with 10uF input cap to GND)
                       ├── EN pin    ──> VIN (always enabled) or to load switch
                       ├── SW pin    ──> 4.7uH inductor ──> VOUT
                       ├── VOUT pin  ──> 5V_BOOST net (with 10uF output cap to GND)
                       ├── FB pin    ──> voltage divider midpoint
                       ├── PG pin    ──> optional power-good signal (or leave NC)
                       └── GND pin   ──> GND (exposed pad also GND)
```

**Voltage divider for 5V output (FB pin):**
```
VOUT ──> R1 (750k) ──> FB pin ──> R2 (150k) ──> GND

V_OUT = 0.5V x (1 + R1/R2) = 0.5 x (1 + 750/150) = 0.5 x 6 = 3.0V
```

> **Important:** Check the TPS61023 datasheet for exact recommended R1/R2 values for 5V output. The internal reference is 500mV. Use TI WEBENCH tool to verify.

**Parts:**
- TPS61023 (LCSC: C84773) — SOT-23-6 or similar small package
- 4.7uH shielded inductor (check datasheet for recommended part)
- 10uF ceramic capacitors x2 (0805 or 1206) — input and output
- 750k + 150k resistors (0402 or 0603) — feedback divider
- 100nF ceramic capacitor (0805) — bypass on VIN

**Tips:**
- Keep the inductor, input cap, and output cap physically close to the IC on the PCB. Long traces cause noise and reduce efficiency.
- Use the exact inductor value and type recommended in the datasheet. Inductors are not interchangeable like resistors.
- The exposed thermal pad on the bottom of the IC must be soldered to GND.
- Use TI WEBENCH (https://www.ti.com/design-resources/design-tools-simulation/webench-power-designer.html) to verify your design and get recommended component values.

---

### Block 4: Soft Power Switch (TPS22918 Load Switch)

Controls power to the Pi via a push button. Allows clean software shutdown.

```
5V_BOOST ──> TPS22918 (VIN)
               │
               ├── ON pin  ──> push button circuit (active high)
               ├── VOUT    ──> 5V_PI net (to Pi Zero header pin 2/4)
               └── GND     ──> GND

Push button latch circuit:
  Button ──> pull-down resistor (100k to GND)
  Button ──> ON pin
  Pi GPIO ──> ON pin (software can hold power on after boot)
```

**Parts:**
- TPS22918 (LCSC: C130340) — SOT-23-5 package
- Tactile push button (6mm SMD or through-hole)
- 100k pull-down resistor (0805)
- 100nF bypass cap on VIN

**How it works:**
1. Press button -> ON pin goes high -> TPS22918 connects VIN to VOUT -> Pi boots
2. Pi GPIO pin drives ON high -> holds power on after button release
3. To shut down: Pi runs shutdown sequence, then releases GPIO -> power cuts

**Tips:**
- This is a simple load switch — the TPS22918 datasheet has a clear reference design.
- If this feels too complex for v1, you can substitute a simple SPDT slide switch between 5V_BOOST and 5V_PI. Just change the footprint and skip the push button circuit.

---

### Block 5: Battery Fuel Gauge (MAX17048 via I2C)

Reports battery state-of-charge (%) over I2C. No external ADC needed.

```
BAT+ ──> MAX17048
           │
           ├── CELL pin ──> BAT+ (battery voltage sense)
           ├── VDD pin  ──> 3.3V (from Pi header pin 1)
           ├── SDA pin  ──> Pi GPIO2 (I2C1 SDA) with 4.7k pull-up to 3.3V
           ├── SCL pin  ──> Pi GPIO3 (I2C1 SCL) with 4.7k pull-up to 3.3V
           ├── ALRT pin ──> Pi GPIO (optional, low battery interrupt)
           ├── QSTRT pin──> leave unconnected or tie to GND
           └── GND pin  ──> GND
```

**Parts:**
- MAX17048 (LCSC: C2682025) — DFN-8 package (small, needs reflow or hot air)
- 2x 4.7k pull-up resistors (0805) — for I2C bus
- 100nF bypass cap on VDD
- 1uF cap on CELL pin (per datasheet)

**Tips:**
- The MAX17048 uses a ModelGauge algorithm — it learns battery behavior over time. No calibration needed.
- I2C address is fixed at 0x36.
- Read the VCELL register for voltage, SOC register for state-of-charge (%).
- The ALRT pin can trigger a GPIO interrupt when battery drops below a threshold — useful for showing low battery warning on the badge display.

---

### Block 6: Pi Zero 2W Header

The Pi Zero plugs into the PCB via a 40-pin header.

```
2x20 female header (2.54mm pitch)
  Pin 1  (3.3V)    ──> 3V3 net (power for MAX17048, pull-ups)
  Pin 2  (5V)      ──> 5V_PI net (from load switch)
  Pin 3  (GPIO2)   ──> I2C1_SDA net
  Pin 4  (5V)      ──> 5V_PI net
  Pin 5  (GPIO3)   ──> I2C1_SCL net
  Pin 6  (GND)     ──> GND
  Pin 7  (GPIO7)   ──> TOUCH_CS net (display touch chip select)
  Pin 8  (GPIO14)  ──> UART_TX net (RFID, future)
  Pin 10 (GPIO15)  ──> UART_RX net (RFID, future)
  Pin 12 (GPIO18)  ──> DISP_BL net (display backlight)
  Pin 19 (GPIO10)  ──> SPI0_MOSI net (display data)
  Pin 22 (GPIO25)  ──> DISP_DC net (display data/command)
  Pin 23 (GPIO11)  ──> SPI0_SCLK net (display clock)
  Pin 24 (GPIO8)   ──> DISP_CS net (display chip select)
  Pin 13 (GPIO27)  ──> DISP_RST net (display reset)
  GND pins (9, 14, 20, 25, 30, 34, 39) ──> GND
```

**Parts:**
- 2x20 pin female header, 2.54mm pitch, through-hole
- **Do not place via SMT assembly** — mark as "hand solder" in BOM

**Tips:**
- Double-check the Pi Zero 2W GPIO pinout against the Raspberry Pi official documentation: https://pinout.xyz
- Not all 40 pins are used. Only connect the ones needed for SPI display, I2C, UART, and power.
- Other pins can be left as pass-through (connected to nothing on the PCB).

---

### Block 7: Display Connector

The 3.5" SPI display connects via a header on top of the Pi.

```
26-pin or custom female header
  Route SPI + control signals from Pi GPIO to display header pins.
  Pin mapping depends on your specific display model.
```

**Parts:**
- Female header matching your display's connector (check display datasheet)
- Through-hole, hand-soldered

**Tips:**
- Many 3.5" SPI displays (like Waveshare) are designed to plug directly onto the Pi's 40-pin header. If using one of these, you may not need a separate display connector — the display plugs directly into the Pi header that's already on your PCB.
- If the display doesn't cover all 40 pins, you can use a shorter header (26-pin) and only route the needed signals.
- Verify your display's pinout before routing. Different manufacturers use different pin assignments.

---

### Block 8: RFID Header (Future)

A 4-pin header for the Chameleon Tiny RFID module. Place the footprint now, populate later.

```
4-pin header (2.54mm):
  Pin 1: VCC (3.3V or 5V, check Chameleon Tiny requirements)
  Pin 2: GND
  Pin 3: GPIO14 (UART TX)
  Pin 4: GPIO15 (UART RX)
```

**Parts:**
- 1x4 pin male or female header, 2.54mm, through-hole
- Not populated in v1

**Tips:**
- Place at the board edge for easy access.
- If the Chameleon Tiny uses USB instead of UART, you may need a different connector. The 4-pin UART header is a safe starting point.

---

### Block 9: Decoupling & Passives

Every IC needs bypass capacitors close to its power pins.

| IC         | Cap Value | Cap Location     |
| ---------- | --------- | ---------------- |
| TP4056     | 1uF       | VCC pin, BAT pin |
| TPS61023   | 10uF      | VIN, VOUT        |
| TPS22918   | 100nF     | VIN              |
| MAX17048   | 100nF     | VDD              |
| MAX17048   | 1uF       | CELL pin         |

Also add:
- 100nF cap between 3.3V and GND near the Pi header
- 10uF bulk cap on the 5V rail

---

### Schematic Completion Checklist

After drawing all blocks:

- [ ] All IC pins are connected or explicitly marked as NC (no connect)
- [ ] All power nets are labeled (5V_USB, BAT+, 5V_BOOST, 5V_PI, 3V3, GND)
- [ ] All signal nets are labeled (SPI0_MOSI, SPI0_SCLK, DISP_CS, I2C1_SDA, etc.)
- [ ] Every IC has decoupling caps on power pins
- [ ] I2C bus has 4.7k pull-up resistors to 3.3V
- [ ] USB-C has 5.1k CC resistors
- [ ] LED current limiting resistors are present
- [ ] Run ERC (Electrical Rules Check) — fix all errors, review all warnings

---

## Phase 2: Assign Footprints & Generate PCB

1. Every schematic symbol needs a physical **footprint** — the pad pattern for soldering
2. Parts from LCSC already have footprints assigned in EasyEDA Pro
3. For through-hole parts (headers, JST connector, button), assign footprints manually
4. Click **"Design" > "Update PCB"** to transfer the schematic to the PCB editor

---

## Phase 3: PCB Layout

### Step 1: Set Board Outline

- Draw a rectangle: **85mm x 55mm** (approximately credit card size)
- Round corners with **2mm radius** for comfort and aesthetics
- Add **4x M2 mounting holes** at corners:
  - Drill diameter: 2.2mm
  - Pad diameter: 4mm
  - Top 2 holes double as lanyard attachment points

### Step 2: Place Components

Place in this order (power path first, then signals):

| Order | Component         | Location                | Side   |
| ----- | ----------------- | ----------------------- | ------ |
| 1     | USB-C connector   | Bottom edge center      | Top    |
| 2     | TP4056 + caps     | Near USB-C              | Top    |
| 3     | DW01A + FS8205    | Near TP4056             | Top    |
| 4     | TPS61023 + inductor| Near TP4056 output     | Top    |
| 5     | TPS22918 + button | Near boost output       | Top    |
| 6     | Status LEDs       | Near USB-C (visible)    | Top    |
| 7     | Pi Zero header    | Center of board         | Top    |
| 8     | Display header    | Aligned with Pi header  | Top    |
| 9     | MAX17048 + caps   | Near battery connector  | Bottom |
| 10    | JST battery conn  | Bottom edge or back     | Bottom |
| 11    | RFID header       | Board edge (side)       | Top    |
| 12    | Decoupling caps   | Close to each IC        | Both   |

**Tips:**
- Power components (TP4056, TPS61023, TPS22918) should be grouped together
- Pi header in the center — it's the largest component and everything connects to it
- Keep the battery connector on the bottom side so the LiPo pouch sits flat behind the PCB

### Step 3: Route Traces

**Trace width guidelines:**

| Net Type          | Width    | Notes                                  |
| ----------------- | -------- | -------------------------------------- |
| GND               | 0.5mm+   | Use copper pour instead of traces      |
| 5V, BAT+ power    | 0.5mm    | Carries up to 1A                       |
| 3.3V power        | 0.3mm    | Lower current                          |
| SPI signals       | 0.25mm   | MOSI, SCLK, CS, DC                    |
| I2C signals       | 0.25mm   | SDA, SCL                               |
| UART signals      | 0.25mm   | TX, RX                                 |
| LED / misc        | 0.25mm   | Low current signals                    |

**Routing order:**
1. Route power traces first (5V, BAT+, 3.3V)
2. Route SPI signals (display)
3. Route I2C signals (fuel gauge)
4. Route remaining signals (UART, LEDs, button)
5. Add ground copper pour last (see below)

### Step 4: Ground Plane

- **Pour copper fill on the entire bottom layer** and connect it to GND
- This is the single most important thing for reducing noise
- Every GND pad connects to this plane via vias
- EasyEDA Pro: select bottom copper layer > "Copper Area" tool > draw around the board > set net to GND

### Step 5: Silkscreen

Add text labels on the silkscreen layer:

- Component references (R1, C1, U1, etc.) — EasyEDA adds these automatically
- Connector labels: "USB-C", "BATT", "RFID", "DISPLAY"
- Pin 1 markers on headers
- Board name: "VRCBadge v1"
- Your name or handle (optional)

### Step 6: Design Rules Check (DRC)

Run DRC before exporting. Fix all errors:

- **Clearance violations** — traces too close together
- **Unrouted nets** — connections you missed
- **Minimum width violations** — traces thinner than fab limits

**JLCPCB minimum specs:**

| Parameter       | Minimum |
| --------------- | ------- |
| Trace width     | 0.127mm (5mil) |
| Trace clearance | 0.127mm (5mil) |
| Via drill       | 0.3mm   |
| Via diameter    | 0.6mm   |
| Hole size       | 0.3mm   |

Stay above these to avoid fabrication issues. The 0.2mm+ widths in our design are safe.

---

## Phase 4: Generate Manufacturing Files & Order

### Export from EasyEDA Pro

1. **Gerber files** — "Fabrication" > "Generate Gerber". This produces the board layers.
2. **BOM (Bill of Materials)** — "Fabrication" > "Export BOM". Lists all components with LCSC part numbers.
3. **CPL (Pick-and-Place)** — "Fabrication" > "Export Pick and Place". Tells the machine where to place each SMD part.

### Order on JLCPCB

1. Go to https://jlcpcb.com and click "Order now"
2. Upload Gerber ZIP file
3. Select options:

| Option          | Value                |
| --------------- | -------------------- |
| Layers          | 2                    |
| Dimensions      | 85 x 55mm            |
| Quantity        | 5 (minimum)          |
| Thickness       | 1.6mm                |
| Solder mask     | Black                |
| Silkscreen      | White                |
| Surface finish  | HASL (lead-free)     |
| SMT Assembly    | Yes                  |
| Assembly side   | Top (or both if needed) |

4. Upload BOM and CPL files when prompted for SMT assembly
5. JLCPCB shows a **3D preview** of component placement — review carefully:
   - Are all components on the correct pads?
   - Are polarized components (ICs, LEDs, caps) oriented correctly?
   - Are through-hole parts excluded from assembly?
6. Confirm and pay

**Expected cost:**
- PCB fabrication (5 pcs): ~$2-5
- SMT assembly: ~$10-20
- Components: ~$5-10
- Shipping: ~$5-15 (economy) or ~$15-25 (express)
- **Total: ~$25-50**

**Turnaround:** 3-5 days fabrication + 5-15 days shipping (economy)

---

## Phase 5: Hand-Solder Through-Hole Parts

After receiving your boards (SMD parts pre-soldered by JLCPCB):

### You will need

- Soldering iron (temperature-controlled, ~350C)
- Solder wire (0.8mm, leaded or lead-free)
- Flux pen (makes soldering easier)
- Helping hands or PCB holder

### Soldering order

| Order | Part                        | Difficulty |
| ----- | --------------------------- | ---------- |
| 1     | 2x20 Pi Zero female header  | Easy       |
| 2     | Display female header        | Easy       |
| 3     | JST PH battery connector     | Easy       |
| 4     | RFID 4-pin header (optional) | Easy       |
| 5     | Tactile push button           | Easy       |

All through-hole parts are straightforward — insert pins, flip board, solder, trim leads.

### After soldering

1. **Visual inspection** — check for solder bridges (especially on the headers)
2. **Continuity test** — use a multimeter to check GND, 5V, 3.3V aren't shorted
3. **Power test** — connect battery, check 5V output with multimeter before plugging in Pi
4. **Plug in Pi Zero 2W** — connect display, power on, verify boot

---

## Time Estimate

| Phase | Task                          | Estimated Time |
| ----- | ----------------------------- | -------------- |
| 0     | Learn EasyEDA Pro basics      | 1-2 hours      |
| 1     | Draw schematic (all blocks)   | 3-5 hours      |
| 2     | Assign footprints, generate   | 30 min         |
| 3     | PCB layout + routing          | 3-5 hours      |
| 4     | Export + order on JLCPCB      | 30 min         |
| —     | Wait for delivery             | 1-3 weeks      |
| 5     | Hand-solder through-hole      | 30-60 min      |
|       | **Total active time**         | **~8-14 hours** |

Spread across a few sessions. Don't rush — mistakes are cheap to fix in schematic, expensive to fix on a manufactured board.

---

## Common Mistakes to Avoid

1. **Wrong footprint** — Always verify the physical footprint matches the actual part you ordered. Check dimensions in the LCSC product page.
2. **Reversed polarity** — Double-check LED, capacitor, and IC orientation. Match pin 1 markers.
3. **Missing decoupling caps** — Every IC needs a 100nF cap close to its power pin. This is non-negotiable.
4. **Traces too thin for power** — Use 0.5mm+ for 5V and battery traces. Thin traces heat up under load.
5. **No ground plane** — Always pour a ground copper fill. Floating ground pins cause noise and erratic behavior.
6. **Forgetting CC resistors on USB-C** — Without 5.1k on CC1/CC2, USB-C chargers won't provide power.
7. **Not running DRC** — Always run Design Rules Check before exporting. Fix every error.
8. **Soldering Pi header backwards** — The female header goes on the PCB top side, Pi plugs in from above with components facing up.
