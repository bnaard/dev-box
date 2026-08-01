# Color Themes

LLMS index: [llms.txt](/aibox/v0.x/v0.29.0/llms.txt)

---

# Themes

aibox supports consistent color theming across all terminal tools. Set a theme in `aibox.toml`:

```toml
[customization]
theme = "gruvbox-dark"
mode = "auto"
```

Or during project initialization:

```bash
aibox init --theme catppuccin-mocha
```

The selected theme is applied to **tmux**, **Vim**, **Yazi**, **lazygit**, and **Starship** simultaneously.

`mode = "auto"` follows the host OS light/dark appearance when a host signal is detectable during `aibox apply`, `aibox up`, or `aibox set theme.*`. Containers do not receive live macOS/Windows/Linux appearance-change events, so rerun one of those commands to regenerate mounted runtime theme files after changing the host appearance. If the host appearance cannot be detected, `auto` preserves the selected concrete theme.

`mode = "light"` and host-light `auto` use the selected theme family's light partner when one exists. Genuinely dark-only themes stay on the selected concrete theme instead of falling back to an unrelated light theme.

## Available Themes

aibox supports the tmux-powerkit popular theme roster plus aibox-specific
extensions:

- `tokyo-night`, `tokyo-night-storm`, `tokyo-night-day`
- `catppuccin-mocha`, `catppuccin-macchiato`, `catppuccin-frappe`, `catppuccin-latte`
- `dracula`, `dracula-soft`, `nord`, `gruvbox-dark`, `gruvbox-light`
- `rose-pine`, `rose-pine-moon`, `rose-pine-dawn`
- `material`, `material-ocean`, `material-palenight`, `material-lighter`, `material-darker`
- `solarized-dark`, `solarized-light`
- `github-dark`, `github-dark-dimmed`, `github-dark-high-contrast`, `github-light`, `github-light-high-contrast`
- `ayu-dark`, `ayu-mirage`, `ayu-light`, `night-owl`, `night-owl-light`, `moonlight`
- `everforest-dark`, `everforest-light`, `kanagawa-wave`, `kanagawa-dragon`, `kanagawa-lotus`
- `min-dark`, `min-light`, `one-dark-pro`, `one-light`, `slack-dark`, `slack-ochin`
- `vitesse-dark`, `vitesse-light`, `vitesse-black`, `vscode-dark-plus`, `vscode-light-plus`
- `andromeeda`, `aurora-x`, `houston`, `laserwave`, `monokai`, `plastic`, `poimandres`, `red`, `snazzy-light`, `synthwave-84`, `vesper`
- `projectious`

## Light/Dark Partners

| Family | Dark variants | Light variant |
|---|---|---|
| Tokyo Night | `tokyo-night`, `tokyo-night-storm` | `tokyo-night-day` |
| Catppuccin | `catppuccin-mocha`, `catppuccin-macchiato`, `catppuccin-frappe` | `catppuccin-latte` |
| Gruvbox | `gruvbox-dark` | `gruvbox-light` |
| Rose Pine | `rose-pine`, `rose-pine-moon` | `rose-pine-dawn` |
| Material | `material`, `material-ocean`, `material-palenight` | `material-lighter` |
| Solarized | `solarized-dark` | `solarized-light` |
| GitHub | `github-dark` | `github-light` |
| Ayu | `ayu-dark`, `ayu-mirage` | `ayu-light` |
| Night Owl | `night-owl` | `night-owl-light` |
| Everforest | `everforest-dark` | `everforest-light` |
| Kanagawa | `kanagawa-wave`, `kanagawa-dragon` | `kanagawa-lotus` |
| Min | `min-dark` | `min-light` |
| One Dark | `one-dark-pro` | `one-light` |
| Slack | `slack-dark` | `slack-ochin` |
| Vitesse | `vitesse-dark`, `vitesse-black` | `vitesse-light` |
| VS Code | `vscode-dark-plus` | `vscode-light-plus` |

Dark-only or single-variant themes with no light partner: `andromeeda`,
`aurora-x`, `houston`, `laserwave`, `monokai`, `moonlight`, `nord`, `plastic`,
`poimandres`, `projectious`, `red`, `snazzy-light`, `synthwave-84`, and `vesper`.

### gruvbox-dark (default)

Retro groove color scheme with warm, earthy tones. High contrast and easy on the eyes.

