"""Process boundary for the long-lived GUI backend.

The backend is intentionally toolkit-free. It owns one serialized BMAP
connection and exposes only validated immutable wire values.
"""

from .protocol import PROTOCOL_VERSION

__all__ = ["PROTOCOL_VERSION"]
