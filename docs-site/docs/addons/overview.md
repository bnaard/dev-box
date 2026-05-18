---
sidebar_position: 1
title: "Overview"
---

# Addons

aibox uses the Debian runtime image family (`base-debian-v0.26.x` for legacy releases, `base-debian-runtime-v0.27.0+` after the image-tag cutover) with **30 composable addons** that install language runtimes, tool bundles, documentation frameworks, and AI coding agents into your container.

## Managing Addons

### Via CLI

```bash
# See all available addons
aibox get addon

# Add an addon (updates aibox.toml and runs apply)
aibox set addon python

# Remove an addon
aibox delete addon python

# View addon details (tools, versions)
aibox describe addon rust

# Emit the machine-readable addon catalog index
aibox describe addon-catalog -o json
```

### Via aibox.toml

```toml
[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.11.15" }

[addons.rust.tools]
rustc = { version = "1.87" }
clippy = {}
rustfmt = {}

[addons.node.tools]
node = { version = "22" }
pnpm = { version = "10" }
```

Each addon has **default-enabled tools** that are included automatically, and **optional tools** you can enable explicitly. Tools with version selection let you pick from curated, tested versions.

`aibox describe addon-catalog -o json` emits the stable `aibox.addon-catalog.v0`
index used by downstream automation. It includes each addon's profile intent,
automation usage class, supported aibox profiles, exported surfaces,
dependencies, and tool metadata. Canonical processkit `Artifact{kind=addon-spec}`
emission remains gated on the upstream processkit schema release.

After editing `aibox.toml`, run `aibox apply` to regenerate the Dockerfile and rebuild.

## Available Addons

### Language Runtimes

| Addon | Default Tools | Optional Tools |
|-------|--------------|----------------|
| `python` | python (3.12/3.13/3.14), uv (0.7/0.11.10/0.11.11/0.11.15) | poetry, pdm |
| `rust` | rustc (1.85/1.87), clippy, rustfmt | — |
| `node` | node (20/22), pnpm (9/10) | yarn, bun |
| `go` | go (1.25/1.26) | — |
| `typst` | typst (0.13/0.14) | — |
| `latex` | texlive-core, texlive-recommended, texlive-fonts, biber, texlive-code, texlive-diagrams, texlive-math | texlive-music, texlive-chemistry |

### Tool Bundles

| Addon | Default Tools | Optional Tools |
|-------|--------------|----------------|
| `infrastructure` | opentofu, ansible, packer | — |
| `git-ui` | gh, lazygit | — |
| `preview-archive` | chafa, timg, poppler-utils, mupdf-tools, entr, p7zip-full, resvg | — |
| `preview-enhanced` | python3-rich, ffmpeg, ghostscript | — |
| `audio-voice` | sox, pulseaudio-utils, ALSA PulseAudio plugins | — |
| `kubernetes` | kubectl, helm, kustomize | k9s |
| `cloud-aws` | aws-cli | — |
| `cloud-gcp` | gcloud-cli | — |
| `cloud-azure` | azure-cli | — |

### Documentation Frameworks

| Addon | Tools |
|-------|-------|
| `docs-mkdocs` | mkdocs + mkdocs-material |
| `docs-zensical` | zensical |
| `docs-docusaurus` | docusaurus |
| `docs-starlight` | starlight |
| `docs-mdbook` | mdbook |
| `docs-hugo` | hugo |

### AI Harnesses

AI harnesses are selected under `[ai]`, not as public addon blocks. aibox still
uses internal install recipes for container CLIs when `install = true`.

```toml
[ai.harness.claude]
enabled = true
install = true

[ai.harness.codex]
enabled = true
install = true

[ai.harness.codex]
enabled = true
install = true
version = "latest"
```

Legacy `[addons.ai-*.tools]` entries are accepted for compatibility, but fresh
scaffolding keeps AI configuration in the `[ai]` section.

## Addons and Skills

As of v0.16.0, all skills live in [processkit](https://github.com/projectious-work/processkit)
and **every project gets every skill installed** under `context/skills/`,
regardless of which addons are active. There is no longer an addon-driven
"auto-deploy a skill" mechanism, and no `[skills].include` to manage.

The relevant skills for each addon's tooling are still in the catalogue —
agents pick them up via skill descriptions, not via addon membership:

| Addon | Naturally relevant skills |
|-------|---------------------------|
| `python` | `python-best-practices`, `fastapi-patterns`, `pandas-polars` |
| `rust` | `rust-conventions`, `concurrency-patterns` |
| `go` | concurrency-patterns and the Go-flavoured patterns shipped upstream |
| `node` | `typescript-patterns`, `tailwind` |
| `latex` / `typst` | `documentation` |
| `git-ui` | `git-workflow` |
| `kubernetes` | `container-orchestration` |
| `infrastructure` | terraform-flavoured patterns shipped upstream |

See [Skills (via processkit)](../skills/index.md) for the full split.

## How Addons Work

When you run `aibox apply`, the CLI:

1. Reads `[addons]` from `aibox.toml`
2. Looks up each addon definition from YAML files in `~/.config/aibox/addons/`
3. Merges your tool selections with addon defaults
4. Generates Dockerfile builder stages (for heavy builds like Rust, LaTeX)
5. Generates runtime `RUN`/`COPY` commands
6. Builds the container image

Addons that need compilation (Rust, LaTeX, infrastructure, Kubernetes) use **multi-stage Docker builds** -- heavy compilation happens in isolated builder stages, and only the final binaries are copied into the runtime image.

### Addon Definition Format

Addon definitions are YAML files stored in `~/.config/aibox/addons/` with category subdirectories (`languages/`, `tools/`, `docs/`, `ai/`). They are installed automatically by the install script and updated when you upgrade aibox.

## Extra Packages

For one-off apt packages not covered by addons, use `extra_packages`:

```toml
[container]
extra_packages = ["universal-ctags", "graphviz", "postgresql-client"]
```

These are installed during `aibox apply` via the generated Dockerfile. They persist across container restarts but are reinstalled on image rebuild.

## Version Selection

Each tool in an addon has a curated list of supported versions. Use `aibox describe addon <name>` to see available versions:

```bash
$ aibox describe addon python
Add-on: python
Recipe version: 1.0.0

  TOOL      DEFAULT    VERSION  SUPPORTED
  python        yes       3.13  3.12, 3.13, 3.14
  uv            yes    0.11.15  0.7, 0.11.10, 0.11.11, 0.11.15
  poetry         no        2.0  1.8, 2.0
  pdm            no       2.22  2.22
```

Tools marked "DEFAULT: yes" are included automatically when you set the addon. Tools marked "no" must be explicitly listed in your `aibox.toml` to be installed.
