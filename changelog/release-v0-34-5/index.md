# v0.34.5 — consistent tmux window separators

> Restores PowerKit separator colors across all themes and makes visual regressions release-blocking.


The v0.34.5 patch restores the intended two-chevron PowerKit window layout:
each arrow now inherits the background of the segment it closes, without
leaking tmux format syntax or inherited dim styling. The regression coverage
runs against every bundled theme on isolated tmux servers and is now mandatory
for releases.

This release also refreshes the deferred Go, Rust, Bun, PDM, OpenTofu,
kubectl, Tau, Zensical, and Rust `cc` dependency pins.

After upgrading, run `aibox apply` to refresh the managed tmux configuration.

[Full v0.34.5 release notes](https://github.com/projectious-work/aibox/releases/tag/v0.34.5)


---
Source: https://projectious-work.github.io/aibox/changelog/release-v0-34-5/index.md
