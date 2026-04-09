# RAG Layer for Context Directory — Research Report — March 2026

Research for semantic search over the aibox `./context/` directory. Evaluates embedding
models, vector storage, MCP integration, existing tools, indexing strategies, and query
interfaces. Recommends the simplest approach that works in a devcontainer without GPU.
Conducted 2026-03-26.

---

## 1. Problem Statement

aibox projects have a `./context/` directory with 25-60+ markdown files: research reports,
decisions, backlog items, work instructions, session notes, and architecture docs. As the
corpus grows, finding relevant information by filename or grep becomes unreliable. A RAG
(Retrieval-Augmented Generation) layer would enable semantic search -- finding content by
meaning rather than exact keywords.

**Current corpus stats (aibox itself):** 61 markdown files, ~792KB total, ~15,000 lines.

---

## 2. Embedding Models for CPU Use

All models below run on CPU without GPU. The key tradeoff is model size vs. quality.

### Comparison Table

| Model | Size | Dimensions | MTEB Score | Runtime | Language | Notes |
|---|---|---|---|---|---|---|
| **all-MiniLM-L6-v2** | 23MB | 384 | ~63 | ONNX / PyTorch | EN | Fastest, smallest, good baseline |
| **bge-small-en-v1.5** | 33MB | 384 | ~63 | ONNX | EN | Slightly better than MiniLM |
| **bge-base-en-v1.5** | 130MB | 768 | ~64 | ONNX | EN | Good quality/size balance |
| **nomic-embed-text-v1.5** | 137MB | 768 (truncatable to 256) | ~62 | ONNX / PyTorch | EN+multi | Matryoshka dims, open weights |
| **nomic-embed-text-v2-moe** | ~280MB | 768 | ~65 | PyTorch | Multilingual | MoE architecture, heavier |
| **Qwen3-Embedding-0.6B** | ~1.2GB | 1024 | ~67 | Candle/ONNX | Multilingual | High quality, too large for devcontainer |

### Runtime Libraries

| Library | Language | Backend | Model Download | Key Advantage |
|---|---|---|---|---|
| **fastembed (Rust)** | Rust | ONNX Runtime | Auto from HuggingFace | Native Rust, 3-5x faster than Python, quantized models |
| **fastembed (Python)** | Python | ONNX Runtime | Auto from HuggingFace | Simple API, ONNX-optimized for CPU |
| **sentence-transformers** | Python | PyTorch | Auto from HuggingFace | Largest ecosystem, easy fine-tuning |
| **Transformers.js** | JS/TS | ONNX Runtime (WASM) | Auto from HuggingFace | Node.js native, used by mcp-local-rag |

### Recommendation

**fastembed (Rust crate)** with `bge-small-en-v1.5` (quantized). Rationale:
- aibox CLI is already Rust -- no new runtime dependency
- 33MB model, ~384-dim vectors, fast on CPU
- ONNX Runtime handles SIMD acceleration automatically
- Quantized variant (`BGESmallENV15Q`) reduces size and latency further

---

## 3. Vector Storage

The corpus is small (~61 files, ~500-1000 chunks). Heavy databases are overkill.

### Comparison Table

| Database | Language | Architecture | RAM (idle) | RAM (search) | Disk | Server Process | Rust Support |
|---|---|---|---|---|---|---|---|
| **sqlite-vec** | C (SQLite ext) | Embedded, file-based | ~5MB | ~10MB | Single .db file | No | Via rusqlite |
| **LanceDB** | Rust core | Embedded, file-based | ~4MB | ~150MB | Lance format dir | No | Native crate |
| **ChromaDB** | Rust (v2) | Client-server or embedded | ~200MB | ~400MB | Directory | Optional | Python/JS SDK |
| **Qdrant** | Rust | Client-server | ~400MB | ~500MB | Directory | Yes (separate) | Native crate |
| **FAISS** | C++ | Library (in-memory) | Varies | Varies | None (manual) | No | C bindings |

### Analysis

**sqlite-vec** is the clear winner for this use case:
- Zero-dependency C extension, works with any SQLite binding
- Single `.db` file -- trivial to backup, sync, gitignore
- Handles tens of thousands of vectors efficiently
- KNN search with multiple distance metrics (cosine, L2, dot)
- Accessible via `rusqlite` in Rust with `bundled` feature
- ~5MB overhead -- invisible in a devcontainer

**LanceDB** is a strong alternative if the corpus grows to thousands of files or needs
multimodal search. Its Rust-native crate is well-maintained, but the Lance format creates
a directory of files rather than a single file.

### Recommendation

**sqlite-vec** via rusqlite. Single file, minimal deps, fits the aibox "simple tools" philosophy.

---

## 4. Existing MCP Tools and Prior Art

### MCP Servers for RAG

