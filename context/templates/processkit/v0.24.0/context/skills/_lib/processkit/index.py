"""SQLite indexer for processkit entity files.

This module powers the ``index-management`` MCP server. It walks the
project's ``context/`` directory, parses every entity file, and writes a
SQLite database that other tools can query.

The schema is intentionally simple — one row per entity, with the most
common fields denormalized into columns and the full spec stashed as
JSON for queries we did not anticipate.
"""
from __future__ import annotations

import json
import math
import sqlite3
import struct
from dataclasses import dataclass
from hashlib import blake2b
from pathlib import Path
from typing import Any, Iterable
import re

from . import entity as entity_mod
from . import paths


SCHEMA_DDL = """
CREATE TABLE IF NOT EXISTS entities (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    api_version TEXT NOT NULL,
    path TEXT NOT NULL,
    storage_location TEXT,
    created TEXT NOT NULL,
    updated TEXT,
    title TEXT,
    state TEXT,
    labels_json TEXT,
    spec_json TEXT NOT NULL,
    body TEXT
);
CREATE INDEX IF NOT EXISTS idx_entities_kind ON entities (kind);
CREATE INDEX IF NOT EXISTS idx_entities_state ON entities (state);
CREATE INDEX IF NOT EXISTS idx_entities_created ON entities (created);

CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL,
    actor TEXT,
    subject TEXT,
    subject_kind TEXT,
    summary TEXT,
    details_json TEXT,
    correlation_id TEXT,
    path TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_events_type ON events (event_type);
CREATE INDEX IF NOT EXISTS idx_events_subject ON events (subject);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (timestamp);

CREATE TABLE IF NOT EXISTS errors (
    path TEXT PRIMARY KEY,
    message TEXT NOT NULL
);
"""

FTS_SCHEMA_DDL = """
CREATE VIRTUAL TABLE IF NOT EXISTS entities_fts USING fts5(
    id UNINDEXED,
    kind UNINDEXED,
    state UNINDEXED,
    title,
    body,
    spec_json,
    tokenize='unicode61'
);
"""

VECTOR_DIMENSIONS = 128

SEMANTIC_SCHEMA_DDL = """
CREATE TABLE IF NOT EXISTS semantic_chunks (
    rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    chunk_id TEXT UNIQUE NOT NULL,
    entity_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    state TEXT,
    path TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    text TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_semantic_chunks_entity ON semantic_chunks (entity_id);
CREATE INDEX IF NOT EXISTS idx_semantic_chunks_kind ON semantic_chunks (kind);
"""

VEC_SCHEMA_DDL = f"""
CREATE VIRTUAL TABLE IF NOT EXISTS entity_vec USING vec0(
    embedding float[{VECTOR_DIMENSIONS}]
);
"""


@dataclass
class IndexStats:
    entities: int
    events: int
    errors: int


def open_db(path: Path | None = None) -> sqlite3.Connection:
    db_path = Path(path) if path else paths.index_db_path()
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    # WAL mode lets multiple MCP servers read while one writes without
    # hitting "database is locked". Persists across connections — only
    # needs to be set once per database file, but PRAGMA is cheap to
    # re-issue. synchronous=NORMAL is the WAL-recommended pairing.
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA synchronous=NORMAL")
    conn.executescript(SCHEMA_DDL)
    _migrate_schema(conn)
    conn.executescript(SEMANTIC_SCHEMA_DDL)
    try:
        conn.executescript(FTS_SCHEMA_DDL)
    except sqlite3.OperationalError:
        # FTS5 is built into supported processkit environments, but keep
        # the index usable on stripped-down SQLite builds.
        pass
    if _load_sqlite_vec(conn):
        try:
            conn.executescript(VEC_SCHEMA_DDL)
        except sqlite3.OperationalError:
            pass
    return conn


