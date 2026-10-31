# pybmap

Python library for the Bose BMAP protocol over Bluetooth
RFCOMM. It controls noise cancellation, EQ, profiles, and device settings on
paired Bose headphones.

This directory holds the Python side of the project. See the
[repository README](../README.md) for the device list and the protocol notes.

## Install

```bash
python -m pip install -e .
python -m pip install pytest
```

## Library

```python
import pybmap

with pybmap.connect() as dev:
    print(dev.battery())
    dev.set_cnc(8)          # QC Ultra 2: CNC level 0-10
    dev.set_eq(3, 0, -2)    # Bass +3, mid flat, treble -2
```

## Backend sidecar

The desktop app uses this package as a long-lived sidecar process:

```bash
python -m pybmap.backend
```

## Tests

```bash
pytest
```

Tests marked `integration` need a paired Bluetooth device.
