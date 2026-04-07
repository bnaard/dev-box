---
sidebar_position: 2
title: "Language Runtimes"
---

# Language Runtime Addons

Language runtimes install compilers, interpreters, and package managers into your container.

## Python

```toml
[addons.python.tools]
python = { version = "3.13" }   # 3.12, 3.13, 3.14
uv = { version = "0.7" }        # 0.6, 0.7
# poetry = { version = "2.0" }  # Optional: 1.8, 2.0
# pdm = { version = "2.22" }    # Optional
```

Installs Python, pip, venv, and [uv](https://github.com/astral-sh/uv) (fast package manager). Poetry and PDM are available but not enabled by default.

## Rust

```toml
[addons.rust.tools]
rustc = { version = "1.87" }    # 1.85, 1.87
clippy = {}                     # Linter (no version selection)
rustfmt = {}                    # Formatter (no version selection)
```

Installs the Rust toolchain via rustup with clippy and rustfmt. Uses a multi-stage Docker build — compilation happens in a builder stage and only the toolchain is copied to the runtime image.

## Node.js

```toml
[addons.node.tools]
node = { version = "22" }       # 20, 22
pnpm = { version = "10" }       # 9, 10
# yarn = { version = "4" }      # Optional
# bun = { version = "1.2" }     # Optional
```

Installs Node.js via NodeSource, plus pnpm as default package manager. Yarn and Bun are available but not enabled by default.

## Go

```toml
[addons.go.tools]
go = { version = "1.26" }       # 1.25, 1.26
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
