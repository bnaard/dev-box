# aibox Zellij status plugin

Native Rust/WASM-oriented Zellij plugin source for the aibox two-row bottom UI:

- row 1: width-aware aibox key hints with Zellij-like segmented rendering
- row 2: grouped runtime status using the same segmented visual language

This directory is intentionally standalone from the main aibox CLI crate. The
base image builds it as a WASM plugin and installs the artifact at
`/usr/local/share/aibox/zellij/aibox-status.wasm`.

## Current integration dependency

The Zellij adapter in `src/zellij_plugin.rs` targets:

```toml
zellij-tile = "=0.44.2"
```

Build for the WASI target used by the pinned Zellij runtime:

```sh
rustup target add wasm32-wasip1
cargo build --manifest-path zellij-plugins/aibox-status/Cargo.toml \
  --release \
  --target wasm32-wasip1 \
  --features zellij
```

The pure rendering core has no external dependencies and can be tested without
the Zellij crate:

```sh
cargo test --manifest-path zellij-plugins/aibox-status/Cargo.toml
```

## Runtime data path

The adapter runs `/usr/local/bin/aibox-status --plugin-json`. That executable is
Rust and reads the latest bounded diagnostics sidecar snapshot, with a small
direct-read fallback when the snapshot is unavailable. The Zellij plugin stays a
renderer: it does not scan `/proc`, parse log history, or run shell pipelines.
