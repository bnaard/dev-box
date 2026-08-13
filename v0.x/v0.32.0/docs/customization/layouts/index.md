# Layouts

LLMS index: [llms.txt](/aibox/v0.x/v0.32.0/llms.txt)

---

# Layouts

aibox ships four tmux layouts. Harness placement follows `[ai].harness_order`:
the 1st harness is the first enabled harness in that order, then the 2nd, 3rd,
and so on. Enabled harnesses missing from `harness_order` are appended in
canonical order.

Generated layouts can include an extended PowerKit status bar with host,
network, development, cloud, resource, and aibox runtime segments.

## Available Layouts

### ai

| Window | Contents |
|-----|----------|
| 1 · work | left 50%: yazi · right 50%: 1st harness |
| 2 · ai | all further harnesses, split as full-height even horizontal panes |
| 3 · lazygit | lazygit, when `git-ui` selects `lazygit` |
| 3/4 · shell | bash |

### dev

| Window | Contents |
|-----|----------|
| 1 · work | left 50%: yazi top 50% / 1st harness bottom 50% · right 50%: shell |
| 2 · lazygit | lazygit, when `git-ui` selects `lazygit` |
| 2/3 · ai | all further harnesses, split as full-height even horizontal panes |
| final · shell | bash |

### focus

| Window | Contents |
|-----|----------|
| 1 · files | yazi |
| 2..n · harness name | one fullscreen window per harness in `harness_order` |
| next · lazygit | lazygit, when `git-ui` selects `lazygit` |
| final · shell | bash |

### cowork

| Window | Contents |
|-----|----------|
| 1 · work | left 50%: yazi · right 50%: shell |
| 2 · ai | all harnesses, split as full-height even horizontal panes |
| 3 · lazygit | lazygit, when `git-ui` selects `lazygit` |

## Setting The Default Layout

```toml
[customization]
layout = "dev"
```

Options: `dev`, `focus`, `cowork`, `ai`.

## Per-Session Override

```bash
aibox up --layout focus
```

This does not change the default in `aibox.toml`.
