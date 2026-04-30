# index-management MCP server

SQLite-backed index over all entity files in the project. Provides
query/search/lookup tools used directly by agents and indirectly by other
processkit MCP servers (which call `upsert_entity` after writing files).

## Tools

| Tool                                                           | Purpose                                          |
|----------------------------------------------------------------|--------------------------------------------------|
| `reindex()`                                                    | Walk `context/`, rebuild the index from scratch  |
| `query_entities(kind?, state?, limit?)`                        | List entities matching filters                   |
| `get_entity(id)`                                               | Fetch one entity by ID                           |
| `search_entities(text, limit?)`                                | FTS5-ranked search with LIKE fallback           |
| `semantic_status()`                                            | Semantic chunk/vector capability and counts      |
| `semantic_search_entities(text, limit?)`                       | sqlite-vec semantic search when available        |
| `hybrid_search_entities(text, limit?)`                         | RRF over FTS5 + semantic results, FTS fallback   |
| `query_events(event_type?, subject?, actor?, limit?)`          | Query the LogEntry events table                  |
| `list_errors()`                                                | Files that failed to parse during last reindex   |
| `stats()`                                                      | Count of entities/events/errors in the index     |

## Database

`<project-root>/context/.cache/processkit/index.sqlite` — gitignored,
rebuildable from source files.

## Running

From a processkit checkout:

```bash
cd /path/to/your/project   # contains AGENTS.md and context/
uv run /path/to/processkit/src/skills/index-management/mcp/server.py
```

When installed by aibox into a consumer project:

```bash
uv run context/skills/index-management/mcp/server.py
```

Either form first cold-starts uv (~5–10s), then runs the MCP server on
STDIO. Subsequent runs are near-instant due to uv's environment cache.

## Limitations

- WAL mode enabled (v0.4.0+); concurrent writes still serialize.
- Search uses SQLite FTS5 when available and falls back to
  `LIKE %text%` for invalid FTS syntax or SQLite builds without FTS5.
- Semantic search uses optional sqlite-vec. When sqlite-vec is not
  installed or loadable, `semantic_search_entities` returns no vector
  results and `hybrid_search_entities` falls back to FTS5.
- No incremental indexing — `reindex()` is a full sweep (BACK-006).

## Configuration

Override the database path:

```toml
# processkit.toml
[index]
path = "context/.cache/processkit/index.sqlite"
```

or via env var:

```bash
PROCESSKIT_INDEX_DB=/tmp/test.sqlite uv run .../server.py
```

(env var support lands in v0.3.1)
