"""Process boundary for the long-lived GUI backend.

The backend is intentionally toolkit-free.  Stage 1 freezes its wire contract;
device discovery and BMAP operations are implemented in Stage 2.
"""

from .protocol import PROTOCOL_VERSION

__all__ = ["PROTOCOL_VERSION"]
