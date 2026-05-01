---
apiVersion: processkit.projectious.work/v1
kind: LogEntry
metadata:
  id: LOG-20260501_0809-TidyWillow-implementation-pushed
  created: '2026-05-01T08:09:31+00:00'
spec:
  event_type: implementation.pushed
  timestamp: '2026-05-01T08:09:31+00:00'
  summary: Implemented and pushed image provenance policy preview projection
  actor: codex
  subject: 39c2d26
  subject_kind: commit
  details:
    branch: main
    remote: origin
    commit: 39c2d26
    scope:
    - added aibox describe image-provenance-policy in json/yaml/table formats
    - emits aibox.image-provenance-policy.v0-preview with GHCR image flavor, concrete
      tag or tag template, generated file paths, runtime version markers, selected
      addons, and release phase command templates
    - handles mutable version pin latest by leaving concrete tag null and exposing
      base-debian-v{version} template
    - documents the projection in CLI and configuration docs
    validation:
    - cargo test --manifest-path cli/Cargo.toml --bin aibox image_provenance
    - cargo check --manifest-path cli/Cargo.toml --bin aibox
    - cargo build --manifest-path cli/Cargo.toml --bin aibox
    - AIBOX_ADDONS_DIR=/workspace/addons cli/target/debug/aibox describe image-provenance-policy
      -o json
    - cargo test --manifest-path cli/Cargo.toml -j1
    - cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings
    - cargo fmt --manifest-path cli/Cargo.toml -- --check
    - git diff --check
---
