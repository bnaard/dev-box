# AI Provider Audit Logging Capabilities

**Status:** Research complete
**Date:** 2026-03-28
**Relates to:** DISC-001, event-log-management skill design
**Purpose:** Determine whether AI coding assistants provide deterministic audit logging
that aibox could integrate with for hybrid (deterministic + probabilistic) event capture.

---

## Key Finding

Most major AI coding providers offer some form of audit logging, but capabilities vary
dramatically. Two clear patterns emerge:

1. **Hooks/OTel providers** (Claude Code, Gemini CLI, Codex CLI): Full event capture
   including prompts, responses, tool calls, file modifications. Can push to external
   destinations via webhooks or OpenTelemetry backends.

2. **Enterprise-only providers** (GitHub Copilot, Cursor, Windsurf, Amazon Q): Audit
   logging gated behind enterprise tiers. Often exclude prompt/response content.

## Comparison Table

| Provider | Audit Logging | Webhook/External | Captures Prompts | Captures Responses | Captures Tool Calls | All Tiers? |
|---|---|---|---|---|---|---|
| **Claude Code** | Yes (hooks + JSONL) | Yes (HTTP hooks) | Yes | Yes | Yes (21 events) | Yes |
| **Gemini CLI** | Yes (OTel + Cloud Audit) | Yes (any OTel backend) | Yes | Yes | Yes | Yes |
| **Codex CLI** | Yes (OTel) | Yes (any OTel backend) | Yes | Yes | Yes | Yes |
| **Amazon Q Dev** | Yes (CloudTrail) | Yes (EventBridge) | No (hidden) | No (hidden) | Yes | Pro only |
| **GitHub Copilot** | Yes (enterprise) | No native webhook | No (excluded) | No (excluded) | Yes (agent sessions) | Enterprise only |
| **Windsurf** | Yes (enterprise DB) | Enterprise self-hosted | Yes | Yes | Unknown | Enterprise only |
| **Aider** | Partial (file-based) | No | Yes (.md history) | Yes | Partial (git commits) | Yes (OSS) |
| **Cursor** | Limited (enterprise) | No | No | No | No | Enterprise only |
| **Continue.dev** | Partial (PostHog) | No | Yes (internal logs) | Yes | No | Enterprise policies |

## Implication for aibox

**Three logging channels available:**

1. **Provider hooks (deterministic):** Claude Code hooks, Gemini OTel, Codex OTel can
   capture every interaction automatically. This is deterministic — if configured, it
   always logs. Configuration goes in aibox.toml or the provider's settings.

2. **Agent event-log skill (probabilistic):** The agent uses the event-log-management
   skill to record process events (state changes, decisions, gate checks). This is
   probabilistic — the agent should always do it, but it's not guaranteed.

3. **aibox sync infrastructure events (deterministic):** `aibox sync` and `aibox lint`
   record infrastructure events (inconsistencies, validation results).

**Recommended integration:**

aibox should support an optional `[audit]` section in `aibox.toml`:

```toml
[audit]
# Provider-level deterministic logging (optional)
provider_hooks = true          # enable provider audit hooks if available
provider_destination = "context/audit/"  # where to store provider logs
# or: provider_webhook = "https://your-logging-endpoint.com/ingest"

# Process-level probabilistic logging (always active via skills)
event_log = "context/events/"  # JSONL event log location

# Infrastructure-level deterministic logging (always active)
sync_log = "context/events/"   # aibox sync/lint events go here too
```

When `provider_hooks = true`, `aibox init` configures the active provider's hooks to
capture tool calls and file modifications. This creates a deterministic audit trail
alongside the probabilistic process event log.

**The hybrid model:**
- Provider hooks capture WHAT happened (every tool call, every file edit) — deterministic
- Agent event-log captures WHY it happened (state changes, decisions, rationale) — probabilistic
- Together they provide complete audit coverage: the "what" is guaranteed, the "why" is best-effort

## Emerging Standards

- **OpenTelemetry GenAI semantic conventions** — emerging standard for AI agent telemetry.
  Gemini CLI and Codex CLI already use it. Could become the universal format.
- **No universal standard yet** for AI coding assistant audit log format.
- **Third-party governance middleware** (MintMCP Gateway, Oasis) emerging as tool-agnostic
  audit layers.
