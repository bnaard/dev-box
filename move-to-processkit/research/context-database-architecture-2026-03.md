# Context Database Architecture Research

**Status:** Research complete
**Date:** 2026-03-26
**Relates to:** BACK-055 (process templates), DEC-016 (composable process packages), process-ontology-primitives-2026-03.md, rag-context-layer-2026-03.md, event-log-design-2026-03.md
**Author:** Researcher agent

---

## 1. Problem Statement

aibox needs a storage layer for process primitives -- work items, events, decisions, projects, metrics, cross-references, and user-defined entities. This layer must satisfy five constraints simultaneously:

1. **Single source of truth** -- one authoritative format, no dual-master synchronization
2. **Git-native** -- data lives in a git repo with meaningful diffs, merges, and code review
3. **Flexible schema** -- user-defined fields per artifact type (like Jira custom fields)
4. **Efficient search** -- structured queries and RAG (vector embeddings)
5. **Human-readable** -- developers can read/understand data without special tooling

This document evaluates storage architectures against these constraints and recommends an approach.

---

## 2. Git and Binary Files -- The Real Story

### 2.1 How Git Stores Objects

Git is fundamentally a content-addressable filesystem. Every file version is stored as a **blob** object, identified by its SHA-1 hash. When first written, blobs are stored as **loose objects** -- individually zlib-compressed files in `.git/objects/`.

As the repository grows, `git gc` (or `git repack`) consolidates loose objects into **packfiles** (`.git/objects/pack/*.pack`). Packfiles use **delta compression**: git finds similar objects, stores one as a base, and the rest as binary deltas. The packing heuristic sorts objects by type, then by filename, then by size -- so successive versions of the same file tend to be packed as deltas against each other.

### 2.2 Binary Files: Where Delta Compression Fails

Delta compression works on raw bytes, not semantic structure. For text files (source code, JSON, markdown), consecutive versions share most bytes, so deltas are tiny. For binary files, the story is different:

- **Structured binary formats** (SQLite, Protocol Buffers, MessagePack): even a single-field change can shift byte offsets throughout the file, causing the entire file to appear different. SQLite in particular reorders pages, updates header counters, and may rearrange B-tree nodes on any write.
- **Compressed formats** (JPEG, PNG, ZIP): compression destroys byte-level similarity. A one-pixel change in an image produces a completely different compressed output.
- **Append-only formats** (log files, JSONL): these compress well because only new bytes are added at the end -- git's delta captures this efficiently.

**Key finding:** Git does NOT do content-aware chunking or deduplication for binary files. It applies the same byte-level delta algorithm to everything. The algorithm works well only when consecutive versions share long runs of identical bytes.

### 2.3 SQLite in Git: Measured Reality

A SQLite database storing 100 work items with 20 fields each is approximately 200-400KB. The problem is not the absolute size but the **diff amplification**: changing one field value in one row can alter the SQLite file by 4KB-16KB (one or more B-tree pages plus the header), none of which is human-readable in `git diff`.

Concrete issues with SQLite in git:

- **No meaningful diffs**: `git diff` shows "Binary files differ." Custom textconv drivers (using `sqlite3 .dump`) produce SQL text diffs, but these are fragile and not mergeable.
- **No meaningful merges**: two branches modifying different rows still produce a binary merge conflict that cannot be resolved automatically.
- **WAL/journal files**: SQLite in WAL mode creates `-wal` and `-shm` files that must be either excluded from git or carefully managed.
- **Non-deterministic output**: the same logical database content can produce different binary files depending on insertion order, vacuum state, and page reuse.
- **Repository bloat over time**: each commit stores a full copy (or a large delta) of the entire database. 100 commits to a 300KB database can mean 5-15MB of pack data, depending on delta efficiency.

**Projects that have tried this:**

- `git-sqlite` (cannadayr/git-sqlite) -- custom diff/merge driver that dumps SQL; works for diffs but merges remain manual.
- `sqldiff` (sqlite.org) -- computes SQL-level diffs between two databases; could be used as a textconv driver.
- The general community consensus (Hacker News, various blog posts): "don't store SQLite in git unless you have no alternative."

