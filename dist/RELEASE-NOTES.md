# aibox v0.32.6 — 2026-08-16

**Summary:** This patch refreshes the v0.x toolchain catalog and makes Yazi's path-copy menu reliably reach the tmux and host clipboard.

## Changed

- Updated curated tool versions, including Go 1.26.6, Yazi 26.8.15, uv 0.12.5, Hugo 0.165.0, pnpm 11.22.0, PDM 2.28.1, Ansible 14.3.1, Helm 4.2.4, Tau 0.3.10, and Zensical 0.0.55.
- Updated the Rust dependency lockfile to the current compatible patch releases.

## Fixed

- Fixed `govulncheck` installation failures caused by the previous Go 1.26.5 pin.
- Fixed Yazi's `c p`, `c d`, `c f`, and `c n` actions so copied paths, directory names, filenames, and stems are sent through `aibox-copy` to both the tmux buffer and host clipboard.

## Upgrade notes

Upgrade the host CLI to v0.32.6 and run `aibox apply` to regenerate the container and managed Yazi configuration.

[v0.32.6]: https://github.com/projectious-work/aibox/compare/v0.32.5...v0.32.6
