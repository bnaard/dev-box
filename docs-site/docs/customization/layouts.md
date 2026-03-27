---
sidebar_position: 3
title: "Layouts"
---

# Layouts

aibox ships four Zellij layouts that control the terminal workspace arrangement. Each layout is optimized for a different workflow.

## Available Layouts

### dev (default)

VS Code-like arrangement with a Yazi file sidebar, Vim editor, and stacked terminal panes. Best for general development where you need file navigation, editing, and terminal access simultaneously.

### focus

One tool per tab, fullscreen. Tabs for vim, lazygit, bash, and your AI provider (e.g., claude). Zero distraction -- each tool gets the entire screen.

### cowork

Side-by-side coding with AI. Yazi and Vim on the left, AI agent pane on the right. Ideal for pair programming sessions where you want to see AI output while editing.

### browse

Yazi-focused layout with a large file preview (60%) and an AI pane (40%). Great for exploring unfamiliar codebases, reviewing files, and asking AI about what you find.

## Setting the Default Layout

Set your preferred layout in `aibox.toml`:

```toml
[customization]
layout = "dev"
```

Options: `dev`, `focus`, `cowork`, `browse`.

## Per-Session Override

Override the default layout when starting a session:

```bash
aibox start --layout focus
```

This does not change the default in `aibox.toml` -- it only applies to the current session.

## Custom Layouts

Layouts are standard Zellij layout files stored in `.aibox-home/.config/zellij/layouts/`. You can edit the built-in layouts or create your own.

See the [Zellij layout documentation](https://zellij.dev/documentation/layouts) for the full layout specification and examples.
