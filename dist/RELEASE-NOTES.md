# aibox v0.16.1 — sync auto-installs processkit, init picks the version

Patch release that fixes a v0.16.0 bug and adds two long-overdue
ergonomics for the `[processkit]` integration.

## Bug fix — `aibox sync` now installs processkit content

In v0.16.0, the only code path that installed processkit content was
`aibox init`. If a user ran `aibox init` while `[processkit].version`
was the default `"unset"` sentinel and then edited `aibox.toml` to
pin a real version, the next `aibox sync` would silently do nothing —
no fetch, no install, no error. The user would see an empty
`context/` directory and have no way to recover short of deleting
`aibox.toml` and re-running init.

`aibox sync` now auto-installs whenever:

- `[processkit].version != "unset"` AND
- there is no `aibox.lock` yet, OR the lock disagrees with `aibox.toml`
  on `(source, version)`

The install is **idempotent** when source and version match the lock:
the gating function `sync_should_install_processkit` short-circuits
and the existing three-way diff path takes over for drift detection.
You can re-run `aibox sync` as often as you like; the second run is
free.

The decision is a pure function (`container::sync_should_install_processkit`)
with five unit tests covering: unset sentinel, no-lock + pinned, lock
matches, lock version stale, lock source changed.

## Init now offers an interactive version picker

`aibox init` previously hard-defaulted `[processkit].version` to the
`"unset"` sentinel, leaving every new project with no content until
the user manually researched which processkit tag to pin. v0.16.1
queries the configured source at init time and shows a menu:

```
✔ Project name · my-project
✔ Work process · managed — small teams with a shared backlog
ℹ Querying available processkit versions at https://github.com/projectious-work/processkit.git...
? processkit version ›
❯ v0.5.1 (latest)
  v0.5.0
  v0.4.0
  v0.3.0
  v0.2.0
  v0.1.0
  unset — skip processkit install (configure later)
```

- The latest version is the default (top entry).
- An explicit `unset — skip processkit install (configure later)`
  escape hatch is always present at the bottom for users who want to
  defer pinning.
- In non-interactive mode (no TTY) the latest version is picked
  automatically.
- If listing fails (network error, no semver tags) the resolver falls
  back to `unset` with a warning. Existing v0.16.0 behavior preserved
  in the failure case.

## New CLI flags on `aibox init`

| Flag | What it does |
|---|---|
| `--processkit-source <url>` | Override the upstream URL. Lists versions from the override (or any compatible repo). Default: `https://github.com/projectious-work/processkit.git`. |
| `--processkit-version <tag>` | Pin a specific tag. Skips the interactive picker entirely. Useful for scripted setup. |
| `--processkit-branch <name>` | Track a moving branch. Wins over `--processkit-version` at fetch time per the existing fetcher contract; the version is still recorded in `aibox.toml` so the project can drop the branch later and have a sensible pin to fall back to. |

Example:

```bash
# Pin a specific version, no prompt
aibox init --name my-app --process managed --processkit-version v0.5.1

# Use a fork instead of upstream
aibox init --name my-app --processkit-source https://github.com/acme/processkit-acme.git

# Track a branch (discouraged, fine for testing pre-release work)
aibox init --name my-app --processkit-branch main
```

## New API: `content_source::list_versions`

Public function:

```rust
pub fn list_versions(source: &str) -> Result<Vec<String>>
```

Strategy:
- **GitHub-hosted sources** (host == `github.com`): GitHub Releases API
  (`https://api.github.com/repos/<org>/<name>/releases?per_page=100`).
- **Anything else** (GitLab, Gitea, self-hosted, file://, ssh://, scp-like):
  `git ls-remote --tags --refs <source>`.

Filters: only tags that parse as semver (with optional leading `v`)
are returned. Sorted descending by semver. Duplicates after the `v`
strip are deduplicated. Six unit tests cover the helpers without
network.

## Quality gates

- 438/438 tests pass (`cargo test`) — five new gating tests + six new
  version-list filter tests
- Zero clippy warnings (`cargo clippy --all-targets -- -D warnings`)
- `cargo audit` clean

## Migration impact

**Backwards compatible.** No config schema changes; no breaking API
changes. Existing v0.16.0 projects pick up the new behavior on the
next `aibox sync`.

If you were stuck on the v0.16.0 bug:

```bash
# Pin a real processkit version
sed -i 's/version  = "unset"/version  = "v0.5.1"/' aibox.toml
# Sync now installs the content
aibox sync
```

## Linked decisions

- **DEC-028** — `aibox sync` auto-installs processkit content; init
  offers an interactive version picker
- **DEC-027** — aibox v0.16.0: rip the bundled process layer
- **DEC-026** — Cache-tracked processkit reference
- **DEC-025** — Generic content-source release-asset fetcher