### 2.4 Git-LFS: Would It Help for SQLite?

Git LFS replaces large files with **pointer files** (tiny text files containing an OID and size) while storing the actual content on a separate server. This solves repository bloat but makes the situation worse for our use case:

- The data is no longer in the repository -- it requires a separate LFS server.
- Diffs show pointer file changes, not content changes.
- Offline workflows break without LFS server access.
- Every collaborator must install Git LFS.
- You cannot `git blame` LFS-tracked content.

**Verdict:** LFS is designed for large media assets, not for source-of-truth data. It is the wrong tool for a context database.

### 2.5 Size Estimates for a Typical Project

| Content | Items | Size per item | Total |
|---------|-------|---------------|-------|
| Work items (JSON) | 100 | ~2KB | ~200KB |
| Events (JSON) | 500 | ~0.5KB | ~250KB |
| Decisions (JSON or MD) | 50 | ~3KB | ~150KB |
| Log entries (JSONL) | 1,000 | ~0.3KB | ~300KB |
| RAG embeddings (384 dims x 4 bytes) | 1,000 chunks | 1.5KB | ~1.5MB |
| **Total (without embeddings)** | | | **~900KB** |
| **Total (with embeddings)** | | | **~2.4MB** |

This is small. Even with generous growth (10x), the entire context database is under 25MB of text/JSON files -- well within git's comfort zone. The embeddings are the largest component but still modest. A SQLite database for the same data would be 1-3MB (more compact) but with all the binary-in-git problems described above.

---

## 3. Jira's Data Model -- Lessons Learned

### 3.1 Core Architecture

Jira's data model centers on the **issue** as the fundamental entity. The core tables are:

- **jiraissue** -- the central table: id, key (e.g., PROJ-123), summary, description, issue type, status, priority, reporter, assignee, created, updated, workflow_id
- **issuelink** -- relationships between issues (blocks, is blocked by, relates to, duplicates)
- **issuetype** -- defines issue types (Bug, Story, Task, Epic, Sub-task, plus custom types)
- **priority** / **resolution** / **status** -- lookup tables for the respective fields
- **project** -- projects with key, lead, and configuration references
- **workflowscheme** -- maps workflows to issue types per project

### 3.2 Custom Fields: The EAV Pattern

Jira uses an **Entity-Attribute-Value (EAV)** model for custom fields, stored across three tables:

- **customfield** -- field definitions (id, name, type, description, default value)
- **customfieldvalue** -- the actual values (issue_id, customfield_id, value as string/number/date)
- **customfieldoption** -- option values for select/multi-select fields

This means a custom field "Estimated Hours" on issue PROJ-123 is not a column on the issue table. It is a row in `customfieldvalue` with `issue_id=123, customfield_id=10042, numbervalue=4.0`.

**Why this matters for aibox:** The EAV pattern allows unlimited custom fields without schema migration. But it has known costs:

- **Query complexity**: finding all issues where `estimated_hours > 3` requires joining through the EAV tables rather than a simple `WHERE` clause.
- **Performance degradation at scale**: Atlassian's own testing shows that response times for Create Issue, Edit Issue, and JQL Search degrade measurably as custom field count grows from 1,400 to 2,800. The degradation is tied to the number of fields in an issue's **context** (Project + Issue Type combination).
- **Sparse matrix problem**: most issues have values for only a fraction of available custom fields, creating a sparse, hard-to-index dataset.

### 3.3 Workflow Engine

Jira workflows are state machines with:

- **Statuses** -- named states (To Do, In Progress, Done, plus custom statuses)
- **Transitions** -- edges between statuses, with optional conditions, validators, and post-functions
- **Workflow schemes** -- map issue types to workflows per project (different types can have different workflows)
- **Screens** -- define which fields are shown/required during each transition

This is powerful but creates a configuration explosion. A large Jira instance can have hundreds of workflows, dozens of schemes, and thousands of field configurations -- leading to the "Jira admin" role becoming a full-time job.

### 3.4 What Makes Jira Powerful

