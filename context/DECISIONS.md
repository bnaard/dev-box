# Decisions Log

Inverse chronological. Each decision has a rationale and alternatives considered.

## DEC-007 — Rename .root/ to .dev-box-home/ (2026-03-19)

**Decision:** Rename the host-side persisted config directory from `.root/` to `.dev-box-home/` for clarity. Backward compat: CLI falls back to `.root/` if it exists and `.dev-box-home/` doesn't.

**Rationale:** `.root/` is ambiguous — doesn't convey purpose. `.dev-box-home/` clearly indicates it's the container user's home directory content.

**Alternatives:** `.dev-box-config/`, `.config-mount/`, `.container-home/`

## DEC-006 — OWNER.md created locally, no symlink (2026-03-19)

**Decision:** OWNER.md is always created as a local file in `context/`, not symlinked from `~/.config/dev-box/`. The `owner` field was removed from `[context]` in dev-box.toml.

**Rationale:** The symlink pattern was confusing. Users can still share OWNER.md content across projects manually if they want.

## DEC-005 — No GitHub Actions (2026-03-16)

**Decision:** All builds and deploys are local. No GitHub Actions workflows.

**Rationale:** Cost avoidance. Local builds are fast enough for the project's scale.

## DEC-004 — Multi-stage TeX Live build (2026-03-16)

**Decision:** TeX Live installed via multi-stage Docker build from CTAN. Builder stage discarded (~2GB install).

**Rationale:** Keeps runtime image size manageable. CTAN install gives full control over package selection.

**Alternatives:** Debian texlive packages (too large, less control), TinyTeX (too minimal)

## DEC-003 — `uname -m` for arch detection (2026-03-16)

**Decision:** Use `uname -m` instead of `TARGETARCH` for architecture detection in Dockerfiles.

**Rationale:** Podman doesn't inject `TARGETARCH` build arg. `uname -m` works everywhere.

## DEC-002 — Alt-based zellij keybindings (2026-03-16)

**Decision:** All zellij keybindings use `Alt` modifier.

**Rationale:** Avoids conflicts with vim and TUI applications that use Ctrl.

## DEC-001 — Rust for CLI, not Python (2026-03-16)

**Decision:** dev-box CLI written in Rust, not Python.

**Rationale:** Single static binary, no runtime dependencies, fast cold start. Matches the "uv" inspiration.

**Alternatives:** Python (runtime dependency), Go (viable but less familiar to owner)
