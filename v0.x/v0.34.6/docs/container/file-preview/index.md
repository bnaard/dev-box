# File Preview


# File Preview

aibox containers ship with Yazi preview configuration and can install
TUI-native preview tools for raster images, vector graphics, PDF, archives, and
video through optional addons. Several formats also support **watch-mode
preview**, where the rendered output updates automatically whenever the source
file changes.

## Overview

There are two independent preview mechanisms:

| Mechanism | When to use |
|-----------|-------------|
| [Yazi file preview](#yazi-file-preview) | Browsing files — preview appears automatically in the right panel as you navigate |
| [Standalone TUI viewers](#standalone-tui-viewers) | Viewing a specific file in a pane, or piping output from a build tool |

Watch-mode (live-updating preview) is available via the standalone tools — see [Watch-Mode Preview](#watch-mode-preview).

---

## Yazi File Preview

When you open Yazi (`Ctrl+g s` from the file manager pane, or via the layout sidebar), files are previewed automatically in the right panel as you navigate. No manual invocation needed.

### Supported formats

| Format | Extensions | Previewer | Requirement |
|--------|-----------|-----------|-------------|
| JPEG / PNG | `.jpg` `.jpeg` `.png` | `image` (built-in) | `chafa` |
| GIF (incl. animated) | `.gif` | `image` (built-in) | `chafa` |
| WebP | `.webp` | `image` (built-in) | `chafa` |
| BMP | `.bmp` | `image` (built-in) | `chafa` |
| TIFF | `.tiff` `.tif` | `image` (built-in) | `chafa` |
| SVG | `.svg` | `svg.yazi` plugin | `resvg` or `rsvg-convert` |
| EPS | `.eps` | `eps.yazi` plugin | `ghostscript` (addon) |
| PDF | `.pdf` | `pdf` (built-in) | `poppler-utils` |
| Markdown | `.md` `.markdown` | `rich-preview.yazi` plugin | `preview-enhanced` addon |
| SQLite | `.sqlite` `.sqlite3` `.db` | `sqlite-preview.yazi` plugin | `data-preview` addon |
| CSV / TSV | `.csv` `.tsv` | `tabular-preview.yazi` plugin | `data-preview` addon |
| Excel | `.xls` `.xlsx` | `tabular-preview.yazi` plugin | `data-preview` addon |
| Video | `.mp4` `.mkv` `.webm` `.avi` | `video` (built-in) | `ffmpeg` (addon) |
| Text / code | most text formats | `code` (built-in) | — |

Raster image, SVG, PDF, archive, and standalone terminal viewers require the
**preview-archive** addon. SQLite, CSV/TSV, and Excel previews require the
**data-preview** addon. Markdown rendering, EPS, and video thumbnails require
the **preview-enhanced** addon, which depends on `preview-archive`:

```bash
aibox set addon preview-archive enabled --apply    # adds chafa, timg, poppler, mutool, entr, p7zip, resvg
aibox set addon data-preview enabled --apply       # adds sqlite3 and csvkit for data previews
aibox set addon preview-enhanced enabled --apply   # adds python3-rich, ffmpeg, ghostscript
```

### How previewer dispatch works

Yazi matches files against a list of `prepend_previewers` in `~/.config/yazi/yazi.toml`. The first matching entry wins:

```toml
[plugin]
prepend_previewers = [
    { url = "*.svg", run = "svg" },
    { url = "*.eps", run = "eps" },
    { url = "*.md",  run = "rich-preview" },
    { url = "*.jpg",  run = "image" },
    { url = "*.jpeg", run = "image" },
    { url = "*.png",  run = "image" },
    { url = "*.gif",  run = "image" },
    { url = "*.webp", run = "image" },
    { url = "*.bmp",  run = "image" },
    { url = "*.tiff", run = "image" },
    { url = "*.tif",  run = "image" },
]
```

Custom plugins (`svg.yazi`, `eps.yazi`) live at `~/.config/yazi/plugins/<name>.yazi/init.lua`. They are seeded into `.aibox-home/.config/yazi/` on first `aibox init`.

### Format notes

**SVG** — converted to PNG by `resvg`, a fast standalone Rust-based SVG renderer bundled as a static binary in `/usr/local/bin/resvg`. The rendered PNG is cached under Yazi's cache directory. If `resvg` is absent from `PATH`, the plugin fails gracefully and Yazi falls back to the text previewer.

**EPS** — rendered to PNG at 150 DPI by `gs` (Ghostscript), then displayed as an image. Result is cached.

**Markdown** — when the **preview-enhanced** addon is enabled, `.md` and `.markdown` files are rendered through `rich-preview.yazi`, which uses Python Rich for terminal-native Markdown rendering. Without the addon, Yazi falls back to its built-in text/code preview.

**PDF** — page 1 is rendered by `pdftoppm` (from `poppler-utils`). Navigate multi-page documents with Yazi's built-in PDF plugin controls.

**Wide text previews** — the seeded Yazi keymap includes a pager shortcut for the selected file using `less -R -S`. `-R` preserves ANSI color from rich/code output; `-S` disables wrapping so long lines can be inspected with horizontal scrolling.

**SQLite** — `sqlite-preview.yazi` opens databases read-only through `sqlite3` and shows schema objects plus table/view columns. It is enabled only when the `data-preview` addon is configured.

**CSV / TSV / Excel** — `tabular-preview.yazi` formats CSV/TSV with `csvlook`; `.xls` and `.xlsx` are converted with `in2csv` before formatting. It is enabled only when the `data-preview` addon is configured.

**`.excalidraw` files** — Excalidraw's native format is JSON. A graphical preview is not possible in a TUI environment. Yazi falls back to the text previewer showing the raw JSON. This is a known limitation — Excalidraw requires a browser to render.

{{< callout type="info" title="Image protocol inside tmux" >}}

Image preview support depends on the host terminal and tmux passthrough
settings. The generated Yazi config favors terminal-safe fallbacks inside tmux;
use half-block mode when you need predictable rendering across terminals.

{{< /callout >}}

---

## Standalone TUI Viewers

These tools are available directly in the shell for viewing a specific file or
integrating into a pipeline when the `preview-archive` addon is enabled.

### chafa — universal image renderer

`chafa` converts images to terminal graphics using Sixel, Kitty protocol, half-block Unicode, or plain ASCII, auto-detecting the best mode for the current terminal.

```bash
# View any raster image
chafa photo.jpg

# Force half-block mode (safe inside tmux)
chafa --format=halfblock diagram.png

# SVG via librsvg (if chafa was compiled with librsvg support)
chafa --format=halfblock logo.svg

# Constrain to a specific cell size
chafa -s 80x40 banner.png
```

**Supported formats:** JPEG, PNG, GIF (animated), WebP, BMP, TIFF, AVIF, and more. SVG support depends on whether `chafa` was built with `librsvg` — run `chafa --version` and check for `SVG: yes`.

### timg — terminal image and document viewer

`timg` renders images, animated GIFs, videos, and **PDFs** (page by page) directly in the terminal.

```bash
# View an image
timg photo.jpg

# View a PDF — renders all pages sequentially
timg document.pdf

# View a specific PDF page (page 2)
timg -p2 document.pdf

# Constrain output size
timg -g 120x40 wide-image.png

# Clear previous output before rendering (useful in watch loops)
timg --clear output.pdf
```

**Supported formats:** JPEG, PNG, GIF (animated), WebP, BMP, TIFF, PDF (via MuPDF), video (via ffmpeg).

---

## Watch-Mode Preview

Watch-mode preview automatically re-renders a file whenever it changes on disk. This is particularly useful for **LaTeX, Typst, and other document workflows** where you write source in one pane and see the rendered output update in real time in another.

The pattern uses `entr` (an inotify-based file watcher) combined with a rasteriser and `timg --clear`:

```
source file changes → entr triggers → rasteriser produces PNG → timg renders PNG in terminal
```

Install the `preview-archive` addon to get the watch-mode tools (`entr`,
`mupdf-tools`, `resvg`, `timg`).

### PDF watch preview ⟳

Run this in a dedicated pane while editing your LaTeX or Typst source:

```bash
pdf-watch output.pdf
```

`pdf-watch` wraps the underlying `entr` + `mutool draw` + `timg --clear` pipeline and re-renders page 1 whenever the PDF changes:

```bash
ls output.pdf | entr -s 'mutool draw -o /tmp/p.png output.pdf 1 && timg --clear /tmp/p.png'
```

| Part | Role |
|------|------|
| `entr` | Watches `output.pdf` for changes (inotify, near-zero CPU at idle) |
| `mutool draw` | MuPDF rasteriser — renders a PDF page to PNG (fast, no X11 needed) |
| `-o /tmp/p.png output.pdf 1` | Output file, input file, page number |
| `timg --clear` | Renders the PNG inline, clearing the previous frame first |

Yazi also seeds a PDF live-watch binding for selected `.pdf` files. It invokes `pdf-watch` so you can start the same live preview directly from the file manager.

**Tips:**

- Change the final `1` to preview a different page number.
- To watch all pages: `mutool draw -o /tmp/p-%d.png output.pdf` (produces `/tmp/p-1.png`, `/tmp/p-2.png`, …); then `timg --clear /tmp/p-*.png`.
- Add `-r 150` to `mutool draw` for higher resolution (default is 72 DPI).
- Use `timg -g 120x40` to constrain the rendered size to fit a specific pane.

### SVG watch preview ⟳

```bash
ls diagram.svg | entr -s 'resvg diagram.svg /tmp/d.png && timg --clear /tmp/d.png'
```

`resvg` converts the SVG to a high-fidelity PNG. Unlike Inkscape or rsvg-convert, `resvg` is a static binary with no runtime dependencies and typically renders in under 100 ms for typical diagrams.

### General pattern

The same `entr` + rasteriser + `timg` pattern works for any file format that has a headless rasteriser:

```bash
# Watch a file and re-render on change
ls <file> | entr -s '<rasterise-command> && timg --clear <output.png>'
```

| Source format | Rasteriser command |
|--------------|-------------------|
| PDF (page 1) | `mutool draw -o /tmp/p.png file.pdf 1` |
| SVG | `resvg file.svg /tmp/p.png` |
| EPS | `gs -dBATCH -dNOPAUSE -sDEVICE=png16m -r150 -sOutputFile=/tmp/p.png file.eps` |

{{< callout type="success" title="Pane layout for watch preview" >}}

In the `dev` or `cowork` layouts, open a new horizontal pane below the editor (`Ctrl+g d`) and run the watch command there. The preview refreshes in that pane every time you save. Use `Ctrl+g =` to resize the pane to taste.

{{< /callout >}}

---

## Format Coverage Summary

| Format | Yazi preview | `chafa` | `timg` | Watch-mode |
|--------|:-----------:|:-------:|:------:|:----------:|
| JPEG / PNG | ✓ | ✓ | ✓ | — |
| GIF (animated) | ✓ | ✓ | ✓ | — |
| WebP | ✓ | ✓ | ✓ | — |
| BMP | ✓ | ✓ | ✓ | — |
| TIFF | ✓ | ✓ | ✓ | — |
| SVG | ✓ (resvg) | ✓ (librsvg) | — | ✓ (resvg) |
| EPS | ✓ (ghostscript) | — | — | ✓ (ghostscript) |
| PDF | ✓ (poppler) | — | ✓ | ✓ (mutool) |
| Video | ✓ (ffmpeg) | — | ✓ | — |
| `.excalidraw` | text fallback | — | — | — |


---
Source: https://projectious-work.github.io/aibox/v0.x/v0.34.6/docs/container/file-preview/index.md
