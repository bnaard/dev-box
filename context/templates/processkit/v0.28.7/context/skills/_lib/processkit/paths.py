"""Locate the project root, processkit checkout, and per-kind directories.

Project layout (consumer side):

    <project-root>/
      AGENTS.md            ← provider-neutral root marker (preferred)
      aibox.toml           ← aibox installer config (accepted as fallback marker)
      context/
        skills/<name>/...           ← installed processkit skills
        skills/_lib/processkit/...  ← installed processkit lib
        schemas/<f>.yaml            ← installed primitive schemas
        state-machines/<f>.yaml     ← installed state machines
        processes/<f>.md            ← installed process definitions
        templates/processkit/<v>/   ← full upstream reference templates
        workitems/, decisions/, logs/, ...
        .cache/processkit/          ← gitignored runtime cache (index DB)

Processkit checkout layout (this repo, when running servers from
a checkout):

    <processkit-root>/
      AGENTS.md
      src/
        context/
          schemas/
          state-machines/
          skills/
      context/

Servers may be invoked in either context. The functions here resolve
paths by walking upward from cwd looking for a project root marker.
AGENTS.md is the preferred marker; aibox.toml is accepted as a
fallback for projects that have not yet adopted AGENTS.md.
"""
from __future__ import annotations

import os
from pathlib import Path

from . import DEFAULT_DIRS

# Markers tried in order. First match wins.
_PROJECT_MARKERS = ("AGENTS.md", "aibox.toml")


def find_project_root(start: Path | str | None = None) -> Path:
    """Walk upward from ``start`` (default: cwd) looking for a project marker.

    Tries AGENTS.md, processkit.toml, and aibox.toml in that order.
    Returns the directory containing the first marker found. If none is
    found, returns the starting directory so callers can still proceed
    with sensible defaults rather than crashing.
    """
    here = Path(start) if start else Path.cwd()
    here = here.resolve()
    if here.is_file():
        here = here.parent
    candidate = here
    while True:
        for marker in _PROJECT_MARKERS:
            if (candidate / marker).is_file():
                return candidate
        if candidate.parent == candidate:
            return here
        candidate = candidate.parent


def find_processkit_root(server_path: Path | str) -> Path | None:
    """Locate the processkit checkout root containing ``server_path``.

    Looks for ``src/context/skills/_lib/processkit/__init__.py`` upward from
    the given server script. Returns the directory containing ``src/`` or None
    if the script is not running from a processkit checkout (e.g. installed by
    aibox into ``context/skills/``).
    """
    here = Path(server_path).resolve().parent
    while True:
        if (
            here
            / "src"
            / "context"
            / "skills"
            / "_lib"
            / "processkit"
            / "__init__.py"
        ).is_file():
            return here
        if here.parent == here:
            return None
        here = here.parent


def context_dir(kind: str, root: Path | str | None = None) -> Path:
    """Return the base directory under ``context/`` for entities of ``kind``.

    Reads directory overrides from processkit.toml (or aibox.toml legacy)
    via config.load_config(); otherwise uses the default subdirectory from
    DEFAULT_DIRS.

    This returns the *root* of the kind's directory, without any sharding
    subdirectories. Use ``entity_path()`` when writing a new entity so that
    sharding is applied automatically.
    """
    from . import config  # local import to avoid circular dependency at module level
    root = Path(root) if root else find_project_root()
    cfg = config.load_config(root)
    override = cfg.directory_for(kind)
    sub = override if override else DEFAULT_DIRS.get(kind, kind.lower())
    return root / "context" / sub


def _parse_iso_timestamp(created_at: str | None):
    from datetime import datetime, timezone

    if created_at:
        ts_str = created_at.replace("Z", "+00:00")
        try:
            return datetime.fromisoformat(ts_str)
        except ValueError:
            pass
    return datetime.now(timezone.utc)


def _render_shard_template(template: str, *, base: Path, ts, state: str | None) -> Path:
    rendered = template.format(
        year=f"{ts.year:04d}",
        month=f"{ts.month:02d}",
        state=state or "",
    ).strip("/")
    if rendered.startswith("context/"):
        return base.parent / rendered.removeprefix("context/")
    return base / rendered


