# aibox v0.28.19 — 2026-07-31

**Summary:** This patch restores `aibox apply` for projects that track the latest published image after the v1 alpha appeared. Users can upgrade normally; no configuration change is required.

## Fixed

- Preserve the complete published image SemVer, including prerelease identifiers, so `v1.0.0-alpha.1` resolves to its real GHCR runtime tag instead of the nonexistent `v1.0.0` tag.
- Add regression coverage for exact prerelease image-tag generation.

## Upgrade notes

Upgrade the host CLI to v0.28.19 and rerun `aibox apply`.

[v0.28.19]: https://github.com/projectious-work/aibox/compare/v0.28.18...v0.28.19