- **Flexible schemas**: any field, any type, any project can be customized independently.
- **Cross-referencing**: issues link to other issues, epics contain stories, components group issues, versions track releases.
- **JQL**: a structured query language that works across all fields including custom fields.
- **Per-project configuration**: workflow schemes, field configurations, and screens can differ per project.
- **Extensibility**: a rich API and marketplace ecosystem.

### 3.5 What Makes Jira Painful

- **Configuration complexity**: schemes-within-schemes-within-schemes create a management burden.
- **Performance at scale**: EAV custom fields, deep permission schemes, and complex JQL queries slow down as instances grow.
- **Vendor lock-in**: data lives in Atlassian's database, export is lossy and painful.
- **Migration difficulty**: moving between Jira instances or away from Jira requires mapping custom fields, workflows, and schemes.
- **UI performance**: large boards, backlogs, and search results are notoriously slow.

### 3.6 How Linear Differs

Linear takes the opposite approach: **opinionated simplicity**.

| Aspect | Jira | Linear |
|--------|------|--------|
| Custom fields | Unlimited, any type | Not supported (fixed schema) |
| Workflow customization | Arbitrary state machines | Preset statuses (Backlog, Todo, In Progress, Done, Cancelled) |
| Issue types | Unlimited custom types | Fixed: Issue, Project, Cycle |
| Configuration model | Per-project schemes | Global, minimal |
| Performance | Degrades at scale | Consistently fast |
| Target audience | Any team, any methodology | Engineering teams, opinionated workflow |

**Lesson for aibox:** Jira proves that flexible schemas are essential for real-world use. Linear proves that you can be opinionated about workflow while keeping things fast and simple. aibox should aim for Jira's schema flexibility with Linear's simplicity -- which means the complexity must live in the data format, not in configuration layers.

---

## 4. Storage Architecture Options

### 4.1 Option A: SQLite (Binary Database)

Store all process primitives in a single SQLite database (`context/.aibox/context.db`).

**Schema approach:** Core columns for universal fields (id, type, status, title, created_at) plus a `custom_fields JSON` column for flexible attributes. SQLite's `json_extract()` and `json_each()` functions allow querying into JSON columns.

**Advantages:**
- Excellent query performance (indexes, JOINs, aggregations)
- Full SQL expressiveness including JSON functions
- Single file, easy to back up
- Mature Rust support via `rusqlite` with `bundled` feature
- sqlite-vec extension for vector search (already chosen for RAG layer)
- Transaction safety (ACID)

**Disadvantages:**
- Binary file in git -- no meaningful diffs, merges, or blame
- Cannot be code-reviewed in a pull request
- Merge conflicts are unresolvable without custom tooling
- Non-deterministic binary output
- Violates "git-native" and "human-readable" constraints

**Verdict:** Excellent as a derived runtime index. Unsuitable as source of truth.

### 4.2 Option B: JSON-Per-Entity Files

One JSON file per entity in a directory structure:

```
context/
  items/
    BACK-001.json
    BACK-002.json
  events/
    EVT-001.json
  decisions/
    DEC-001.json
```

**Advantages:**
- Perfect git integration -- each file tracked individually, clean diffs, blame per field
- Human-readable (formatted JSON)
- Flexible schema -- any field can be added to any file
- Merge conflicts are rare (different entities = different files) and resolvable when they occur
- Cross-references are just ID strings
- JSON Schema validation for flexible-but-validated schemas
- Trivially parseable in any language

**Disadvantages:**
- No built-in query capability -- CLI must scan all files for search/filter
- Large directories (1000+ files) slow down `ls` and some editors
- No referential integrity without external validation
- JSON is more verbose than YAML for metadata
- No narrative content support (descriptions are escaped strings in JSON)

**Prior art:**
- `git-issues` (steviee/git-issues) -- stores issues as markdown files with YAML frontmatter under `.issues/`, zero-padded ID + slug filenames. Bidirectional relations, AI-agent friendly.
- `Markdown Projects` (markdownprojects.com) -- stores project state as human-readable files under `.mdp/`. Every issue is a markdown file with YAML frontmatter.
- `json-repo` (iotshaman/json-repo) -- in-memory repository pattern persisting to JSON files.
- `Sir` (Hacker News "git-diff-able JSON database on yer filesystem") -- JSON files on the filesystem as a database.

