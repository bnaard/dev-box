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
ARG TMUX_POWERKIT_REF=6ac71f0d22827af7324e527a3ad69334f80f0641
ARG TMUX_SENSIBLE_REF=25cb91f42d020f675bb0a2ce3fbd3a5d96119efa
ARG VIM_TMUX_NAVIGATOR_REF=e41c431a0c7b7388ae7ba341f01a0d217eb3a432
ARG TPM_REF=e261deb1b47614eed3400089ce7197dc68acc4eb
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
    sh -s -- -y --profile minimal --default-toolchain 1.96.1

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
ARG ZOXIDE_VERSION=0.10.0
RUN ARCH=$(uname -m) && \
    curl -fsSL --retry 5 --retry-delay 3 --retry-all-errors \
      "https://github.com/ajeetdsouza/zoxide/releases/download/v${ZOXIDE_VERSION}/zoxide-${ZOXIDE_VERSION}-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin zoxide && \
    chmod +x /usr/local/bin/zoxide

# ── fzf (uses amd64/arm64 instead of x86_64/aarch64) ────────────────────────
FROM fetch-base AS fetch-fzf
ARG FZF_VERSION=0.73.1
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
ARG STARSHIP_VERSION=1.26.0
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
# Foundation image — OS + toolchain base shared across runtime variants.
# =============================================================================
FROM debian:trixie-slim AS foundation

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
COPY --from=ghcr.io/astral-sh/uv:0.11.26 /uv /uvx /usr/local/bin/
ENV UV_CACHE_DIR=/tmp/aibox/uv-cache

ARG AIBOX_IMAGE_SOURCE_SHA=unknown
ARG AIBOX_FOUNDATION_SOURCE_SHA=unknown
ARG AIBOX_RUNTIME_SOURCE_SHA=unknown
ARG AIBOX_IMAGE_BUILD_VERSION=unknown
LABEL org.projectious-work.aibox.image-kind="foundation" \
      org.projectious-work.aibox.image-source-sha="${AIBOX_IMAGE_SOURCE_SHA}" \
      org.projectious-work.aibox.image-foundation-source-sha="${AIBOX_FOUNDATION_SOURCE_SHA}" \
      org.projectious-work.aibox.image-runtime-source-sha="${AIBOX_RUNTIME_SOURCE_SHA}" \
      org.projectious-work.aibox.image-build-version="${AIBOX_IMAGE_BUILD_VERSION}"

# =============================================================================
# Runtime image — managed defaults and full dev-container behavior.
# =============================================================================
FROM foundation AS runtime

# ── Bake default configs into the image ───────────────────────────────────────
# These act as fallback defaults when no host directory is mounted.
# When docker-compose mounts .aibox-home/.vim / .aibox-home/.config/tmux etc.
# over these paths, the host copy takes full precedence.

# Vim — colorschemes from builder stage, vimrc from build context.
# Copy themes into both the image home and Vim's system runtimepath so a
# host-mounted ~/.vim can never hide the configured colorscheme.
RUN mkdir -p /home/aibox/.vim/undo
# The active Vim colorscheme is the aibox-generated ~/.vim/colors/aibox.vim
# (written by `aibox theme`). The upstream colorschemes below are kept as
# manual fallbacks for users running `:colorscheme <name>` interactively.
COPY --from=fetch-vim-colors /vim-colors/colors/                                 /home/aibox/.vim/colors/
COPY --from=fetch-vim-colors /vim-colors/pack/themes/start/dracula/colors/       /home/aibox/.vim/pack/themes/start/dracula/colors/
COPY --from=fetch-vim-colors /vim-colors/pack/themes/start/dracula/autoload/     /home/aibox/.vim/pack/themes/start/dracula/autoload/
COPY --from=fetch-vim-colors /vim-colors/colors/                                 /usr/share/vim/vimfiles/colors/
COPY --from=fetch-vim-colors /vim-colors/pack/themes/start/dracula/colors/       /usr/share/vim/vimfiles/colors/
COPY --from=fetch-vim-colors /vim-colors/pack/themes/start/dracula/autoload/     /usr/share/vim/vimfiles/autoload/
COPY config/vimrc /home/aibox/.vim/vimrc
COPY config/inputrc /home/aibox/.inputrc

# Create config directories in a single layer
RUN mkdir -p /home/aibox/.config/tmux \
             /home/aibox/.tmux/plugins \
             /home/aibox/.local/share/tmux/resurrect \
             /home/aibox/.cache/starship \
             /home/aibox/.cache/uv \
             /home/aibox/.config/yazi/plugins \
             /home/aibox/.config/state \
             /home/aibox/.config/git \
             /home/aibox/.local/bin \
             /tmp/aibox/uv-cache && \
    chown -R aibox:aibox /tmp/aibox

