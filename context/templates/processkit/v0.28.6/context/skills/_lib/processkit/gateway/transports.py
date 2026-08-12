"""Serialization helpers for gateway-exposed transport metadata."""

from __future__ import annotations

from enum import Enum
from typing import Any


class TransportKind(str, Enum):
    """Known processkit gateway transport modes."""

    STDIO = "stdio"
    HTTP = "http"
    IN_PROCESS = "in-process"


ANNOTATION_FIELDS = (
    "title",
    "readOnlyHint",
    "destructiveHint",
    "idempotentHint",
    "openWorldHint",
)


JsonScalar = str | int | float | bool | None
JsonValue = JsonScalar | list["JsonValue"] | dict[str, "JsonValue"]


def _json_safe(value: Any) -> JsonValue | None:
    if isinstance(value, (str, int, float, bool)) or value is None:
        return value
    if isinstance(value, list):
        return [
            item
            for raw_item in value
            if (item := _json_safe(raw_item)) is not None
        ]
    if isinstance(value, dict):
        return {
            str(key): item
            for key, raw_item in value.items()
            if (item := _json_safe(raw_item)) is not None
        }
    return None


def annotation_metadata(annotations: Any) -> dict[str, JsonValue] | None:
    """Serialize MCP tool annotations to JSON-safe metadata.

    Supports pydantic models, plain dictionaries, and simple objects with
    the standard MCP annotation attributes used by FastMCP tools.
    """

    if annotations is None:
        return None

    if hasattr(annotations, "model_dump"):
        raw_data = annotations.model_dump(exclude_none=True)
    elif isinstance(annotations, dict):
        raw_data = annotations
    else:
        raw_data = {
            field: getattr(annotations, field)
            for field in ANNOTATION_FIELDS
            if hasattr(annotations, field)
        }

    return {
        str(key): safe_value
        for key, raw_value in raw_data.items()
        if (safe_value := _json_safe(raw_value)) is not None
    }
