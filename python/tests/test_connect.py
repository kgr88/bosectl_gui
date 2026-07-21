"""Connection setup behavior."""

import pybmap
from pybmap.errors import BmapConnectionError


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

    connection = pybmap.connect(
        mac="68:F2:1F:00:00:01",
        device_type="qc_ultra2",
        transport_factory=FakeTransport,
    )

    assert [attempt.channel for attempt in attempts] == [2, 1]
    assert attempts[0].closed is True
    assert connection._transport is attempts[1]
