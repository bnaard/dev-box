# aibox v0.29.0 — 2026-08-01

**Summary:** This minor release adds Tau as a first-class coding-agent harness for users who want its readable, provider-neutral terminal workflow inside an aibox dev container. Enable the new tau harness and rebuild the container; existing configurations remain compatible.

## Added

- Add tau to the supported AI harness catalog and generated aibox.toml configuration.
- Install the pinned tau-ai 0.3.5 package through the shared uv tool environment and persist Tau state under ~/.tau.
- Project processkit command adapters into Tau's Agent Skills-compatible .agents/skills/ directory.
- Add Tau to terminal profiles, tmux layout coverage, addon publication, release pin validation, and user documentation.

## Changed

- Provider-backend reporting now states that Tau loads root AGENTS.md instructions and Agent Skills but does not currently provide a built-in MCP client.

## Upgrade notes

Add { harness = "tau", enable = true, install = true } to [ai].harnesses, then run aibox apply.

[v0.29.0]: https://github.com/projectious-work/aibox/compare/v0.28.19...v0.29.0