**Failure modes:**
- ID collisions if two branches create entities simultaneously (mitigated by prefix namespacing)
- Performance degrades past ~5,000 files in a single directory (mitigated by subdirectories)
- No atomic multi-file transactions (git commit is the transaction boundary)

### 4.3 Option C: JSONL (JSON Lines)

Append-only files with one JSON record per line:

```
context/
  items.jsonl      # all work items, one per line
  events.jsonl     # all events, one per line
  decisions.jsonl  # all decisions, one per line
```

**Advantages:**
- Git-friendly for appends (new lines = clean diffs)
- Compact (no per-file overhead)
- Easy streaming parse
- Good for event logs and audit trails

**Disadvantages:**
- Editing existing records requires rewriting the entire file -- produces large, noisy diffs
- Merge conflicts when two branches append to the same file simultaneously
- Harder to find a specific record (must scan the file)
- Less human-readable than individual files (must find the right line)
- Not well-suited for entities that are frequently updated (work items change status often)

**Verdict:** Good for append-only data (event log, audit trail). Poor for mutable entities.

### 4.4 Option D: Markdown with YAML Frontmatter

Each entity is a markdown file with structured YAML frontmatter and a narrative body:

```yaml
---
id: BACK-001
type: work-item
status: in-progress
priority: must
created: 2026-03-26
updated: 2026-03-26
assignee: developer
tags: [security, must-fix]
references: [BACK-002, DEC-015]
custom:
  estimated_hours: 4
  component: cli
  sprint: 2026-W13
---
# Security review

Conduct a comprehensive security review of the CLI, covering input validation,
file operations, network calls, and supply chain concerns.

## Scope

- Input validation for container names, hostnames, package names
- Symlink following and path traversal in backup/restore
- TLS verification in network calls
...
```

**Advantages:**
- Git-native (plain text files, clean diffs, blame, merge)
- Human-readable -- perhaps the MOST readable of all options
- Flexible schema via the `custom:` map in frontmatter (any key-value pairs)
- Machine-parseable via YAML frontmatter parsing (well-established libraries)
- Narrative content in markdown body -- descriptions, rationale, analysis
- RAG can index the markdown body directly (same chunking strategy as existing context files)
- CLI can query the YAML metadata for structured search
- Already the pattern used by SKILL.md files in aibox
- Already the pattern used by git-issues, Markdown Projects, and multiple other tools
- Editors (VS Code, Obsidian, etc.) render frontmatter nicely
- Static site generators (Docusaurus, Jekyll, Hugo) consume this format natively

**Disadvantages:**
- YAML parsing is slower than JSON parsing (but negligible for <1000 files)
- YAML has some gotchas (Norway problem: `no` parsed as boolean, unquoted strings)
- Frontmatter fields need a defined schema to avoid chaos
- Query performance requires scanning all files (same as JSON-per-entity)
- Less structured than JSON for complex nested data

**Prior art:**
- Jekyll, Hugo, Gatsby, Docusaurus -- all use markdown + frontmatter as their data model
- Obsidian -- uses YAML frontmatter for metadata, supports Dataview plugin for SQL-like queries over frontmatter
- git-issues -- markdown files with YAML frontmatter
- Markdown Projects -- `.mdp/` directory with markdown + frontmatter
- aibox itself -- SKILL.md files already use this exact pattern

### 4.5 Option E: SurrealDB Embedded

SurrealDB is a multi-model database (document, graph, relational, time-series) written in Rust that can run embedded in-process.

**Advantages:**
- Rust-native -- first-class crate, no FFI
- Multi-model: document store with graph relationships and SQL-like query language (SurrealQL)
- Schema-flexible (schemaless or schemafull modes)
- Embedded mode uses SurrealKV (custom key-value store) or RocksDB
- Rich query capabilities: JOINs, subqueries, graph traversals

**Disadvantages:**
- Stores data in binary format (SurrealKV or RocksDB) -- same git problems as SQLite
- Large dependency (the surrealdb crate pulls in significant transitive dependencies)
- Still maturing -- API changes between versions
- Overkill for <1000 entities
- Not human-readable, not git-diffable

