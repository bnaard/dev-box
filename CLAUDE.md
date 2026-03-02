# CLAUDE.md — dev-box Hand-off Document

This document captures the key decisions, architecture, and implementation
details of the `dev-box` project for continuity in Claude Code sessions.

---

## Project Purpose

A minimal, reproducible, programming-focused dev-container based on
**Debian Trixie Slim**, managed via Podman (with Docker compatibility).
The container is used both as a **VS Code Dev Container** and as a
**standalone terminal environment** launched via a shell script.

---

## Repository Layout

```
<project-root>/               ← mounted as /workspace inside the container
├── .devcontainer/
│   ├── Dockerfile
│   ├── docker-compose.yml
│   ├── devcontainer.json
│   └── config/               ← source-of-truth config templates (tracked in git)
│       ├── vimrc
│       ├── gitconfig
│       └── zellij/
│           ├── config.kdl
│           ├── themes/
│           │   └── gruvbox.kdl
│           └── layouts/
│               └── dev.kdl
├── scripts/
│   └── dev.sh                ← container lifecycle manager
├── .root/                    ← host-side persisted config (NOT tracked in git)
│   ├── .ssh/                 → mounted as /root/.ssh (read-only)
│   ├── .vim/                 → mounted as /root/.vim
│   ├── .config/
│   │   ├── zellij/           → mounted as /root/.config/zellij
│   │   └── git/              → mounted as /root/.config/git
└── .gitignore                ← must exclude .root/
```

**Key principle:** The project root itself is `/workspace` inside the
container — there is no separate `workspace/` subdirectory.

---

## Installed Tools

| Tool | Version / Source | Purpose |
|------|-----------------|---------|
| `zellij` | 0.43.1 — prebuilt musl binary from GitHub releases | Terminal multiplexer |
| `vim` | Debian package (`vim` + `vim-runtime`) | Editor |
| `git` | Debian package | Version control |
| `lazygit` | Debian package | Git TUI |
| `curl` | Debian package | HTTP client |
| `jq` | Debian package | JSON processor |
| `tzdata` | Debian package | Timezone data |
| `ca-certificates` | Debian package | TLS root certificates |
| `locales` | Debian package | Locale support (en_US.UTF-8) |
| `less` | Debian package | Pager |
| `bash-completion` | Debian package | Shell completions |

---

## Dockerfile — Key Decisions

### Multi-stage build

- **Stage 1 (`builder`):** Downloads the official Zellij prebuilt binary
  from GitHub releases. Uses `uname -m` for arch detection (returns
  `aarch64` or `x86_64` directly, matching Zellij's release filename
  convention). This works reliably across Docker, Podman, and Buildah
  without relying on the `TARGETARCH` build arg (which Podman does not
  inject automatically).
- **Stage 2 (`runtime`):** Pure Debian Trixie Slim with apt packages.
  Only the Zellij binary is copied from the builder.

### Why prebuilt binary instead of compiling from source

Compiling Zellij from source (`cargo install --locked zellij`) was the
original approach but was abandoned because:
- The final linker step requires ~8+ GB RAM
- On Apple Silicon (aarch64) under Podman/Docker Desktop, the default VM
  memory cap caused SIGKILL (OOM) after ~870 seconds of successful compilation
- The official musl-static binary is self-contained (no glibc dependency),
  equally portable, and takes seconds to download

To upgrade Zellij: change `ARG ZELLIJ_VERSION=0.43.1` in the Dockerfile.

### Suppressing build warnings

Three categories of warnings were addressed:

**1. debconf frontend warnings** (`TERM is not set`, `Readline not found`, etc.)
- Cause: `docker build` has no TTY; debconf tries Dialog → Readline → Teletype
  and fails all three
- Fix: `ENV DEBIAN_FRONTEND=noninteractive` before all `apt-get` calls

**2. `perl: warning: Setting locale failed`**
- Cause: `LANG=en_US.UTF-8` and `LC_ALL=en_US.UTF-8` were set in `ENV`
  *before* `locale-gen` ran, so Perl invoked by dpkg found a non-existent locale
- Fix: Move locale `ENV` vars to a separate block *after* the `RUN` that
  calls `locale-gen`