def _migrate_schema(db: sqlite3.Connection) -> None:
    """Apply lightweight additive migrations for existing index DBs."""
    columns = {
        row["name"]
        for row in db.execute("PRAGMA table_info(entities)").fetchall()
    }
    if "storage_location" not in columns:
        db.execute("ALTER TABLE entities ADD COLUMN storage_location TEXT")


def reindex(root: Path | None = None, db: sqlite3.Connection | None = None) -> IndexStats:
    """Walk ``<root>/context/`` and rebuild the index from scratch."""
    root = root or paths.find_project_root()
    own = db is None
    if db is None:
        db = open_db()
    db.execute("DELETE FROM entities")
    db.execute("DELETE FROM events")
    db.execute("DELETE FROM errors")
    if _has_fts(db):
        db.execute("DELETE FROM entities_fts")
    if _has_vec(db):
        db.execute("DELETE FROM entity_vec")
    db.execute("DELETE FROM semantic_chunks")
    n_entities = 0
    n_events = 0
    n_errors = 0
    context_dir = root / "context"
    if not context_dir.is_dir():
        if own:
            db.commit()
            db.close()
        return IndexStats(0, 0, 0)
    for path in sorted(context_dir.rglob("*.md")):
        try:
            ent = entity_mod.load(path)
        except entity_mod.NotAnEntityError:
            # Not a processkit entity (no frontmatter, or no apiVersion key).
            # Silently skip — README, INDEX, SKILL.md, SERVER.md, etc.
            continue
        except entity_mod.EntityError as e:
            # Claims to be a processkit entity (has apiVersion) but is malformed.
            db.execute(
                "INSERT OR REPLACE INTO errors (path, message) VALUES (?, ?)",
                (str(path), str(e)),
            )
            n_errors += 1
            continue
        _insert_entity(db, ent)
        n_entities += 1
        if ent.kind == "LogEntry":
            _insert_event(db, ent)
            n_events += 1
    for manifest_path in sorted((context_dir / "archives").rglob("*.json")):
        try:
            archived_entities, archived_events = _insert_archive_manifest(
                db,
                manifest_path,
            )
            n_entities += archived_entities
            n_events += archived_events
        except Exception as e:
            db.execute(
                "INSERT OR REPLACE INTO errors (path, message) VALUES (?, ?)",
                (str(manifest_path), str(e)),
            )
            n_errors += 1
    db.commit()
    if own:
        db.close()
    return IndexStats(n_entities, n_events, n_errors)


def _insert_entity(
    db: sqlite3.Connection,
    ent,
    *,
    storage_location: str | None = None,
) -> None:
    spec = ent.spec or {}
    title = spec.get("title") or spec.get("name")
    spec_json = json.dumps(spec, default=str)
    db.execute(
        """
        INSERT OR REPLACE INTO entities
        (
            id, kind, api_version, path, storage_location, created, updated,
            title, state, labels_json, spec_json, body
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            ent.id,
            ent.kind,
            ent.apiVersion,
            str(ent.path) if ent.path else "",
            storage_location,
            str(ent.created),
            str(ent.updated) if ent.updated else None,
            title,
            spec.get("state"),
            json.dumps(ent.labels) if ent.labels else None,
            spec_json,
            ent.body or None,
        ),
    )
    _upsert_fts(db, ent, title=title, spec_json=spec_json)
    _upsert_semantic(
        db,
        ent,
        title=title,
        spec_json=spec_json,
        storage_location=storage_location,
    )


def _insert_archive_manifest(
    db: sqlite3.Connection,
    manifest_path: Path,
) -> tuple[int, int]:
    data = json.loads(manifest_path.read_text(encoding="utf-8"))
    archive_path = data.get("archive_path")
    archive_id = data.get("archive_id") or manifest_path.stem
    entity_count = 0
    event_count = 0
    for item in data.get("entities") or []:
        metadata = dict(item.get("metadata") or {})
        spec = dict(item.get("spec") or {})
        original_path = item.get("original_path") or item.get("path") or ""
        member_path = item.get("member_path") or original_path
        location = item.get("storage_location")
        if not location:
            location = f"{archive_path}::{member_path}" if archive_path else archive_id
        ent = entity_mod.Entity(
            apiVersion=item.get("apiVersion") or item.get("api_version") or "",
            kind=item.get("kind") or "",
            metadata=metadata,
            spec=spec,
            body=item.get("body_index") or "",
            path=Path(original_path) if original_path else None,
        )
        _insert_entity(db, ent, storage_location=location)
        entity_count += 1
        if ent.kind == "LogEntry":
            _insert_event(db, ent)
            event_count += 1
    return entity_count, event_count


def _has_fts(db: sqlite3.Connection) -> bool:
    row = db.execute(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'entities_fts'"
    ).fetchone()
    return row is not None


def _upsert_fts(
    db: sqlite3.Connection,
    ent,
    *,
    title: str | None,
    spec_json: str,
) -> None:
    """Mirror an entity row into the optional FTS5 search table."""
    if not _has_fts(db):
        return
    db.execute("DELETE FROM entities_fts WHERE id = ?", (ent.id,))
    db.execute(
        """
        INSERT INTO entities_fts (id, kind, state, title, body, spec_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            ent.id,
            ent.kind,
            (ent.spec or {}).get("state"),
            title or "",
            ent.body or "",
            spec_json,
        ),
    )


