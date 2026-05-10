---
apiVersion: processkit.projectious.work/v2
kind: Migration
metadata:
  id: disabled-harness-state-REJECTED
  created: '2026-05-09T12:41:00+00:00'
spec:
  source: aibox
  state: rejected
  rejected_reason: Trigger condition no longer holds — `[ai.harness.claude]` is enabled
    in aibox.toml. Likely emitted during a transient state when claude was briefly
    removed; aibox apply did not auto-clear when claude was re-added. Applying would
    purge `.aibox-home/.claude` (live state of the running session's harness).
  source_api_version: processkit.projectious.work/v1
  source_processkit_version: unknown
  target_api_version: processkit.projectious.work/v2
  target_processkit_version: 2.0.0-alpha.1
  apply_mode: one-shot
---

# Migration: disabled AI-harness state cleanup (REJECTED)

> **REJECTED 2026-05-09:** Trigger condition no longer holds — `[ai.harness.claude]` is enabled in aibox.toml. The migration was emitted during a transient state when claude was briefly removed; `aibox apply` did not auto-clear when claude was re-added. Applying would purge `.aibox-home/.claude`, which is the live state of the running session's harness. Kept here for audit trail.

## Original body (for reference)

> **SAFETY: Do not execute host actions automatically.**
> **Discuss the cleanup with the project owner before applying it.**

**Status:** rejected (was: pending)

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