**Verdict:** Interesting technology but violates the git-native and human-readable constraints. Same fundamental problem as SQLite for source of truth.

### 4.6 Option F: Other Embedded Databases

| Database | Language | Storage | Git-friendly? | Notes |
|----------|----------|---------|---------------|-------|
| **PoloDB** | Rust | Binary file | No | MongoDB-like API, ~500KB RAM, but binary storage |
| **redb** | Pure Rust | Binary file (B-tree) | No | Key-value store, stable format, but binary |
| **sled** | Rust | Binary directory | No | Modern embedded DB, but binary + directory |
| **TinyDB** | Python | JSON file | Partially | Single JSON file, not great for git diffs at scale |
| **LowDB** | Node.js | JSON file | Partially | Same as TinyDB -- single file rewrites on every change |
| **DuckDB** | C++ | Binary file | No | Analytical/columnar, wrong use case |
| **libSQL** | C (Rust bindings) | Binary file | No | SQLite fork, same binary problems |

**Verdict:** All binary-format databases share the same fundamental incompatibility with git. JSON-backed databases (TinyDB, LowDB) store everything in a single file that is rewritten on every change -- bad for git diffs. None of these satisfy the constraints.

---

## 5. The Hybrid Architecture: Markdown Source + SQLite Index

The analysis above points to a clear conclusion: **no single storage engine satisfies all five constraints**. Text files (markdown, JSON) satisfy git-native, human-readable, and flexible schema. Databases (SQLite, SurrealDB) satisfy efficient search and query performance. The solution is a hybrid:

**Markdown+frontmatter files are the source of truth. SQLite is a derived runtime index.**

### 5.1 Source of Truth: Markdown + YAML Frontmatter

```
context/
  items/
    BACK-001.md          # work item with frontmatter + description
    BACK-002.md
  events/
    EVT-001.md           # event with frontmatter + details
  decisions/
    DEC-016.md           # decision with frontmatter + rationale
  projects/
    PROJ-001.md          # project with frontmatter + overview
  log/
    events.jsonl         # append-only event log (lightweight events)
```

Each file has YAML frontmatter for structured metadata and a markdown body for narrative content. The `custom:` map in frontmatter allows arbitrary user-defined fields.

### 5.2 Derived Index: SQLite + sqlite-vec

```
context/.aibox/
  index.db             # SQLite index (gitignored)
  vectors.db           # Vector embeddings (gitignored, or same file)
```

The CLI builds the index by scanning all markdown files, parsing frontmatter, and inserting into SQLite tables. The index supports:

- **Structured queries**: `aibox query "status:todo priority:must"` -- SQL query against indexed frontmatter fields
- **Full-text search**: SQLite FTS5 over markdown body text
- **Vector search**: sqlite-vec for RAG embeddings (already designed in rag-context-layer-2026-03.md)
- **Cross-reference resolution**: `aibox refs BACK-001` -- find all items that reference a given ID
- **Aggregations**: `aibox stats` -- counts by status, priority, type

The index is **derived, not authoritative**. It can be deleted and rebuilt at any time:

```bash
aibox index rebuild    # scan all context/ files, rebuild index.db
```

### 5.3 Write Path

1. User (or AI agent) creates/edits a markdown file in `context/items/BACK-NNN.md`
2. The CLI command `aibox item create` or `aibox item update` writes the file
3. After writing, the CLI updates the SQLite index incrementally (upsert the changed record)
4. The file is committed to git -- the markdown file IS the commit payload

### 5.4 Read Path

1. **Quick queries** hit the SQLite index (fast, sub-millisecond)
2. **Full content** reads the markdown file directly
3. **Semantic search** uses sqlite-vec embeddings
4. If the index is stale or missing, the CLI rebuilds it from the markdown files

### 5.5 Schema Validation

Use JSON Schema (or a YAML-compatible equivalent) to validate frontmatter:

```yaml
# context/.aibox/schemas/work-item.schema.yaml
type: object
required: [id, type, status, title, created]
properties:
  id:
    type: string
    pattern: "^BACK-\\d{3,}$"
  type:
    const: work-item
  status:
    enum: [draft, todo, in-progress, blocked, done, cancelled, archived]
  priority:
    enum: [must, should, could, wont]
  title:
    type: string
  created:
    type: string
    format: date
  tags:
    type: array
    items: { type: string }
  references:
    type: array
    items: { type: string }
  custom:
    type: object
    additionalProperties: true   # <-- flexible schema!
```

