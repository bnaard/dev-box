---
apiVersion: processkit.projectious.work/v2
kind: Migration
metadata:
  id: MIG-DISABLED-HARNESS-STATE
  created: 2026-05-13 13:12:37.530986+00:00
  updated: '2026-05-13T13:13:36+00:00'
spec:
  source: aibox
  source_api_version: processkit.projectious.work/v1
  source_processkit_version: unknown
  target_api_version: processkit.projectious.work/v2
  target_processkit_version: 0.25.13
  kind: runtime
  state: rejected
  apply_mode: one-shot
  generated_by: aibox apply
  generated_at: 2026-05-13 13:12:37.530986+00:00
  summary: Disabled AI-harness state cleanup requires owner review
  rejected_reason: This migration would delete local disabled-harness Claude runtime
    state under .aibox-home/.claude. The dogfood project has already rejected this
    cleanup as safer to retain; no destructive host cleanup applied.
  rejected_at: '2026-05-13T13:13:36+00:00'
---

# Migration: disabled AI-harness state cleanup

> **SAFETY: Do not execute host actions automatically.**
> **Discuss the cleanup with the project owner before applying it.**

**Status:** pending

## Summary

One or more AI harnesses that previously had state on the host are no longer listed in `[ai].harnesses`. Their `.aibox-home` config directories and MCP-registration files are still on disk.

`aibox apply` did NOT delete this state because `[apply].purge_disabled_harness_state` is `false` (the default).

## What would be removed

### claude (claude no longer enabled)

- `.aibox-home/.claude`

## How to apply this cleanup

1. Review the list above with the project owner.
2. Either:
   - re-enable the harness in `aibox.toml` if the removal was unintentional, OR
   - set `[apply].purge_disabled_harness_state = true` in `aibox.toml` and run `aibox apply` again.
3. Move this file to `context/migrations/applied/` once handled.
