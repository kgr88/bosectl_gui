"""Serialized backend service surface.

Stage 1 deliberately contains no Bluetooth implementation.  This class fixes the
dispatcher shape so Stage 2 can fill in device behavior without changing IPC.
"""

import platform

from pybmap import __version__

from .protocol import METHODS, PROTOCOL_VERSION, ProtocolError


class BackendService(object):
    def __init__(self):
        self.stopping = False

    def handle(self, method, params):
        if method not in METHODS:
            raise ProtocolError("UNKNOWN_METHOD", "unknown method")
        if self.stopping and method != "shutdown":
            raise ProtocolError("SHUTTING_DOWN", "backend is shutting down")
        if method == "hello":
            return {
                "backend_version": __version__,
                "python_version": platform.python_version(),
                "protocol_version": PROTOCOL_VERSION,
                "operations": list(METHODS),
            }
        if method == "shutdown":
            self.stopping = True
            return {"ok": True}
        raise ProtocolError(
            "BACKEND_UNAVAILABLE",
            "%s is part of the frozen Stage 1 contract and is implemented in Stage 2" % method,
        )
