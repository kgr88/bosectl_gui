"""Connection setup behavior."""

import pybmap
from pybmap.errors import BmapConnectionError


VALID_BATTERY_RESPONSE = b"\x02\x02\x03\x04\x50\xff\xff\x00"


def test_qc_ultra2_falls_back_to_alternate_rfcomm_channel():
    attempts = []

    class FakeTransport:
        def __init__(self, mac, channel):
            self.mac = mac
            self.channel = channel
            self.closed = False
            attempts.append(self)

        def connect(self):
            if self.channel == 2:
                raise BmapConnectionError("RFCOMM channel rejected")

        def close(self):
            self.closed = True

        def send_recv(self, packet):
            return VALID_BATTERY_RESPONSE

    connection = pybmap.connect(
        mac="68:F2:1F:00:00:01",
        device_type="qc_ultra2",
        transport_factory=FakeTransport,
        channel_resolver=lambda _mac: [],
    )

    assert [attempt.channel for attempt in attempts] == [2, 1]
    assert attempts[0].closed is True
    assert connection._transport is attempts[1]


def test_sdp_channel_is_tried_before_unique_catalog_fallbacks():
    attempts = []

    class FakeTransport:
        def __init__(self, mac, channel):
            self.channel = channel
            attempts.append(channel)

        def connect(self): pass
        def close(self): pass
        def send_recv(self, packet): return VALID_BATTERY_RESPONSE

    connection = pybmap.connect(
        mac="68:F2:1F:00:00:01", device_type="qc_ultra2",
        transport_factory=FakeTransport, channel_resolver=lambda _mac: [7, 2],
    )

    assert attempts == [7]
    assert connection.rfcomm_channel == 7


def test_connected_non_bmap_endpoint_is_rejected_before_fallback():
    attempts = []

    class FakeTransport:
        def __init__(self, mac, channel):
            self.channel = channel
            self.closed = False
            attempts.append(self)

        def connect(self): pass
        def close(self): self.closed = True
        def send_recv(self, packet):
            if self.channel == 7:
                return b"\x05\x01\x03\x01\x00"
            return VALID_BATTERY_RESPONSE

    connection = pybmap.connect(
        mac="68:F2:1F:00:00:01", device_type="qc_ultra2",
        transport_factory=FakeTransport, channel_resolver=lambda _mac: [7],
    )

    assert [attempt.channel for attempt in attempts] == [7, 2]
    assert attempts[0].closed is True
    assert connection.rfcomm_channel == 2
