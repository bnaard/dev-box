# LaTeX Build and Preview

LLMS index: [llms.txt](/aibox/v0.x/v0.28.19/llms.txt)

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
# document = "overview" # optional compatibility-route default
allow_public = false
```

Supported engines are `lualatex`, `pdflatex`, `xelatex`, and `tectonic`.
Continuous watch mode uses `latexmk`, so `tectonic` configurations support
one-shot builds only.

## Build and watch

Run these commands **inside the development container**. `aibox apply` generates
them under `.aibox-home/.local/bin`, which is already on the container `PATH`:

```bash
aibox-latex-build                 # every configured document
aibox-latex-build overview        # one configured document
aibox-latex-watch overview        # foreground watcher; stop with Ctrl-C
```

Builds use non-interactive, file-and-line-error, halt-on-error flags. Watch mode
adds `latexmk -pvc -view=none`, leaving browser preview ownership to the sidecar.
`TEXMFVAR` and `TEXMFCONFIG` live below `latex.cache_dir`, so TeX does not write
mutable state to the user's global TeX tree.

Compilation never runs on the host or in the preview sidecar. The scripts use
the TeX installation in the main development container, keep the watch process
in the foreground, and write PDFs below each configured `output_dir`. Run one
watcher per document when several documents should rebuild concurrently.

Running `aibox apply` adds a concise, conditional aibox-managed LaTeX section
to `AGENTS.md`. This is the primary instruction surface for AI agents and gives
them the configured container commands, companion health check, preview URLs,
and document paths without adding another root-level guidance file.

## Live PDF preview

When preview is enabled, `aibox apply` generates a dedicated Compose sidecar and
port mapping. Host-side `aibox up` starts that sidecar, which serves every
configured document from the project workspace mounted read-only. The sidecar
does not contain a TeX toolchain and cannot modify the workspace.
`latex.preview.document` chooses the preferred document used by compatibility
routes; it does not exclude the others.

Inspect its logs with:

```bash
docker compose -f .devcontainer/docker-compose.yml logs <project-container-name>-latex-preview
```

`aibox down` stops the sidecar with the rest of the Compose project. A
subsequent `aibox up` starts it again. With multiple documents, the root URL
displays a selection page. Each PDF also has a stable direct URL:

```text
http://127.0.0.1:8765/
http://127.0.0.1:8765/documents/overview/
http://127.0.0.1:8765/documents/appendix/
```

The stable URL is `http://127.0.0.1:8765/`. The service watches the completed
PDF output for every document, waits for its metadata to remain stable across
multiple polls, and then sends a document-specific SSE revision event. This
prevents the browser from fetching a partially-written PDF. The browser
requests a versioned PDF URL on reload so a stale cache cannot hide a new
build. Page and zoom state are retained separately for each document.

The viewer uses the pinned
[`@embedpdf/snippet`](https://www.embedpdf.com/docs/snippet/getting-started)
browser package. Its default toolbar provides navigation, zoom, search,
thumbnails, and outline support when the PDF contains an outline. The browser
must be able to reach jsDelivr to load the pinned viewer module; the PDF itself
is served by the local preview sidecar.

### Host and network access

The preview helper listens on `0.0.0.0:8765` inside its isolated sidecar. The
generated Compose mapping publishes that internal port only on the configured
host address and port. The secure default is therefore accessible from the
host browser but not from other machines:

```toml
[latex.preview]
enabled = true
bind = "127.0.0.1"
port = 8765
allow_public = false
```

Run `aibox apply` after changing this configuration, then run `aibox up` on the
host. Do not add a manual override mapping;
aibox generates `127.0.0.1:8765:8765` in `.devcontainer/docker-compose.yml`.

For access from another machine on the local network, explicitly expose the
unauthenticated endpoint:

```toml
[latex.preview]
enabled = true
bind = "0.0.0.0"
port = 8765
allow_public = true
```

This generates `0.0.0.0:8765:8765`. Open `http://<host-ip>:8765/` and ensure
the host firewall permits the port. Prefer the loopback default plus SSH
forwarding when possible.

### Remote hosts over SSH

The default bind address is loopback and is suitable for SSH forwarding:

```bash
ssh -L 8765:127.0.0.1:8765 user@remote-host
```

Then open `http://127.0.0.1:8765/` locally. Publishing on a non-loopback host
address is rejected unless `allow_public = true` is also set. That opt-in
exposes an unauthenticated PDF endpoint; prefer an SSH tunnel.
