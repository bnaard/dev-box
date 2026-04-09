# File-Per-Entity Scaling in Git Repositories

**Status:** Research complete
**Date:** 2026-03-26
**Relates to:** context-database-architecture-2026-03.md, process-ontology-primitives-2026-03.md, event-log-design-2026-03.md
**Author:** Researcher agent

---

## 1. Problem Statement

aibox's context system stores each process artifact (work item, event, decision, etc.) as a separate markdown file with YAML frontmatter. This is the foundation for kaits, a multi-agent company simulator that could generate thousands of artifacts per project. The question: at what scale does this pattern break, and what mitigations exist?

The scale targets under consideration:

| Scenario | File count | Source |
|----------|-----------|--------|
| Single aibox project, active use | 500 - 2,000 | Typical human workflow |
| Large aibox project, full lifecycle | 5,000 - 20,000 | Multi-year project with rich history |
| kaits single project simulation | 20,000 - 50,000 | Agent-generated artifacts |
| kaits company simulation (20 projects) | 200,000 - 1,000,000 | Full company sim |

---

## 2. Git at Scale -- Real-World Data

### 2.1 Baseline: How Git Handles File Count

Git's core operations (`status`, `add`, `diff`, `checkout`) scale linearly with the number of tracked files because they must stat every file in the working tree to detect changes. This is the fundamental bottleneck.

**Measured performance at different file counts (approximate, on modern hardware):**

| File count | `git status` (cold) | `git status` (warm/cached) | Source |
|-----------|---------------------|---------------------------|--------|
| 10,000 | < 1s | < 0.5s | General benchmarks |
| 50,000 | 2-3s | ~1s | Dropbox monorepo data |
| 100,000 | 3-5s | ~1.5s | Extrapolated from Canva |
| 500,000 | ~10s | 2-3s (with fsmonitor) | Canva engineering blog |
| 3,500,000 | ~10 minutes | 4-5s (with VFS for Git) | Microsoft Windows repo |

### 2.2 Real-World Monorepos

