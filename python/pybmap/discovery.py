"""Platform discovery for paired BMAP devices.

Linux keeps the existing ``bluetoothctl`` path.  Windows uses the documented
Bluetooth desktop API through ``ctypes`` so the backend has no optional runtime
dependency.  Discovery never guesses an unknown device configuration.
"""

import ctypes
import re
import subprocess
import sys
from collections import namedtuple
from ctypes import wintypes

from .catalog import BMAP_UUID, lookup_device


DiscoveredDevice = namedtuple(
    "DiscoveredDevice",
    ["address", "name", "service_uuid", "rfcomm_channel", "product_id",
     "device_type", "paired", "connected"],
)


class BluetoothDiscoveryError(Exception):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code


def discover_devices(platform=None):
    """Return all paired BMAP candidates as platform-neutral records."""
    platform = platform or sys.platform
    if platform == "win32":
        return _scan_windows_devices()
    if platform.startswith("linux"):
        return _scan_linux_devices()
    return []


def find_bmap_device():
    """Compatibility API returning the preferred ``(address, config)`` tuple."""
    candidates = discover_devices()
    for device in candidates:
        if device.connected and device.device_type:
            return (device.address, device.device_type)
    for device in candidates:
        if device.device_type:
            return (device.address, device.device_type)
    return (None, None)


def _scan_linux_devices():
    devices = []
    try:
        result = subprocess.run(
            ["bluetoothctl", "devices", "Paired"],
            capture_output=True, text=True, timeout=5,
        )
        for line in result.stdout.strip().splitlines():
            parts = line.split(None, 2)
            if len(parts) < 2:
                continue
            address = parts[1]
            fallback_name = parts[2] if len(parts) > 2 else "Bose device"
            info = subprocess.run(
                ["bluetoothctl", "info", address],
                capture_output=True, text=True, timeout=3,
            ).stdout
            is_audio = "audio-headset" in info or "audio-headphones" in info
            if not (is_audio and BMAP_UUID.lower() in info.lower()):
                continue
            name_match = re.search(r"^\s*Name:\s*(.+)$", info, re.MULTILINE)
            product_id = _product_id_from_modalias(info)
            device_type = _device_type_from_product(product_id)
            channel = _channel_for_type(device_type)
            devices.append(DiscoveredDevice(
                address, name_match.group(1).strip() if name_match else fallback_name,
                BMAP_UUID, channel, product_id, device_type, True,
                "Connected: yes" in info,
            ))
    except (FileNotFoundError, subprocess.TimeoutExpired, OSError):
        pass
    return devices


# Kept for callers and tests that used the original private helper.
def _scan_paired_devices():
    return [(d.address, d.device_type, d.connected) for d in _scan_linux_devices()]


def _product_id_from_modalias(info_text):
    match = re.search(
        r"Modalias:\s*bluetooth:v[0-9A-Fa-f]{4}p([0-9A-Fa-f]{4})", info_text
    )
    return int(match.group(1), 16) if match else None


def _device_type_from_product(product_id):
    entry = lookup_device(product_id) if product_id is not None else None
    return entry.config if entry and entry.config else None


def _detect_device_type(info_text):
    """Compatibility helper; importantly, unknown devices remain unknown."""
    return _device_type_from_product(_product_id_from_modalias(info_text))


def _device_from_name(name):
    """Resolve only model names that unambiguously select a supported config."""
    normalized = " ".join(name.lower().split())
    if "quietcomfort 35" in normalized or re.search(r"\bqc35(?: ii)?\b", normalized):
        product_id = 0x4020 if (" ii" in normalized or "qc35 ii" in normalized) else 0x400C
        return product_id, "qc35"
    if re.search(r"\bqc ultra 2(?: hp| headphones)?\b", normalized):
        return 0x4082, "qc_ultra2"
    if ("quietcomfort ultra" in normalized or "qc ultra" in normalized) and (
            "2nd gen" in normalized or "second gen" in normalized or normalized.endswith(" 2")):
        return (0x4062 if "earbud" in normalized else 0x4082), "qc_ultra2"
    return None, None


