---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260517_1026-StableOtter-ai-execution-policy-schema
  created: '2026-05-17T10:26:45+00:00'
  labels:
    area: cli-config
    harnesses:
    - codex
    - claude
    - aider
    - opencode
    release: post-v0.26.6
  updated: '2026-05-17T10:46:26+00:00'
spec:
  title: Implement AI execution policy schema for harness sandbox settings
  state: done
  type: story
  priority: high
  description: 'Accepted design: add stable aibox AI execution policy vocabulary with
    global defaults and per-harness overrides using filesystem/approval/network axes.
    Map Codex filesystem=container-full to sandbox_mode="danger-full-access" so trusted
    devcontainers can keep .git writable inside Codex tool execution, while keeping
    MCP permission vocabulary separate from execution policy.'
  started_at: '2026-05-17T10:45:36+00:00'
  completed_at: '2026-05-17T10:46:26+00:00'
---

## Transition note (2026-05-17T10:45:36+00:00)

Implementation started after accepted design. Plan: split parser/schema work to team worker, implement projection/rendering/docs locally, verify with focused tests plus cargo check/clippy.


## Transition note (2026-05-17T10:46:21+00:00)

Implementation and verification complete; moving through required review state before terminal close.


## Transition note (2026-05-17T10:46:26+00:00)

Closed after verification. Tests passed: cargo fmt --all --check, cargo check, cargo clippy --all-targets -- -D warnings, cargo test, git diff --check.
