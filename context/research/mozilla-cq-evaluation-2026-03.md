# Mozilla cq Evaluation — March 2026

Evaluation of Mozilla.ai's cq ("colloquy") tool for potential integration with aibox. cq is a shared agent knowledge system — described as "Stack Overflow for AI agents" — that lets coding agents persist, share, and query collective knowledge to avoid rediscovering the same failures independently. Assessed against aibox's skill system, context architecture, and addon model.

---

## Sources

| # | URL | Topic |
|---|---|---|
| 1 | https://github.com/mozilla-ai/cq | GitHub repository |
| 2 | https://blog.mozilla.ai/cq-stack-overflow-for-agents/ | Mozilla.ai launch blog post |
| 3 | https://github.com/mozilla-ai/cq/blob/main/docs/architecture.md | Architecture documentation |
| 4 | https://github.com/mozilla-ai/cq/blob/main/docs/CQ-Proposal.md | Full proposal document |
| 5 | https://www.technology.org/2026/03/25/mozilla-ai-agents-share-knowledge-before-wasting-tokens-coding/ | Technology.org coverage |
| 6 | https://gigazine.net/gsc_news/en/20260325-cq-stack-overflow-for-ai/ | Gigazine coverage |
| 7 | https://winbuzzer.com/2026/03/25/mozilla-launches-cq-stack-overflow-for-ai-agents-xcxwbn/ | WinBuzzer coverage |

---

## 1. What Is cq

cq is an open standard and reference implementation for shared agent learning. The core idea: before an AI coding agent tackles unfamiliar work (an API integration, a CI/CD config, a framework quirk), it queries a shared knowledge commons. If another agent has already learned something — e.g., "Stripe returns 200 with an error body for rate-limited requests, not 429" — the querying agent knows this before writing a single line of code.

The name derives from "colloquy" (structured dialogue) and amateur radio's "CQ" general call ("any station, respond").

### Key Properties

- **Local-first**: Works offline with a private SQLite database; no server required for basic use
- **Three-tier knowledge model**: Local (private) -> Team/Org (shared within company) -> Global Commons (public, community-governed)
- **Model-agnostic**: Works with any LLM or agent framework
- **Open standard**: Apache 2.0 license, open protocol and formats
- **Knowledge unit lifecycle**: Pitfall -> Workaround -> Tool Recommendation -> Tool Gap Signal

---

## 2. Architecture and Technical Details

### Runtime Boundaries

| Component | Technology | Purpose |
|-----------|-----------|---------|
| Agent plugin | SKILL.md + hooks.json | Behavioral triggers, post-error auto-queries |
| Local MCP server | Python/FastMCP + SQLite | Knowledge storage, query/propose/confirm cycle |
| Team API | Python/FastAPI + Postgres (Docker) | Cross-agent shared knowledge within org |
| Global commons | Federated (design phase) | Public community-governed knowledge |

### MCP Tools Exposed

Six tools via stdio/MCP protocol:

- **`query`** — Search local, team, and global stores for relevant knowledge
- **`propose`** — Submit new knowledge units discovered during coding
- **`confirm`** — Validate existing knowledge (increases confidence score)
- **`flag`** — Report stale or incorrect knowledge
- **`reflect`** — Session mining: retrospectively analyze a coding session for shareable insights
- **`status`** — Display store statistics

### Knowledge Unit Schema

Structured JSON with:
- `insight` (summary / detail / action tripartite)
- `context` (language, frameworks, environment, pattern tags)
- `evidence` (severity, confidence 0.0-1.0, confirmation count, timestamps)
- `provenance` (proposer DID, graduation history with approver timestamps)
- `lifecycle` (status, kind, staleness policy, supersession relations)

### Guardrails

Integrated at ingestion, graduation, and retrieval:
- PII detection and prompt injection filtering
- Factual consistency and security checks at graduation
- Disputed/stale knowledge flagging at retrieval
- Uses Mozilla's `any-guardrail` model-agnostic framework

### Installation

```bash
# Claude Code plugin (primary method)
claude plugin marketplace add mozilla-ai/cq
claude plugin install cq

# Or via Agent Skills standard
npx skills add mozilla-ai/cq --skill cq

# Dependencies: uv (Python), jq (for OpenCode)
```