The `custom:` field has `additionalProperties: true`, giving users Jira-like custom field flexibility without Jira's EAV complexity. The CLI validates frontmatter on write, catching errors before commit.

---

## 6. Vector Embeddings Storage

### 6.1 Size Analysis

| Scenario | Chunks | Dims | Bytes/vector | Total |
|----------|--------|------|-------------|-------|
| Small project (20 files) | 200 | 384 | 1,536 | 300KB |
| Medium project (60 files) | 600 | 384 | 1,536 | 900KB |
| Large project (200 files) | 2,000 | 384 | 1,536 | 3MB |
| With metadata overhead (JSON) | 2,000 | 384 | ~2,000 | 4MB |

Embeddings are small enough to store as a JSON file, but JSON encoding of float arrays is verbose (each float like `0.0234375` takes 9-12 bytes as text vs 4 bytes as binary). A 2,000-chunk embedding file would be ~15MB in JSON vs ~3MB in binary.

### 6.2 Storage Options

**Option A: SQLite (sqlite-vec)** -- already recommended by the RAG research. Embeddings stored in a gitignored `.aibox/index.db` alongside the structured index. Binary format is fine because embeddings are derived data, not source of truth.

**Option B: JSON file** -- `context/.aibox/vectors.json`. Human-inspectable but large and slow to parse. Not recommended.

**Option C: Binary format in gitignored file** -- custom binary or NumPy-compatible format. Fast but adds complexity without benefit over SQLite.

**Recommendation:** Store embeddings in the same SQLite database as the structured index. Both are derived, both are gitignored, and sqlite-vec is already selected. One file, one rebuild command.

---

## 7. Migration from Current Format

aibox currently stores context in monolithic markdown files:

- `context/BACKLOG.md` -- all work items in a markdown table
- `context/DECISIONS.md` -- all decisions in a single file
- `context/PROJECTS.md` -- all projects in a single file

The new architecture would split these into per-entity files:

```
# Before (current)
context/BACKLOG.md          # 100 items in one table

# After (proposed)
context/items/BACK-001.md   # one item per file
context/items/BACK-002.md
...
context/items/BACK-100.md
context/BACKLOG.md           # can be retained as a generated overview/index
```

Migration can be incremental:
1. Keep existing monolithic files as the source of truth initially
2. Add per-entity files alongside them
3. Once the CLI fully supports per-entity operations, deprecate the monolithic files
4. Optionally generate monolithic overview files from per-entity files (reversed source of truth)

---

## 8. Comparison Matrix

| Criterion | SQLite (binary) | JSON-per-entity | JSONL | Markdown+frontmatter | SurrealDB |
|-----------|-----------------|-----------------|-------|----------------------|-----------|
| **Git-native** | Poor (binary, no diff/merge) | Excellent | Good (append) / Poor (edit) | Excellent | Poor (binary) |
| **Human-readable** | No (requires tooling) | Good (formatted JSON) | Moderate (dense lines) | Excellent (rendered everywhere) | No (binary) |
| **Flexible schema** | Good (JSON columns) | Excellent (any fields) | Excellent (any fields) | Excellent (custom: map) | Excellent (schemaless mode) |
| **Query performance** | Excellent (SQL + indexes) | Poor (file scan) | Poor (line scan) | Poor (file scan) | Excellent (SurrealQL) |
| **RAG support** | Excellent (sqlite-vec) | Good (index files same as MD) | Moderate | Excellent (MD body = RAG input) | Good (built-in vector) |
| **Merge conflicts** | Fatal (binary) | Rare (per-file) | Common (same file) | Rare (per-file) | Fatal (binary) |
| **Narrative content** | Awkward (text in columns) | Awkward (escaped strings) | Awkward (escaped strings) | Native (markdown body) | Awkward (text in documents) |
| **Tooling ecosystem** | Excellent (SQL tools) | Good (JSON tools) | Moderate | Excellent (editors, SSGs, Obsidian) | Growing |
| **Rust library support** | Excellent (rusqlite) | Excellent (serde_json) | Excellent (serde_json) | Good (serde_yaml + pulldown-cmark) | Good (surrealdb crate) |
| **As derived index** | Excellent | N/A | N/A | N/A | Good |
| **As source of truth** | Poor | Good | Moderate | Excellent | Poor |

