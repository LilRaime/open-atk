# ATK / VXE (Compx) Protocol Specification

## Transport Layer Framing

OpenATK uses direct USB Human Interface Device (HID) vendor reports to exchange telemetry and hardware configuration with ATK / VXE / VGN devices.

- **Target HID Collection:** Interface 1, `Usage Page 0xFF02`, `Usage 0x0002` (`Col05`).
- **Report ID:** `0x08`
- **Wire Report Size:** Strictly 17 bytes (`[Report ID (1 byte), Payload (16 bytes)]`).
- **Wire Framing Format:**
  ```text
  Host -> Device: 08 <cmd> <status> <addr_hi> <addr_lo> <len> <data...> <checksum>
  Device -> Host: 08 <cmd> <status> <addr_hi> <addr_lo> <len> <data...> <checksum>
  ```
- **Checksum Calculation:**
  The checksum is stored in wire byte 16 (`payload[15]`). The sum of all 17 wire bytes modulo 256 equals `0x55`:
  $$\text{checksum} = \left(0x55 - \left(\text{Report\_ID} + \sum_{i=0}^{14} \text{payload}[i]\right)\right) \pmod{256}$$

---

## Verified Protocol Commands (Opcodes)

| Opcode | Command Constant | Direction | Description |
|---|---|---|---|
| `0x04` | `CMD_BATTERY` | Host $\to$ Device | Query battery percentage, charging state, and voltage in mV |
| `0x10` | `CMD_CID_MID` | Host $\to$ Device | Query Client ID (CID) and Model ID (MID) |
| `0x12` | `CMD_VERSION` | Host $\to$ Device | Query MCU firmware version string |
| `0x03` | `CMD_RECEIVER_ONLINE` | Host $\to$ Device | Query wireless receiver online status and paired RF ID |
| `0x08` | `CMD_READ_EEPROM` | Host $\to$ Device | Read configuration registers (up to 10 bytes per chunk) |
| `0x07` | `CMD_WRITE_EEPROM` | Host $\to$ Device | Write configuration registers to onboard EEPROM |
| `0x0E` | `CMD_CURRENT_PROFILE` | Host $\to$ Device | Read currently active onboard profile index (0..3) |
| `0x0F` | `CMD_SET_PROFILE` | Host $\to$ Device | Switch active onboard profile |

---

## Command Payload Structures

### 1. Battery Telemetry (`0x04`)
- **Request:** `[0x08, 0x04, 0x00, 0x00, 0x00, 0x00, ...]`
- **Response Structure:**
  - `payload[4]`: Battery charge percentage (`0` to `100`%)
  - `payload[5]`: Charging flag (`0x01` = Charging, `0x00` = Discharging)
  - `payload[6..7]`: Battery voltage in millivolts, big-endian unsigned 16-bit integer (e.g., `[0x10, 0x05]` = 4101 mV $\approx$ 4.10 V)

### 2. Device Identity (`0x10`)
- **Response Structure:**
  - `payload[4]`: Client ID (`CID`), e.g., `0x02`
  - `payload[5]`: Model ID (`MID`), e.g., `0x1B` (`27` = VXE R1 Pro Max)

### 3. Firmware Version (`0x12`)
- **Response Structure:**
  - `payload[4..]`: Null-terminated ASCII version string (e.g., `"v1.21"`)

### 4. Wireless Receiver Status (`0x03`)
- **Response Structure:**
  - `payload[4]`: Mouse link state (`0x01` = Mouse online & connected, `0x00` = Mouse sleeping/offline)
  - `payload[5..7]`: 3-byte Radio Frequency ID (e.g., `01 0F 6B`)

### 5. EEPROM Read Transaction (`0x08`)
- **Request Structure:**
  - `payload[0]`: `0x08`
  - `payload[1]`: `0x00`
  - `payload[2]`: Address High Byte (`(addr >> 8) & 0xFF`)
  - `payload[3]`: Address Low Byte (`addr & 0xFF`)
  - `payload[4]`: Read Length in bytes (`1` to `10`)
- **Response Structure:**
  - `payload[0]`: `0x08`
  - `payload[1]`: Status (`0x00` = Success)
  - `payload[2..3]`: Echoed Register Address
  - `payload[4]`: Length of data returned
  - `payload[5..5+len]`: Raw register bytes

### 6. EEPROM Write Transaction (`0x07`)
- **Request Structure:**
  - `payload[0]`: `0x07`
  - `payload[1]`: `0x00`
  - `payload[2]`: Address High Byte
  - `payload[3]`: Address Low Byte
  - `payload[4]`: Data Length in bytes
  - `payload[5..5+len]`: Data bytes to write
- **Response Structure:**
  - Echo confirmation with status byte `0x00`

---

## Complete EEPROM Register Map

Registers marked with `[val, 0x55 - val]` use self-verifying paired checksums where the second byte must sum to `0x55` with the value byte.

### 1. System & Performance Registers

