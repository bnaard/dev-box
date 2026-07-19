---
title: "LaTeX Build and Preview"
---

# LaTeX Build and Preview

The `latex` addon supplies TeX Live, `latexmk`, bibliography tools, fonts, and
common package groups. aibox adds a project-level build contract on top so
humans and agents use the same engine, cache, output paths, and document names.

## Configure documents

Enable the addon and declare one or more documents:

```toml
[addons.latex.tools]
texlive-core = {}
texlive-recommended = {}
texlive-fonts = {}
biber = {}

[latex]
engine = "lualatex"
cache_dir = ".latex-cache"
options = []

[[latex.documents]]
name = "overview"
source = "docs/overview/overview.tex"
output_dir = ".latex-cache/overview"

[[latex.documents]]
name = "appendix"
source = "docs/appendix/appendix.tex"
output_dir = ".latex-cache/appendix"

[latex.preview]
enabled = true
engine = "embedpdf"
bind = "127.0.0.1"
port = 8765
document = "overview" # optional; defaults to the first configured document
allow_public = false
```

Supported engines are `lualatex`, `pdflatex`, `xelatex`, and `tectonic`.
Continuous watch mode uses `latexmk`, so `tectonic` configurations support
one-shot builds only.

## Build and watch

```bash
aibox latex build                 # every configured document
aibox latex build overview
aibox latex watch overview
aibox latex status
aibox latex status -o json
```

Builds use non-interactive, file-and-line-error, halt-on-error flags. Watch mode
adds `latexmk -pvc -view=none`, leaving browser preview ownership to aibox.
`TEXMFVAR` and `TEXMFCONFIG` live below `latex.cache_dir`, so TeX does not write
mutable state to the user's global TeX tree.

The CLI uses a host TeX installation when the configured engine is available.
Otherwise it executes the same command inside the running project container.
Source and output paths are project-relative and may contain spaces.

`aibox latex status` reports output readiness, active watchers and previews,
the preview URL, and the latest error line from each TeX log. PID files prevent
accidentally starting a second watcher or preview for the same document.

Running `aibox apply` also writes `AIBOX-LATEX.md` in the project root. This
managed file gives AI agents the configured commands and document paths without
duplicating processkit-owned instructions.

## Live PDF preview

When preview is enabled, `aibox up` starts it in the background for
`latex.preview.document`, or for the first configured document when that key is
omitted. The log is stored under `.aibox/latex/`. Start the same service
explicitly in the foreground when working without `aibox up`:

```bash
aibox preview latex overview
```

`aibox down` stops a background or foreground preview registered for the
project. A subsequent `aibox up` starts it again.

The stable URL is `http://127.0.0.1:8765/`. The service watches the completed
PDF output, waits for its metadata to remain stable across multiple polls, and
then sends an SSE revision event. This prevents the browser from fetching a
partially-written PDF. The browser requests a versioned PDF URL on reload so a
stale cache cannot hide a new build.

The viewer uses the pinned
[`@embedpdf/snippet`](https://www.embedpdf.com/docs/snippet/getting-started)
browser package. Its default toolbar provides navigation, zoom, search,
thumbnails, and outline support when the PDF contains an outline. The browser
must be able to reach jsDelivr to load the pinned viewer module; the PDF itself
is served only by the local aibox process.

### Remote hosts

The default bind address is loopback and is suitable for SSH forwarding:

```bash
ssh -L 8765:127.0.0.1:8765 user@remote-host
```

Then open `http://127.0.0.1:8765/` locally. Binding to a non-loopback address is
rejected unless `allow_public = true` is also set. That opt-in exposes an
unauthenticated PDF endpoint; prefer an SSH tunnel.