---

## 9. Rust Library Support

### For the Recommended Architecture (Markdown+Frontmatter + SQLite Index)

**Source of truth (reading/writing markdown+frontmatter):**

| Crate | Purpose | Maturity |
|-------|---------|----------|
| `serde_yaml` | Parse/serialize YAML frontmatter | Stable, widely used |
| `gray_matter` | Extract frontmatter from markdown (like Node.js gray-matter) | Maintained |
| `pulldown-cmark` | Parse markdown body | Stable, reference implementation |
| `serde_json` / `serde` | Serialize structured data, handle `custom:` fields as `Value` | Stable |
| `jsonschema` | Validate frontmatter against JSON Schema | Maintained |

**Derived index (SQLite + vector search):**

| Crate | Purpose | Maturity |
|-------|---------|----------|
| `rusqlite` (with `bundled` feature) | SQLite access | Stable, widely used |
| `sqlite-vec` (via rusqlite) | Vector similarity search | Stable enough for embedded use |
| `fastembed` | Generate embeddings (bge-small-en-v1.5) | Maintained, ONNX-based |

**Frontmatter parsing approach in Rust:**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_yaml::Value;

#[derive(Serialize, Deserialize)]
struct WorkItem {
    id: String,
    #[serde(rename = "type")]
    item_type: String,
    status: String,
    title: String,
    created: String,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    references: Vec<String>,
    #[serde(default)]
    custom: HashMap<String, Value>,  // flexible schema!
}
```

The `custom: HashMap<String, Value>` field gives Jira-like custom field support with zero schema migration cost. Users can add any key-value pairs, and the CLI can query them via the SQLite index.

---

## 10. Recommendation

### Primary: Markdown + YAML Frontmatter (source of truth) + SQLite (derived index)

This architecture satisfies all five constraints:

| Constraint | How it is satisfied |
|------------|-------------------|
| **Single source of truth** | Markdown files are authoritative. SQLite is derived and gitignored. |
| **Git-native** | Markdown files produce clean diffs, support blame, merge cleanly, and are code-reviewable. |
| **Flexible schema** | The `custom:` map in YAML frontmatter allows arbitrary user-defined fields. JSON Schema validates the known fields. |
| **Efficient search** | SQLite index enables structured queries, FTS5 for full-text, sqlite-vec for vector search. Index rebuilds from source files in seconds. |
| **Human-readable** | Markdown is universally readable. Frontmatter is clear metadata. No tooling required to read. |

### Why Not JSON-Per-Entity?

JSON-per-entity was the second-best option. Markdown+frontmatter wins because:

1. **Narrative content is first-class**: work items, decisions, and events all benefit from rich descriptions. In JSON, descriptions are escaped strings. In markdown, they are the natural document body.
2. **Existing pattern**: aibox's SKILL.md files already use YAML frontmatter. Standardizing on this pattern across all context artifacts creates consistency.
3. **Editor/tool support**: every code editor, every static site generator, and tools like Obsidian understand markdown+frontmatter natively.
4. **RAG integration**: the markdown body is directly indexable by the RAG chunking strategy already designed (heading-based chunking from rag-context-layer-2026-03.md). JSON values are not.

### Why Not Just Keep Monolithic Markdown Files?

The current BACKLOG.md (single table with all items) works for small projects but does not scale:

- Merge conflicts when two agents edit different items simultaneously
- No per-item git blame or history
- No structured query capability without parsing markdown tables
- No validation (typos in status values go unnoticed)
- No custom fields (the table format is rigid)

### Implementation Phases

1. **Phase 1**: Define schemas for core primitives (work-item, decision, event, project) as JSON Schema files
2. **Phase 2**: Implement frontmatter read/write in CLI (`aibox item create`, `aibox item show`, `aibox item list`)
3. **Phase 3**: Build SQLite index with incremental updates
4. **Phase 4**: Integrate vector search (reuse RAG layer design)
5. **Phase 5**: Migration tool to convert existing BACKLOG.md / DECISIONS.md to per-entity files
6. **Phase 6**: Generate overview files (BACKLOG.md as a computed view) for backward compatibility

### JSONL for Append-Only Data

One refinement: use **JSONL** for truly append-only data like the event log. Events are written once and never edited, making JSONL's append-friendly, single-file format ideal. The event log can coexist with per-entity markdown files:

```
context/
  items/BACK-001.md        # markdown+frontmatter (mutable)
  decisions/DEC-016.md     # markdown+frontmatter (mutable)
  log/events.jsonl         # JSONL (append-only)
