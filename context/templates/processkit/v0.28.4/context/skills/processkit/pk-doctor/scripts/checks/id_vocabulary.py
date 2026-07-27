"""ID vocabulary and shorthand health checks."""

from __future__ import annotations

import sqlite3
import sys
from pathlib import Path

from .common import CheckResult


CATEGORY = "id_vocabulary"
NORMAL_PAIR_TARGET = 50_000
HIGH_VOLUME_TARGET = 250_000
CONFIGURED_KIND_MODE = "double_adjective"


def _add_processkit_lib_to_path(repo_root: Path) -> None:
    for candidate in (
        repo_root / "src" / "context" / "skills" / "_lib",
        repo_root / "context" / "skills" / "_lib",
        repo_root / "src" / "lib",
        repo_root / "_lib",
    ):
        if not (candidate / "processkit" / "__init__.py").is_file():
            continue
        if str(candidate) not in sys.path:
            sys.path.insert(0, str(candidate))
            return
        return


def _sqlite_vec_available() -> bool:
    try:
        import sqlite3 as _sqlite3
        import sqlite_vec  # type: ignore

        conn = _sqlite3.connect(":memory:")
        try:
            conn.enable_load_extension(True)
            sqlite_vec.load(conn)
            conn.enable_load_extension(False)
            return True
        finally:
            conn.close()
    except Exception:
        return False


def _indexed_ids(repo_root: Path) -> list[str]:
    db_path = repo_root / "context" / ".cache" / "processkit" / "index.sqlite"
    if not db_path.is_file():
        return []
    conn = sqlite3.connect(db_path)
    try:
        rows = conn.execute("SELECT id FROM entities ORDER BY id").fetchall()
        return [str(row[0]) for row in rows if row and row[0]]
    finally:
        conn.close()


def _configured_palette_kinds(ids_module) -> tuple[str, ...]:
    if hasattr(ids_module, "configured_palette_kinds"):
        return tuple(ids_module.configured_palette_kinds())
    palettes = getattr(ids_module, "_KIND_PALETTES", {})
    return tuple(sorted(palettes))


def _palette_for_kind(ids_module, kind: str) -> tuple[str, ...]:
    if hasattr(ids_module, "palette_for_kind"):
        return tuple(ids_module.palette_for_kind(kind))
    palettes = getattr(ids_module, "_KIND_PALETTES", {})
    return tuple(palettes.get(kind, ()))


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    _add_processkit_lib_to_path(repo_root)
    try:
        from processkit import ids
    except Exception as exc:
        return [CheckResult(
            severity="WARN",
            category=CATEGORY,
            id="id-vocabulary.import-failed",
            message=f"could not import processkit ID helpers: {exc}",
            suggested_fix=(
                "run pk-doctor from a processkit checkout with "
                "src/context/skills/_lib available"
            ),
        )]

    existing = _indexed_ids(repo_root)
    results: list[CheckResult] = []

    kind_reports = []
    for kind in _configured_palette_kinds(ids):
        tags = _palette_for_kind(ids, kind)
        report = ids.vocabulary_capacity_report(
            palette_tags=tags,
            allocation_mode=CONFIGURED_KIND_MODE,
            existing=existing,
        )
        kind_reports.append({
            "kind": kind,
            "palette_tags": list(tags),
            "capacity": int(report["capacity"]),
            "report": report,
        })

    low_kind_reports = [
        item for item in kind_reports
        if int(item["capacity"]) < NORMAL_PAIR_TARGET
    ]
    min_kind_capacity = min(
        (int(item["capacity"]) for item in kind_reports),
        default=0,
    )
    if low_kind_reports:
        results.append(CheckResult(
            severity="WARN",
            category=CATEGORY,
            id="id-vocabulary.configured-kind-capacity-low",
            message=(
                f"{len(low_kind_reports)} configured kind palette(s) are below "
                f"the {NORMAL_PAIR_TARGET}+ durable shorthand target"
            ),
            suggested_fix=(
                "add more tagged nouns or adjectives for the listed kind "
                "palette(s), or assign a broader palette"
            ),
            extra={
                "allocation_mode": CONFIGURED_KIND_MODE,
                "target": NORMAL_PAIR_TARGET,
                "low_kinds": low_kind_reports,
                "configured_kinds": kind_reports,
            },
        ))
    else:
        results.append(CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="id-vocabulary.configured-kind-capacity-ok",
            message=(
                "configured kind palettes meet durable shorthand target "
                f"(minimum {min_kind_capacity} combinations)"
            ),
            extra={
                "allocation_mode": CONFIGURED_KIND_MODE,
                "target": NORMAL_PAIR_TARGET,
                "configured_kinds": kind_reports,
            },
        ))

    workitem_tags = _palette_for_kind(ids, "WorkItem")
    workitem_double = ids.vocabulary_capacity_report(
        palette_tags=workitem_tags,
        allocation_mode="double_adjective",
        existing=existing,
    )
    if int(workitem_double["capacity"]) < HIGH_VOLUME_TARGET:
        results.append(CheckResult(
            severity="WARN",
            category=CATEGORY,
            id="id-vocabulary.high-volume-capacity-low",
            message=(
                "WorkItem double-adjective palette is below the "
                f"{HIGH_VOLUME_TARGET} high-volume target"
            ),
            suggested_fix=(
                "add more tagged nouns or adjectives before enabling "
                "high-volume mode"
            ),
            extra=workitem_double,
        ))
    else:
        results.append(CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="id-vocabulary.high-volume-capacity-ok",
            message=(
                "WorkItem double-adjective palette is ready for high-volume "
                f"allocation ({workitem_double['capacity']} combinations)"
            ),
            extra=workitem_double,
        ))

    ambiguities = ids.detect_lexical_ambiguities(existing)
    for token, matches in sorted(ambiguities.items())[:20]:
        results.append(CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="id-vocabulary.lexical-token-historical-ambiguity",
            message=(
                f"historical lexical shorthand {token!r} resolves to "
                f"{len(matches)} IDs; use full IDs when referring to them"
            ),
            entity_ref=token,
            action_required=False,
            extra={"matches": matches[:10]},
        ))

    blocked: list[tuple[str, str, list[str]]] = []
    for entity_id in existing:
        token = ids.lexical_token_from_id(entity_id)
        if not token:
            continue
        if (
            hasattr(ids, "is_managed_lexical_token")
            and not ids.is_managed_lexical_token(token)
        ):
            continue
        words = ids.blocked_words_in_token(token)
        if words:
            blocked.append((entity_id, token, words))
    for entity_id, token, words in blocked[:20]:
        results.append(CheckResult(
            severity="WARN",
            category=CATEGORY,
            id="id-vocabulary.blocked-word",
            message=f"{entity_id} uses blocked process word(s) in {token!r}: {words}",
            entity_ref=entity_id,
            suggested_fix=(
                "avoid operational/process vocabulary in future generated IDs"
            ),
        ))

    if _sqlite_vec_available():
        results.append(CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="id-vocabulary.semantic-index-ready",
            message="sqlite-vec is available for RAG-assisted palette selection",
        ))
    else:
        results.append(CheckResult(
            severity="WARN",
            category=CATEGORY,
            id="id-vocabulary.semantic-index-unavailable",
            message=(
                "sqlite-vec is unavailable; RAG-assisted ID palette selection "
                "will fall back to deterministic lexical scoring"
            ),
            suggested_fix="install sqlite-vec in the MCP runtime",
        ))

    return results