# tmux
RUN mkdir -p /usr/local/share/aibox/tmux/plugins
COPY --from=fetch-tmux-plugins /tmux-plugins/ /usr/local/share/aibox/tmux/plugins/
# ── Temporary local patch: surface-spacer fix for adjacent segment separators ──
# When @powerkit_elements_spacing="both" the closing separator glyph and the
# next segment's opening glyph are adjacent with mismatched backgrounds,
# producing a doubled-arrow rendering artefact.  The fix emits a space cell at
# spacing_bg between them so both the I2 invariant and the powerline convention
# are satisfied.
#
# Tracked by BACK-20260515_1503-RefinedIvy.  Drop this patch block (and bump
# TMUX_POWERKIT_REF past the fix) once the upstream PR is merged.
RUN _f=/usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/renderer/segment_builder.sh; \
    grep -q "AIBOX-FIX-GH-SEPARATOR-MARKER" "$_f" || \
    sed -i \
      's|output+="#\[fg=\${current_spacing_fg},bg=\${prev_bg}\]\${spacing_sep}#\[none\]"|output+="#[fg=${current_spacing_fg},bg=${prev_bg}]${spacing_sep}" # AIBOX-FIX-GH-SEPARATOR-MARKER\n                output+="#[fg=${current_spacing_fg},bg=${current_spacing_bg}] #[none]"|' \
      "$_f"
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
# Override upstream Kubernetes/cloud plugins so the status bar reads local
# context only and never runs live auth/network probes during render.
COPY config/tmux/powerkit-plugins/kubernetes.sh /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/kubernetes.sh
COPY config/tmux/powerkit-plugins/cloud.sh      /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/cloud.sh
COPY config/tmux/powerkit-plugins/github.sh     /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/github.sh
# aibox forge plugin: multi-provider replacement for git+github (BACK-20260515_1503-RefinedIvy).
COPY config/tmux/powerkit-plugins/forge.sh      /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/forge.sh
COPY config/tmux/powerkit-plugins/modelstatus_provider.sh /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/modelstatus_provider.sh
RUN for provider in openai anthropic google mistral deepseek cohere xai alibaba aws meta microsoft minimax moonshot nvidia xiaomi zai; do \
      cp /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/modelstatus_provider.sh \
        "/usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/modelstatus_${provider}.sh"; \
    done
COPY config/tmux/tmux.conf /home/aibox/.config/tmux/tmux.conf
RUN ln -sf /home/aibox/.config/tmux/tmux.conf /home/aibox/.tmux.conf && \
    for plugin in /usr/local/share/aibox/tmux/plugins/*; do \
      ln -s "$plugin" "/home/aibox/.tmux/plugins/$(basename "$plugin")"; \
    done && \
    mkdir -p /home/aibox/.vim/pack/aibox/start \
             /usr/share/vim/vimfiles/pack/aibox/start && \
    ln -sf /usr/local/share/aibox/tmux/plugins/vim-tmux-navigator \
      /home/aibox/.vim/pack/aibox/start/vim-tmux-navigator && \
    ln -sf /usr/local/share/aibox/tmux/plugins/vim-tmux-navigator \
      /usr/share/vim/vimfiles/pack/aibox/start/vim-tmux-navigator

# Yazi — `theme.toml` is overwritten by `aibox theme` at apply time, but a
# baseline copy ships in the image so a freshly built container without an
# .aibox-home mount still renders. The per-flavor bundled themes are no
# longer needed (themes are palette-generated CLI-side).
COPY config/yazi/yazi.toml             /home/aibox/.config/yazi/yazi.toml
COPY config/yazi/keymap.toml           /home/aibox/.config/yazi/keymap.toml
COPY config/yazi/theme.toml            /home/aibox/.config/yazi/theme.toml
COPY config/yazi/init.lua              /home/aibox/.config/yazi/init.lua
COPY config/yazi/plugins/              /home/aibox/.config/yazi/plugins/

# Helper scripts (chmod via COPY --chmod to avoid extra layer)
COPY --chmod=755 config/bin/open-in-editor.sh /usr/local/bin/open-in-editor
COPY --chmod=755 config/bin/aibox-preview.sh   /usr/local/bin/aibox-preview
COPY --chmod=755 config/bin/vim-loop.sh       /usr/local/bin/vim-loop
COPY --chmod=755 config/bin/pdf-watch.sh      /usr/local/bin/pdf-watch
COPY --chmod=755 config/bin/aibox-latex-preview.py /usr/local/bin/aibox-latex-preview
COPY --chmod=755 config/bin/aibox-log-viewer.sh /usr/local/bin/aibox-log-viewer
COPY --chmod=755 config/bin/aibox-log-viewer-vim.sh /usr/local/bin/aibox-log-viewer-vim
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

ARG AIBOX_IMAGE_SOURCE_SHA=unknown
ARG AIBOX_FOUNDATION_SOURCE_SHA=unknown
ARG AIBOX_RUNTIME_SOURCE_SHA=unknown
ARG AIBOX_IMAGE_BUILD_VERSION=unknown
LABEL org.projectious-work.aibox.image-source-sha="${AIBOX_IMAGE_SOURCE_SHA}" \
      org.projectious-work.aibox.image-kind="runtime" \
      org.projectious-work.aibox.image-foundation-source-sha="${AIBOX_FOUNDATION_SOURCE_SHA}" \
      org.projectious-work.aibox.image-runtime-source-sha="${AIBOX_RUNTIME_SOURCE_SHA}" \
      org.projectious-work.aibox.image-build-version="${AIBOX_IMAGE_BUILD_VERSION}"

# Keep the container alive — VS Code execs into it; tmux is launched via the terminal profile
CMD ["sleep", "infinity"]
