# Language Runtimes

LLMS index: [llms.txt](/aibox/v0.x/v0.31.2/llms.txt)

---

# Language Runtime Addons

Language runtimes install compilers, interpreters, and package managers into your container.

## Python

```toml
[addons.python.tools]
python = { version = "3.14" }   # 3.12, 3.13, 3.14
uv = { version = "0.12.0" }    # 0.7, 0.11.10, 0.11.11, 0.11.15, 0.11.19, 0.11.26, 0.12.0
# poetry = { version = "2.4.1" } # Optional: 1.8, 2.0, 2.4.1
# pdm = { version = "2.28.0" }   # Optional: 2.22, 2.26.9, 2.27.0, 2.28.0
```

Installs Python, pip, venv, and [uv](https://github.com/astral-sh/uv) (fast package manager). The base image pins uv to the curated default instead of following a floating `latest` image tag. Poetry and PDM are available but not enabled by default.

## Rust

```toml
[addons.rust.tools]
rustc = { version = "1.97.1" }  # 1.90, 1.91, 1.92, 1.93, 1.94, 1.94.1, 1.96.0, 1.96.1, 1.97.1
clippy = {}                     # Linter (no version selection)
rustfmt = {}                    # Formatter (no version selection)
```

Installs the Rust toolchain via rustup with clippy and rustfmt. Uses a multi-stage Docker build — compilation happens in a builder stage and only the toolchain is copied to the runtime image.

## Node.js

```toml
[addons.node.tools]
node = { version = "26" }       # 20, 22, 24, 26
pnpm = { version = "11.21.0" }  # 9, 10, 11.1.3, 11.5.2, 11.10.0, 11.18.0, 11.20.0, 11.21.0
# yarn = { version = "4.17.0" } # Optional: 4, 4.16.0, 4.17.0
# bun = { version = "1.3.14" }  # Optional
```

Installs Node.js via NodeSource, plus pnpm as default package manager. Yarn and Bun are available but not enabled by default.

## Go

```toml
[addons.go.tools]
go = { version = "1.26.5" }     # 1.25, 1.26, 1.26.3, 1.26.4, 1.26.5
```

Installs Go and sets up GOPATH.

### Production Go groups

Go projects can compose production tooling without bloating the base runtime:

```toml
[addons.go]

[addons.go.infrastructure]

[addons.go.quality.tools]
staticcheck = { enabled = false } # optional per-tool override

[addons.go.supply-chain]

[addons.go.release]

# Optional nested container engine without the other infrastructure defaults:
[addons.go.infrastructure.tools]
opentofu = { enabled = false }
ansible = { enabled = false }
packer = { enabled = false }
podman = {}
```

`quality` installs `goimports`, `staticcheck`, `golangci-lint`,
`govulncheck`, and `gosec`, plus the Linux compiler and libc headers required
by `go test -race ./...`. `supply-chain` adds `gitleaks`, `osv-scanner`,
`syft`, `grype`, and `cosign`. `release` adds GoReleaser, ShellCheck, and
Hadolint. The built-in `gofmt`, `go vet`, `go test`, fuzzing, and coverage
commands remain part of the Go toolchain and need no extra binary.
The optional `podman` selection installs a rootless container engine and its
Compose provider. The explicit disables keep OpenTofu, Ansible, and Packer out
when the project only needs nested container operations.

The same `infrastructure`, `security`/`supply-chain`, and `release` group names
are accepted below every language addon. Go additionally exposes `quality`
and its `lint` alias.

## Typst

```toml
[addons.typst.tools]
typst = { version = "0.15.0" }  # 0.13.1, 0.14.2, 0.15.0
```

Installs the [Typst](https://typst.app/) typesetting system for modern document creation.

## LaTeX

```toml
[addons.latex.tools]
texlive-core = {}               # Base TeX Live installation
texlive-recommended = {}        # Common packages
texlive-fonts = {}              # Font packages
biber = {}                      # Bibliography processor
texlive-code = {}               # Code listing packages
texlive-diagrams = {}           # TikZ, PGF, circuit diagrams
texlive-math = {}               # Math packages
# texlive-music = {}            # Optional: LilyPond, MusiXTeX
# texlive-chemistry = {}        # Optional: chemfig, mhchem
```

Installs TeX Live via a multi-stage Docker build. The full TeX Live installation happens in a builder stage and only the final tree is copied to the runtime image, keeping layer sizes manageable.

Use the [LaTeX build and preview workflow](./latex-workflow.md) to define named
documents, run reproducible `latexmk` builds and watchers inside the development
container, and serve completed PDFs through the read-only EmbedPDF sidecar.

### Emoji rendering with LuaLaTeX

The runtime image ships `fonts-noto-color-emoji` so emoji glyphs render
correctly in LuaLaTeX documents. To wire it up, add the following to
your preamble:

```latex
\directlua{luaotfload.add_fallback("emojifallback",{"NotoColorEmoji:mode=harf;"})}
\setmainfont{FreeSans}[Scale=0.95,RawFeature={fallback=emojifallback}]
```

Without the fallback configured, emoji characters render as
missing-glyph boxes even though the font is installed.