| Address | Length | Parameter | Description |
|---|---|---|---|
| `0x0000` | 2 bytes | **Polling Rate** | `[rate_opcode, 0x55 - rate_opcode]` |
| `0x0002` | 2 bytes | **DPI Stage Count** | `[count, 0x55 - count]` (1 to 6 stages) |
| `0x0004` | 2 bytes | **Active DPI Stage** | `[index, 0x55 - index]` (0-indexed) |
| `0x000A` | 2 bytes | **Lift-Off Distance (LOD)** | `[lod_code, 0x55 - lod_code]` (`1` = 1.0 mm, `2` = 2.0 mm, `3` = 0.7 mm) |

#### Polling Rate Opcodes
- `0x08` = 125 Hz
- `0x04` = 250 Hz
- `0x02` = 500 Hz
- `0x01` = 1000 Hz
- `0x10` = 2000 Hz
- `0x20` = 4000 Hz
- `0x40` = 8000 Hz

---

### 2. DPI Stage Records (`0x000C` – `0x0020`)

Each stage record occupies 4 bytes:
`[xb, yb, mode, checksum]`

- **Axis Decoding Algorithm:**
  ```rust
  fn decode_dpi_axis(byte: u8, nibble: u8) -> u32 {
      let count = (byte as u32) | (((nibble as u32 >> 2) & 0x03) << 8);
      let doubled = (nibble & 1) != 0;
      let step = if doubled { 100 } else { 50 };
      (count + 1) * step
  }
  ```
- `mode`: High nibble contains `Y_nibble`, low nibble contains `X_nibble`.
- `checksum`: `0x55.wrapping_sub(xb).wrapping_sub(yb).wrapping_sub(mode)`.

| Address | Length | Stage |
|---|---|---|
| `0x000C` | 4 bytes | DPI Stage 1 |
| `0x0010` | 4 bytes | DPI Stage 2 |
| `0x0014` | 4 bytes | DPI Stage 3 |
| `0x0018` | 4 bytes | DPI Stage 4 |
| `0x001C` | 4 bytes | DPI Stage 5 |
| `0x0020` | 4 bytes | DPI Stage 6 |

---

### 3. DPI Stage Indicator Colors (`0x002C`)
- **Length:** 16 bytes
- **Layout:** Sequential RGB records for each DPI stage color.

---

### 4. DPI Indicator LED Animation (`0x004C`)
- **Length:** 8 bytes (4 self-verifying pairs of `[val, 0x55 - val]`):
  - Offset `0..1`: LED Mode (`0` = Off / Closed, `1` = Constant, `2` = Breathing)
  - Offset `2..3`: Brightness (`0` = Low, `1` = Medium, `2` = High)
  - Offset `4..5`: Speed (`0` = Slow, `1` = Medium, `2` = Fast)
  - Offset `6..7`: State flag

---

### 5. Advanced Sensor & Switch Parameters (`0x00A9`)
- **Length:** 10 bytes (5 self-verifying pairs of `[val, 0x55 - val]`):
  - Offset `0..1`: **Debounce Delay** (`0` to `20` ms)
  - Offset `2..3`: **Motion Sync** (`1` = Enabled, `0` = Disabled)
  - Offset `4..5`: **Sleep Timeout** in 10-second units:
    - `3` = 30 seconds
    - `6` = 1 minute
    - `12` = 2 minutes
    - `18` = 3 minutes
    - `30` = 5 minutes
    - `60` = 10 minutes
    - `90` = 15 minutes
    - `120` = 20 minutes
    - `180` = 30 minutes
  - Offset `6..7`: **Straight Line Correction** (Angle Snapping: `1` = Enabled, `0` = Disabled)
  - Offset `8..9`: **Ripple Correction** (Anti-jitter smoothing: `1` = Enabled, `0` = Disabled)

---

### 6. Sensor Mode & RF Transmission (`0x00B5`)
- **Length:** 6 bytes
- Offset `4..5`: **Ultra Long Range Mode** `[val, 0x55 - val]` (`1` = Enabled / Enhanced RF transmission power, `0` = Disabled / Standard).

---

## Universal Hardware Profile Schema (`open_atk_profile.json`)

Both `open-atk` (desktop GUI) and `vxectl` (CLI) exchange hardware configurations using the standardized JSON schema below:

```json
{
  "model": "VXE R1 PRO MAX",
  "profile_name": "OpenATK Profile",
  "polling_rate_hz": 1000,
  "active_stage": 1,
  "stage_count": 4,
  "stages_dpi": [
    { "stage": 1, "x": 800, "y": 800, "color_rgb": [255, 0, 0] },
    { "stage": 2, "x": 800, "y": 800, "color_rgb": [0, 0, 255] },
    { "stage": 3, "x": 1600, "y": 1600, "color_rgb": [0, 255, 0] },
    { "stage": 4, "x": 3200, "y": 3200, "color_rgb": [255, 255, 0] }
  ],
  "lod_mm": 1.0,
  "debounce_ms": 8,
  "motion_sync": true,
  "straight_line_correction": false,
  "ripple_control": false,
  "ultra_long_range": false,
  "sleep_time_idx": 1,
  "dpi_light_mode": 0,
  "dpi_light_brightness": 2,
  "dpi_light_speed": 1,
  "button_mappings": []
}
```
