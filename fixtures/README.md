# Test Fixtures

BMAP packet captures used as reference data for the Python tests.

## Structure

- `packets/<device>/` — Captured request/response snapshots (JSON files from
  `python/tools/bmap-capture.py`). The capture tool writes new files to
  `packets/`. Move each file into the folder for its device.

## Packet format

Each capture JSON file contains snapshots of all readable BMAP function
blocks taken before and after a specific setting change. See
`python/tools/bmap-capture.py` for the capture tool.
