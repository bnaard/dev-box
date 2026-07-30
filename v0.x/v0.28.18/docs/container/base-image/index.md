# Base Image

LLMS index: [llms.txt](/aibox/v0.x/v0.28.18/llms.txt)

---

# Base Image

The base image is the foundation for all aibox container flavors. It provides a complete, opinionated development environment built on **Debian Trixie Slim**.

## Installed Tools

| Tool | Version / Source | Purpose |
|------|-----------------|---------|
| tmux | Debian package | Terminal multiplexer |
| Yazi | 25.4.8 (prebuilt binary from GitHub releases) | Terminal file manager |
| Vim | Debian package (`vim` + `vim-runtime`) | Editor |
| Git | Debian package | Version control |
| Claude CLI | Official install script | AI assistant |
| ripgrep (`rg`) | Debian package | Fast recursive search (grep replacement) |
| fd | Debian package | Fast file finder (find replacement) |
| bat | Debian package | Syntax-highlighting cat replacement |
| eza | Debian package | Modern ls replacement with git integration |
| zoxide | Debian package | Smarter cd that learns your habits |
| fzf | Debian package | Fuzzy finder for files, history, and more |
| delta | Debian package | Syntax-highlighting diff viewer (used by git) |
| starship | Prebuilt binary | Minimal, fast shell prompt with context |
| curl | Debian package | HTTP client |
| jq | Debian package | JSON processor |
| less | Debian package | Pager |
| unzip | Debian package | Archive extraction |
| iproute2 | Debian package | Network route/interface inspection for status segments |
| iputils-ping | Debian package | ICMP latency checks for status segments |
| bash-completion | Debian package | Shell completions |
| ca-certificates | Debian package | TLS root certificates |
| locales | Debian package | Locale support (en_US.UTF-8) |
| tzdata | Debian package | Timezone data |