**3. `update-alternatives: warning: skip creation of .../vim.1.gz`**
- Cause: `debian:trixie-slim` strips man pages, but vim's post-install
  script tries to register man page symlinks via `update-alternatives`
- Fix: Write explicit path exclusions to `/etc/dpkg/dpkg.cfg.d/excludes`
  *before* any `apt-get install`:
  ```dockerfile
  RUN printf 'path-exclude=/usr/share/man/*\n\
  path-exclude=/usr/share/groff/*\n\
  path-exclude=/usr/share/info/*\n\
  path-exclude=/usr/share/lintian/*\n' \
      > /etc/dpkg/dpkg.cfg.d/excludes
  ```

### CMD

```dockerfile
CMD ["sleep", "infinity"]
```

The container must stay alive and idle so VS Code (and `dev.sh`) can
exec into it. Zellij is never the container entrypoint — it is launched
by the terminal profile in `devcontainer.json` or directly by `dev.sh`.

---

## Configuration Persistence Strategy

All user configuration is persisted on the **host** under `.root/` and
bind-mounted into the container. The Dockerfile bakes identical defaults
into the image as a fallback — if no mounts are present, the container
still works out of the box.

On first run, `dev.sh` auto-seeds `.root/` from `config/` templates.

### Vim

- `vimrc` lives at `/root/.vim/vimrc` (not `~/.vimrc`)
- Vim 7.4+ checks `~/.vim/vimrc` automatically — no extra configuration needed
- The single mount `.root/.vim → /root/.vim` covers: config, undo history,
  netrw bookmarks, and any future plugins

### Git

- Config lives at `/root/.config/git/config` (XDG path, not `~/.gitconfig`)
- `GIT_CONFIG_GLOBAL=/root/.config/git/config` is set in `docker-compose.yml`
- Using a directory mount (rather than a single-file mount) allows a
  `credentials` file to coexist alongside `config`

### Zellij

- Config directory: `/root/.config/zellij/`
- Contains: `config.kdl`, `themes/gruvbox.kdl`, `layouts/dev.kdl`
- Zellij caches compiled plugins in this directory at runtime — the mount
  persists the cache across container recreations

---

## Zellij Configuration

### Theme
Gruvbox dark (`themes/gruvbox.kdl`).

### Key bindings
All primary bindings use `Alt` as modifier to avoid conflicts with vim
and other TUI apps running inside panes:

| Key | Action |
|-----|--------|
| `Alt s` | Open Strider file picker (floating) |
| `Alt m` | Open session manager (floating) |
| `Alt h/j/k/l` | Navigate panes |
| `Alt n/d/r` | New pane / split down / split right |
| `Alt x` | Close pane |
| `Alt f` | Toggle fullscreen |
| `Alt t / Alt w` | New tab / close tab |
| `Alt [/]` | Previous / next tab |
| `Alt 1-5` | Jump to tab N |
| `Alt u` | Enter scroll mode |
| `Alt /` | Search scrollback |
| `Ctrl q` | Quit Zellij |

### Default layout (`layouts/dev.kdl`)
Three tabs open on startup:
- **dev:** Strider sidebar (15%) | vim (80% height) | bottom row: bash terminal + claude code (20% height, split 50/50)
- **git:** Full-pane lazygit
- **shell:** Clean bash terminal

---

## Vim Configuration

Notable settings in `config/vimrc`:
- Space as leader key
- Relative + absolute line numbers
- 4-space indentation (2-space override for YAML, JSON, KDL, HTML, CSS, JS)
- `undofile` with undo directory at `/root/.vim/undo`
- No swap files (clean container environment)
- `colorcolumn=88` (Black/PEP8 default)
- Greps use ripgrep if available (`set grepprg=rg\ --vimgrep\ --smart-case`)
- Netrw in tree mode, no banner, 25% width
- `colorscheme desert` (ships with vim-runtime, no plugins needed)

---

## `dev.sh` — Script Architecture

**Location:** `scripts/dev.sh`  
**Called from:** project root (`./scripts/dev.sh <command>`)