### Configuration

Three environment variables: `CQ_LOCAL_DB_PATH`, `CQ_TEAM_ADDR`, `CQ_TEAM_API_KEY`.

---

## 3. Repository Health

| Metric | Value | Assessment |
|--------|-------|-----------|
| Stars | ~750 | Strong launch signal for a 1-week-old project |
| Forks | 23 | Early community interest |
| Commits | 65 | Active development |
| Latest release | v0.4.0 (2026-03-23) | Rapid iteration |
| Open issues | 61 | Typical for early-stage exploratory project |
| License | Apache 2.0 | Fully compatible with aibox |
| Language | Python 74%, TypeScript 21% | Standard AI tooling stack |
| Status | Exploratory prototype | Not production-ready |
| Created | Early March 2026 | Less than 1 month old |
| Backing | Mozilla.ai | Credible organization, strong open-source track record |

---

## 4. Relationship to aibox

### 4.1 cq vs. aibox Skills

**Not overlapping. Complementary at a different layer.**

aibox skills (SKILL.md files) provide **procedural instructions** — how to manage a backlog, how to write decision records, how to do code review. They are executable behavioral specifications that tell an agent *how to perform a process*.

cq provides **declarative knowledge** — facts, pitfalls, workarounds, and tool recommendations that agents have learned empirically. It tells an agent *what to watch out for* when performing a task.

| Dimension | aibox skills | cq knowledge |
|-----------|-------------|--------------|
| Nature | Procedural ("how to do X") | Declarative ("X has pitfall Y") |
| Authorship | Human-curated, version-controlled | Agent-proposed, human-approved |
| Scope | Project/team process | Cross-project technical facts |
| Format | SKILL.md (YAML frontmatter + markdown) | Knowledge Unit JSON schema |
| Lifecycle | Versioned releases | Confidence-scored, graduated |
| Distribution | aibox skill registry | MCP server + tiered stores |

### 4.2 cq vs. aibox Context System

aibox's context system (CLAUDE.md, BACKLOG.md, DECISIONS.md, work-instructions/) stores **project-specific state and process declarations**. cq stores **cross-project technical knowledge** that is not tied to any single codebase.

The two systems address fundamentally different questions:
- aibox context: "What are *this project's* decisions, backlog, and conventions?"
- cq knowledge: "What have *all agents everywhere* learned about Stripe's API, Three.js r128, or Postgres timeout defaults?"

There is no meaningful overlap. cq does not replace or compete with any part of aibox's context directory.

### 4.3 cq vs. aibox Addons

aibox addons install **development tools** into the container image (Python, Node.js, Rust toolchains, linters, databases). cq is not a development tool in this sense — it is an MCP server that runs alongside the agent. It could be packaged as an addon (installing the cq MCP server into the container), but the fit is loose since cq's primary integration point is the agent plugin, not the container environment.

---

## 5. Integration Options

### Option A: aibox Addon (Container-Level)

Package cq's MCP server as an aibox addon that gets installed into the devcontainer. The addon would install `uv`, clone the cq repo, and configure the MCP server to start automatically.

**Pros**: Consistent with aibox's addon model; container-reproducible.
**Cons**: cq's primary value comes from the agent plugin (SKILL.md + hooks), not just the server. The addon alone would be incomplete — users would still need to install the Claude Code plugin separately. Addons are for container tooling, not agent behavior.

**Verdict**: Poor fit. An addon would only deliver half the value.

### Option B: aibox Skill (Agent-Level)

Create an aibox skill (`cq-integration`) that:
1. Instructs the agent to use the cq MCP server when available
2. Defines triggers for when to query cq (before API integrations, after errors, when using unfamiliar frameworks)
3. Defines when to propose knowledge back (after debugging sessions, discovering undocumented behavior)

This could wrap or complement cq's own SKILL.md with aibox-specific conventions (e.g., propose knowledge to cq when updating DECISIONS.md with a technical decision).