GitHub CLI (`gh`) and lazygit are provided by the optional
[`git-ui` addon](../addons/tool-bundles.md#git-ui), not by the base image. Add
`[addons.git-ui.tools]` to `aibox.toml` when a project needs those tools.

Audio tools are provided by the internal `audio-voice` recipe, which is selected
automatically when `[audio] enabled = true` and `install = true`. File-preview and archive helpers
such as `chafa`, `timg`, `poppler-utils`, `mupdf-tools`, `entr`, and
`resvg` are provided by the optional `preview-archive` addon.

## Build Architecture

The Dockerfile keeps the runtime stage on Debian Trixie Slim and installs tmux
from the distro package set. That keeps the terminal multiplexer on the same
security update path as the rest of the base image and avoids a separate
prebuilt-binary fetch stage.

## tmux Configuration

### Plugin Policy

aibox-managed tmux plugins are preinstalled and pinned by the generated runtime
and image build. TPM is documented only as a user convenience layer for adding
personal tmux plugins after initialization; aibox does not rely on TPM to
install or update managed plugins.

`tmux-resurrect` and `tmux-continuum` are installed and available in the image,
but disabled by default until the workspace persistence policy is decided. Users
can opt into them in local tmux config, but generated layouts should not assume
session resurrection is active.

### Key Bindings

All bindings use `Ctrl+g` as a leader key — press `Ctrl+g`, release, then press the action key. This avoids conflicts with macOS Option key (which produces special characters like `@`, `€`, `|`) and with Vim/bash Ctrl bindings.

| Key | Action |
|-----|--------|
| `Ctrl+g` then `h/j/k/l` | Navigate panes (vim-style) |
| `Ctrl+g` then `n` | New pane |
| `Ctrl+g` then `d` | Split down |
| `Ctrl+g` then `r` | Split right |
| `Ctrl+g` then `x` | Close focused pane |
| `Ctrl+g` then `f` | Toggle fullscreen |
| `Ctrl+g` then `z` | Toggle pane frames |
| `Ctrl+g` then `e` | Toggle embed/floating |
| `Ctrl+g` then `=` / `-` | Resize pane (increase / decrease) |
| `Ctrl+g` then `t` | New window |
| `Ctrl+g` then `w` | Close window |
| `Ctrl+g` then `[` / `]` | Previous / next window |
| `Ctrl+g` then `1-5` | Jump to window N |
| `Ctrl+g` then `i` / `o` | Move window left / right |
| `Ctrl+g` then `s` | Session chooser |
| `Ctrl+g` then `m` | Session manager |
| `Ctrl+g` then `u` | Enter scroll mode |
| `Ctrl+g` then `/` | Search scrollback |
| `Ctrl+q` | Quit tmux |

Press `Escape` or `Ctrl+g` again to cancel the leader and return to normal mode.

### Layouts

aibox ships four tmux layouts. Select one with `aibox up --layout <name>` (the default is `dev`). Layouts include harness windows based on the enabled entries in `[ai].harnesses` and `[ai].harness_order`; they include the **lazygit** window only when the `git-ui` addon selects `lazygit`.

#### dev (default)

<div class="asciinema" data-cast="/aibox/screencasts/layout-dev.cast" data-poster="npt:4" data-autoplay="false" data-controls="false" data-fit="width"></div>

Window **work** has Yazi and the 1st harness stacked on the left, with shell on the right. Further harnesses use the **ai** window; lazygit and shell get their own windows.

#### focus -- one tool per window, fullscreen

<div class="asciinema" data-cast="/aibox/screencasts/layout-focus.cast" data-poster="npt:4" data-autoplay="false" data-controls="false" data-fit="width"></div>

Each tool gets the entire screen in its own window. Switch with `Ctrl+g [/]` or `Ctrl+g 1-5`.

Windows: **files** (yazi) | one window per harness | optional **lazygit** | **shell**

#### cowork -- side-by-side coding with AI

<div class="asciinema" data-cast="/aibox/screencasts/layout-cowork.cast" data-poster="npt:4" data-autoplay="false" data-controls="false" data-fit="width"></div>

Window **work** has Yazi on the left and shell on the right. The **ai** window contains all harnesses split evenly across full-height panes; the lazygit window is generated when enabled.

### Opening Files from Yazi

- **`Enter`** -- opens file in vim in-place (suspends Yazi, `:q` returns to Yazi). Works in all layouts.
- **`e`** -- opens file in a full-screen vim popup and returns to Yazi when vim exits.

<div class="alert alert-secondary" role="alert"><div class="h4 alert-heading" role="heading">tmux vs Yazi</div>



`Ctrl+g` then `s` opens the tmux session chooser. The sidebar file manager in
all layouts is **Yazi**, an external terminal file manager with richer features
(preview, bulk operations, async I/O).

</div>


### Theme

Gruvbox dark, defined in `themes/gruvbox.conf`.

## Vim Configuration

Notable settings baked into the image:

- **Leader key:** Space
- **Line numbers:** Relative + absolute (hybrid)
- **Indentation:** 4 spaces default, 2 spaces for YAML, JSON, KDL, HTML, CSS, JavaScript
- **Undo:** Persistent undo files stored in `/home/aibox/.vim/undo`
- **No swap files** -- clean container environment
- **Color column** at 88 (Black/PEP8 default)
- **Grep program:** ripgrep if available (`rg --vimgrep --smart-case`)
- **Netrw:** Tree mode, no banner, 25% width
- **Colorscheme:** `desert` (ships with vim-runtime, no plugins needed)

## Git Configuration

Git config lives at `/home/aibox/.config/git/config` (XDG path, not `~/.gitconfig`). The environment variable `GIT_CONFIG_GLOBAL` is set in the generated `docker-compose.yml` to point to this location.

Using a directory mount (rather than a single-file mount) allows a `credentials` file to coexist alongside `config`.

## AI Coding Agents

AI coding agents (Claude, Codex, Aider, Gemini, and others) are **not**
pre-installed in the base image. They are installed per-project when you select
them in the ordered `[ai].harnesses` list, for example
`{ harness = "claude", enable = true, install = true }`.

## Audio Support

Set `[audio] enabled = true` to enable audio bridging. aibox then selects the
internal `audio-voice` recipe and configures the PulseAudio environment for the
container. See [Audio Support](audio.md) for setup details.

## Configuration Persistence

All user configuration is persisted on the host under `.aibox-home/` and bind-mounted into the container:

| Host Path | Container Path | Contents |
|-----------|---------------|----------|
| `.aibox-home/.ssh/` | `/home/aibox/.ssh` (read-only) | SSH keys |
| `.aibox-home/.vim/` | `/home/aibox/.vim` | Vim config and undo history |
| `.aibox-home/.config/` | `/home/aibox/.config` | Git, tmux, Yazi, prompt, and tool config |
| `.aibox-home/.cache/` | `/home/aibox/.cache` | Runtime caches |
| `.aibox-home/.local/` | `/home/aibox/.local` | Helper scripts, state, and local data |
| `.aibox-home/.tmux/` | `/home/aibox/.tmux` | tmux plugin and socket state |

The Dockerfile bakes identical defaults into the image as a fallback. If no mounts are present, the container still works out of the box.

On first `aibox init` or `aibox up`, the `.aibox-home/` directory is auto-seeded
from built-in templates. User-owned files are retained. Files explicitly
managed by aibox may be refreshed on `aibox apply` when their upstream template
changes.

Tool credentials saved below `/home/aibox/.config`, including a GitHub CLI
login stored in `/home/aibox/.config/gh`, therefore survive container
replacement and image rebuilds. They remain local secret-bearing files rather
than encrypted storage. See [GitHub authentication](../reference/local-config.md#github-authentication)
for the security tradeoffs and the recommended scoped-PAT alternative.

## File Preview

The base image ships the Yazi configuration and preview plugins. Install the
optional `preview-archive` addon for raster/SVG/PDF/archive helper binaries and
`preview-enhanced` for Markdown, EPS, video, and Ghostscript
support. PDF and SVG also support **watch-mode preview** when the required
preview tools are selected.

See the dedicated [File Preview](file-preview.md) page for full documentation, including format coverage, standalone tools (`chafa`, `timg`), and the PDF/SVG watch-mode patterns.

## Container Entrypoint

```dockerfile
CMD ["sleep", "infinity"]
```

The container stays alive and idle. Both VS Code and `aibox up` exec into it. tmux is never the container entrypoint -- it is launched on attach.