def _channel_for_type(device_type):
    return {"qc_ultra2": 2, "qc35": 8}.get(device_type)


def _format_bluetooth_address(value):
    return ":".join("%02X" % ((value >> shift) & 0xFF) for shift in range(40, -1, -8))


def _scan_windows_devices():
    """Enumerate paired Windows radios/devices and retain BMAP services.

    ``BluetoothEnumerateInstalledServices`` reports the installed SDP service
    GUIDs.  The desktop API does not expose the RFCOMM channel, so it is selected
    only after an unambiguous supported model resolution.
    """
    if sys.platform != "win32":
        return []

    class SYSTEMTIME(ctypes.Structure):
        _fields_ = [(name, wintypes.WORD) for name in (
            "year", "month", "day_of_week", "day", "hour", "minute", "second", "milliseconds"
        )]

    class BLUETOOTH_DEVICE_INFO(ctypes.Structure):
        _fields_ = [
            ("dwSize", wintypes.DWORD), ("Address", ctypes.c_ulonglong),
            ("ulClassofDevice", wintypes.ULONG), ("fConnected", wintypes.BOOL),
            ("fRemembered", wintypes.BOOL), ("fAuthenticated", wintypes.BOOL),
            ("stLastSeen", SYSTEMTIME), ("stLastUsed", SYSTEMTIME),
            ("szName", wintypes.WCHAR * 248),
        ]

    class BLUETOOTH_DEVICE_SEARCH_PARAMS(ctypes.Structure):
        _fields_ = [
            ("dwSize", wintypes.DWORD), ("fReturnAuthenticated", wintypes.BOOL),
            ("fReturnRemembered", wintypes.BOOL), ("fReturnUnknown", wintypes.BOOL),
            ("fReturnConnected", wintypes.BOOL), ("fIssueInquiry", wintypes.BOOL),
            ("cTimeoutMultiplier", ctypes.c_ubyte), ("hRadio", wintypes.HANDLE),
        ]

    class BLUETOOTH_FIND_RADIO_PARAMS(ctypes.Structure):
        _fields_ = [("dwSize", wintypes.DWORD)]

    bth = ctypes.WinDLL("bthprops.cpl", use_last_error=True)
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    # ctypes assumes unannotated parameters are 32-bit C ``int`` values.  The
    # Bluetooth enumeration tokens and radio handles are pointer-sized, so that
    # default truncates 64-bit Windows handles (or raises OverflowError before
    # the API is called).
    bth.BluetoothFindFirstRadio.argtypes = [
        ctypes.POINTER(BLUETOOTH_FIND_RADIO_PARAMS), ctypes.POINTER(wintypes.HANDLE)
    ]
    bth.BluetoothFindFirstRadio.restype = wintypes.HANDLE
    bth.BluetoothFindNextRadio.argtypes = [
        wintypes.HANDLE, ctypes.POINTER(wintypes.HANDLE)
    ]
    bth.BluetoothFindNextRadio.restype = wintypes.BOOL
    bth.BluetoothFindFirstDevice.argtypes = [
        ctypes.POINTER(BLUETOOTH_DEVICE_SEARCH_PARAMS),
        ctypes.POINTER(BLUETOOTH_DEVICE_INFO),
    ]
    bth.BluetoothFindFirstDevice.restype = wintypes.HANDLE
    bth.BluetoothFindNextDevice.argtypes = [
        wintypes.HANDLE, ctypes.POINTER(BLUETOOTH_DEVICE_INFO)
    ]
    bth.BluetoothFindNextDevice.restype = wintypes.BOOL
    bth.BluetoothFindRadioClose.argtypes = [wintypes.HANDLE]
    bth.BluetoothFindRadioClose.restype = wintypes.BOOL
    bth.BluetoothFindDeviceClose.argtypes = [wintypes.HANDLE]
    bth.BluetoothFindDeviceClose.restype = wintypes.BOOL
    bth.BluetoothEnumerateInstalledServices.argtypes = [
        wintypes.HANDLE,
        ctypes.POINTER(BLUETOOTH_DEVICE_INFO),
        ctypes.POINTER(wintypes.DWORD),
        ctypes.c_void_p,
    ]
    bth.BluetoothEnumerateInstalledServices.restype = wintypes.DWORD
    kernel32.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel32.CloseHandle.restype = wintypes.BOOL
    radio_params = BLUETOOTH_FIND_RADIO_PARAMS(ctypes.sizeof(BLUETOOTH_FIND_RADIO_PARAMS))
    radio = wintypes.HANDLE()
    radio_find = bth.BluetoothFindFirstRadio(ctypes.byref(radio_params), ctypes.byref(radio))
    if not radio_find:
        raise BluetoothDiscoveryError(
            "BLUETOOTH_UNAVAILABLE", "No enabled Windows Bluetooth radio is available"
        )

    records = {}
    try:
        while True:
            search = BLUETOOTH_DEVICE_SEARCH_PARAMS(
                ctypes.sizeof(BLUETOOTH_DEVICE_SEARCH_PARAMS), True, True, False,
                True, False, 1, radio,
            )
            info = BLUETOOTH_DEVICE_INFO()
            info.dwSize = ctypes.sizeof(info)
            device_find = bth.BluetoothFindFirstDevice(ctypes.byref(search), ctypes.byref(info))
            if device_find:
                try:
                    while True:
                        if info.fAuthenticated or info.fRemembered:
                            services = _windows_service_guids(bth, radio, info)
                            if BMAP_UUID.lower() in services:
                                address = _format_bluetooth_address(info.Address)
                                product_id, device_type = _device_from_name(info.szName)
                                records[address] = DiscoveredDevice(
                                    address, info.szName or "Bose device", BMAP_UUID,
                                    _channel_for_type(device_type), product_id, device_type,
                                    True, bool(info.fConnected),
                                )
                        info = BLUETOOTH_DEVICE_INFO()
                        info.dwSize = ctypes.sizeof(info)
                        if not bth.BluetoothFindNextDevice(device_find, ctypes.byref(info)):
                            break
                finally:
                    bth.BluetoothFindDeviceClose(device_find)
            next_radio = wintypes.HANDLE()
            if not bth.BluetoothFindNextRadio(radio_find, ctypes.byref(next_radio)):
                break
            kernel32.CloseHandle(radio)
            radio = next_radio
    finally:
        kernel32.CloseHandle(radio)
        bth.BluetoothFindRadioClose(radio_find)
    return list(records.values())


def _windows_service_guids(bth, radio, device_info):
    """Return installed service GUID strings for a Windows device."""
    count = wintypes.DWORD(0)
    bth.BluetoothEnumerateInstalledServices(
        radio, ctypes.byref(device_info), ctypes.byref(count), None
    )
    if not count.value:
        return set()
    guid_array = (ctypes.c_byte * (16 * count.value))()
    result = bth.BluetoothEnumerateInstalledServices(
        radio, ctypes.byref(device_info), ctypes.byref(count), ctypes.byref(guid_array)
    )
    if result != 0:
        if result == 5:
            raise BluetoothDiscoveryError("ACCESS_DENIED", "Windows denied Bluetooth service discovery")
        return set()
    raw = bytes(guid_array)
    return {_guid_bytes_to_text(raw[i:i + 16]) for i in range(0, count.value * 16, 16)}


def _guid_bytes_to_text(raw):
    data1 = int.from_bytes(raw[0:4], "little")
    data2 = int.from_bytes(raw[4:6], "little")
    data3 = int.from_bytes(raw[6:8], "little")
    data4 = raw[8:]
    return "%08x-%04x-%04x-%s-%s" % (
        data1, data2, data3, data4[:2].hex(), data4[2:].hex()
    )
