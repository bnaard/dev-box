---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260424_0058-ToughGrove-feature-global-mcp-permissions
  created: '2026-04-24T00:58:47+00:00'
  updated: '2026-04-24T11:26:26+00:00'
spec:
  title: 'Feature: Global MCP permissions allow/deny list in aibox.toml'
  state: done
  type: story
  priority: medium
  description: |
    **Problem:** Claude Code and other harnesses continuously prompt for permission to use processkit MCP servers, despite these being aibox-shipped, trusted content. Users must manually add ~26 individual MCP tool permissions and repeat for each harness.

    **Scope:** Implement provider-independent MCP permission configuration in aibox.toml to eliminate redundant prompts across all supported harnesses.

    ## Comprehensive Harness Analysis (8 Harnesses)

    | Harness | Permission Model | Format | Key Characteristics |
    | :-- | :-- | :-- | :-- |
    | **Claude Code** | Simple allowlist | `permissions.allow[]` in `.claude/settings.local.json` | Individual entries (e.g. `mcp__processkit-workitem-management__create_workitem`); no pattern support |
    | **OpenCode** | 3-state + wildcards | `allow/ask/deny` in `.opencode/config.toml` | Supports `mcp__processkit-*` glob wildcards; per-agent overrides; first-match-wins semantics |
    | **Continue IDE** | Ask-first default | Per-tool mode in `continue/config.json` | Default "Ask" mode; "Automatic" mode for tool agents; safety-first design |
    | **Cursor IDE** | Per-tool approval | `allowedMcpServers[]` in `.cursor/settings.json` | Prompts on each tool use; `autoApprove` option available (note: trusts entry names, not validated commands) |
    | **Gemini CLI** | Dual allowlist + blocklist | `includeTools[]` / `excludeTools[]` in config | Principle of least privilege emphasis; intersection semantics (both must match) |
    | **GitHub Copilot** | CLI flags + enterprise | `--allow-tool`, `--deny-tool`, `--available-tools` | Enterprise registry-based control; command-line flag overrides; policy-scoped |
    | **Aider** | Fine-grained per-tool | Per-tool agent permissions | Strict principle of least privilege; tool-level control for each agent |
    | **Codex** | Trust level (simple) | `trust_level = "trusted"` in `.codex/config.toml` | Project-scoped; no per-tool granularity; simplest model but least flexible |

    **Current State:**
    - aibox seeds keybindings.json but NOT settings.local.json with MCP permissions
    - Manual workaround requires: 26 individual `mcp__processkit-*` entries in Claude Code + 1 Bash fallback
    - Each harness requires separate manual config across 8+ supported tools
    - Users face repeated permission prompts despite aibox-shipped, trusted content
    - No provider-independent configuration point exists

    ## Proposed 3-Tier Architecture

    ### Tier 1: Global Defaults
    ```toml
    [mcp]
    default_mode = "allow"  # "allow", "ask", "deny"
    allow_patterns = ["mcp__processkit-*", "bash"]
    deny_patterns = []
    ```

    ### Tier 2: Pattern-Based Allow/Deny Lists
    - Glob-style pattern matching (first-match-wins semantics)
    - Override global defaults for specific tool families
    - Support both `mcp__*` pattern-based servers and Bash hook fallback

    ### Tier 3: Per-Harness Overrides
    ```toml
    [mcp.harness."claude-code"]
    enabled = true
    mode = "allow"  # Override global default if needed
    extra_patterns = []  # Add harness-specific patterns

    [mcp.harness."codex"]
    enabled = true
    trust_level = "trusted"  # Codex uses trust_level instead of allow lists

    [mcp.harness."opencode"]
    enabled = true
    mode = "allow"
    deny_patterns = []  # Can restrict specific tools per harness
    ```

    ## Implementation Strategy (4 Phases)

    ### Phase 1: Core Infrastructure (seed.rs)
    - Add `struct McpConfig` with allow_patterns, deny_patterns, default_mode
    - Implement glob pattern matching and expansion logic
    - Create `expand_mcp_patterns()` function: takes pattern list, returns concrete tool names
    - First-match-wins semantics for overlapping patterns

    ### Phase 2: Per-Harness Generators
    Create one generator function per harness:
    - `generate_claude_code_permissions()` → `.claude/settings.local.json` update
    - `generate_opencode_permissions()` → `.opencode/config.toml` generation
    - `generate_continue_permissions()` → `continue/config.json` tool modes
    - `generate_cursor_permissions()` → `.cursor/settings.json` generation
    - `generate_gemini_permissions()` → Dual allow/deny lists
    - `generate_github_copilot_flags()` → CLI flags and policies
    - `generate_aider_permissions()` → Per-tool agent config
    - `generate_codex_permissions()` → `trust_level` + fallback allow list

    ### Phase 3: Integration
    - Call all generators from container seed lifecycle
    - Merge with existing harness configs (never overwrite)
    - Log warnings if patterns don't expand to any tools
    - Preserve user edits: only update generated sections

    ### Phase 4: Documentation
    - Update aibox.toml reference docs with `[mcp]` section
    - Update AGENTS.md with per-harness permission semantics
    - Add troubleshooting guide: "Why am I still seeing permission prompts?"
    - Document per-harness override patterns

    ## Security Considerations

    - **Principle of Least Privilege:** Default to `deny` mode if not specified
    - **Pattern Validation:** Warn if patterns match zero tools (typo detection)
    - **Aibox Content Trust:** Document that aibox-shipped MCP tools are trusted (no additional validation needed)
    - **User Override:** Users can always restrict further with deny_patterns or per-harness overrides
    - **Future-Proof:** Design supports adding new harnesses without code changes (config-driven)

    ## Benefits

    - Eliminates ~26 permission prompts per harness per session
    - One central configuration point (aibox.toml) for all harnesses
    - Provider-independent design: works across Claude Code, OpenCode, Continue, Cursor, Gemini, GitHub Copilot, Aider, Codex
    - Reduces user friction without security loss (aibox content is trusted)
    - Pattern-based configuration supports future tool additions without manual edits
    - Per-harness overrides allow fine-grained control where needed
    - Backward compatible: default allows all processkit tools, user can restrict further

    ## Related Decisions
    - See DEC-20260424_0052-ToughGrove (if created) for architecture approval
  started_at: '2026-04-24T11:26:19+00:00'
  completed_at: '2026-04-24T11:26:26+00:00'
---

## Transition note (2026-04-24T11:26:23+00:00)

All 4 phases complete: Phase 1 core infrastructure (McpConfig, pattern matching), Phase 2 (8 harness generators), Phase 3 (integration into seed.rs), Phase 4 (documentation). Feature delivers global MCP permission configuration across 8 supported harnesses with provider-independent [mcp.permissions] section in aibox.toml.
