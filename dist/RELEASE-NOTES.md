# aibox v0.14.2

CLI patch release. Fixes a layout-sync bug, adds a new "ai" zellij layout, and
re-releases the macOS binaries (the v0.14.2 macOS artifacts from a prior
incomplete release attempt are superseded by this clean release).

## Bug fix: layout setting in `aibox.toml` now actually applies

Two places hard-coded the layout to `dev` and ignored `[customization] layout`
in `aibox.toml`. After this release the configured layout is honored
everywhere:

- **`.aibox-home/.config/zellij/config.kdl`** — `default_layout` is now
  substituted from `aibox.toml` at seed time. Previously the seeded config
  always wrote `default_layout "dev"`, so any zellij invocation that did not
  pass `--layout` (e.g. opening a new VS Code terminal) silently fell back
  to `dev`.
- **`.devcontainer/devcontainer.json`** — VS Code's zellij terminal profile
  is now generated with the configured layout in its `--layout` argument.
  Previously it was hard-coded to `["--layout", "dev"]`.

After upgrading to v0.14.2, run `aibox sync` to regenerate both files. Your
chosen layout will then take effect for `aibox start`, VS Code's zellij
terminal profile, and any other path into zellij.

## New layout: `ai`

A new zellij layout for AI-first workflows: **yazi on the left, AI agent on
the right**, vertical split with no editor on the first screen.

```
┌──────────────────┬────────────────────────────┐
│                  │                            │
│  yazi (40%)      │  claude (60%)              │
│                  │                            │
│                  │                            │
├──────────────────┴────────────────────────────┤
│  status-bar                                   │
└───────────────────────────────────────────────┘

  Tab 1: ai   Tab 2: editor   Tab 3: git   Tab 4: shell
```

The editor still lives in tab 2 (fullscreen vim), so opening a file from
yazi via `e` brings up the editor exactly like the `browse` layout. Tabs 3
and 4 are git and shell as in every other layout.

If multiple AI providers are configured, they are stacked in the right pane
(same convention as `cowork` and `browse`).

Select with:

```bash
aibox start --layout ai
```

or set as default in `aibox.toml`:

```toml
[customization]
layout = "ai"
```

## Container Images

- `ghcr.io/projectious-work/aibox:base-debian-v0.14.2`
- `ghcr.io/projectious-work/aibox:base-debian-latest`

The base image is rebuilt with the new version label only — there are no
content changes from `v0.14.1`. Users on the v0.14.1 image can stay on it;
the v0.14.2 CLI is fully compatible.

## CLI Binaries

- `aibox-v0.14.2-aarch64-unknown-linux-gnu.tar.gz`
- `aibox-v0.14.2-x86_64-unknown-linux-gnu.tar.gz`
- `aibox-v0.14.2-aarch64-apple-darwin.tar.gz` (added in Phase 2)
- `aibox-v0.14.2-x86_64-apple-darwin.tar.gz` (added in Phase 2)

## Upgrading

```bash
# Install the new CLI (Linux)
curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash

# Or on macOS, follow Phase 2 instructions on the host

# In each project, regenerate the seeded config and devcontainer files:
cd <project>
aibox sync
```

After `aibox sync`, the layout in `aibox.toml` is the source of truth for
all zellij invocations.
