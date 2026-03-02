# PCB Layout

## Board Specification

- Width: 105mm
- Height: 80mm
- Corner radius: 2mm
- Mounting holes: 2mm diameter, one at each corner, 3.5mm from each edge

## Display Module Dimensions

Measurements from Waveshare 3.5" RPi LCD (F) specsheet.

### Panel Dimensions (front view, landscape orientation)

| Dimension | Value | Notes |
|-----------|-------|-------|
| Lens (outer) width | 92.44 +/- 0.20 mm | Glass/lens edge to edge |
| Lens (outer) height | 61.00 +/- 0.20 mm | Glass/lens edge to edge |
| Touch active area width | 73.84 +/- 0.10 mm | Usable touch/display region |
| Touch active area height | 49.36 +/- 0.10 mm | Usable touch/display region |
| Bezel left/right | 9.30 mm | Lens edge to active area |
| Bezel top/bottom | 5.82 mm | Lens edge to active area |

### PCB Dimensions (back view)

| Dimension | Value | Notes |
|-----------|-------|-------|
| PCB width | 56.00 mm | Back PCB behind the lens |
| PCB component area | 49.00 mm | Active component region |
| PCB left edge offset | 2.50 mm | From lens edge to PCB edge |
| PCB right edge offset | 3.50 mm | From lens edge to PCB edge |
| Connector row height | 3.46 mm | GH1.25 13-pin connector area |
| Mounting hole pattern | M2.50 | 4 holes on back PCB |
| Mounting hole spacing (vertical) | 27.73 mm | Between hole centers |
| Connector offset from edge | 2.31 mm | Horizontal offset |
| Component spacing | 2.55 mm | Between connector groups |

### Stack Height (side profile)

| Layer | Thickness | Notes |
|-------|-----------|-------|
| Touch panel + LCD | 3.95 mm | Glass + LCD panel combined |
| Air gap / clearance | 0.80 mm | Between LCD and back PCB |
| Back PCB + components | 6.60 mm | PCB with tallest component |
| **Total stack** | **~14.60 mm** | Front glass to back component tip |

### FPC Cable

| Dimension | Value | Notes |
|-----------|-------|-------|
| FPC exit direction | Bottom edge | In portrait orientation |
| FPC extension length | ~24.00 mm | Flexible cable extends from panel |
| FPC width | ~15.28 mm | Approximate |

## Orientation

All directions described as viewed from the front (LCD-facing) side.
Display is mounted in landscape orientation (92.44mm horizontal, 61.00mm vertical).

## Component Placement

- J6 (display connector): front side, left edge
- J5 (USB-C): back side, right edge, as close to ESP32 as possible
- J4 (Battery connector): back side, top-left edge
- U9 (ESP32-S3-WROOM-1U): antenna extends outside the frame
- SW3 (RESET) + SW4 (BOOT): back side, side by side, near ESP32, clickable from frame

## Clearance Notes

- Display stack is ~14.60mm total height -- back components must not exceed 6.60mm from PCB surface
- The display lens (92.44 x 61.00mm) is larger than the back PCB (56.00mm wide) -- the PCB board must extend beyond the lens width to accommodate mounting and connectors
- FPC cable routes from display bottom edge -- ensure clearance for the cable bend radius
- Board size (105 x 80mm) provides ~12.78mm margin on each side of the lens horizontally and ~9.50mm margin vertically for mounting holes and edge components
