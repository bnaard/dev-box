# Prompt Presets


# Starship Prompt Presets

aibox includes 8 [Starship](https://starship.rs) prompt presets that work with any theme. Set a preset in `aibox.toml`:

```toml
[customization]
prompt = "default"
```

## Available Presets

### default

Full-featured two-line prompt with directory, git branch/status, language versions, and command duration. Uses Nerd Font symbols.

```
 ~/workspace/myproject  main ✓  v1.75.0  took 2s
❯
```

---

### plain

Same information as `default` but uses ASCII characters only — no Nerd Font or special font needed. Works in any terminal.

```
~/workspace/myproject [main +1 !2] [v1.75.0] took 2s
>
```

Good for remote SSH sessions or environments without font customization.

---

### minimal

Directory and git branch only, with a `❯` indicator. Two-line. For distraction-free, low-noise work.

```
~/workspace/myproject on main
❯
```

---

### nerd-font

Rich prompt with Nerd Font icons for OS, language runtimes, git status, Docker context, and system info. Requires a [Nerd Font](https://www.nerdfonts.com/) installed on the host terminal.

```
 ~/workspace  main  +1 !2   v1.75.0  🐳 dev  3s
❯
```

---

### pastel

One-line pastel powerline prompt inspired by Starship's Pastel Powerline preset. Directory, git, language runtimes, command duration, and character appear inline in connected colored segments. Nerd Font recommended.

```
 ~/workspace/myproject  main +1 
❯
```

### powerline-pastel

Explicit name for the one-line pastel powerline prompt. The legacy `pastel-powerline` name is still accepted as an alias.

---

### bracketed

Each segment wrapped in square brackets — `[dir] [branch] [status]`. Clean, structured appearance without special fonts. A good alternative to `plain` with more visual structure.

```
[~/workspace/myproject] [main] [+1 !2]
❯
```

---

### arrow

Airline/powerline-style prompt with hard chevron separators (`►`). Segments for directory, git branch, and git status appear as connected colored blocks, with command duration shown inline. Requires a Nerd Font or Powerline-patched font.

```
 ~/workspace/myproject  main +1 !2  took 3s
❯
```

---

## Changing Presets

1. Edit `aibox.toml`:
   ```toml
   [customization]
   prompt = "arrow"
   ```

2. Run apply:
   ```bash
   aibox apply
   ```

The Starship config is regenerated at `.aibox-home/.config/starship.toml`. Colors are derived from the active theme.

## Font Requirements

| Preset | Font requirement |
|--------|-----------------|
| `default` | Nerd Font recommended (for `❯` symbol) |
| `plain` | Any font — ASCII only |
| `minimal` | Nerd Font recommended (for `❯` symbol) |
| `nerd-font` | Nerd Font required |
| `pastel` | Nerd Font or Powerline font required |
| `powerline-pastel` | Nerd Font or Powerline font required |
| `bracketed` | Any font — no special glyphs |
| `arrow` | Nerd Font or Powerline font required |

Install a Nerd Font from [nerdfonts.com](https://www.nerdfonts.com/) and configure it in your terminal emulator to use icon-based presets.


---
Source: https://projectious-work.github.io/aibox/docs/customization/prompts/index.md