| Tool | Embedding | Storage | Local? | Notes |
|---|---|---|---|---|
| **[mcp-local-rag](https://github.com/shinpr/mcp-local-rag)** | all-MiniLM-L6-v2 (Transformers.js) | LanceDB | Yes | Node.js, ~200MB idle, semantic chunking |
| **[MCP-Markdown-RAG](https://github.com/Zackriya-Solutions/MCP-Markdown-RAG)** | ~50MB model (unspecified) | Milvus Lite | Yes | Python/uv, heading-based chunking, incremental indexing |
| **[claude-context](https://github.com/zilliztech/claude-context)** | OpenAI text-embedding-3-small | Zilliz Cloud (Milvus) | No | Requires API keys, hybrid BM25+vector search |
| **[qdrant-rag-mcp](https://github.com/ancoleman/qdrant-rag-mcp)** | Various | Qdrant | Yes (server) | Requires running Qdrant instance |
| **[rag-code-mcp](https://github.com/doITmagic/rag-code-mcp)** | Ollama local LLMs | Qdrant | Yes (server) | Multi-language code focus, heavy |

### Other Tools

| Tool | Type | Notes |
|---|---|---|
| **Greptile** | SaaS | Full codebase RAG, dependency graphs, cloud-hosted or air-gapped |
| **Sourcegraph Cody** | SaaS/self-hosted | Code intelligence with embeddings, enterprise-focused |
| **code-graph-rag** | Local | AST + knowledge graph + vector search, multi-language |

### Key Insight

MCP-Markdown-RAG is closest to what we need (markdown-focused, heading-based chunking,
incremental indexing), but it uses Python and Milvus. mcp-local-rag is more mature but
uses Node.js and LanceDB. Neither fits naturally into a Rust CLI.

**Building a thin Rust-native implementation is preferable** -- it avoids adding a Node.js
or Python runtime dependency to aibox and keeps the tool self-contained.

---

## 5. Architecture Design

### Chunking Strategy

Markdown files have natural structure. Chunking by heading sections preserves semantic
coherence better than fixed-size splits.

```
# Document Title           --> chunk 1 (title + intro paragraphs)
## Section A               --> chunk 2 (heading + body)
### Subsection A.1         --> chunk 3 (heading + body)
## Section B               --> chunk 4 (heading + body)
```

Each chunk stores:
- `file_path`: relative path from context root
- `heading_path`: e.g., "Document Title > Section A > Subsection A.1"
- `content`: raw text of the section
- `embedding`: vector from fastembed
- `file_mtime`: for incremental updates
- `chunk_hash`: content hash for change detection

### Database Schema (sqlite-vec)

```sql
CREATE TABLE chunks (
    id INTEGER PRIMARY KEY,
    file_path TEXT NOT NULL,
    heading_path TEXT NOT NULL,
    content TEXT NOT NULL,
    chunk_hash TEXT NOT NULL,
    file_mtime INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE VIRTUAL TABLE chunk_vectors USING vec0(
    chunk_id INTEGER PRIMARY KEY,
    embedding FLOAT[384]
);

CREATE INDEX idx_chunks_file ON chunks(file_path);
CREATE INDEX idx_chunks_hash ON chunks(chunk_hash);
```

### Search Flow

```
Query text
  --> fastembed encode --> query vector (384 dims)
  --> sqlite-vec KNN search (cosine distance, top-k=10)
  --> return ranked chunks with file_path, heading_path, content snippet
```

---

## 6. Indexing Strategy

### When to Index

| Trigger | Mechanism | Notes |
|---|---|---|
| **`aibox search --reindex`** | On-demand full reindex | Explicit user action, ~2-5 seconds for 61 files |
| **`aibox search <query>`** | Incremental before search | Check file mtimes, re-embed only changed files |
| **`aibox sync`** | Post-sync hook | Reindex after container sync completes |
| **File watcher** | inotify/fswatch | Overkill for this corpus size, skip for v1 |

### Incremental Indexing Algorithm

1. Scan `./context/` for all `.md` files
2. For each file, compare `mtime` against stored `file_mtime`
3. If changed: re-chunk, re-hash, re-embed changed chunks, delete removed chunks
4. If unchanged: skip
5. Delete chunks for files that no longer exist

**Expected performance:** ~50ms per file (chunk + embed on CPU), full reindex of 61 files
takes ~3 seconds. Incremental update of 1-2 changed files takes <200ms.

### Index Location

Store at `./context/.aibox/search.db` (gitignored). The `.aibox/` directory can also
hold other derived data in the future.

---

## 7. Query Interface

### Option A: CLI Subcommand (Recommended for v1)

```bash
aibox search "how do we handle addon dependencies"
aibox search "what decisions were made about versioning"
aibox search --reindex    # force full reindex
aibox search --top 5 "brand guidelines"
```

Output format:
```
 1. context/research/addon-dependency-design-2026-03.md
    > Dependency Resolution > Algorithm
    Score: 0.87 | "Dependencies between addons are resolved using..."

 2. context/DECISIONS.md
    > DEC-042: Addon dependency ordering
    Score: 0.82 | "Decided to use topological sort for..."
```

### Option B: MCP Server (Recommended for v2)

Expose semantic search as an MCP tool so Claude Code can query it automatically.

```json
{
  "mcpServers": {
    "aibox-search": {
      "command": "aibox",
      "args": ["mcp-serve", "--tools", "search"]
    }
  }
}
```

MCP tool definition:
```json
{
  "name": "context_search",
  "description": "Semantic search over project context documents",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": { "type": "string", "description": "Natural language search query" },
      "top_k": { "type": "integer", "default": 5 }
    },
    "required": ["query"]
  }
}
```

This enables Claude Code to automatically search context when working on tasks, without
the user needing to manually find and paste relevant files.

### Option C: Agent Skill

Teach agents to use `aibox search` by adding instructions to CLAUDE.md:
```
When you need context about this project, run: aibox search "your query"
```

This is the lowest-effort integration and works with any AI coding tool.

---

## 8. Scope

### What to Index

| Scope | Files | Size | Priority |
|---|---|---|---|
| `./context/` (non-archive) | ~30 files | ~400KB | v1 -- primary use case |
| `./context/archive/` | ~30 files | ~400KB | v1 -- include but lower rank |
| `./skills/` descriptions | ~20 files | ~100KB | v2 -- useful for "what skill does X" |
| `./docs-site/docs/` | ~15 files | ~200KB | v2 -- user-facing docs |
| Source code (Rust) | ~30 files | ~300KB | v3 -- needs code-aware chunking |

### Ranking Boost

Apply a recency/importance boost:
- Active files (`BACKLOG.md`, `DECISIONS.md`, `PROJECTS.md`) get 1.2x boost
- Archive files get 0.8x weight
- Research reports and work instructions get 1.0x (default)

---

## 9. Recommended Implementation Plan

### Phase 1: CLI Search (v0.15)

**Effort:** ~3-5 days

1. Add `fastembed` and `rusqlite` (with sqlite-vec) to `cli/Cargo.toml`
2. Implement markdown heading-based chunker
3. Implement incremental indexing with mtime + hash check
4. Implement `aibox search <query>` subcommand
5. Store index at `./context/.aibox/search.db`
6. Add `.aibox/` to default `.gitignore` template

**Dependencies to add:**
```toml
fastembed = "5"          # ~ONNX embedding, ~33MB model download on first run
rusqlite = { version = "0.32", features = ["bundled"] }
# sqlite-vec loaded as extension at runtime
```

### Phase 2: MCP Integration (v0.16)

**Effort:** ~2-3 days (after Phase 1)

1. Add `aibox mcp-serve` subcommand using MCP protocol over stdio
2. Expose `context_search` tool
3. Auto-configure in `.devcontainer/devcontainer.json`
4. Claude Code / Cursor / other MCP clients can query context automatically

### Phase 3: Extended Scope (v0.17+)

- Index skills metadata and docs
- Hybrid search (BM25 keyword + vector)
- Cross-project search (multiple context directories)
- Web UI for browsing indexed content

---

## 10. Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| ONNX model download on first run (~33MB) | Low | Cache in `~/.cache/fastembed/`, warn user on first run |
| fastembed Rust crate maturity | Medium | Well-maintained (Qdrant team), active releases, fallback to Python fastembed |
| sqlite-vec availability | Low | Distribute as bundled extension or use LanceDB as fallback |
| Embedding quality for technical markdown | Low | bge-small-en works well on technical English; upgrade model later if needed |
| Build time increase from ONNX | Medium | Feature-gate behind `search` cargo feature flag |
| Devcontainer image size increase | Low | ~40MB for model + ONNX runtime, negligible vs base image |

---

## 11. Decision

**Recommended approach:** Rust-native implementation using `fastembed` (bge-small-en-v1.5)
+ `sqlite-vec` via rusqlite. Markdown heading-based chunking with incremental indexing.
CLI subcommand for v1, MCP server for v2.

This is the simplest approach that:
- Requires no additional runtime (no Python, no Node.js, no Docker services)
- Works offline after first model download
- Runs on CPU in any devcontainer
- Fits naturally into the existing `aibox` CLI
- Produces a single index file that is easy to manage
- Can be extended to MCP for automatic agent integration

---

## Sources

- [fastembed Rust crate](https://crates.io/crates/fastembed) -- ONNX-based embedding in Rust
- [fastembed-rs GitHub](https://github.com/Anush008/fastembed-rs) -- Rust library for vector embeddings
- [sqlite-vec GitHub](https://github.com/asg017/sqlite-vec) -- Vector search SQLite extension
- [LanceDB Rust crate](https://crates.io/crates/lancedb) -- Embedded vector database
- [mcp-local-rag](https://github.com/shinpr/mcp-local-rag) -- Local RAG MCP server (Node.js + LanceDB)
- [MCP-Markdown-RAG](https://github.com/Zackriya-Solutions/MCP-Markdown-RAG) -- Markdown semantic search MCP server
- [claude-context](https://github.com/zilliztech/claude-context) -- Code search MCP for Claude Code
- [Best Open-Source Embedding Models Benchmarked](https://supermemory.ai/blog/best-open-source-embedding-models-benchmarked-and-ranked/)
- [Best Vector Databases 2026](https://encore.dev/articles/best-vector-databases)
- [sqlite-vec Stable Release](https://alexgarcia.xyz/blog/2024/sqlite-vec-stable-release/index.html)