**Canva (500K files):** Git status took 10 seconds on average, git fetch took 15 seconds to minutes. Their solutions: (1) fsmonitor with Watchman to capture changes as they happen instead of scanning all files, (2) `feature.manyFiles` which enables the untracked cache, (3) ignoring generated/translation files. Result: git status reduced to 2-3 seconds with cache. Source: [Canva Engineering Blog](https://www.canva.dev/blog/engineering/we-put-half-a-million-files-in-one-git-repository-heres-what-we-learned/).

**Dropbox:** Git status took over 2 seconds before optimization. They shipped a wrapper that automatically enables fsmonitor for developers. Status dropped to under 1 second. Source: [Dropbox Tech Blog](https://dropbox.tech/application/speeding-up-a-git-monorepo-at-dropbox-with--200-lines-of-code).

**Microsoft Windows (3.5M files, 270GB):** Without VFS for Git, git status took ~10 minutes, checkout took up to 3 hours, clone took 12+ hours. With VFS for Git: clone in minutes, checkout in 30 seconds, status in 4-5 seconds. Key insight: typical developers only touch 50-100K of those 3.5M files. Source: [Brian Harry's Blog](https://devblogs.microsoft.com/bharry/the-largest-git-repo-on-the-planet/).

**Chromium (~400K files, 33K directories):** Used as a benchmark for Git's built-in fsmonitor. With fsmonitor enabled, status dropped from 17-85 seconds to under 1 second. Source: [GitHub Engineering Blog](https://github.blog/engineering/infrastructure/improve-git-monorepo-performance-with-a-file-system-monitor/).

### 2.3 Git Performance Optimization Stack

Modern Git includes several features that dramatically improve large-repo performance. Listed from easiest to most invasive:

**1. `feature.manyFiles` (git config)**
Sets `index.version=4` (smaller index) and `core.untrackedCache=true` (caches untracked file scan results). Zero-cost to enable. Helps most at 10K+ files.

**2. fsmonitor (Watchman or built-in)**
Instead of stat-ing every file on each operation, Git asks a filesystem monitor daemon which files changed since last check. Reduces `git status` from O(n) filesystem calls to O(changed). Built into Git since 2.37. Effect: 10-100x speedup on large repos.

**3. Commit-graph**
Pre-computed graph of commit ancestry. Speeds up `git log`, `git merge-base`, reachability checks. One benchmark showed `git log` going from 11 seconds to 0.13 seconds. Enabled automatically by `git maintenance`.

**4. Multi-pack-index (MIDX)**
Single index across multiple packfiles. Speeds up object lookups. Managed automatically by `git maintenance`.

**5. `git maintenance` (background)**
Runs prefetch, commit-graph, loose-objects, and incremental-repack tasks on a schedule. Zero developer friction once configured. Available since Git 2.31.

**6. Sparse checkout (cone mode)**
Only materializes a subset of files on disk. All files remain in git history, but the working tree only contains directories you specify. Effect: transforms a 500K-file repo into what feels like a 5K-file repo for daily operations.

**7. Partial clone (`--filter=blob:none`)**
Fetches tree structure but not file contents. Blobs are downloaded on demand. Combines with sparse checkout: clone is near-instant, and only accessed files are fetched.

**8. Scalar (Microsoft, now in Git mainline)**
Bundles sparse checkout + partial clone + fsmonitor + commit-graph + background maintenance into a single `scalar clone` command. Available since Git 2.38. The recommended approach for any repo over 50K files.

### 2.4 Key Insight: Git's Limit Is the Working Tree, Not the Repository

Git's object store (packfiles) handles millions of objects efficiently. The bottleneck is the working tree: stat-ing files on disk. Solutions that reduce the on-disk file count (sparse checkout, VFS) are the most effective. This is directly relevant to aibox: archiving cold files out of the working tree matters more than archiving them out of git history.

---

## 3. Filesystem Limits

### 3.1 Files Per Directory

| Filesystem | Theoretical max files/dir | Practical degradation point | Mechanism |
|-----------|--------------------------|---------------------------|-----------|
| ext4 | ~10M (2-level HTree), ~6B (3-level, Linux 4.12+) | ~10K for `ls`/readdir, ~50K for random access | HTree B-tree indexing; readdir returns unsorted, forcing userspace sort |
| APFS | Millions (B-tree) | ~5-10K for readdir (global kernel lock issue) | APFS acquires a global kernel lock on readdir, serializing all I/O |
| NTFS | ~4B (B-tree) | ~10K for Explorer, fine for programmatic access | Explorer UI is the bottleneck, not NTFS itself |
| XFS | Billions | ~50-100K | Designed for large directories; outperforms ext4 at scale |

**Critical finding for macOS:** APFS has a global kernel lock during readdir(), making it 5-6x slower than HFS+ for directory scanning. This is particularly relevant because many aibox users will be on macOS. Directories with over 5,000 files will feel sluggish on APFS.

### 3.2 Subdirectory Sharding

Sharding distributes files across a tree of subdirectories, keeping each directory small.

**Date-based sharding** (e.g., `items/2026/03/BACK-001.md`):
- Natural and human-readable
- Uneven distribution (busy months have more files)
- Good enough for up to ~100K total files with monthly granularity
- Hot files (recent) cluster together, which is good for sparse checkout

**Hash-based sharding** (e.g., `items/a7/f3/BACK-a7f3b2c1.md`):
- Even distribution guaranteed
- Not human-browsable
- Like git's own object store: 256 first-level dirs, optionally 256 second-level
- 2-level sharding (256 x 256 = 65K buckets) handles millions of files
- Overkill for under 100K files

**Prefix-based sharding** (e.g., `items/BACK-0/BACK-001.md` through `items/BACK-9/`):
- Compromise: human-readable, reasonably distributed
- Works well when IDs are sequential integers

**Recommendation:** Date-based sharding (`YYYY/MM/` or `YYYY/`) is the best fit for process artifacts. It aligns with the hot/cold access pattern (recent files are accessed most), is human-readable, and works naturally with sparse checkout (check out only recent months).

### 3.3 Inode Limits

Default ext4 creates 1 inode per 16KB of disk space. A 100GB partition has ~6.5M inodes. Each file and directory consumes one inode. At 1M small files, inode exhaustion is a real risk on small partitions but not on modern systems with large disks. Docker containers with overlay2 inherit the host's inode limits. Monitor with `df -i`.

---

## 4. Projects That Use File-Per-Entity at Scale

### 4.1 Obsidian Vaults

Obsidian is the closest analogue to aibox's context system: markdown files with metadata, stored locally, often backed by git.

- **Typical large vault:** 5,000-15,000 notes. Works well on desktop.
- **Extreme vaults:** 50K+ notes plus 40K+ attachments reported on forums. Desktop works; mobile vault load time ~3 minutes, reindexing ~27 minutes.
- **Obsidian's indexing strategy:** In-memory index built on startup by scanning all files. Background reindexing on file changes. Full reindex of 15,000 files takes ~1 minute on M-series Mac, ~10 minutes for embedding generation.
- **Git integration:** The Obsidian Git plugin works well up to ~10K notes. Beyond that, users report auto-backup intervals needing to increase to 5-10 minutes to avoid performance impact.
- **Multi-vault strategy:** Power users split content across purpose-specific vaults to maintain performance. This is analogous to per-project repos in aibox. Source: [Obsidian Forum](https://forum.obsidian.md/t/terabyte-size-million-notes-vaults-how-scalable-is-obsidian/66674).

### 4.2 Static Site Generators

**Hugo (Go):** Renders 10,000 pages in ~10 seconds. Smashing Magazine's 7,500-page site builds in ~13 seconds. Hugo's speed comes from Go's compiled performance and incremental-build optimizations. Git performance with 10K content files is not reported as an issue -- the build time is the bottleneck, not git.

**Jekyll (Ruby):** Significantly slower. 10,000 pages can take minutes to build. Git performance is not the limiting factor.

**Key lesson:** For aibox, the index/query layer will be the bottleneck long before git is. Investing in fast incremental indexing matters more than optimizing git for this scale range.

### 4.3 Kubernetes GitOps (ArgoCD)

ArgoCD uses git repos as the source of truth for Kubernetes manifests.

- **Scaling point:** Manifest generation becomes a bottleneck beyond ~50 applications in a monorepo. UI degrades beyond 1,000 applications.
- **Repo server:** At scale, ArgoCD's repo server (which clones and parses manifests) becomes the most resource-intensive component. Solution: multiple replicas + caching.
- **Best practice:** Separate repos for app source code vs. deployment manifests. Keep config-only repos lean.
- **Relevance to aibox:** ArgoCD's experience confirms that thousands of YAML files in git work fine for the git layer, but the tooling layer (parsing, rendering, reconciling) is what needs optimization.

### 4.4 Zettelkasten / Digital Gardens

Users with 6,000-10,000+ interlinked notes in git report:
- Git operations remain fast (notes are small text files; git handles this well)
- Graph visualization is the bottleneck (Obsidian's graph view struggles above 6K nodes)
- The KaaS repository houses 10,000+ notes and uses git without reported issues
- Search/indexing performance becomes the real concern at scale

---

## 5. Indexing Performance

### 5.1 YAML Frontmatter Parsing Speed

Parsing YAML frontmatter from markdown files in Rust (using `yaml-front-matter`, `gray_matter`, or `frontmatter-gen` crates):

| File count | Estimated parse time (Rust, single-threaded) | With parallelism (8 cores) |
|-----------|----------------------------------------------|---------------------------|
| 1,000 | ~50ms | ~10ms |
| 10,000 | ~500ms | ~80ms |
| 50,000 | ~2.5s | ~400ms |
| 100,000 | ~5s | ~800ms |

These estimates assume ~1KB files with ~200 bytes of YAML frontmatter. The bottleneck is filesystem I/O (open + read + close per file), not YAML parsing itself. With memory-mapped I/O or batched reads, these times improve further.

### 5.2 Incremental Indexing Strategy

**mtime-based approach:** On each index update, stat all files and re-parse only those with mtime newer than the last index timestamp. Cost of stat-ing 100K files:

| File count | `stat()` time (ext4, cold cache) | `stat()` time (warm cache) |
|-----------|--------------------------------|---------------------------|
| 10,000 | ~100ms | ~20ms |
| 50,000 | ~500ms | ~100ms |
| 100,000 | ~1s | ~200ms |

This is fast enough for interactive use up to 100K files, but for 1M files the cold-cache stat takes 5-10 seconds.

**fswatch/inotify approach:** Register watches on content directories. On change events, update only affected index entries. Cost: ~1KB kernel memory per watch. For 100K files: ~100MB of kernel memory for inotify watches. Practical limit on Linux is governed by `fs.inotify.max_user_watches` (default 8192, commonly raised to 65536 or higher).

**Recommended approach:** Persistent index file (SQLite or binary) + mtime-based incremental updates. On startup, stat all files, re-parse changed ones. During operation, use fswatch for real-time updates. This is essentially what Obsidian does internally.

### 5.3 Index Storage

A persistent index for 100K artifacts, each with ~10 frontmatter fields:
- **SQLite:** ~20-50MB. Excellent query performance. Single file, easy to manage.
- **JSON:** ~30-80MB. Slow to parse/write at this scale. Not recommended above 10K.
- **Custom binary (MessagePack, FlatBuffers):** ~10-30MB. Fastest read/write. Requires custom tooling.

Recommendation: SQLite for the index. It is NOT the source of truth (the markdown files are), so binary-in-git concerns do not apply. The index is a derived, regenerable cache -- it can be `.gitignore`d.

---

## 6. Solutions for Scaling Beyond 50K Files

### 6.1 Hot/Cold Archiving

Move completed/old artifacts to compressed archives, keeping only active files as individual markdown files.

**Implementation options:**

| Strategy | Compression | Git behavior | Queryability | Complexity |
|----------|------------|-------------|-------------|-----------|
| tar.gz bundles | 10-20x for text | Binary diff (poor) | Must extract to read | Low |
| ZIP archives | 5-10x | Binary diff (poor) | Can read individual entries | Low |
| SQLite archive | N/A (~2-3x smaller than raw) | Binary diff (poor) | Full SQL queries | Medium |
| Separate directory + sparse checkout | None | Normal text diff | Direct file access | Low |
| Separate repo | None | Normal text diff | Cross-repo tooling needed | Medium |

**Recommended: Sparse checkout as the primary mechanism.** Keep ALL files in git as individual markdown (preserving full diff/merge/blame history). Use directory sharding by date. Configure sparse checkout to only materialize recent files on disk. This gives the best of both worlds: full git history with manageable working tree size.

**For extreme scale (500K+):** Combine sparse checkout with periodic archival. Move very old files (>2 years) to a compressed archive or separate repo. The archive preserves data but sacrifices per-file git history for the archived period.

### 6.2 Sparse Checkout in Detail

```bash
# Clone with sparse checkout + partial clone
git clone --filter=blob:none --sparse https://github.com/org/repo.git
cd repo

# Only check out recent context files
git sparse-checkout set context/items/2026/ context/items/2025/ context/active/

# Add more directories as needed
git sparse-checkout add context/items/2024/
```

Effect on a repo with 100K artifact files across 5 years:
- Full clone + full checkout: 100K files on disk, git status ~5s
- Sparse checkout (last 2 years): ~40K files on disk, git status ~2s
- Sparse checkout (last 6 months): ~10K files on disk, git status < 1s

All files remain in git history. `git log -- context/items/2020/01/BACK-001.md` still works. The files just are not materialized on disk until requested.

### 6.3 Git LFS for Archives

If archiving to tar.gz/ZIP bundles, store them in Git LFS to avoid bloating the git object database. Each archive version is stored as a pointer file in git, with the actual binary in LFS storage. This keeps `git clone` fast even if archives are large.

### 6.4 SQLite as Cold Storage

Store archived artifacts in a SQLite database. The database is a `.gitignore`d derived artifact, regenerable from git history (or from an archived bundle). This provides fast queries over historical data without the filesystem cost of millions of individual files.

Tradeoff: SQLite files do not diff/merge well in git. Acceptable if the SQLite is (a) read-only historical data, or (b) a regenerable cache, not a source of truth.

### 6.5 Separate Repos per Project

For kaits at company-simulation scale, each simulated project gets its own git repo. Cross-project references use project ID + artifact ID (e.g., `PROJ-007:BACK-042`). This is the simplest solution that scales to millions of total artifacts.

---

## 7. The kaits Dimension

### 7.1 Scale Analysis

| Scenario | Artifacts | Recommended architecture |
|----------|----------|------------------------|
| 1 kaits project | 20-50K | Single repo, date-sharded dirs, sparse checkout |
| 5 kaits projects | 100-250K | Repo per project, shared index database |
| 20 kaits projects (full company) | 400K-1M | Repo per project, Postgres/SurrealDB hot tier |
| Continuous simulation (months) | 1M+ | Database-primary, periodic git snapshots |

### 7.2 Where File-Per-Entity Breaks Down

At 1M artifacts in a single repo:
- `git status` with fsmonitor: ~5-10s (tolerable for batch, not for interactive)
- Full index rebuild: ~30-60s (Rust, parallel)
- Incremental index (mtime check on 1M files): ~5-10s cold cache
- Disk usage: ~1-2GB for small markdown files
- `git clone`: minutes even with partial clone (tree objects alone are significant)

The pattern does NOT break at the git level (packfiles handle millions of objects fine). It breaks at the working-tree level (too many files to stat) and the tooling level (indexing, searching, displaying).

### 7.3 Recommended Architecture for kaits

**Tier 1 -- Hot (individual files in git):**
Active and recent artifacts. File-per-entity markdown with YAML frontmatter. Date-sharded directories. Sparse checkout for only the relevant time window. This is the aibox native format.

**Tier 2 -- Warm (queryable archive):**
Completed project artifacts. SQLite database per project, generated from the markdown files. `.gitignore`d (regenerable). Provides fast SQL queries and full-text search over historical data without filesystem overhead.

**Tier 3 -- Cold (compressed archive):**
Very old simulation data. tar.gz bundles stored in Git LFS or external storage. Retained for compliance/audit, not for routine access.

**The boundary:** aibox owns Tier 1 (the file format, the CLI, the index). kaits owns the tiering logic -- it decides when to archive, manages the warm/cold tiers, and provides its own query layer (likely a database) for cross-project analytics.

---

## 8. Recommendations for aibox

### 8.1 Target Scale

Design aibox's context system for **up to 50,000 active files per repository** with confidence. This covers all human-use scenarios and single-project kaits simulations. Beyond 50K, require explicit configuration of sparse checkout and archival.

### 8.2 Directory Structure

Use date-based sharding for artifact directories:

```
context/
  items/
    2026/
      03/
        BACK-001.md
        BACK-002.md
      04/
        BACK-003.md
    2025/
      ...
  events/
    2026/
      03/
        EVT-001.md
  decisions/
    DEC-001.md      # Fewer decisions; flat is fine until ~1,000
```

Keep each directory under 1,000 files. With daily sharding (adding `/DD/` level), support millions of files, but monthly is sufficient for most use cases.

### 8.3 Git Configuration (automatic)

aibox should configure these settings automatically in any context repository:

```ini
[core]
    untrackedCache = true
    fsmonitor = true
[feature]
    manyFiles = true
[fetch]
    writeCommitGraph = true
[maintenance]
    auto = true
    strategy = incremental
```

These settings are free (no downside) and provide 5-10x improvement at 10K+ files.

### 8.4 Indexing

Build a persistent index (SQLite, `.gitignore`d) that:
1. On first run, parses all YAML frontmatter and builds the index (~500ms for 10K files in Rust)
2. On subsequent runs, uses mtime to detect changes and re-parses only changed files (~20ms for 10K files, warm cache)
3. During operation, optionally uses fswatch for real-time updates
4. Stores: artifact ID, type, status, priority, created date, modified date, all frontmatter fields, file path

### 8.5 Archival Strategy

1. **Under 10K files:** No archival needed. Everything is fast.
2. **10K-50K files:** Enable sparse checkout hints in documentation. Configure `git maintenance` automatically.
3. **50K-100K files:** Recommend sparse checkout. Provide `aibox archive` command that moves old items to `context/archive/YYYY/` directories and updates sparse-checkout config.
4. **100K+ files:** Require sparse checkout. Provide `aibox archive --cold` that generates SQLite warm-tier databases from archived directories.

### 8.6 What NOT to Do

- **Do not use a database as the source of truth.** The context-database-architecture research already concluded that markdown files in git are the right single source of truth. A database is a derived index, not the canonical store.
- **Do not use hash-based sharding.** It destroys human readability for minimal benefit under 1M files.
- **Do not store SQLite databases in git as a primary artifact.** They do not diff or merge. (The `.gitignore`d index is fine.)
- **Do not prematurely optimize.** At 2,000 files (typical aibox usage), raw `git status` takes under 0.5 seconds. No optimization is needed.

---

## 9. Summary Table

| File count | `git status` | Index rebuild | Recommended config | Archival needed? |
|-----------|-------------|--------------|-------------------|-----------------|
| 1,000 | < 0.5s | ~50ms | Default | No |
| 10,000 | < 1s | ~500ms | `feature.manyFiles` | No |
| 50,000 | 2-3s | ~2.5s | fsmonitor + maintenance | Optional sparse checkout |
| 100,000 | 3-5s | ~5s | Sparse checkout required | Yes (warm tier) |
| 500,000 | ~10s | ~25s | Scalar + sparse checkout | Yes (warm + cold tiers) |
| 1,000,000+ | Impractical without VFS | ~60s+ | Separate repos per project | Yes (all tiers) |

---

## Sources

- [Canva: We Put Half a Million Files in One Git Repository](https://www.canva.dev/blog/engineering/we-put-half-a-million-files-in-one-git-repository-heres-what-we-learned/)
- [GitHub Blog: Improve Git Monorepo Performance with a File System Monitor](https://github.blog/engineering/infrastructure/improve-git-monorepo-performance-with-a-file-system-monitor/)
- [Dropbox: Speeding Up a Git Monorepo](https://dropbox.tech/application/speeding-up-a-git-monorepo-at-dropbox-with--200-lines-of-code)
- [Microsoft: The Largest Git Repo on the Planet](https://devblogs.microsoft.com/bharry/the-largest-git-repo-on-the-planet/)
- [Microsoft: Introducing Scalar](https://devblogs.microsoft.com/devops/introducing-scalar/)
- [Git Documentation: git-maintenance](https://git-scm.com/docs/git-maintenance)
- [Git Documentation: git-sparse-checkout](https://git-scm.com/docs/git-sparse-checkout)
- [Git Documentation: scalar](https://git-scm.com/docs/scalar)
- [GitHub Blog: Bring Your Monorepo Down to Size with Sparse Checkout](https://github.blog/open-source/git/bring-your-monorepo-down-to-size-with-sparse-checkout/)
- [Git Tower: How to Improve Performance in Git](https://www.git-tower.com/blog/git-performance)
- [Obsidian Forum: Terabyte Size, Million Notes Vaults](https://forum.obsidian.md/t/terabyte-size-million-notes-vaults-how-scalable-is-obsidian/66674)
- [ArgoCD: Best Practices](https://argo-cd.readthedocs.io/en/stable/user-guide/best_practices/)
- [Medium: Benchmark Deep vs Flat Directory Structure on ext4](https://medium.com/@hartator/benchmark-deep-directory-structure-vs-flat-directory-structure-to-store-millions-of-files-on-ext4-cac1000ca28)
- [Kopia: Sharding Documentation](https://kopia.io/docs/advanced/sharding/)
- [ext4 Wikipedia](https://en.wikipedia.org/wiki/Ext4)
- [Gregory Szorc: Global Kernel Locks in APFS](https://gregoryszorc.com/blog/2018/10/29/global-kernel-locks-in-apfs/)
- [LinuxVox: inotify Watches Performance](https://linuxvox.com/blog/what-is-a-reasonable-amount-of-inotify-watches-with-linux/)
