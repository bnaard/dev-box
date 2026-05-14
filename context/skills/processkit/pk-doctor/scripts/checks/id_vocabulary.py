"""ID vocabulary and shorthand health checks."""

from __future__ import annotations

import sqlite3
import sys
from pathlib import Path

from .common import CheckResult


CATEGORY = "id_vocabulary"
NORMAL_PAIR_TARGET = 50_000
HIGH_VOLUME_TARGET = 250_000


def _add_processkit_lib_to_path(repo_root: Path) -> None:
    for candidate in (
        repo_root / "src" / "context" / "skills" / "_lib",
        repo_root / "context" / "skills" / "_lib",
        repo_root / "src" / "lib",
        repo_root / "_lib",
    ):
        if (
            (candidate / "processkit" / "__init__.py").is_file()
            and str(candidate) not in sys.path
        ):
            sys.path.insert(0, str(candidate))
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

    default_report = ids.vocabulary_capacity_report(existing=existing)
    default_capacity = int(default_report["capacity"])
    if default_capacity < NORMAL_PAIR_TARGET:
        results.append(CheckResult(
            severity="WARN",
            category=CATEGORY,
            id="id-vocabulary.default-pair-capacity-low",
            message=(
                f"default two-word vocabulary has {default_capacity} pair(s); "
                f"target for durable human shorthand is {NORMAL_PAIR_TARGET}+"
            ),
            suggested_fix=(
                "enable tagged palettes and use double_adjective mode for "
                "high-volume entity kinds"
            ),
            extra=default_report,
        ))
    else:
        results.append(CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="id-vocabulary.default-pair-capacity-ok",
            message=f"default two-word vocabulary has {default_capacity} pair(s)",
            extra=default_report,
        ))

    workitem_tags = ids.palette_for_kind("WorkItem")
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
            severity="WARN",
            category=CATEGORY,
            id="id-vocabulary.lexical-token-ambiguous",
            message=f"lexical shorthand {token!r} resolves to {len(matches)} IDs",
            entity_ref=token,
            suggested_fix=(
                "prefer full IDs for these entities; future allocations should "
                "reserve lexical tokens globally"
            ),
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
