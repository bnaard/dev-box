---
sidebar_position: 2
title: "Language Runtimes"
---

# Language Runtime Addons

Language runtimes install compilers, interpreters, and package managers into your container.

## Python

```toml
[addons.python.tools]
python = { version = "3.14" }   # 3.12, 3.13, 3.14
uv = { version = "0.11.19" }    # 0.7, 0.11.10, 0.11.11, 0.11.15, 0.11.19
# poetry = { version = "2.4.1" } # Optional: 1.8, 2.0, 2.4.1
# pdm = { version = "2.27.0" }   # Optional: 2.22, 2.26.9, 2.27.0
```

Installs Python, pip, venv, and [uv](https://github.com/astral-sh/uv) (fast package manager). The base image pins uv to the curated default instead of following a floating `latest` image tag. Poetry and PDM are available but not enabled by default.

## Rust

```toml
[addons.rust.tools]
rustc = { version = "1.96.0" }  # 1.90, 1.91, 1.92, 1.93, 1.94, 1.94.1, 1.96.0
clippy = {}                     # Linter (no version selection)
rustfmt = {}                    # Formatter (no version selection)
```

Installs the Rust toolchain via rustup with clippy and rustfmt. Uses a multi-stage Docker build — compilation happens in a builder stage and only the toolchain is copied to the runtime image.

## Node.js

```toml
[addons.node.tools]
node = { version = "26" }       # 20, 22, 24, 26
pnpm = { version = "11.5.2" }   # 9, 10, 11.1.3, 11.5.2
# yarn = { version = "4.16.0" } # Optional: 4, 4.16.0
# bun = { version = "1.3.14" }  # Optional
```

Installs Node.js via NodeSource, plus pnpm as default package manager. Yarn and Bun are available but not enabled by default.

## Go

```toml
[addons.go.tools]
go = { version = "1.26.4" }     # 1.25, 1.26, 1.26.3, 1.26.4
```

Installs Go and sets up GOPATH.

## Typst

```toml
[addons.typst.tools]
typst = { version = "0.14" }    # 0.13, 0.14
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