- **Background:** `#282828` (dark brown-gray)
- **Accent:** `#D79921` (warm yellow)
- **Style:** Dark, warm, retro

<div class="asciinema" data-cast="/aibox/screencasts/theme-gruvbox-dark.cast" data-poster="npt:2" data-loop="true" data-theme="gruvbox-dark" data-fit="width"></div>

### catppuccin-mocha

Soothing pastel theme with a dark background. The most popular modern terminal theme.

- **Background:** `#1E1E2E` (deep purple-black)
- **Accent:** `#89B4FA` (soft blue)
- **Style:** Dark, pastel, modern

<div class="asciinema" data-cast="/aibox/screencasts/theme-catppuccin-mocha.cast" data-poster="npt:2" data-loop="true" data-theme="catppuccin-mocha" data-fit="width"></div>

### catppuccin-latte

Light variant of Catppuccin. Clean and readable in bright environments.

- **Background:** `#EFF1F5` (warm white)
- **Accent:** `#1E66F5` (vivid blue)
- **Style:** Light, pastel, modern

<div class="asciinema" data-cast="/aibox/screencasts/theme-catppuccin-latte.cast" data-poster="npt:2" data-loop="true" data-theme="catppuccin-latte" data-fit="width"></div>

### dracula

Dark theme with vibrant colors. A classic among developers.

- **Background:** `#282A36` (dark gray-blue)
- **Accent:** `#BD93F9` (purple)
- **Style:** Dark, vibrant, bold

<div class="asciinema" data-cast="/aibox/screencasts/theme-dracula.cast" data-poster="npt:2" data-loop="true" data-theme="dracula" data-fit="width"></div>

### tokyo-night

Inspired by Tokyo's night lights. Clean and modern with blue tones.

- **Background:** `#1A1B26` (deep blue-black)
- **Accent:** `#7AA2F7` (bright blue)
- **Style:** Dark, cool, modern

<div class="asciinema" data-cast="/aibox/screencasts/theme-tokyo-night.cast" data-poster="npt:2" data-loop="true" data-theme="tokyo-night" data-fit="width"></div>

### nord

Arctic, north-bluish color palette. Minimalist and calm.

- **Background:** `#2E3440` (dark blue-gray)
- **Accent:** `#88C0D0` (frost blue)
- **Style:** Dark, cool, minimalist

<div class="asciinema" data-cast="/aibox/screencasts/theme-nord.cast" data-poster="npt:2" data-loop="true" data-theme="nord" data-fit="width"></div>

### projectious

The projectious.work brand theme. Deep navy base with a vivid orange accent.

- **Background:** `#1d3352` (midnight navy)
- **Accent:** `#E05232` (ember orange)
- **Midtone:** `#546a82` (slate blue)
- **Style:** Dark, professional

<!-- recording pending -->

## How It Works

Each theme is a coordinated set of config files applied to all tools when `aibox apply`, `aibox up`, or `aibox set theme.*` regenerates managed runtime files:

| Tool | Config file | What's themed |
|------|------------|---------------|
| **tmux** | `.config/tmux/themes/<name>.conf` | Pane borders, status bar, window colors |
| **Vim** | `.vim/colors/<name>.vim` | Syntax highlighting, UI elements |
| **Yazi** | `.config/yazi/theme.toml` | File colors, status bar, selection |
| **lazygit** | `.config/lazygit/config.yml` | Borders, selection, diff colors |
| **Starship** | `.config/starship.toml` | Prompt segment colors |

Claude Code inherits terminal colors automatically — no separate theme file needed.

## Changing Themes

To switch light/dark mode in an existing project:

```bash
aibox set theme.mode auto
aibox set theme.mode light
aibox set theme.mode dark
aibox set theme.name tokyo-night
```

This updates `[customization].mode` in `aibox.toml` and regenerates the mounted runtime theme files under `.aibox-home/`. The running container is not stopped.

If the project tmux session is running, refresh and attach it without stopping the container:

```bash
aibox set theme.mode dark --restart-session
```

<div class="alert alert-secondary" role="alert"><div class="h4 alert-heading" role="heading">Theme files are force-updated by apply</div>


`aibox apply` and `aibox set theme.mode/name` overwrite theme-dependent config files (tmux theme, Vim colorscheme, Yazi theme, lazygit config, Starship config) to match the selected theme. You do not need to rebuild or restart the container to change themes; running TUI processes may need to be restarted.
</div>
