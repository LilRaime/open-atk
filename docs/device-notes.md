# Device Research & Hardware Notes

## Verified Hardware Support Matrix

OpenATK dynamically detects and supports gaming mice across the ATK, VXE, and VGN product families using unified vendor protocols over USB HID.

### Recognized Vendor IDs (VID)
- `0x3554`: Primary Compx / ATK / VXE Vendor ID
- `0x373B`: Alternative ATK Vendor ID

### Supported Model Families

| Family | Models | Optical Sensor | Max Polling | Wireless PID | Wired PID | CLI Aliases |
|---|---|---|---|---|---|---|
| **VXE R1 Series** | VXE R1 SE | PAW3395SE | 1000 Hz | `0xF580` | `0xF581` | `r1se`, `se` |
| | VXE R1 SE+ | PAW3395SE | 1000 Hz | `0xF58E` | `0xF58F` | `r1se+`, `r1sep` |
| | VXE R1 | PAW3395 | 1000 Hz | `0xF584` | `0xF585` | `r1` |
| | VXE R1 Pro | PAW3395 | 4000 Hz | `0xF588` | `0xF589` | `r1p`, `pro` |
| | VXE R1 Pro Max | PAW3395 | 4000 Hz | `0xF58A`, `0x1085` | `0xF58C` | `r1pm`, `promax` |
| **VGN Dragonfly F1** | F1 / Pro / Pro Max / MOBA | PAW3395 | 4000 Hz | Various (`0x3554`) | Various | `f1`, `f1p`, `f1pm`, `f1m`, `dragonfly` |
| **ATK Blazing Sky F1** | F1 / F1 Pro / F1 Ultimate | PAW3950 / Ultra | 8000 Hz | Various (`0x3554`) | Various | `f1`, `f1u`, `f1pro` |
| **ATK Blazing Sky X1** | X1 / X1 Pro / X1 Ultra | PAW3950 / Ultra | 8000 Hz | Various (`0x3554`) | Various | `x1`, `x1p`, `x1pm`, `x1u` |
| **ATK Mad R Series** | Mad R / Mad R Major / Mad R+ | PAW3395 / PAW3950 | 8000 Hz | Various (`0x3554`) | Various | `madr`, `madrmajor`, `madr+` |

---

## Baseline Reference Sample (VXE R1 Pro Max)

- **Target Device:** **ATK / VXE R1 Pro Max**
- **Protocol Identifiers:**
  - `CID` (Client ID): `2`
  - `MID` (Model ID): `27`
- **Firmware Version:** `v1.21`
- **Optical Sensor:** PixArt PAW3395 (50 – 26,000 DPI native, configurable up to 30,000 DPI in 50 DPI steps)
- **Wireless Receiver Identifiers:**
  - Standard Dongle PID: `0xF58A` (1K Nordic Dongle)
  - High-Speed Dongle PID: `0x1085` (4K/8K Compx Dongle)
  - Wired Mode PID: `0xF58C`
  - Radio Frequency ID (RF ID): `01 0F 6B`
  - Manufacturer: Compx (Nordic nRF MCU)

---

## Control Channel & Transport Specifications

- **Transport Mechanism:** USB HID Feature/Output reports via Report ID `0x08`.
- **Wire Report Length:** Exactly 17 bytes (`[Report ID (1 byte), Payload (16 bytes)]`).
- **Configuration Collection:** Interface 1, `Usage Page 0xFF02`, `Usage 0x0002` (`Col05`).
- **Checksum Invariant:** The sum of all 17 wire bytes modulo 256 is strictly equal to `0x55`:

  ```math
  \left(0x55 - \left(\text{Report\_ID} + \sum_{i=0}^{14} \text{payload}[i]\right)\right) \pmod{256}
  ```

---

## Complete Hardware Register & EEPROM Map

All configuration settings are stored in onboard non-volatile EEPROM. Writes survive power cycles, receiver unplugging, and system reboots.