def _load_sqlite_vec(db: sqlite3.Connection) -> bool:
    """Load sqlite-vec if the optional package is installed."""
    try:
        import sqlite_vec  # type: ignore
    except Exception:
        return False
    try:
        db.enable_load_extension(True)
        sqlite_vec.load(db)
        db.enable_load_extension(False)
        return True
    except Exception:
        try:
            db.enable_load_extension(False)
        except Exception:
            pass
        return False


def _has_vec(db: sqlite3.Connection) -> bool:
    row = db.execute(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'entity_vec'"
    ).fetchone()
    if row is None:
        return False
    try:
        db.execute("SELECT rowid FROM entity_vec LIMIT 0").fetchall()
        return True
    except sqlite3.OperationalError:
        return False


def _tokenize(text: str) -> list[str]:
    return re.findall(r"[a-zA-Z0-9_]+", text.lower())


def _embed_text(text: str) -> list[float]:
    """Deterministic local embedding used for provider-neutral indexing.

    This is intentionally simple: hashed bag-of-words into a fixed-size
    normalized vector. Projects can later replace this with provider or
    local model embeddings without changing the sqlite-vec storage shape.
    """
    vec = [0.0] * VECTOR_DIMENSIONS
    for tok in _tokenize(text):
        digest = blake2b(tok.encode("utf-8"), digest_size=8).digest()
        bucket = int.from_bytes(digest[:4], "little") % VECTOR_DIMENSIONS
        sign = 1.0 if digest[4] & 1 else -1.0
        vec[bucket] += sign
    norm = math.sqrt(sum(v * v for v in vec))
    if norm:
        vec = [v / norm for v in vec]
    return vec


def _serialize_vector(vec: list[float]) -> bytes:
    try:
        import sqlite_vec  # type: ignore

        return sqlite_vec.serialize_float32(vec)
    except Exception:
        return struct.pack(f"{len(vec)}f", *vec)


def _semantic_chunks(ent, *, title: str | None, spec_json: str) -> list[str]:
    text_parts = [title or "", ent.body or "", spec_json]
    text = "\n\n".join(p for p in text_parts if p)
    paragraphs = [p.strip() for p in re.split(r"\n\s*\n", text) if p.strip()]
    chunks: list[str] = []
    current = ""
    for para in paragraphs:
        if len(current) + len(para) + 2 <= 1200:
            current = f"{current}\n\n{para}".strip()
        else:
            if current:
                chunks.append(current)
            current = para[:1200]
    if current:
        chunks.append(current)
    return chunks or ([text[:1200]] if text else [])