```

This matches the event log design already documented in event-log-design-2026-03.md.

---

## Sources

- [Git Internals - Packfiles](https://git-scm.com/book/en/v2/Git-Internals-Packfiles)
- [Git's Delta Compression Algorithm: Technical Deep Dive](https://www.linkedin.com/pulse/gits-delta-compression-algorithm-technical-deep-dive-maheshwari)
- [How Git Compresses Files](https://www.aviator.co/blog/how-git-compresses-files/)
- [Should SQLite *.db Files Be Stored in a Git Repository?](https://www.designcise.com/web/tutorial/should-sqlite-db-files-be-stored-in-a-git-repository)
- [Tracking SQLite Database Changes in Git](https://ongardie.net/blog/sqlite-in-git/)
- [git-sqlite: Custom diff and merge driver for SQLite](https://github.com/cannadayr/git-sqlite)
- [How to Use git diff with an SQLite3 Database](https://dunkels.com/adam/git-diff-sqlite3/)
- [Why SQLite Does Not Use Git](https://sqlite.org/whynotgit.html)
- [Git LFS - How It Works](https://www.perforce.com/blog/vcs/how-git-lfs-works)
- [Why You Shouldn't Use Git LFS](https://gregoryszorc.com/blog/2021/05/12/why-you-shouldn%27t-use-git-lfs/)
- [About the Jira Database Schema](https://developer.atlassian.com/server/jira/platform/database-schema/)
- [Jira Database: Custom Fields](https://developer.atlassian.com/server/jira/platform/database-custom-fields/)
- [Read This Before Adding Custom Fields to Large Jira Instances](https://medium.com/exness-blog/why-are-sysops-and-devops-hesitant-to-add-custom-fields-to-large-jira-instances-8f3808605951)
- [The Jira Object Model](https://thejiraguy.com/2022/03/16/the-jira-object-model/)
- [Optimize Your Custom Fields (Atlassian)](https://confluence.atlassian.com/enterprise/optimize-your-custom-fields-1540234569.html)
- [Linear vs Jira Comparison (Nuclino)](https://www.nuclino.com/solutions/linear-vs-jira)
- [Linear vs Jira: Which Tool Wins?](https://www.laneapp.co/blog/jira-vs-linear-which-tool-wins)
- [Markdown Projects](https://www.markdownprojects.com/)
- [git-issues: Repository-based Issue Tracker](https://github.com/steviee/git-issues)
- [Git as a NoSQL Database](https://www.kenneth-truyers.net/2016/10/13/git-nosql-database/)
- [So You Want a Git Database? (DoltHub)](https://www.dolthub.com/blog/2021-11-26-so-you-want-git-database/)
- [SurrealDB Embedding in Rust](https://surrealdb.com/docs/surrealdb/embedding/rust)
- [The Power of SurrealDB Embedded](https://surrealdb.com/blog/the-power-of-surrealdb-embedded)
- [PoloDB: An Embedded JSON Database](https://www.polodb.org/)
- [redb: Embedded Key-Value Database in Pure Rust](https://github.com/cberner/redb)
- [Vector Embedding Storage Requirements (Milvus)](https://milvus.io/ai-quick-reference/what-are-the-storage-requirements-for-embeddings)
- [Git LFS (Atlassian Tutorial)](https://www.atlassian.com/git/tutorials/git-lfs)
