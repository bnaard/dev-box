# syntax=docker/dockerfile:1
# =============================================================================
# Builder stages — one per tool for parallel BuildKit downloads and granular
# cache invalidation. Bumping one version no longer re-downloads all others.
# =============================================================================
FROM debian:trixie-slim AS fetch-base

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates unzip \
    && rm -rf /var/lib/apt/lists/*

# ── tmux plugins ─────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-tmux-plugins
ARG TMUX_POWERKIT_REF=139be6bbd57dbedfc6c534e72a440147ad0ab4d4
ARG TMUX_SENSIBLE_REF=25cb91f42d020f675bb0a2ce3fbd3a5d96119efa
ARG VIM_TMUX_NAVIGATOR_REF=e41c431a0c7b7388ae7ba341f01a0d217eb3a432
ARG TPM_REF=99469c4a9b1ccf77fade25842dc7bafbc8ce9946
ARG TMUX_CONTINUUM_REF=0698e8f4b17d6454c71bf5212895ec055c578da0
ARG TMUX_YANK_REF=acfd36e4fcba99f8310a7dfb432111c242fe7392
ARG TMUX_RESURRECT_REF=cff343cf9e81983d3da0c8562b01616f12e8d548
RUN set -eux; \
    install_plugin() { \
      repo="$1"; ref="$2"; dest="$3"; \
      mkdir -p "/tmux-plugins/${dest}"; \
      curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors "https://github.com/${repo}/archive/${ref}.tar.gz" \
        | tar -xz --strip-components=1 -C "/tmux-plugins/${dest}"; \
    }; \
    install_plugin fabioluciano/tmux-powerkit "${TMUX_POWERKIT_REF}" tmux-powerkit; \
    install_plugin tmux-plugins/tmux-sensible "${TMUX_SENSIBLE_REF}" tmux-sensible; \
    install_plugin christoomey/vim-tmux-navigator "${VIM_TMUX_NAVIGATOR_REF}" vim-tmux-navigator; \
    install_plugin tmux-plugins/tpm "${TPM_REF}" tpm; \
    install_plugin tmux-plugins/tmux-continuum "${TMUX_CONTINUUM_REF}" tmux-continuum; \
    install_plugin tmux-plugins/tmux-yank "${TMUX_YANK_REF}" tmux-yank; \
    install_plugin tmux-plugins/tmux-resurrect "${TMUX_RESURRECT_REF}" tmux-resurrect

# ── aibox runtime status helpers ─────────────────────────────────────────────
FROM debian:trixie-slim AS build-aibox-runtime-tools

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl build-essential pkg-config \
    && rm -rf /var/lib/apt/lists/*

ENV CARGO_HOME=/cargo RUSTUP_HOME=/rustup PATH="/cargo/bin:${PATH}"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --profile minimal --default-toolchain 1.94

COPY config/bin/aibox_status_core.rs config/bin/aibox-status.rs config/bin/aibox-diagnostics.rs /build/aibox-runtime-tools/
RUN mkdir -p /build/aibox-runtime-tools/bin && \
    rustc --edition=2021 -D warnings -C opt-level=2 \
      /build/aibox-runtime-tools/aibox-status.rs \
      -o /build/aibox-runtime-tools/bin/aibox-status && \
    rustc --edition=2021 -D warnings -C opt-level=2 \
      /build/aibox-runtime-tools/aibox-diagnostics.rs \
      -o /build/aibox-runtime-tools/bin/aibox-diagnostics

# ── Yazi ─────────────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-yazi
ARG YAZI_VERSION=26.5.6
RUN ARCH=$(uname -m) && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors \
      "https://github.com/sxyazi/yazi/releases/download/v${YAZI_VERSION}/yazi-${ARCH}-unknown-linux-musl.zip" \
      -o /tmp/yazi.zip && \
    unzip -q /tmp/yazi.zip -d /tmp/yazi && \
    mv /tmp/yazi/yazi-${ARCH}-unknown-linux-musl/yazi /usr/local/bin/yazi && \
    ln -sf /usr/local/bin/yazi /usr/local/bin/ya && \
    chmod +x /usr/local/bin/yazi && \
    rm -rf /tmp/yazi /tmp/yazi.zip

# ── ripgrep (no aarch64 musl build — use gnu) ────────────────────────────────
FROM fetch-base AS fetch-ripgrep
ARG RIPGREP_VERSION=15.1.0
RUN ARCH=$(uname -m) && \
    RG_LIBC=$(case $ARCH in x86_64) echo musl;; *) echo gnu;; esac) && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors \
      "https://github.com/BurntSushi/ripgrep/releases/download/${RIPGREP_VERSION}/ripgrep-${RIPGREP_VERSION}-${ARCH}-unknown-linux-${RG_LIBC}.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "ripgrep-${RIPGREP_VERSION}-${ARCH}-unknown-linux-${RG_LIBC}/rg" && \
    chmod +x /usr/local/bin/rg

# ── fd ───────────────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-fd
ARG FD_VERSION=10.4.2
RUN ARCH=$(uname -m) && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors \
      "https://github.com/sharkdp/fd/releases/download/v${FD_VERSION}/fd-v${FD_VERSION}-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "fd-v${FD_VERSION}-${ARCH}-unknown-linux-musl/fd" && \
    chmod +x /usr/local/bin/fd

# ── bat ──────────────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-bat
ARG BAT_VERSION=0.26.1
RUN ARCH=$(uname -m) && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors \
      "https://github.com/sharkdp/bat/releases/download/v${BAT_VERSION}/bat-v${BAT_VERSION}-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "bat-v${BAT_VERSION}-${ARCH}-unknown-linux-musl/bat" && \
    chmod +x /usr/local/bin/bat

# ── eza (no aarch64 musl build — use gnu) ────────────────────────────────────
FROM fetch-base AS fetch-eza
ARG EZA_VERSION=0.23.4
RUN ARCH=$(uname -m) && \
    EZA_SUFFIX=$(case $ARCH in x86_64) echo "unknown-linux-musl";; aarch64) echo "unknown-linux-gnu";; esac) && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors \
      "https://github.com/eza-community/eza/releases/download/v${EZA_VERSION}/eza_${ARCH}-${EZA_SUFFIX}.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/eza

# ── zoxide ───────────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-zoxide
ARG ZOXIDE_VERSION=0.9.9
RUN ARCH=$(uname -m) && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors \
      "https://github.com/ajeetdsouza/zoxide/releases/download/v${ZOXIDE_VERSION}/zoxide-${ZOXIDE_VERSION}-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin zoxide && \
    chmod +x /usr/local/bin/zoxide

# ── fzf (uses amd64/arm64 instead of x86_64/aarch64) ────────────────────────
FROM fetch-base AS fetch-fzf
ARG FZF_VERSION=0.72.0
RUN ARCH=$(uname -m) && \
    FZF_ARCH=$(case $ARCH in aarch64) echo arm64;; x86_64) echo amd64;; esac) && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors \
      "https://github.com/junegunn/fzf/releases/download/v${FZF_VERSION}/fzf-${FZF_VERSION}-linux_${FZF_ARCH}.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/fzf

# ── delta (no aarch64 musl build — use gnu) ──────────────────────────────────
FROM fetch-base AS fetch-delta
ARG DELTA_VERSION=0.19.2
RUN ARCH=$(uname -m) && \
    DELTA_LIBC=$(case $ARCH in x86_64) echo musl;; *) echo gnu;; esac) && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors \
      "https://github.com/dandavison/delta/releases/download/${DELTA_VERSION}/delta-${DELTA_VERSION}-${ARCH}-unknown-linux-${DELTA_LIBC}.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "delta-${DELTA_VERSION}-${ARCH}-unknown-linux-${DELTA_LIBC}/delta" && \
    chmod +x /usr/local/bin/delta

# ── starship ─────────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-starship
ARG STARSHIP_VERSION=1.25.1
RUN ARCH=$(uname -m) && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors \
      "https://github.com/starship/starship/releases/download/v${STARSHIP_VERSION}/starship-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/starship

# ── Vim colorschemes ─────────────────────────────────────────────────────────
# Separate stage keeps network fetches out of the runtime build graph.
# Uses curl for dracula instead of git clone (~4 MB savings).
FROM fetch-base AS fetch-vim-colors
RUN mkdir -p /vim-colors/colors \
      /vim-colors/pack/themes/start/dracula/colors \
      /vim-colors/pack/themes/start/dracula/autoload && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors -o /vim-colors/colors/gruvbox.vim \
      https://raw.githubusercontent.com/morhetz/gruvbox/master/colors/gruvbox.vim && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors -o /vim-colors/colors/catppuccin_mocha.vim \
      https://raw.githubusercontent.com/catppuccin/vim/main/colors/catppuccin_mocha.vim && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors -o /vim-colors/colors/catppuccin_latte.vim \
      https://raw.githubusercontent.com/catppuccin/vim/main/colors/catppuccin_latte.vim && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors -o /vim-colors/colors/tokyonight.vim \
      https://raw.githubusercontent.com/ghifarit53/tokyonight-vim/master/colors/tokyonight.vim && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors -o /vim-colors/colors/nord.vim \
      https://raw.githubusercontent.com/nordtheme/vim/main/colors/nord.vim && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors -o /vim-colors/pack/themes/start/dracula/colors/dracula.vim \
      https://raw.githubusercontent.com/dracula/vim/master/colors/dracula.vim && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors -o /vim-colors/pack/themes/start/dracula/autoload/dracula.vim \
      https://raw.githubusercontent.com/dracula/vim/master/autoload/dracula.vim

# =============================================================================
# Runtime — minimal dev-container base image
# =============================================================================
# Estimated layer sizes (arm64, approximate):
#   debian:trixie-slim base .............. ~75 MB
#   Core apt (curl, git, vim, etc.) ...... ~105 MB
#   Static binaries (10 tools) ........... ~85 MB
#   Vim colorschemes ..................... ~0.1 MB
#   Config files ......................... ~0.1 MB
#   ─────────────────────────────────────────────
#   Total estimate ....................... ~365 MB
#
# Preview/archive and audio helpers are opt-in addons; keep the base focused on
# core aibox/processkit operation plus the interactive shell/file-manager core.
# =============================================================================
FROM debian:trixie-slim AS runtime

# Suppress debconf warnings during build — no TTY available in docker build
ENV DEBIAN_FRONTEND=noninteractive

# Set timezone early so tzdata installs silently
ENV TZ=Europe/Berlin

# Explicitly declare path exclusions before any package is installed.
# debian:trixie-slim already strips man pages, but making it explicit here
# silences the update-alternatives warnings about missing man page symlinks
# that would otherwise appear during vim installation.
RUN printf 'path-exclude=/usr/share/man/*\n\
path-exclude=/usr/share/groff/*\n\
path-exclude=/usr/share/info/*\n\
path-exclude=/usr/share/lintian/*\n' \
    > /etc/dpkg/dpkg.cfg.d/excludes

# ── System packages ──────────────────────────────────────────────────────────
# Kept as a single RUN to share one apt-get update and minimize layer overhead.
# Groups are documented for future reference if slim image variants are needed.
RUN apt-get update && apt-get install -y --no-install-recommends \
    # ── Core utilities (~35 MB) ──
    curl \
    ca-certificates \
    tzdata \
    locales \
    jq \
    sudo \
    gosu \
    procps \
    iproute2 \
    iputils-ping \
    tmux \
    xclip \
    xsel \
    wl-clipboard \
    # ── Sandboxing support (~1 MB) ──
    # OpenAI Codex CLI uses bubblewrap on Linux. Ship the Debian package so
    # Codex does not have to fall back to its vendored binary.
    bubblewrap \
    # ── Editor (~25 MB) ──
    vim \
    # ── Version control (~40 MB) ──
    git \
    # ── Shell quality-of-life (~5 MB) ──
    less \
    unzip \
    file \
    bash-completion \
    # ── Log viewer (~3 MB) ──
    # lnav for optional structured inspection in the `Prefix L` tmux popup.
    # The aibox-log-viewer helper defaults to a jq-formatted, mouse-enabled
    # less view and can use lnav via AIBOX_LOG_VIEWER=lnav.
    lnav \
    # Preview/archive helpers live in the preview-archive addon.
    # Audio helpers live in the audio-voice addon.
    # ── Python (~50 MB) ──
    # Debian Trixie ships python3 = 3.13.x as the default. python3
    # and python3-venv are unconditionally installed in the base image
    # since v0.16.5 (DEC-036) so processkit's PEP 723 MCP server scripts
    # work via `uv run` regardless of whether the python addon is
    # enabled. The python addon (addons/languages/python.yaml) now
    # provides ADDITIONAL Python tooling on top of this minimum.
    python3 \
    python3-venv \
    && sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen \
    && locale-gen \
    && ln -snf /usr/share/zoneinfo/${TZ} /etc/localtime \
    && echo ${TZ} > /etc/timezone \
    && rm -rf /var/lib/apt/lists/*

# Set locale vars only after locale-gen has run, so perl/dpkg never see
# an LC_ALL pointing to a non-existent locale during the build
ENV LANG=en_US.UTF-8 \
    LANGUAGE=en_US:en \
    LC_ALL=en_US.UTF-8 \
    TERM=xterm-256color \
    COLORTERM=truecolor \
    SHELL=/bin/bash

# ── Non-root user ───────────────────────────────────────────────────────────
RUN groupadd --gid 1000 aibox \
    && useradd --uid 1000 --gid aibox --shell /bin/bash --create-home aibox \
    && echo "aibox ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/aibox \
    && chmod 0440 /etc/sudoers.d/aibox

# ── Copy binaries from parallel builder stages ──────────────────────────────
COPY --from=fetch-yazi     /usr/local/bin/yazi     /usr/local/bin/
RUN ln -sf /usr/local/bin/yazi /usr/local/bin/ya
COPY --from=fetch-ripgrep  /usr/local/bin/rg       /usr/local/bin/
COPY --from=fetch-fd       /usr/local/bin/fd       /usr/local/bin/
COPY --from=fetch-bat      /usr/local/bin/bat      /usr/local/bin/
COPY --from=fetch-eza      /usr/local/bin/eza      /usr/local/bin/
COPY --from=fetch-zoxide   /usr/local/bin/zoxide   /usr/local/bin/
COPY --from=fetch-fzf      /usr/local/bin/fzf      /usr/local/bin/
COPY --from=fetch-delta    /usr/local/bin/delta    /usr/local/bin/
COPY --from=fetch-starship /usr/local/bin/starship /usr/local/bin/
COPY --from=build-aibox-runtime-tools /build/aibox-runtime-tools/bin/aibox-status /usr/local/bin/
COPY --from=build-aibox-runtime-tools /build/aibox-runtime-tools/bin/aibox-diagnostics /usr/local/bin/
# ── uv (Python package/project manager — ~25 MB) ────────────────────────────
# Distroless image ships only the static binaries. Pulled into the base
# image since v0.16.5 (DEC-036) so processkit's PEP 723 MCP server
# scripts run via `uv run` out of the box. Works against the
# externally-installed python3 from apt above.
COPY --from=ghcr.io/astral-sh/uv:0.11.11 /uv /uvx /usr/local/bin/

# ── Bake default configs into the image ───────────────────────────────────────
# These act as fallback defaults when no host directory is mounted.
# When docker-compose mounts .aibox-home/.vim / .aibox-home/.config/tmux etc.
# over these paths, the host copy takes full precedence.

# Vim — colorschemes from builder stage, vimrc from build context.
# Copy themes into both the image home and Vim's system runtimepath so a
# host-mounted ~/.vim can never hide the configured colorscheme.
RUN mkdir -p /home/aibox/.vim/undo
COPY --from=fetch-vim-colors /vim-colors/colors/                                 /home/aibox/.vim/colors/
COPY --from=fetch-vim-colors /vim-colors/pack/themes/start/dracula/colors/       /home/aibox/.vim/pack/themes/start/dracula/colors/
COPY --from=fetch-vim-colors /vim-colors/pack/themes/start/dracula/autoload/     /home/aibox/.vim/pack/themes/start/dracula/autoload/
COPY config/vim-colors/projectious.vim                                           /home/aibox/.vim/colors/projectious.vim
COPY --from=fetch-vim-colors /vim-colors/colors/                                 /usr/share/vim/vimfiles/colors/
COPY --from=fetch-vim-colors /vim-colors/pack/themes/start/dracula/colors/       /usr/share/vim/vimfiles/colors/
COPY --from=fetch-vim-colors /vim-colors/pack/themes/start/dracula/autoload/     /usr/share/vim/vimfiles/autoload/
COPY config/vim-colors/projectious.vim                                           /usr/share/vim/vimfiles/colors/projectious.vim
COPY config/vimrc /home/aibox/.vim/vimrc

# Create config directories in a single layer
RUN mkdir -p /home/aibox/.config/tmux \
             /home/aibox/.tmux/plugins \
             /home/aibox/.local/share/tmux/resurrect \
             /home/aibox/.cache/starship \
             /home/aibox/.cache/uv \
             /home/aibox/.config/yazi/plugins \
             /home/aibox/.config/state \
             /home/aibox/.config/git \
             /home/aibox/.local/bin

# tmux
RUN mkdir -p /usr/local/share/aibox/tmux/plugins
COPY --from=fetch-tmux-plugins /tmux-plugins/ /usr/local/share/aibox/tmux/plugins/
COPY config/tmux/powerkit-plugins/aibox.sh /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/aibox.sh
# aibox-metrics path-a split: one plugin script per metric segment so each
# renders with chevron/color-rotation styling (BACK-20260508_1603-QuietCedar).
COPY config/tmux/powerkit-plugins/aibox_log.sh  /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/aibox_log.sh
COPY config/tmux/powerkit-plugins/aibox_oom.sh  /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/aibox_oom.sh
COPY config/tmux/powerkit-plugins/aibox_proc.sh /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/aibox_proc.sh
COPY config/tmux/powerkit-plugins/aibox_ai.sh   /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/aibox_ai.sh
COPY config/tmux/powerkit-plugins/aibox_mcp.sh  /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/aibox_mcp.sh
COPY config/tmux/powerkit-plugins/aibox_mig.sh  /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/aibox_mig.sh
# Override upstream uptime.sh so the status segment reports container uptime
# from PID 1 rather than host/VM kernel uptime from /proc/uptime.
COPY config/tmux/powerkit-plugins/uptime.sh     /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/uptime.sh
# Override upstream netspeed.sh so the segment uses fixed-width rate fields.
COPY config/tmux/powerkit-plugins/netspeed.sh   /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/netspeed.sh
COPY config/tmux/tmux.conf /home/aibox/.config/tmux/tmux.conf
RUN ln -sf /home/aibox/.config/tmux/tmux.conf /home/aibox/.tmux.conf && \
    for plugin in /usr/local/share/aibox/tmux/plugins/*; do \
      ln -s "$plugin" "/home/aibox/.tmux/plugins/$(basename "$plugin")"; \
    done && \
    mkdir -p /home/aibox/.vim/pack/aibox/start && \
    ln -sf /usr/local/share/aibox/tmux/plugins/vim-tmux-navigator \
      /home/aibox/.vim/pack/aibox/start/vim-tmux-navigator

# Yazi
COPY config/yazi/yazi.toml             /home/aibox/.config/yazi/yazi.toml
COPY config/yazi/keymap.toml           /home/aibox/.config/yazi/keymap.toml
COPY config/yazi/theme.toml            /home/aibox/.config/yazi/theme.toml
COPY config/yazi/init.lua              /home/aibox/.config/yazi/init.lua
COPY config/yazi/themes/               /home/aibox/.config/yazi/themes/
COPY config/yazi/plugins/              /home/aibox/.config/yazi/plugins/

# Helper scripts (chmod via COPY --chmod to avoid extra layer)
COPY --chmod=755 config/bin/open-in-editor.sh /usr/local/bin/open-in-editor
COPY --chmod=755 config/bin/aibox-preview.sh   /usr/local/bin/aibox-preview
COPY --chmod=755 config/bin/vim-loop.sh       /usr/local/bin/vim-loop
COPY --chmod=755 config/bin/pdf-watch.sh      /usr/local/bin/pdf-watch
COPY --chmod=755 config/bin/aibox-log-viewer.sh /usr/local/bin/aibox-log-viewer
COPY --chmod=755 config/bin/aibox-status-toggle.sh /usr/local/bin/aibox-status-toggle
COPY --chmod=755 config/bin/aibox-tmux-session.sh /usr/local/bin/aibox-tmux-session
# xdg-open shim — headless containers have no browser, so any tool that
# calls xdg-open (gh auth login, git credential helpers, opencode, ...)
# would otherwise hit a confusing "Failed opening a web browser" error.
# The shim prints the URL with copy-this-to-your-host instructions and
# exits 0 so the caller thinks the browser opened. See DEC-031.
COPY --chmod=755 config/bin/xdg-open.sh       /usr/local/bin/xdg-open

# Quick reference cheatsheet
COPY config/cheatsheet.txt             /home/aibox/.config/cheatsheet.txt

# lnav format definition for aibox NDJSON log
RUN mkdir -p /home/aibox/.config/lnav/formats/aibox
COPY config/lnav/aibox.json            /home/aibox/.config/lnav/formats/aibox/aibox.json

# Git — use XDG path so the mount target is a directory, not a single file
COPY config/gitconfig /home/aibox/.config/git/config

# Audio — PulseAudio config for Claude Code voice support
COPY config/asoundrc /home/aibox/.asoundrc

# Shell — bashrc with aliases and tool integrations
COPY config/bashrc /home/aibox/.bashrc

# Fix ownership of all config files copied as root
RUN chown -R aibox:aibox /home/aibox

# ── Entrypoint for UID/GID remapping ───────────────────────────────────────
COPY --chmod=755 config/entrypoint.sh /usr/local/bin/entrypoint.sh

# ── Tool install locations ───────────────────────────────────────────────────
# Addon-provided executables should land in canonical system locations.
# Keep user-local bin on PATH as a convenience for ad-hoc installs, but do not
# rely on it for scaffolded tools.
ENV UV_TOOL_BIN_DIR="/usr/local/bin" \
    XDG_STATE_HOME="/home/aibox/.config/state" \
    PATH="${PATH}:/home/aibox/.local/bin"

# ── Entrypoint ────────────────────────────────────────────────────────────────
# Root so the entrypoint can remap UID/GID if needed before dropping to aibox
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
WORKDIR /workspace

# Keep the container alive — VS Code execs into it; tmux is launched via the terminal profile
CMD ["sleep", "infinity"]
