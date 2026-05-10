"""Permission metadata derived from MCP tool annotations."""

from __future__ import annotations

from typing import Any


def annotation_metadata(annotations: Any) -> dict[str, Any] | None:
    """Return JSON-serializable MCP annotation metadata."""
    if annotations is None:
        return None

    if hasattr(annotations, "model_dump"):
        data = annotations.model_dump(exclude_none=True)
    elif isinstance(annotations, dict):
        data = annotations
    else:
        data = {
            key: getattr(annotations, key)
            for key in (
                "title",
                "readOnlyHint",
                "destructiveHint",
                "idempotentHint",
                "openWorldHint",
            )
            if hasattr(annotations, key)
        }

    return {
        str(key): value
        for key, value in data.items()
        if isinstance(value, (str, int, float, bool, list, dict, type(None)))
    }


def permission_class(annotations: Any) -> str:
    """Classify a tool coarsely for gateway policy filtering."""
    data = annotation_metadata(annotations) or {}
    if data.get("readOnlyHint") is True:
        return "read-only"
    if data.get("destructiveHint") is True:
        return "destructive-write"
    return "guarded-write"
