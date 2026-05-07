---
sidebar_position: 3
title: "Layouts"
---

# Layouts

aibox ships six Zellij layouts that control the terminal workspace arrangement. Each layout is optimized for a different workflow.

Generated layouts include AI-agent tabs based on enabled `[ai.harness.<name>]` tables. The
fullscreen **git** tab is generated only when the `git-ui` addon selects
`lazygit`, so tab numbering shifts when that optional tab is omitted.

Every generated layout also includes a key-hint row plus an aibox runtime
status row. The default `shell` mode uses Zellij's built-in status bar plus
`aibox-status --watch` for runtime groups such as memory, CPU pressure, load
average, processes, filesystem, uptime, and project state. The optional
`sidecar` mode uses the experimental aibox Zellij WASM plugin for custom key
and status rows.

Configure the status presentation in `aibox.toml`:

```toml
[customization.zellij_status]
mode = "shell" # shell | sidecar | disabled
```

`shell` uses Zellij's built-in status bar plus `aibox-status --watch`.
`sidecar` selects the two-row WASM plugin: a custom Zellij-style keybar with
expanded leader-key hints plus a matching segmented runtime status row.
`disabled` omits aibox-managed status rows from generated layouts.

## Available Layouts

### dev (default)

VS Code-like arrangement: Yazi file browser on the left (40%) and Vim editor on the right (60%). Claude and supporting tools live on their own dedicated tabs.

![dev layout diagram](/img/layouts/layout-dev.svg)

| Tab | Contents |
|-----|----------|
| 1 · dev | yazi (40%) · vim (60%) |
| 2 · claude | AI agent — fullscreen |
| 3 · git | lazygit — fullscreen, when `git-ui` selects `lazygit` |
| 4 · shell | bash — fullscreen |

Best for general development where you need file navigation, editing, and terminal access simultaneously.

<div class="asciinema" data-cast="/aibox/screencasts/layout-dev.cast" data-poster="npt:2" data-loop="true" data-fit="width"></div>

---

### focus

One tool per tab, fullscreen. Zero distraction — each tool gets the entire screen.

![focus layout diagram](/img/layouts/layout-focus.svg)

| Tab | Contents |
|-----|----------|
| 1 · files | yazi — fullscreen |
| 2 · editor | vim — fullscreen |
| 3 · claude | AI agent — fullscreen |
| 4 · git | lazygit — fullscreen, when `git-ui` selects `lazygit` |
| 5 · shell | bash — fullscreen |

Ideal when you want total focus and switch between tools with a single keypress.

<div class="asciinema" data-cast="/aibox/screencasts/layout-focus.cast" data-poster="npt:2" data-loop="true" data-fit="width"></div>

---

### cowork

Side-by-side coding with AI. Yazi and Vim share the left half (stacked), the AI agent fills the right half. Both panes stay visible as you work.

![cowork layout diagram](/img/layouts/layout-cowork.svg)

| Tab | Contents |
|-----|----------|
| 1 · cowork | left 50%: yazi (top 40%) / vim (bottom 60%) · right 50%: AI agent |
| 2 · git | lazygit — fullscreen, when `git-ui` selects `lazygit` |
| 3 · shell | bash — fullscreen |

Ideal for pair-programming sessions where you want to see AI output while editing.

<div class="asciinema" data-cast="/aibox/screencasts/layout-cowork.cast" data-poster="npt:2" data-loop="true" data-fit="width"></div>

---

### cowork-swap

Cowork with AI and Vim positions swapped: the AI agent and Yazi share the left column, Vim takes the larger right column.

![cowork-swap layout diagram](/img/layouts/layout-cowork-swap.svg)

| Tab | Contents |
|-----|----------|
| 1 · cowork-swap | left 40%: yazi (top 40%) / AI agent (bottom 60%) · right 60%: vim |
| 2 · git | lazygit — fullscreen, when `git-ui` selects `lazygit` |
| 3 · shell | bash — fullscreen |

Use this when the editor deserves more horizontal space and the AI pane plays a supporting role.

<!-- recording pending -->

---

### browse

Yazi-focused layout with a large file preview pane (top 60%) and an AI pane below (bottom 40%). Horizontal split — no editor on the main tab.

![browse layout diagram](/img/layouts/layout-browse.svg)

| Tab | Contents |
|-----|----------|
| 1 · browse | yazi (top 60%) / AI agent (bottom 40%) |
| 2 · editor | vim — fullscreen |
| 3 · git | lazygit — fullscreen, when `git-ui` selects `lazygit` |
| 4 · shell | bash — fullscreen |

Great for exploring unfamiliar codebases, reviewing files, and asking AI about what you find.

<!-- recording pending -->

---

### ai

AI-first layout: Yazi and the AI agent share the main tab side by side (50/50 vertical split). The editor is available on its own tab when needed.

![ai layout diagram](/img/layouts/layout-ai.svg)

| Tab | Contents |
|-----|----------|
| 1 · ai | yazi (50%) · AI agent (50%) |
| 2 · editor | vim — fullscreen |
| 3 · git | lazygit — fullscreen, when `git-ui` selects `lazygit` |
| 4 · shell | bash — fullscreen |

Best for AI-heavy sessions where file navigation and AI interaction are the primary loop and the editor is consulted occasionally.

<!-- recording pending -->

---

## Setting the Default Layout

Set your preferred layout in `aibox.toml`:

```toml
[customization]
layout = "dev"
```

Options: `dev`, `focus`, `cowork`, `cowork-swap`, `browse`, `ai`.

## Per-Session Override

Override the default layout when starting a session:

```bash
aibox up --layout focus
```

This does not change the default in `aibox.toml` — it only applies to the current session.

## Custom Layouts

Layouts are standard Zellij layout files stored in `.aibox-home/.config/zellij/layouts/`. You can edit the built-in layouts or add your own files there.

See the [Zellij layout documentation](https://zellij.dev/documentation/layouts) for the full layout specification.
