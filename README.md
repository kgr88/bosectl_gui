# bosectl

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/aaronsb/bosectl)](https://github.com/aaronsb/bosectl/releases/latest)
[![Devices](https://img.shields.io/badge/Devices-2_supported_·_14_known-green)](docs/architecture.md#device-catalog)
[![Python 3](https://img.shields.io/badge/Python-3-3572A5.svg)](python/)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows-0078D4.svg)](desktop/)

**Control Bose headphones from Windows — no Bose app, no cloud, no account.**

A Python library implementing the Bose BMAP protocol
over Bluetooth RFCOMM. Full control over noise cancellation, EQ, spatial
audio, button mapping, profiles, and device settings through a direct
connection to the headphones.

> **This is not an exploit.** We use the BMAP protocol's standard SETGET
> operator, which the headphones accept without authentication. No keys
> are extracted, no encryption is broken, no traffic is replayed.

## Supported Devices

| Device | NC Control | EQ | Spatial | Profiles | Buttons | Status |
|--------|-----------|-----|---------|----------|---------|--------|
| **QC Ultra Headphones 2** | CNC 0-10 slider | 3-band | room/head | 7 custom slots | Shortcut remap | Verified |
| **QuietComfort 35 / 35 II** | ANR off/high/wind/low | — | — | — | Action remap (VPA/ANC) | Verified |

### Device Roadmap

The library includes a [device catalog](docs/architecture.md#device-catalog)
of all known BMAP-capable Bose products. These are recognized by Bluetooth
product ID but don't have tested configurations yet — contributions welcome:

| Device | Codename | Category | PID |
|--------|----------|----------|-----|
| Noise Cancelling Headphones 700 | goodyear | Headphones | `0x4024` |
| QuietComfort 45 | duran | Headphones | `0x4039` |
| QuietComfort Headphones | prince | Headphones | `0x4075` |
| QuietComfort Ultra Headphones | lonestarr | Headphones | `0x4066` |
| QuietComfort Earbuds II | smalls | Earbuds | `0x4064` |
| QuietComfort Ultra Earbuds | scotty | Earbuds | `0x4072` |
| Ultra Open Earbuds | serena | Earbuds | `0x4068` |
| SoundLink Flex | phelps | Speaker | `0xBC59` |
| SoundLink Flex 2 | mathers | Speaker | `0xBC61` |

Adding a new device is a configuration entry — no library code changes needed.
See [Adding a New Device](docs/architecture.md#adding-a-new-device).

## Quick Start

### Library Usage

```python
import pybmap

with pybmap.connect() as dev:
    print(dev.battery())            # 80
    print(dev.name())               # "Obsidian Countess"
    dev.set_anr("high")             # QC35: full noise cancellation
    dev.set_cnc(8)                  # QC Ultra 2: CNC level 0-10
    dev.set_eq(3, 0, -2)            # Bass +3, mid flat, treble -2
    dev.set_buttons(0x10, 4, 2)     # Remap Action button to ANC
```

### Device Catalog API

```python
import pybmap

# Look up any known Bose device by product ID
dev = pybmap.lookup_device(0x4082)
print(dev.name)       # "QuietComfort Ultra Headphones (2nd Gen)"
print(dev.codename)   # "wolverine"

# USB identification
pybmap.usb_ids(0x4082)    # (0x05A7, 0x4082)

# Check support status
pybmap.is_supported(0x4082)  # True — has tested config
pybmap.is_supported(0x4039)  # False — QC45, recognized but untested
pybmap.supported_devices()   # [wolfcastle, baywolf, edith, wolverine]
pybmap.known_devices()       # full catalog
```

## Installation

### Prerequisites

- **Windows** 10 or 11
- **Bluetooth** adapter (built-in or USB)
- **Bose headphones** paired in Windows Bluetooth settings

### From Source

```bash
git clone https://github.com/aaronsb/bosectl.git
cd bosectl/python
python -m pip install -e .
python -m pip install pytest
pytest             # Run the Python tests
```

The backend auto-detects paired Bose devices by their BMAP service UUID —
no MAC address configuration needed, even with renamed headphones.

## Architecture

The library uses this layered design:

```
Application → BmapConnection → Device Config → Transport → Protocol → Bluetooth RFCOMM
```

- **Protocol** — binary BMAP packet codec
- **Transport** — RFCOMM socket with drain mode for async responses
- **Device Config** — data-only description of each headphone model (addresses, parsers, quirks)
- **BmapConnection** — typed API that dispatches to the right address/parser per device
- **Catalog** — all known Bose BMAP devices with product IDs, codenames, and USB identifiers

Device differences (RFCOMM channel, init packets, feature availability) are
expressed as config data, not code branches. Adding a new device is a
config entry pointing to existing parsers.

Full documentation: **[docs/architecture.md](docs/architecture.md)**

## How It Works

Bose headphones speak **BMAP** (Bose Messaging and Protocol) over
Bluetooth RFCOMM. The protocol is organized into function blocks
(groups of features) and operators (read, write, action).

The key insight: while Bose gates SET (operator 0) behind cloud-mediated
ECDH authentication, **SETGET** (operator 2) and **START** (operator 5)
are unauthenticated on the Settings and AudioModes blocks. This gives
full control over every user-facing setting.

Full protocol reference: **[NOTES.md](NOTES.md)**

### How We Found This

1. Connected over RFCOMM and probed all channels — QC Ultra 2 firmware has
   responded on channels 2 and 1, while QC35 responded on channel 8
2. Captured Bluetooth HCI traffic while toggling settings in the Bose app
3. DNS-hijacked the cloud API and noticed mode switching still worked — the app uses START, not SET
4. Systematically tested every operator on every function block to map the auth boundary

## Project Structure

```
├── python/pybmap/       # Python library and backend sidecar
├── docs/                # Architecture guide, device docs
├── NOTES.md             # Protocol reverse engineering notes
└── fixtures/            # Captured protocol data
```

## Building & Testing

Run these commands from `python/`:

```bash
pytest                          # All Python tests
python -m build                 # sdist and wheel in python/dist/
```

## License

MIT