def _upsert_semantic(
    db: sqlite3.Connection,
    ent,
    *,
    title: str | None,
    spec_json: str,
    storage_location: str | None = None,
) -> None:
    rows = db.execute(
        "SELECT rowid FROM semantic_chunks WHERE entity_id = ?",
        (ent.id,),
    ).fetchall()
    if _has_vec(db):
        for row in rows:
            db.execute("DELETE FROM entity_vec WHERE rowid = ?", (row["rowid"],))
    db.execute("DELETE FROM semantic_chunks WHERE entity_id = ?", (ent.id,))

    chunks = _semantic_chunks(ent, title=title, spec_json=spec_json)
    for ordinal, chunk in enumerate(chunks):
        chunk_id = f"{ent.id}#{ordinal}"
        cur = db.execute(
            """
            INSERT INTO semantic_chunks
            (chunk_id, entity_id, kind, state, path, ordinal, text)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            """,
            (
                chunk_id,
                ent.id,
                ent.kind,
                (ent.spec or {}).get("state"),
                storage_location or (str(ent.path) if ent.path else ""),
                ordinal,
                chunk,
            ),
        )
        if _has_vec(db):
            db.execute(
                "INSERT INTO entity_vec (rowid, embedding) VALUES (?, ?)",
                (cur.lastrowid, _serialize_vector(_embed_text(chunk))),
            )


