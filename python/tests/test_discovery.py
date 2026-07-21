import uuid
from types import SimpleNamespace

from pybmap.catalog import BMAP_UUID
from pybmap.discovery import (
    _device_from_name, _format_bluetooth_address, _guid_bytes_to_text,
    discover_devices,
)


def test_linux_discovery_returns_structured_supported_device(monkeypatch):
    paired = "Device AA:BB:CC:DD:EE:FF My QC35 II\n"
    info = """Name: My QuietComfort 35 II
Connected: yes
Icon: audio-headphones
UUID: 00000000-deca-fade-deca-deafdecacaff
Modalias: bluetooth:v05A7p4020d0000
"""

    def run(command, **kwargs):
        return SimpleNamespace(stdout=paired if command[:2] == ["bluetoothctl", "devices"] else info)

    monkeypatch.setattr("pybmap.discovery.subprocess.run", run)
    devices = discover_devices(platform="linux")
    assert len(devices) == 1
    assert devices[0].address == "AA:BB:CC:DD:EE:FF"
    assert devices[0].device_type == "qc35"
    assert devices[0].rfcomm_channel == 8
    assert devices[0].connected is True


def test_windows_address_and_guid_wire_formats_match_platform_contract():
    assert _format_bluetooth_address(0xAABBCCDDEEFF) == "AA:BB:CC:DD:EE:FF"
    assert _guid_bytes_to_text(uuid.UUID(BMAP_UUID).bytes_le) == BMAP_UUID


def test_windows_qc_ultra_2_headphones_alias_is_unambiguous():
    assert _device_from_name("Bose QC Ultra 2 HP") == (0x4082, "qc_ultra2")
