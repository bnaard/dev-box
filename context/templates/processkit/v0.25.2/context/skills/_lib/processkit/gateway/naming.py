"""Tool naming and collision handling for processkit gateway mode."""

from __future__ import annotations

from collections.abc import Iterable
from typing import Any


def collision_groups(entries: Iterable[dict[str, Any]]) -> dict[str, list[str]]:
    groups: dict[str, list[str]] = {}
    for entry in entries:
        source_tool = entry["source_tool"]
        groups.setdefault(source_tool, []).append(entry["source_skill"])
    return groups


def registered_name(
    entry: dict[str, Any],
    used_names: set[str],
    duplicate_index: int,
) -> str:
    original_name = entry["source_tool"]
    if duplicate_index == 0 and original_name not in used_names:
        return original_name

    prefix = entry["source_skill"].replace("-", "_")
    candidate = f"{prefix}__{original_name}"
    if candidate not in used_names:
        return candidate

    suffix = 2
    while f"{candidate}_{suffix}" in used_names:
        suffix += 1
    return f"{candidate}_{suffix}"