### Path resolution
```bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
```
All paths derive from `PROJECT_ROOT` — the script works regardless of
the caller's working directory.

### Runtime detection
Detects Podman or Docker automatically:
```bash
if command -v podman &>/dev/null; then
  COMPOSE_BIN="podman compose"
  RUNTIME_BIN="podman"
elif command -v docker &>/dev/null; then
  COMPOSE_BIN="docker compose"
  RUNTIME_BIN="docker"
fi
```

### Container state detection
Uses the runtime's `inspect` directly (not `compose ps`) because
`podman compose ps` silently omits stopped containers in some versions:
```bash
state=$(${RUNTIME_BIN} inspect --format '{{.State.Status}}' "${CONTAINER_NAME}" 2>/dev/null || true)
```
The container name is read from `docker-compose.yml` at runtime via
`_init_names()` — it is never hardcoded in the script.

### Subcommands

| Command | Behaviour |
|---------|-----------|
| `build` | Builds the image; accepts `--no-cache` |
| `start` | Ensures dirs exist, seeds configs, starts container if needed, attaches via zellij |
| `stop` | Stops the container; data in `.root/` is preserved |
| `attach` | Execs into a running container and launches zellij |
| `status` | Reports running / stopped / missing |
| `help` | Prints usage |

### Readiness wait
After `up` or `start`, the script polls `container_status` every 500ms
for up to 7.5 seconds before attempting `exec`. This prevents a race
where `exec` runs before the container process has fully started.

### Config seeding
On first `start` or `build`, missing files in `.root/` are seeded from
`config/` templates. Existing files are never overwritten.

---

## VS Code Dev Container Integration

**`devcontainer.json`** sits alongside `docker-compose.yml` in `.devcontainer/`.

Critical settings:
```json
{
  "overrideCommand": true,
  "customizations": {
    "vscode": {
      "settings": {
        "terminal.integrated.defaultProfile.linux": "zellij",
        "terminal.integrated.profiles.linux": {
          "zellij": {
            "path": "/usr/local/bin/zellij",
            "args": ["--layout", "dev"]
          },
          "bash": { "path": "/bin/bash" }
        }
      }
    }
  }
}
```

`overrideCommand: true` suppresses the container's `CMD` during VS Code
attach. Every new VS Code terminal opens zellij automatically; plain bash
is available via the terminal profile dropdown.

---

## Known Issues and Gotchas

**Podman vs Docker Compose output format**
`podman compose ps` format varies by version and may omit stopped containers.
Always use `podman inspect` / `docker inspect` for state checks in scripts.

**Stale image cache**
If the container exits immediately after `up`, the cached image likely has
the old `CMD ["zellij", "--layout", "dev"]`. Rebuild with:
```bash
./scripts/dev.sh build --no-cache
```

**`.root/` must be in `.gitignore`**
The `.root/` directory contains SSH keys and personal config — it must
never be committed. Ensure `.gitignore` contains `.root/`.

**SSH keys not found warning**
`dev.sh start` warns if `.root/.ssh/` is empty. This is non-fatal.
Copy your keys manually:
```bash
cp ~/.ssh/id_ed25519{,.pub} .root/.ssh/
chmod 700 .root/.ssh && chmod 600 .root/.ssh/id_ed25519
```

**Zellij version pinning**
Zellij is pinned to `0.43.1`. To upgrade, change `ARG ZELLIJ_VERSION`
in `.devcontainer/Dockerfile` and run `./scripts/dev.sh build --no-cache`.

---

## Open Topics / Future Work

- Language-specific layers (Python, Rust, Node, etc.) should be added as
  separate Dockerfiles or compose overrides on top of this base image,
  not baked in here
- LSP integration (e.g. `vim-lsp` or `coc.nvim`) is intentionally absent
  from the base image for the same reason
- The tpope vim plugin suite (`vim-commentary`, `vim-surround`,
  `vim-sleuth`, `vim-fugitive`) was discussed as a useful addition;
  they can be cloned into `.root/.vim/pack/plugins/start/` on the host
  without modifying the image
- Consider adding `universal-ctags`, `ripgrep`, `fzf`, and `fd-find`
  to the apt install list for a richer vim experience
