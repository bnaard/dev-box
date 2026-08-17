# Custom Themes

LLMS index: [llms.txt](/aibox/v0.x/v0.33.1/llms.txt)

---

# Creating Custom Themes

aibox ships 7 built-in themes. You can create a custom theme by adding entries to the CLI source code.

## Theme Structure

Each theme defines colors for 5 tools:

| Tool | Config Location | Format |
|------|----------------|--------|
| **tmux** | `.config/tmux/themes/<name>.conf` | tmux style settings |
| **Vim** | `.vim/colors/<name>.vim` | Vim colorscheme |
| **Yazi** | `.config/yazi/theme.toml` | TOML with hex colors |
| **lazygit** | `.config/lazygit/config.yml` | YAML gui.theme section |
| **Starship** | `.config/starship.toml` | TOML with palette |

## Color Mapping

A theme needs these terminal color slots for tmux:

| Slot | Purpose |
|------|---------|
| `fg` | Default foreground text |
| `bg` | Background |
| `black` | Dark background variant |
| `red` | Errors, unstaged changes |
| `green` | Success, staged changes |
| `yellow` | Warnings, search highlights |
| `blue` | Primary accent |
| `magenta` | Secondary accent |
| `cyan` | Tertiary accent, links |
| `white` | Bright foreground |
| `orange` | Special highlights |

## Adding a Theme

To add a new theme to aibox, you need to modify `cli/src/themes.rs` (theme data) and `cli/src/config.rs` (Theme enum). See the existing themes as reference patterns.

The `projectious` theme (`cli/src/themes.rs`) is a good starting point — it uses a simple palette with clear semantic mappings.

## Manual Overrides

If you don't want to modify the CLI, you can manually edit the config files in `.aibox-home/` after `aibox apply`. Note that `aibox apply` will overwrite theme-dependent files, so manual edits need to be reapplied after each apply.