def _insert_event(db: sqlite3.Connection, ent) -> None:
    spec = ent.spec or {}
    db.execute(
        """
        INSERT OR REPLACE INTO events
        (
            id, timestamp, event_type, actor, subject, subject_kind, summary,
            details_json, correlation_id, path
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            ent.id,
            str(spec.get("timestamp", ent.created)),
            spec.get("event_type", ""),
            spec.get("actor"),
            spec.get("subject"),
            spec.get("subject_kind"),
            spec.get("summary"),
            json.dumps(spec.get("details", {}), default=str),
            spec.get("correlation_id"),
            str(ent.path) if ent.path else "",
        ),
    )


def upsert_entity(db: sqlite3.Connection, ent) -> None:
    """Insert or update a single entity (used after a write)."""
    _insert_entity(db, ent)
    if ent.kind == "LogEntry":
        _insert_event(db, ent)
    db.commit()


def existing_ids(db: sqlite3.Connection, kind: str) -> set[str]:
    """Return all entity IDs of the given kind that the index currently knows.

    Used by id-management for collision avoidance during ID generation.
    Assumes the index is fresh — every MCP server upserts the index in
    place after each write, so this is true in normal operation. If
    files have been edited outside the MCP path, call ``reindex()``
    first.
    """
    rows = db.execute(
        "SELECT id FROM entities WHERE kind = ?",
        (kind,),
    ).fetchall()
    return {r["id"] for r in rows}


def query_entities(
    db: sqlite3.Connection,
    *,
    kind: str | None = None,
    state: str | None = None,
    limit: int = 50,
) -> list[dict[str, Any]]:
    sql = (
        "SELECT id, kind, title, state, created, updated, path,"
        " storage_location FROM entities WHERE 1=1"
    )
    params: list[Any] = []
    if kind:
        sql += " AND kind = ?"
        params.append(kind)
    if state:
        sql += " AND state = ?"
        params.append(state)
    sql += " ORDER BY created DESC LIMIT ?"
    params.append(int(limit))
    return [dict(row) for row in db.execute(sql, params).fetchall()]


def get_entity(db: sqlite3.Connection, id: str) -> dict[str, Any] | None:
    row = db.execute(
        (
            "SELECT id, kind, title, state, created, updated, path,"
            " storage_location, spec_json, labels_json FROM entities WHERE id = ?"
        ),
        (id,),
    ).fetchone()
    if not row:
        return None
    out = dict(row)
    if out.get("spec_json"):
        try:
            out["spec"] = json.loads(out["spec_json"])
        except Exception:
            pass
    if out.get("labels_json"):
        try:
            out["labels"] = json.loads(out["labels_json"])
        except Exception:
            pass
    return out


def resolve_entity(
    db: sqlite3.Connection,
    partial_id: str,
    *,
    kind: str | None = None,
) -> tuple[dict[str, Any] | None, list[str]]:
    """Resolve a partial or word-pair ID to a single entity.

    Agents and humans often refer to entities by a shortened form — either
    omitting the slug suffix or using just the word-pair component. This
    function tries three resolution strategies in order:

    1. **Exact match** — ``id = partial_id``
    2. **Prefix match** — ``id LIKE '{partial_id}-%'`` (handles a missing slug)
    3. **Word-pair match** — ``id LIKE '%-{partial_id}-%' OR id LIKE
       '%-{partial_id}'`` (handles a bare word-pair like ``StoutCrow``)

    Returns ``(row, candidates)`` where:

    - ``(row, [])``   — exactly one match; ``row`` is the full entity dict
    - ``(None, [])``  — no match at all
    - ``(None, ids)`` — ambiguous; ``ids`` lists all matching entity IDs
    """

    def _escape_like(s: str) -> str:
        # Escape LIKE metacharacters so literal underscores in IDs (e.g.
        # "20260410_1050") are not treated as single-character wildcards.
        return s.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_")

    def _fetch(where: str, params: list[Any]) -> list[dict[str, Any]]:
        sql = (
            "SELECT id, kind, title, state, created, updated, path,"
            " storage_location, spec_json, labels_json FROM entities WHERE " + where
        )
        if kind:
            sql += " AND kind = ?"
            params = list(params) + [kind]
        rows = db.execute(sql, params).fetchall()
        result = []
        for row in rows:
            out = dict(row)
            if out.get("spec_json"):
                try:
                    out["spec"] = json.loads(out["spec_json"])
                except Exception:
                    pass
            result.append(out)
        return result

    # 1. Exact match.
    rows = _fetch("id = ?", [partial_id])
    if rows:
        return rows[0], []

    # 2. Prefix match — handles a missing slug suffix, e.g.
    #    "BACK-20260410_1050-StoutCrow" → "BACK-20260410_1050-StoutCrow-create-..."
    escaped = _escape_like(partial_id)
    rows = _fetch("id LIKE ? ESCAPE '\\'", [escaped + "-%"])
    if len(rows) == 1:
        return rows[0], []
    if len(rows) > 1:
        return None, [r["id"] for r in rows]

    # 3. Word-pair / substring match — handles a bare word-pair, e.g.
    #    "StoutCrow" → "BACK-20260410_1050-StoutCrow-create-brand-design-skill"
    rows = _fetch(
        "(id LIKE ? ESCAPE '\\' OR id LIKE ? ESCAPE '\\')",
        ["%-" + escaped + "-%", "%-" + escaped],
    )
    if len(rows) == 1:
        return rows[0], []
    if len(rows) > 1:
        return None, [r["id"] for r in rows]

    return None, []


def search_entities(
    db: sqlite3.Connection,
    text: str,
    *,
    limit: int = 50,
) -> list[dict[str, Any]]:
    if _has_fts(db):
        try:
            rows = db.execute(
                """
                SELECT
                    e.id, e.kind, e.title, e.state, e.created, e.path,
                    e.storage_location
                FROM entities_fts f
                JOIN entities e ON e.id = f.id
                WHERE entities_fts MATCH ?
                ORDER BY rank
                LIMIT ?
                """,
                (text, int(limit)),
            ).fetchall()
            return [dict(r) for r in rows]
        except sqlite3.OperationalError:
            # Preserve the previous forgiving substring-search behaviour
            # for punctuation-heavy or otherwise invalid FTS5 queries.
            pass

    like = f"%{text}%"
    rows = db.execute(
        """
        SELECT id, kind, title, state, created, path, storage_location
        FROM entities
        WHERE id LIKE ? OR title LIKE ? OR body LIKE ? OR spec_json LIKE ?
        ORDER BY created DESC LIMIT ?
        """,
        (like, like, like, like, int(limit)),
    ).fetchall()
    return [dict(r) for r in rows]


def semantic_status(db: sqlite3.Connection) -> dict[str, Any]:
    """Return semantic-index capability and row counts."""
    chunks = db.execute("SELECT COUNT(*) FROM semantic_chunks").fetchone()[0]
    return {
        "chunks": chunks,
        "sqlite_vec_available": _has_vec(db),
        "dimensions": VECTOR_DIMENSIONS,
        "embedding": "local-hashed-bag-of-words",
    }


def semantic_search_entities(
    db: sqlite3.Connection,
    text: str,
    *,
    limit: int = 50,
) -> list[dict[str, Any]]:
    """Search semantic chunks via sqlite-vec when available."""
    if not _has_vec(db):
        return []
    rows = db.execute(
        """
        SELECT
            e.id, e.kind, e.title, e.state, e.created, e.path,
            e.storage_location,
            MIN(v.distance) AS distance
        FROM entity_vec v
        JOIN semantic_chunks c ON c.rowid = v.rowid
        JOIN entities e ON e.id = c.entity_id
        WHERE v.embedding MATCH ? AND k = ?
        GROUP BY e.id
        ORDER BY distance ASC
        LIMIT ?
        """,
        (
            _serialize_vector(_embed_text(text)),
            max(int(limit) * 4, int(limit)),
            int(limit),
        ),
    ).fetchall()
    return [dict(r) for r in rows]


def hybrid_search_entities(
    db: sqlite3.Connection,
    text: str,
    *,
    limit: int = 50,
) -> list[dict[str, Any]]:
    """Combine FTS5 and sqlite-vec results with reciprocal-rank fusion."""
    keyword = search_entities(db, text, limit=max(int(limit) * 4, int(limit)))
    semantic = semantic_search_entities(db, text, limit=max(int(limit) * 4, int(limit)))
    if not semantic:
        return keyword[: int(limit)]

    scores: dict[str, float] = {}
    rows: dict[str, dict[str, Any]] = {}
    for rank, row in enumerate(keyword, start=1):
        rows.setdefault(row["id"], dict(row))
        scores[row["id"]] = scores.get(row["id"], 0.0) + 1.0 / (60 + rank)
    for rank, row in enumerate(semantic, start=1):
        merged = rows.setdefault(row["id"], dict(row))
        if "distance" in row:
            merged["semantic_distance"] = row["distance"]
        scores[row["id"]] = scores.get(row["id"], 0.0) + 1.0 / (60 + rank)

    ranked = sorted(scores, key=lambda entity_id: scores[entity_id], reverse=True)
    out = []
    for entity_id in ranked[: int(limit)]:
        row = rows[entity_id]
        row["hybrid_score"] = scores[entity_id]
        out.append(row)
    return out


def query_events(
    db: sqlite3.Connection,
    *,
    event_type: str | None = None,
    subject: str | None = None,
    actor: str | None = None,
    limit: int = 50,
) -> list[dict[str, Any]]:
    sql = "SELECT id, timestamp, event_type, actor, subject, summary FROM events WHERE 1=1"
    params: list[Any] = []
    if event_type:
        sql += " AND event_type = ?"
        params.append(event_type)
    if subject:
        sql += " AND subject = ?"
        params.append(subject)
    if actor:
        sql += " AND actor = ?"
        params.append(actor)
    sql += " ORDER BY timestamp DESC LIMIT ?"
    params.append(int(limit))
    return [dict(row) for row in db.execute(sql, params).fetchall()]


def list_errors(db: sqlite3.Connection) -> list[dict[str, Any]]:
    return [dict(r) for r in db.execute("SELECT path, message FROM errors").fetchall()]
