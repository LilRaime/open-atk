# OpenATK Architecture

## Modular Structure (Cargo Workspace)

```text
open-atk/
├── Cargo.toml
├── crates/
│   ├── app/           # Native desktop graphical interface (egui/eframe)
│   │   ├── src/
│   │   │   ├── views/           # Modular view components (key change, macros, DPI, parameters, config, other)
│   │   │   ├── device_selector  # Multi-device discovery & dropdown selector
│   │   │   ├── icons.rs         # Embedded rasterized icons and brand badges
│   │   │   ├── macro_player.rs  # Low-level OS hook & macro player execution
│   │   │   ├── models.rs        # UI domain models & state definitions
│   │   │   ├── preferences.rs   # Persistent user preferences (close to tray, autostart)
│   │   │   ├── sidebar.rs       # Navigation sidebar & responsive status indicator
│   │   │   ├── state.rs         # Global reactive application state
│   │   │   ├── theme.rs         # Dark graphite UI theme & styling tokens
│   │   │   ├── tray.rs          # System tray integration with Win32 message loop restore
│   │   │   └── worker.rs        # Non-blocking background worker thread for hardware I/O
│   │   └── Cargo.toml
│   ├── core/          # Domain models, event dispatching, and service coordination
│   ├── device/        # Hardware device lifecycle, scanning, and connection management
│   │   ├── src/
│   │   │   ├── scanner.rs       # Multi-VID/PID discovery and connection mode resolution
│   │   │   └── lib.rs           # AtkDevice handle, register read/write, high-level API
│   │   └── Cargo.toml
│   ├── hid/           # Transport abstraction layer (hidapi wrapper, report read/write)
│   ├── atk-protocol/  # Pure protocol layer: packets, commands, parser, and checksums
│   │   ├── src/
│   │   │   ├── battery.rs       # Battery parsing & voltage decoders
│   │   │   ├── buttons.rs       # Button mapping constants & actions
│   │   │   ├── checksum.rs      # Modulo-256 (0x55) algorithm
│   │   │   ├── commands.rs      # Outbound command builders
│   │   │   ├── constants.rs     # Vendor IDs, Product IDs, Register addresses
│   │   │   ├── dpi.rs           # DPI pack/unpack, bitfield decoders, stage configs
│   │   │   ├── packet.rs        # 17-byte wire packet structure
│   │   │   ├── parser.rs        # Incoming packet parser & state machine
│   │   │   ├── polling.rs       # Polling rate opcodes (125 Hz - 8000 Hz)
│   │   │   └── response.rs      # Parsed device response variants
│   │   └── Cargo.toml
│   ├── macro-engine/  # Macro definitions, recording sequences, and playback models
│   ├── profile/       # Profile configuration models, DPI stages, and sensor tuning
│   ├── storage/       # Local configuration persistence (portable app profiles)
│   └── vxectl/        # High-performance CLI management utility
│       ├── src/
│       │   ├── cli.rs           # Clap CLI parser & command declarations
│       │   ├── commands.rs      # Subcommand execution dispatchers
│       │   ├── device.rs        # Model identification, short aliases, multi-device selection
│       │   ├── profile.rs       # Profile export/import (100% compatible with OpenATK GUI)
│       │   ├── setup.rs         # Auto-installation of udev rules & shell auto-completions
│       │   ├── status.rs        # Rich terminal status formatting & telemetry
│       │   └── theme.rs         # Unicode box drawing, ANSI colors, and battery bar styling
│       └── Cargo.toml
├── docs/
│   ├── architecture.md          # System architecture and workspace design
│   ├── device-notes.md          # Hardware models, VIDs/PIDs, and sensor registers
│   └── protocol.md              # Low-level HID wire protocol specification
└── packaging/
    ├── 71-open-atk.rules        # Linux udev rules for rootless USB HID access
    ├── install.sh               # One-line Linux installer script
    └── open-atk.desktop         # freedesktop.org desktop application launcher
```

---

## Architectural Design Principles

1. **Protocol Purity:** The `atk-protocol` crate has zero dependencies on `hidapi` or UI frameworks. It acts as a pure, deterministic serializer/deserializer and protocol parser, making unit testing and cross-compilation straightforward.
2. **Transport Isolation:** `atk-hid` is strictly responsible for byte transfers across HID collections. It abstracts platform-specific USB details on Windows and Linux, identifying and filtering candidate devices by Vendor IDs (`0x3554`, `0x373B`) and Configuration Collection `Usage Page 0xFF02`, `Usage 0x0002`.
3. **UI Thread Decoupling via Background Worker:** The desktop UI never executes blocking multi-register sweeps on the egui render loop. Device communication is handled by a background worker thread (`worker.rs`) using crossbeam/mpsc channels. Responses arrive via event queues, maintaining 60+ FPS without UI hitching.
4. **Hardware Safety Validation:** All user inputs (DPI increments, debounce time, polling rate, sleep timeout, LOD) are validated against hardware tolerances in `atk-profile` before any write command is dispatched to device EEPROM.
5. **Universal JSON Profile Interoperability:** Configuration backups use an identical JSON schema (`open_atk_profile.json`) across both the graphical `open-atk` suite and the headless `vxectl` CLI. A profile exported from the command line can be applied directly in the GUI and vice-versa.
6. **System-wide Macro Execution:** Macro reproduction is driven by an OS-level background hook and input synthesis (`WH_MOUSE_LL` + `SendInput` on Windows), ensuring macros trigger reliably in all games and applications without locking physical switches.
7. **Native Platform Integration:** 
   - **Windows:** System tray minimize/restore integrates directly with the Win32 API (`FindWindowW`, `ShowWindow`, `SetForegroundWindow`, `BringWindowToTop`) to reliably restore minimized windows.
   - **Linux:** Rootless HID access via udev rules (`71-open-atk.rules`), standard desktop shortcuts (`open-atk.desktop`), and shell completion generation for Bash, Zsh, Fish, and PowerShell.