def entity_path(
    kind: str,
    entity_id: str,
    created_at: str | None = None,
    root: Path | str | None = None,
    state: str | None = None,
    storage_location: str = "live",
) -> Path:
    """Return the full file path for a new entity, applying sharding rules.

    When date-based sharding is configured for ``kind``, entities are
    written into ``context/<subdir>/{year}/{month}/`` rather than the flat
    ``context/<subdir>/`` directory. The ``created_at`` parameter (an ISO
    8601 string) is used to derive the year and month. If ``created_at``
    is None, the current UTC date is used.

    Falls back to ``context_dir(kind) / entity_id.md`` when sharding is
    not configured.
    """
    from . import config  # local import to avoid circular dependency

    root = Path(root) if root else find_project_root()
    base = context_dir(kind, root)
    cfg = config.load_config(root)

    shard = cfg.sharding_for(kind)
    if shard:
        ts = _parse_iso_timestamp(created_at)
        terminal_states = set(shard.get("archive_states") or [])
        if storage_location == "archive" or (state and state in terminal_states):
            archive_template = shard.get("archive_template")
            if archive_template:
                return _render_shard_template(
                    archive_template,
                    base=base,
                    ts=ts,
                    state=state,
                ) / f"{entity_id}.md"

        pattern = shard.get("scheme") or shard.get("pattern")
        template = shard.get("template")
        if not template and isinstance(shard.get("pattern"), str):
            pattern_value = shard["pattern"]
            if "{" in pattern_value or "/" in pattern_value:
                template = pattern_value
        if pattern in {"date", "date-shard"}:
            if template:
                shard_dir = _render_shard_template(
                    template,
                    base=base,
                    ts=ts,
                    state=state,
                )
            else:
                shard_dir = base / f"{ts.year:04d}" / f"{ts.month:02d}"
            return shard_dir / f"{entity_id}.md"
        if pattern == "lifecycle-shard" and state:
            if template:
                shard_dir = _render_shard_template(
                    template,
                    base=base,
                    ts=ts,
                    state=state,
                )
            else:
                shard_dir = base / state
            return shard_dir / f"{entity_id}.md"

    return base / f"{entity_id}.md"


def storage_location_for_path(
    kind: str,
    path: Path | str,
    root: Path | str | None = None,
) -> str:
    """Return the v2 storage location label for an entity path."""
    from . import config  # local import to avoid circular dependency

    root = Path(root) if root else find_project_root()
    p = Path(path)
    try:
        rel = p.resolve().relative_to((root / "context").resolve())
    except ValueError:
        return "external"
    if rel.parts and rel.parts[0] == "archives":
        return "archive"
    cfg = config.load_config(root)
    shard = cfg.sharding_for(kind)
    if not shard:
        return "live"
    archive_template = str(shard.get("archive_template") or "")
    archive_prefix = archive_template.removeprefix("context/").split("{", 1)[0].strip("/")
    if archive_prefix and str(rel).startswith(archive_prefix):
        return "archive"
    return "live"


def primitive_schemas_dir(root: Path | str | None = None) -> Path | None:
    """Where the JSON-Schema YAML files live.

    Tries the consumer's installed location (``context/schemas/``)
    first, then the processkit checkout (``src/context/schemas/``).
    Returns None if neither exists.
    """
    root = Path(root) if root else find_project_root()
    consumer = root / "context" / "schemas"
    if consumer.is_dir():
        return consumer
    processkit = root / "src" / "context" / "schemas"
    if processkit.is_dir():
        return processkit
    return None


def state_machines_dir(root: Path | str | None = None) -> Path | None:
    """Where state machine YAML files live.

    Tries the consumer's installed location (``context/state-machines/``)
    first, then the processkit checkout
    (``src/context/state-machines/``). Returns None if neither
    exists.
    """
    root = Path(root) if root else find_project_root()
    consumer = root / "context" / "state-machines"
    if consumer.is_dir():
        return consumer
    processkit = root / "src" / "context" / "state-machines"
    if processkit.is_dir():
        return processkit
    return None


def index_db_path(root: Path | str | None = None) -> Path:
    """Path to the SQLite index database (gitignored runtime cache).

    Reads an override from processkit.toml ``[index] path`` if set.
    Otherwise uses the default ``context/.cache/processkit/index.sqlite``.
    Creates the parent directory as a side effect.
    """
    from . import config  # local import to avoid circular at module level
    root = Path(root) if root else find_project_root()
    cfg = config.load_config(root)
    if cfg.index_path:
        db = root / cfg.index_path
    else:
        db = root / "context" / ".cache" / "processkit" / "index.sqlite"
    db.parent.mkdir(parents=True, exist_ok=True)
    return db


def env_override(name: str) -> Path | None:
    """Return an env-var path override if set, else None."""
    value = os.environ.get(name)
    return Path(value) if value else None
