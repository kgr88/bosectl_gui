"""RFCOMM Bluetooth socket transport for BMAP devices."""

import errno
import socket

from .errors import BmapConnectionError, BmapTimeoutError

RFCOMM_CHANNEL = 2  # BMAP protocol is always on RFCOMM channel 2


class RfcommTransport:
    """Raw RFCOMM Bluetooth socket transport.

    Manages the Bluetooth socket lifecycle and provides send/receive
    with optional multi-packet draining for async responses.

    Usage:
        with RfcommTransport("68:F2:1F:XX:XX:XX") as transport:
            resp = transport.send_recv(packet_bytes)
    """

    def __init__(self, mac, channel=RFCOMM_CHANNEL, timeout=3.0,
                 connect_timeout=None, response_timeout=None,
                 drain_timeout=0.5, socket_factory=None):
        self.mac = mac
        self.channel = channel
        self.connect_timeout = timeout if connect_timeout is None else connect_timeout
        self.timeout = timeout if response_timeout is None else response_timeout
        self.drain_timeout = drain_timeout
        self._socket_factory = socket_factory or socket.socket
        self._sock = None
        self._buffer = bytearray()

    def connect(self):
        """Open the RFCOMM socket to the device."""
        try:
            self._sock = self._socket_factory(
                socket.AF_BLUETOOTH, socket.SOCK_STREAM, socket.BTPROTO_RFCOMM
            )
            self._sock.settimeout(self.connect_timeout)
            self._sock.connect((self.mac, self.channel))
            self._sock.settimeout(self.timeout)
        except OSError as e:
            if self._sock is not None:
                try:
                    self._sock.close()
                except OSError:
                    pass
            self._sock = None
            raise _translate_socket_error(e, "connect to %s" % self.mac) from e

    def close(self):
        """Close the socket."""
        if self._sock:
            try:
                self._sock.close()
            except OSError:
                pass
            self._sock = None
            self._buffer.clear()

    def _recv_into_buffer(self):
        chunk = self._sock.recv(4096)
        if not chunk:
            raise BmapConnectionError("Device closed the RFCOMM connection")
        self._buffer.extend(chunk)

    def _recv_frame(self):
        """Read exactly one BMAP frame, preserving extra stream bytes."""
        while len(self._buffer) < 4:
            self._recv_into_buffer()
        frame_length = 4 + self._buffer[3]
        while len(self._buffer) < frame_length:
            self._recv_into_buffer()
        frame = bytes(self._buffer[:frame_length])
        del self._buffer[:frame_length]
        return frame

    def send_recv(self, packet, drain=False):
        """Send a BMAP packet and receive the response.

        Args:
            packet: Raw bytes to send.
            drain: If True, keep reading until the socket times out.
                   Needed for commands that return multiple STATUS
                   messages (e.g., GetAll).

        Returns:
            Raw response bytes (may contain multiple concatenated packets
            if drain=True).

        Raises:
            BmapTimeoutError: If no response is received.
        """
        if not self._sock:
            raise BmapConnectionError("Not connected")
        try:
            self._sock.sendall(packet)
            data = self._recv_frame()
        except socket.timeout:
            if self._buffer:
                self._buffer.clear()
                raise BmapConnectionError("Device returned a truncated BMAP frame")
            raise BmapTimeoutError("No response from device")
        except OSError as e:
            raise _translate_socket_error(e, "communicate with device") from e

        if drain:
            previous_timeout = self._sock.gettimeout()
            self._sock.settimeout(self.drain_timeout)
            try:
                while True:
                    data += self._recv_frame()
            except (socket.timeout, BlockingIOError):
                if self._buffer:
                    self._buffer.clear()
                    raise BmapConnectionError("Device returned a truncated BMAP frame")
            except OSError as e:
                raise _translate_socket_error(e, "drain device responses") from e
            finally:
                self._sock.settimeout(previous_timeout)

        return data

    def __enter__(self):
        self.connect()
        return self

    def __exit__(self, *exc):
        self.close()


def _translate_socket_error(error, action):
    """Translate platform socket failures without leaking unstable errno text."""
    code = getattr(error, "winerror", None) or getattr(error, "errno", None)
    timed_out = {errno.ETIMEDOUT, 10060}
    denied = {errno.EACCES, errno.EPERM, 10013}
    unavailable = {errno.EAFNOSUPPORT, errno.EPROTONOSUPPORT, 10047, 10043}
    lost = {errno.ECONNRESET, errno.ENOTCONN, errno.ECONNABORTED, 10053, 10054, 10057}
    if code in timed_out:
        return BmapTimeoutError("Timed out while attempting to %s" % action)
    if code in denied:
        return BmapConnectionError("Bluetooth access was denied while attempting to %s" % action)
    if code in unavailable:
        return BmapConnectionError("Bluetooth RFCOMM is unavailable")
    if code in lost:
        return BmapConnectionError("The Bluetooth device connection was lost")
    return BmapConnectionError("Failed to %s" % action)
