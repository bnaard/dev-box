"""Gateway session helpers."""

from __future__ import annotations

import os


def session_id(default: str | None = None) -> str:
    """Return the explicit processkit session id for this connection."""
    value = os.environ.get("PROCESSKIT_SESSION_ID") or default
    if value:
        return value
    return f"pid-{os.getpid()}"
