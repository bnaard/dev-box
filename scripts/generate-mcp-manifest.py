#!/usr/bin/env python3
"""Generate context/.processkit-mcp-manifest.json for pk-doctor checks."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


GATEWAY_CONFIG_REL = Path(
    "context/skills/processkit/processkit-gateway/mcp/mcp-config.json"
)
HEADER_OPEN = "# /// script"
HEADER_CLOSE = "# ///"


def _canonical_json(data: object) -> str:
    return json.dumps(data, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def _sha256_of_json(path: Path) -> str:
    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)
    return hashlib.sha256(_canonical_json(data).encode("utf-8")).hexdigest()


def _aggregate(entries: list[dict[str, str]]) -> str:
    joined = "\n".join(entry["sha256"] for entry in entries)
    return hashlib.sha256(joined.encode("utf-8")).hexdigest()


def _collect_configs(root: Path, *, gateway: bool) -> list[dict[str, str]]:
    skills_root = root / "context" / "skills"
    entries: list[dict[str, str]] = []
    seen: set[str] = set()
    for pattern in ("*/*/mcp/mcp-config.json", "*/mcp/mcp-config.json"):
        for cfg in sorted(skills_root.glob(pattern)):
            rel = cfg.relative_to(root)
            is_gateway = rel == GATEWAY_CONFIG_REL
            if is_gateway != gateway:
                continue
            rel_str = rel.as_posix()
            if rel_str in seen:
                continue
            seen.add(rel_str)
            entries.append({"path": rel_str, "sha256": _sha256_of_json(cfg)})
    entries.sort(key=lambda entry: entry["path"])
    return entries


def _collect_entries(root: Path) -> list[dict[str, str]]:
    """Backward-compatible alias used by pk-doctor smoke tests."""
    return _collect_configs(root, gateway=False)


def _collect_gateway_entries(root: Path) -> list[dict[str, str]]:
    """Backward-compatible alias used by pk-doctor smoke tests."""
    return _collect_configs(root, gateway=True)


def _extract_header(path: Path) -> str | None:
    lines = path.read_text(encoding="utf-8").splitlines()
    in_block = False
    block: list[str] = []
    for line in lines:
        stripped = line.strip()
        if not in_block:
            if stripped == HEADER_OPEN:
                in_block = True
                block.append(line)
            continue
        block.append(line)
        if stripped == HEADER_CLOSE:
            return "\n".join(block) + "\n"
    return None


def _collect_server_headers(root: Path) -> list[dict[str, str]]:
    entries: list[dict[str, str]] = []
    for server in sorted((root / "context" / "skills").glob("*/*/mcp/server.py")):
        header = _extract_header(server)
        if header is None:
            continue
        entries.append({
            "path": server.relative_to(root).as_posix(),
            "sha256": hashlib.sha256(header.encode("utf-8")).hexdigest(),
        })
    entries.sort(key=lambda entry: entry["path"])
    return entries


def main() -> int:
    root = Path.cwd()
    per_skill = _collect_configs(root, gateway=False)
    per_gateway = _collect_configs(root, gateway=True)
    manifest = {
        "schema_version": 1,
        "per_skill": per_skill,
        "per_gateway": per_gateway,
        "per_server_header": _collect_server_headers(root),
        "aggregate_sha256": _aggregate(per_skill),
    }
    out = root / "context" / ".processkit-mcp-manifest.json"
    out.write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(f"wrote {out.relative_to(root)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