| Address | Length | Parameter | Value Range / Description |
|---|---|---|---|
| `0x0000` | 2 bytes | **Polling Rate** | `[opcode, 0x55 - opcode]` (`0x08`=125Hz, `0x04`=250Hz, `0x02`=500Hz, `0x01`=1000Hz, `0x10`=2000Hz, `0x20`=4000Hz, `0x40`=8000Hz) |
| `0x0002` | 2 bytes | **DPI Stage Count** | `[count, 0x55 - count]` (Range: `1` to `6` stages) |
| `0x0004` | 2 bytes | **Active DPI Stage** | `[index, 0x55 - index]` (0-indexed, `0` to `count - 1`) |
| `0x000A` | 2 bytes | **Lift-Off Distance (LOD)** | `[lod, 0x55 - lod]` (`1` = 1.0 mm, `2` = 2.0 mm, `3` = 0.7 mm) |
| `0x000C` | 4 bytes | **DPI Stage 1 Record** | `[xb, yb, mode_nibbles, checksum]` (Decoded in 50 DPI steps) |
| `0x0010` | 4 bytes | **DPI Stage 2 Record** | 4-byte packed record |
| `0x0014` | 4 bytes | **DPI Stage 3 Record** | 4-byte packed record |
| `0x0018` | 4 bytes | **DPI Stage 4 Record** | 4-byte packed record |
| `0x001C` | 4 bytes | **DPI Stage 5 Record** | 4-byte packed record |
| `0x0020` | 4 bytes | **DPI Stage 6 Record** | 4-byte packed record |
| `0x002C` | 16 bytes | **DPI Stage LED Colors** | RGB color values assigned to each DPI stage |
| `0x004C` | 8 bytes | **DPI Indicator LED Config** | 4 pairs `[val, 0x55 - val]`: Mode (Off/Constant/Breathing), Brightness (0..2), Speed (0..2), Reserved |
| `0x0060` | 24 bytes | **Button Mappings** | 6 button records of 4 bytes each (Left, Right, Middle, Forward, Back, DPI) |
| `0x00A9` (0) | 2 bytes | **Debounce Delay** | `[ms, 0x55 - ms]` at offset 0 (Valid range: `0` to `20` ms) |
| `0x00A9` (2) | 2 bytes | **Motion Sync** | `[val, 0x55 - val]` at offset 2 (`1` = Enabled, `0` = Disabled) |
| `0x00A9` (4) | 2 bytes | **Sleep Timeout** | `[units, 0x55 - units]` at offset 4 (10-second units: `3`=30s, `6`=1m, `12`=2m, `18`=3m, `30`=5m, `60`=10m, `90`=15m, `120`=20m, `180`=30m) |
| `0x00A9` (6) | 2 bytes | **Straight Line Correction** | `[val, 0x55 - val]` at offset 6 (Angle Snapping: `1` = Enabled, `0` = Disabled) |
| `0x00A9` (8) | 2 bytes | **Ripple Correction** | `[val, 0x55 - val]` at offset 8 (Sensor anti-jitter: `1` = Enabled, `0` = Disabled) |
| `0x00B5` (4) | 2 bytes | **Ultra Long Range Mode** | `[val, 0x55 - val]` at offset 4 (RF transmission power boost: `1` = Enabled, `0` = Disabled) |

---

## Connection Modes & Dongle Telemetry

1. **Wired USB-C Mode:**
   - Active when plugged directly into the host PC.
   - Provides simultaneous battery charging and zero-latency communication.
   - Identified by wired PIDs (e.g., `0xF58C`).

2. **2.4 GHz Wireless Receiver Mode:**
   - Active via 1K or 4K/8K USB dongle.
   - Queried via opcode `0x03` (`CMD_RECEIVER_ONLINE`). Returns receiver online status and paired RF ID (3 bytes).
   - If the mouse enters deep sleep or is switched off, the dongle responds with `0x00` in the online flag while keeping the receiver connection open.
   - Battery telemetry is queried via opcode `0x04`: returns charge percentage (`0`–`100%`), charging flag (`1`=charging, `0`=discharging), and battery voltage in millivolts (`mV`).