**Pros**: Natural fit with aibox's skill architecture. Skills are exactly where behavioral instructions live. Could add value beyond cq's own SKILL.md by integrating with aibox's process model.
**Cons**: Depends on cq being installed separately (either via their plugin or standalone MCP). Creates a dependency on an external, immature project.

**Verdict**: Best fit if we integrate at all. But premature given cq's maturity.

### Option C: Documentation / Recommendation Only

Document cq in aibox's docs as a compatible companion tool. Add a section to the docs-site explaining how to use cq alongside aibox. No code changes.

**Pros**: Zero maintenance burden. Users who want cq can install it independently. No coupling to an immature project.
**Cons**: No direct value-add from aibox.

**Verdict**: Appropriate for the current moment.

### Option D: Skip Entirely

Ignore cq for now and revisit when it matures.

**Pros**: No effort. No risk of coupling to a project that may pivot or stall.
**Cons**: Misses the signal that shared agent knowledge is an emerging category.

---

## 6. Value Assessment

### What cq Solves That aibox Does Not

aibox provides reproducible environments and structured AI context for *this project*. cq provides cross-project empirical knowledge. The gap cq fills:

1. **Avoiding known pitfalls**: An agent working in an aibox container on a Stripe integration would benefit from knowing Stripe's rate-limit response quirk — this is not something aibox's context system captures.
2. **Reducing wasted tokens**: Agents that check cq before attempting unfamiliar work avoid trial-and-error cycles that consume context window and compute.
3. **Team knowledge sharing**: Multiple aibox users within an organization could share agent-learned insights via cq's team tier, complementing aibox's per-project context with cross-project institutional knowledge.

### What Limits cq's Value Today

1. **Maturity**: Less than 1 month old, self-described as "exploratory prototype." The knowledge commons is empty or near-empty.
2. **Trust model unproven**: The three-tier graduation system with DIDs, reputation scoring, and anti-poisoning is ambitious but entirely theoretical at this stage.
3. **Quality risk**: AI agents are unreliable at accurately describing their own reasoning. A commons full of low-quality or incorrect knowledge entries could harm rather than help.
4. **Global commons chicken-and-egg**: The system's value scales with the amount of knowledge contributed. Until critical mass is reached, the query tool returns nothing useful.
5. **Python dependency**: cq requires `uv` and Python, which may not be present in all aibox container flavors (e.g., the Rust or LaTeX images).

---

## 7. Recommendation

**Short-term (now): Option C — Documentation only.**

- Add a brief mention in aibox docs that cq exists as a compatible companion tool for shared agent knowledge
- No code changes, no addon, no skill
- The project is too young (< 1 month, exploratory prototype) to warrant integration effort

**Medium-term (Q3 2026): Reassess for Option B (skill) if cq reaches v1.0.**

Conditions to trigger reassessment:
- cq reaches stable release (v1.0+)
- Global commons has meaningful knowledge density (>1000 confirmed knowledge units)
- The trust/graduation model is operational (not just designed)
- At least 2 other major agent platforms have adopted cq (beyond Claude Code)
- Mozilla.ai demonstrates sustained maintenance (not a one-off launch)

If those conditions are met, create an `cq-integration` skill that bridges aibox's process model with cq's knowledge model — e.g., automatically querying cq when the agent encounters framework-specific work, and proposing knowledge to cq when the agent discovers novel patterns during aibox-managed projects.

**What to watch**: cq's most interesting idea is not the tool itself but the *category* it represents — shared agent knowledge as an open standard. If cq fails but the category succeeds (e.g., another tool fills this niche), aibox should integrate the winner. Track the category, not just the project.

---

## 8. Summary

| Question | Answer |
|----------|--------|
| What is cq? | Open standard for shared agent knowledge (pitfalls, workarounds, tool recommendations) |
| Does it compete with aibox? | No — complementary. Different layer entirely. |
| Does it overlap with aibox skills? | No — skills are procedural; cq is declarative knowledge |
| Does it overlap with aibox context? | No — context is project-specific; cq is cross-project |
| Best integration path | aibox skill (Option B), when cq matures |
| Should we integrate now? | No — too immature. Document only. |
| Should we track it? | Yes — the category (shared agent knowledge) is significant |
