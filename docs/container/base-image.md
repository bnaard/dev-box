# Base Image

The base image is the foundation for all dev-box container flavors. It provides a complete, opinionated development environment built on **Debian Trixie Slim**.

## Installed Tools

| Tool | Version / Source | Purpose |
|------|-----------------|---------|
| Zellij | 0.43.1 (prebuilt binary from GitHub releases) | Terminal multiplexer |
| Yazi | 25.4.8 (prebuilt binary from GitHub releases) | Terminal file manager |
| Vim | Debian package (`vim` + `vim-runtime`) | Editor |
| Git | Debian package | Version control |
| lazygit | Debian package | Git TUI |
| GitHub CLI (`gh`) | Debian package | GitHub integration |
| Claude CLI | Official install script | AI assistant |
| curl | Debian package | HTTP client |
| jq | Debian package | JSON processor |
| less | Debian package | Pager |
| unzip | Debian package | Archive extraction |
| bash-completion | Debian package | Shell completions |
| sox | Debian package | Audio processing |
| pulseaudio-utils | Debian package | Audio bridging |
| ca-certificates | Debian package | TLS root certificates |
| locales | Debian package | Locale support (en_US.UTF-8) |
| tzdata | Debian package | Timezone data |

## Build Architecture

The Dockerfile uses a multi-stage build:

- **Stage 1 (builder):** Downloads the official Zellij prebuilt binary from GitHub releases. Architecture detection uses `uname -m`, which returns `aarch64` or `x86_64` directly -- matching the Zellij release filename convention. This works reliably across Docker, Podman, and Buildah.

- **Stage 2 (runtime):** Pure Debian Trixie Slim with apt packages. Only the Zellij binary is copied from the builder stage.

!!! note "Why prebuilt instead of compiled"
    Compiling Zellij from source requires 8+ GB RAM during the final linker step. On Apple Silicon under Podman/Docker Desktop, the default VM memory cap causes OOM kills. The official musl-static binary is equally portable and downloads in seconds.

## Zellij Configuration

### Key Bindings

All primary bindings use `Alt` as modifier to avoid conflicts with Vim and other TUI applications:

| Key | Action |
|-----|--------|
| `Alt s` | Open Strider file picker (floating pane) |
| `Alt m` | Open session manager (floating) |
| `Alt h/j/k/l` | Navigate panes (vim-style) |
| `Alt n` | New pane |
| `Alt d` | Split down |
| `Alt r` | Split right |
| `Alt x` | Close focused pane |
| `Alt f` | Toggle fullscreen |
| `Alt z` | Toggle pane frames |
| `Alt e` | Toggle embed/floating |
| `Alt t` | New tab |
| `Alt w` | Close tab |
| `Alt [` / `Alt ]` | Previous / next tab |
| `Alt 1-5` | Jump to tab N |
| `Alt =` / `Alt -` | Resize pane (increase / decrease) |
| `Alt i` / `Alt o` | Move tab left / right |
| `Alt u` | Enter scroll mode |
| `Alt /` | Search scrollback |
| `Ctrl q` | Quit Zellij |

### Layouts

dev-box ships three IDE layouts. Select one when attaching via `zellij --layout <name>` (the default is `dev`). All layouts include shared tabs for **git** (lazygit), **shell** (extra bash), and **help** (cheatsheet).

#### dev (default, VS Code-like)

```
┌──────────┬───────────────────────────────┐
│          │         vim editor            │
│  yazi    │                               │
│  (20%)   ├───────────────────────────────┤
│          │  ┊bash┊claude┊  stacked       │
├──────────┴───────────────────────────────┤
│  status-bar                              │
└──────────────────────────────────────────┘
```

Yazi file manager on the left, Vim on the top right, bash and Claude Code stacked below.

#### focus (minimal distraction)

```
┌──────────┬───────────────────────────────┐
│          │                               │
│  yazi    │  ┊bash┊claude┊vim┊            │
│  (20%)   │  stacked (one visible)        │
│          │                               │
├──────────┴───────────────────────────────┤
│  status-bar                              │
└──────────────────────────────────────────┘
```

Yazi on the left, three tools stacked on the right (bash, Claude Code, Vim) with only one visible at a time.

#### assist (Claude-focused)

```
┌──────────┬──────────────┬────────────────┐
│          │              │                │
│  yazi    │  ┊bash┊      │    vim         │
│  (20%)   │  ┊claude┊    │               │
│          │  stacked     │                │
├──────────┴──────────────┴────────────────┤
│  status-bar                              │
└──────────────────────────────────────────┘
```

Yazi on the left, stacked bash/Claude Code in the center, Vim on the right. Emphasizes the Claude interface for AI-assisted workflows.

!!! note "Strider vs Yazi"
    `Alt s` opens the built-in **Strider** file picker as a floating overlay (Zellij plugin). The sidebar file manager in all layouts is **Yazi**, an external terminal file manager with richer features (preview, bulk operations, async I/O).

### Theme

Gruvbox dark, defined in `themes/gruvbox.kdl`.

## Vim Configuration

Notable settings baked into the image:

- **Leader key:** Space
- **Line numbers:** Relative + absolute (hybrid)
- **Indentation:** 4 spaces default, 2 spaces for YAML, JSON, KDL, HTML, CSS, JavaScript
- **Undo:** Persistent undo files stored in `/root/.vim/undo`
- **No swap files** -- clean container environment
- **Color column** at 88 (Black/PEP8 default)
- **Grep program:** ripgrep if available (`rg --vimgrep --smart-case`)
- **Netrw:** Tree mode, no banner, 25% width
- **Colorscheme:** `desert` (ships with vim-runtime, no plugins needed)

## Git Configuration

Git config lives at `/root/.config/git/config` (XDG path, not `~/.gitconfig`). The environment variable `GIT_CONFIG_GLOBAL` is set in the generated `docker-compose.yml` to point to this location.

Using a directory mount (rather than a single-file mount) allows a `credentials` file to coexist alongside `config`.

## Claude CLI

The Claude CLI is installed via the official install script during image build. It is available at `/root/.local/bin/claude` and added to `PATH`.

## Audio Support

The base image includes `sox` and `pulseaudio-utils` for audio bridging, enabling Claude Code's voice features inside the container. See [Audio Support](audio.md) for setup details.

## Configuration Persistence

All user configuration is persisted on the host under `.dev-box-home/` and bind-mounted into the container:

| Host Path | Container Path | Contents |
|-----------|---------------|----------|
| `.dev-box-home/.ssh/` | `/root/.ssh` (read-only) | SSH keys |
| `.dev-box-home/.vim/` | `/root/.vim` | Vim config, undo history, plugins |
| `.dev-box-home/.config/git/` | `/root/.config/git` | Git config and credentials |
| `.dev-box-home/.config/zellij/` | `/root/.config/zellij` | Zellij config, themes, layouts, plugin cache |
| `.dev-box-home/.config/yazi/` | `/root/.config/yazi` | Yazi file manager config and keymap |

The Dockerfile bakes identical defaults into the image as a fallback. If no mounts are present, the container still works out of the box.

On first `dev-box init` or `dev-box start`, the `.dev-box-home/` directory is auto-seeded from built-in templates. Existing files are never overwritten.

## Container Entrypoint

```dockerfile
CMD ["sleep", "infinity"]
```

The container stays alive and idle. Both VS Code and `dev-box start` exec into it. Zellij is never the container entrypoint -- it is launched on attach.
