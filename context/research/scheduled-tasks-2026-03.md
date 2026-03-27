# Scheduled / Recurring Tasks for aibox -- Research Report

**Date:** 2026-03-26
**Status:** Draft

---

## 1. Problem Statement

aibox provides reproducible containerized development environments with built-in AI context
structure. Users increasingly want AI agents to perform work autonomously on a schedule --
nightly code audits, daily standup generation, periodic dependency updates -- without manual
invocation. Today, aibox has no scheduling primitive. This report surveys the landscape of
AI agent scheduling, evaluates design options, and recommends a practical approach that
preserves aibox's provider-independence principle.

---

## 2. Landscape Survey

### 2.1 Claude Code -- Headless Mode and Scheduled Tasks

Claude Code supports non-interactive execution via the `-p` (print) flag, which runs without
a TTY and is suitable for cron or CI integration. Key facts:

- **Headless mode:** `claude -p "audit this codebase for security issues"` runs a one-shot
  task and exits. Uses ~512 MB (vs 1-2 GB for the interactive TUI).
- **Desktop scheduled tasks:** Claude Code Desktop (macOS/Windows only) supports persistent
  scheduled tasks that survive restarts. Not available on Linux.
- **CLI `/loop` command:** Session-scoped scheduling that runs a prompt repeatedly at an
  interval. Expires after 3 days if the Desktop app is not running.
- **Background Agents:** Available on all paid plans. Cloud-based execution of tasks without
  requiring a local machine.
- **Linux pattern:** Set up standard cron jobs that invoke `claude -p` in headless mode.
  This is the recommended approach for server/container environments.

**Relevance to aibox:** The `-p` flag is the key integration point. aibox can invoke any
provider's CLI in non-interactive mode from a scheduler. Claude Code does not provide its
own cross-platform persistent scheduler -- it relies on OS-level cron or its Desktop app.

Sources:
- [Run Claude Code programmatically](https://code.claude.com/docs/en/headless)
- [Claude Code Scheduled Tasks Guide](https://claudefa.st/blog/guide/development/scheduled-tasks)
- [runCLAUDErun - Scheduler for Claude Code on macOS](https://runclauderun.com/)

### 2.2 Aider -- Watch Mode (File-Based Triggering)

Aider uses a file-watching model rather than time-based scheduling:

- **`--watch-files` flag:** Monitors all repo files for special AI comment markers.
- **AI comment syntax:** One-liner comments starting or ending with `AI!` (execute) or
  `AI?` (answer). Example: `// AI! refactor this function to use async/await`
- **Processing:** When a marker is detected, Aider collects all AI comments as instructions
  and executes them. Files larger than 1 MB are ignored; gitignore patterns are respected.
- **No time-based scheduling:** Aider has no built-in cron or interval capability. Watch
  mode is event-driven (file change), not time-driven.

**Relevance to aibox:** The file-watching pattern is complementary to cron-based scheduling.
aibox could support both: time-based triggers (cron) and event-based triggers (file watch).
The AI comment marker pattern is provider-specific to Aider and not portable.

Sources:
- [Aider Watch Mode Documentation](https://aider.chat/docs/usage/watch.html)
- [File Watching and AI Comments (DeepWiki)](https://deepwiki.com/Aider-AI/aider/4.2-file-watching-and-ai-comments)

### 2.3 GitHub Agentic Workflows

GitHub launched Agentic Workflows in technical preview (February 2026):

- **Markdown-based workflow definition:** Instead of YAML, workflows are written in plain
  Markdown placed in `.github/workflows/`. Natural language task descriptions.
- **`gh aw` CLI:** Converts Markdown workflows into GitHub Actions `.lock.yml` files.
- **Trigger types:** Issues, PRs, pushes, comments, manual dispatch, and **schedules**
  (cron expressions with timezone support as of March 2026).
- **Provider-agnostic:** Supports GitHub Copilot (default), Claude, and Codex as execution
  engines. The AI agent runs in a containerized GitHub Actions runner.
- **Security model:** Read-only permissions by default, tool allowlists, output sanitization
  layer before applying changes. Write operations require explicit permission grants.
- **Status:** Technical preview. May change significantly. Requires careful human supervision.

**Relevance to aibox:** GitHub Agentic Workflows demonstrate a compelling pattern: declarative
task definitions (Markdown), standard scheduling (cron), provider-agnostic execution. However,
they are tightly coupled to GitHub Actions infrastructure. aibox needs a self-contained
solution that works outside GitHub.

Sources:
- [GitHub Agentic Workflows Documentation](https://github.github.com/gh-aw/)
- [GitHub Blog: Automate repository tasks](https://github.blog/ai-and-ml/automate-repository-tasks-with-github-agentic-workflows/)
- [How They Work](https://github.github.com/gh-aw/introduction/how-they-work/)
- [GitHub Actions March 2026: timezone support for cron](https://github.blog/changelog/2026-03-19-github-actions-late-march-2026-updates/)

### 2.4 OpenAI Codex -- Automations

The Codex app (desktop + cloud) has first-class automation support:

- **Automations:** Defined by frequency/trigger, instructions (prompt or skill), and an
  optional agent personality. Results land in a "review queue" tab.
- **Use cases at OpenAI:** Daily issue triage, CI failure summaries, daily release briefs,
  bug scanning.
- **Local vs cloud execution:** Currently, automations run on the developer's machine at
  scheduled times (app must be running, project on disk). OpenAI is extending this to
  cloud-based scheduling (no local machine needed).
- **Worktree support:** In git repos, automations can run on the local project or on a
  new worktree to avoid conflicts with active development.
- **CLI non-interactive mode:** `codex -p "prompt"` for headless execution, similar to
  Claude's `-p` flag.

**Relevance to aibox:** Codex Automations confirm the pattern: define task + schedule,
execute via CLI, collect results. The local-vs-cloud distinction is important -- aibox
containers are always running, so local execution is natural. The worktree pattern is
valuable for tasks that modify code.

Sources:
- [Codex Automations Documentation](https://developers.openai.com/codex/app/automations)
- [Codex Non-Interactive Mode](https://developers.openai.com/codex/noninteractive)
- [Introducing the Codex App](https://openai.com/index/introducing-the-codex-app/)

### 2.5 Cron-Based AI Agent Patterns

The broader ecosystem has converged on several scheduling patterns for AI agents:

**Pattern 1: Standard Cron**
Time-based triggers (`0 2 * * *`). Best for daily reports, cleanup tasks, audits. Simple,
well-understood, no special infrastructure. The agent process starts fresh each run.

**Pattern 2: Interval Loop**
Agent runs every X minutes/hours. Common for monitoring. Risk of drift if previous job
runs long. Simpler than cron but less precise.

**Pattern 3: Event-Driven Triggers**
Execution in response to external events (file upload, webhook, git push). Not time-based
but complementary to cron.

**Pattern 4: Adaptive Scheduling**
The agent sets its own next wakeup time. Efficient for variable workloads. Agent writes a
`wake_up_time` to a state file; a master scheduler reads it.

**Pattern 5: Heartbeat**
Periodic poll (e.g., every 30 minutes). Lightweight check: "anything need attention?"
Agent checks a task file, glances at recent context, acts or stays silent. Good for
batching -- one heartbeat can check multiple conditions.

**Cron vs Heartbeat decision framework:**
- Use **cron** when exact timing matters, tasks are self-contained, and isolation is needed.
- Use **heartbeat** when tasks benefit from conversational context, batching, or flexibility.
- Most production systems use a mix of both.

**Key production concerns:**
- Timeouts are mandatory (LLMs can hang). Always wrap scheduled jobs in strict timeouts.
- AI agents are stateful -- they often need to "remember" what they did for the next run.
  State persistence between runs is a design requirement.
- Cost control: heartbeats that always invoke an LLM are expensive. "Cheap checks first,
  model only when needed" is the recommended pattern.

Sources:
- [Heartbeats vs Cron: Two Patterns](https://dev.to/ryancwynar/heartbeats-vs-cron-two-patterns-for-scheduling-autonomous-ai-work-1l0)
- [AI Agent Job Scheduling: Best Patterns 2026](https://fast.io/resources/ai-agent-job-scheduling/)
- [Seven Hosting Patterns for AI Agents](https://james-carr.org/posts/2026-03-01-agent-hosting-patterns/)
- [How We Built a CRON Scheduler for AI Agents at Scale](https://blog.geta.team/how-we-built-a-cron-scheduler-for-ai-agents-at-scale/)

### 2.6 MCP (Model Context Protocol) -- Scheduling Status

MCP does not currently have scheduling or trigger concepts in the specification:

- **2026 roadmap:** Triggers and event-driven updates are listed in the "On the Horizon"
  section -- recognized as valuable but not among the four priority areas (transport
  scalability, agent communication, governance, enterprise readiness).
- **Current state:** Clients learn about server-side changes by polling or holding SSE
  connections. No standardized callback/webhook mechanism.
- **Future direction:** The MCP team invites community-formed Working Groups to lead work
  on triggers and event-driven updates through SEPs (Spec Enhancement Proposals).

**Relevance to aibox:** MCP will not provide scheduling primitives in the near term. aibox
should not wait for MCP triggers. However, MCP tools could be *invoked* by scheduled tasks
(e.g., a cron job that calls an MCP server to fetch data, then passes it to an AI agent).

Sources:
- [The 2026 MCP Roadmap](http://blog.modelcontextprotocol.io/posts/2026-mcp-roadmap/)
- [MCP Roadmap - Official](https://modelcontextprotocol.io/development/roadmap)

---

## 3. Use Cases

### 3.1 Scheduled Code Audits
Run nightly, report findings. Example: `claude -p "review src/ for security issues, write
findings to context/audit-$(date +%F).md"` triggered by a 2 AM cron job. Results persist
in the context directory for human review.

### 3.2 Recurring Test Runs with AI Analysis
After nightly test suite execution, pipe test output to an AI agent for failure analysis
and suggested fixes. Two-step: `cargo test 2>&1 | claude -p "analyze these test failures"`.

### 3.3 Daily Project Summaries (Standup Generation)
Every morning at 8 AM, generate a standup from git log, open issues, and context files.
Writes to `context/STANDUPS.md`. Fits naturally with aibox's existing standup context
package.

### 3.4 Periodic Dependency Updates
Weekly: check for outdated dependencies, create a branch with updates, run tests, report
results. Requires git worktree support to avoid disrupting active development.

### 3.5 Overnight Refactoring Tasks
Long-running refactoring assigned before end-of-day, executed overnight. Requires timeout
guards, worktree isolation, and result review queue.

### 3.6 Scheduled Documentation Review
Weekly review of docs for staleness, broken links, missing coverage. Writes a report to
the context directory.

---

## 4. Design Options

### Option A: CLI Command

```
aibox schedule "run /audit every day at 2am"
aibox schedule list
aibox schedule remove <id>
```

**How it works:** Natural language parsed into a cron expression. Creates a crontab entry
or systemd timer inside the container. The scheduled job runs `claude -p` (or whichever
AI provider is configured) with the specified prompt/skill.

| Criterion | Assessment |
|---|---|
| Provider independence | Good -- invokes whichever provider CLI is installed |
| Container lifecycle | Weak -- cron/systemd state lost on container rebuild; must persist schedules to a file and restore on start |
| Security | Medium -- runs as container user; inherits all container permissions and API keys |
| Observability | Medium -- cron logs to syslog; needs structured output redirection |

**Pros:** Intuitive UX. Natural language scheduling is user-friendly.
**Cons:** NLP parsing is fragile and provider-dependent. Container lifecycle is the biggest
weakness -- schedules must survive `aibox sync` / container rebuild.

### Option B: Config in aibox.toml

```toml
[[schedule]]
name = "nightly-audit"
cron = "0 2 * * *"
command = "/audit"
provider = "claude"       # optional, defaults to first configured provider
timeout = "10m"
workdir = "worktree"      # "local" or "worktree"

[[schedule]]
name = "daily-standup"
cron = "0 8 * * 1-5"
command = "generate standup from git log and open issues"
output = "context/STANDUPS.md"
```

**How it works:** `aibox sync` reads `[[schedule]]` entries and generates crontab entries
or systemd timer units. Schedules are declarative, version-controlled, and survive container
rebuilds because they are regenerated from config on every sync.

| Criterion | Assessment |
|---|---|
| Provider independence | Excellent -- `provider` field selects runtime; defaults to first in `[ai].providers` |
| Container lifecycle | Excellent -- schedules live in aibox.toml (host-mounted); regenerated on sync/rebuild |
| Security | Medium -- same as Option A; API keys must be available in the container environment |
| Observability | Good -- can generate structured log output; integrate with event log (BACK-073) |

**Pros:** Declarative, reproducible, version-controlled. Survives container lifecycle.
Natural fit with aibox's existing config-driven model. Team members share the same schedules.
**Cons:** Less flexible than imperative CLI for ad-hoc scheduling. Requires `aibox sync`
after config changes.

### Option C: Skill-Based (Teach Agents to Use System Cron)

Provide a skill (like the existing skills in `templates/skills/`) that teaches AI agents
how to create and manage cron jobs or systemd timers inside the container.

**How it works:** The skill's SKILL.md contains reference material on crontab syntax,
systemd timer units, and best practices. The AI agent, when asked to schedule a task,
creates the appropriate system configuration itself.

| Criterion | Assessment |
|---|---|
| Provider independence | Good -- any provider can read the skill |
| Container lifecycle | Poor -- agent-created cron jobs are not tracked; lost on rebuild |
| Security | Poor -- agent has direct access to scheduling primitives; risk of runaway tasks |
| Observability | Poor -- no centralized tracking of what is scheduled |

**Pros:** Zero implementation cost. Leverages existing skill infrastructure.
**Cons:** Fragile, non-reproducible, invisible. The opposite of aibox's declarative
philosophy. Agent might create conflicting or redundant schedules.

### Option D: Integration with Provider-Native Scheduling

Delegate scheduling to the AI provider's own infrastructure:
- Claude Code Background Agents (cloud)
- Codex Automations (desktop/cloud)
- GitHub Agentic Workflows (GitHub Actions)

**How it works:** aibox generates provider-specific configuration files. For GitHub Agentic
Workflows, it would create `.github/workflows/*.md` files. For Codex, it would configure
automations via the Codex app.

| Criterion | Assessment |
|---|---|
| Provider independence | Poor -- each provider has a different mechanism; tight coupling |
| Container lifecycle | N/A -- scheduling runs outside the container |
| Security | Varies -- depends on provider's security model |
| Observability | Good -- providers have built-in dashboards and notification systems |

**Pros:** Leverages battle-tested infrastructure. No scheduler to maintain.
**Cons:** Violates aibox's provider-independence principle. Requires accounts and
subscriptions. Fragments scheduling across multiple systems if user has multiple providers.

### Option E: Companion Scheduler Container

A lightweight sidecar container (analogous to the preview companion in PROJ-004) that runs
a scheduler process and executes tasks by invoking the main container via `docker exec`.

**How it works:** A minimal container (alpine + crond or Go/Rust binary) reads schedule
definitions from a shared volume (aibox.toml or a generated schedule file). On each
trigger, it runs `docker exec <main-container> claude -p "..."` or equivalent.

| Criterion | Assessment |
|---|---|
| Provider independence | Good -- executes via docker exec into the main container |
| Container lifecycle | Good -- scheduler container is independent; main container can be rebuilt without losing schedules |
| Security | Good -- scheduler has limited permissions; can restrict to docker exec only |
| Observability | Good -- dedicated log stream; can integrate with notification systems |

**Pros:** Clean separation of concerns. Scheduler survives main container rebuilds.
Can run even when the main container is being rebuilt (queues tasks).
**Cons:** Adds operational complexity (another container to manage). Docker-in-Docker or
Docker socket access required for `docker exec`. Overkill for simple use cases.

---

## 5. Comparison Matrix

| Criterion | A: CLI | B: Config | C: Skill | D: Integration | E: Companion |
|---|---|---|---|---|---|
| Provider independence | Good | Excellent | Good | Poor | Good |
| Container lifecycle | Weak | Excellent | Poor | N/A | Good |
| Security | Medium | Medium | Poor | Varies | Good |
| Observability | Medium | Good | Poor | Good | Good |
| Implementation cost | Medium | Medium | Low | High | High |
| User experience | Good | Good | Poor | Medium | Medium |
| Reproducibility | Poor | Excellent | Poor | Medium | Good |
| Team sharing | Poor | Excellent | Poor | Medium | Good |

---

## 6. Recommendation

**Primary: Option B (Config in aibox.toml) with Option A as a convenience layer.**

The reasoning:

1. **Config-driven scheduling (Option B) is the natural fit for aibox.** The entire tool
   is built on the principle that `aibox.toml` is the single source of truth. Schedules
   belong there alongside ports, addons, skills, and context packages. `aibox sync`
   already regenerates `.devcontainer/` from config -- adding crontab/timer generation is
   a natural extension.

2. **Provider independence is achieved through indirection.** The schedule config specifies
   *what* to run, not *which AI*. A `command` field contains the prompt or skill reference;
   the runtime resolves it to `claude -p`, `codex -p`, `aider --message`, or whichever
   provider is configured. A `provider` override field allows per-task provider selection.

3. **Container lifecycle is solved by regeneration.** Because schedules are declared in
   `aibox.toml` (which lives on the host-mounted workspace), they survive container
   rebuilds. `aibox sync` regenerates crontab entries every time. No state loss.

4. **CLI convenience (Option A) as sugar.** `aibox schedule add` could be a thin wrapper
   that appends a `[[schedule]]` entry to `aibox.toml` and runs `aibox sync`. This gives
   users both the declarative config path and the imperative CLI path.

### Implementation Sketch

**Phase 1: Core scheduling**
- Add `[[schedule]]` table array to aibox.toml schema.
- Fields: `name`, `cron`, `command`, `provider` (optional), `timeout` (optional, default
  5m), `workdir` (optional: `local` | `worktree`), `output` (optional: path for results).
- `aibox sync` generates a crontab file at `.devcontainer/crontab` and a
  `postCreateCommand` step that installs it.
- Execution wrapper script: handles timeout enforcement, output capture, exit code logging.

**Phase 2: Observability**
- Each scheduled run appends to a structured log (JSON lines) at
  `context/.schedule-log.jsonl` with timestamp, task name, exit code, duration, output path.
- `aibox schedule status` reads the log and displays recent run history.
- Integration with the event log system (BACK-073) if implemented.

**Phase 3: CLI convenience**
- `aibox schedule add --name "nightly-audit" --cron "0 2 * * *" --command "/audit"`
- `aibox schedule list` -- reads `[[schedule]]` from aibox.toml.
- `aibox schedule remove <name>` -- removes the entry and runs sync.
- `aibox schedule run <name>` -- immediate one-shot execution for testing.

**Phase 4: Advanced features**
- Worktree isolation for code-modifying tasks (create git worktree, run task, report diff).
- Notification hooks (write to a file, post to webhook, send to MCP server).
- Heartbeat mode: `type = "heartbeat"` with an `interval` field instead of `cron`,
  implementing the cheap-check-first pattern.
- Dependency chains: task B runs only after task A succeeds.

### Example Configuration

```toml
# --- Scheduled tasks ---
[[schedule]]
name    = "nightly-audit"
cron    = "0 2 * * *"
command = "review src/ for security vulnerabilities and write findings to context/audit-latest.md"
timeout = "10m"

[[schedule]]
name    = "daily-standup"
cron    = "0 8 * * 1-5"
command = "generate a standup summary from the last 24h of git commits and open backlog items"
output  = "context/STANDUPS.md"
timeout = "5m"

[[schedule]]
name     = "weekly-deps"
cron     = "0 3 * * 0"
command  = "check for outdated dependencies, update them, run tests, and report results"
workdir  = "worktree"
timeout  = "15m"

[[schedule]]
name    = "doc-review"
cron    = "0 4 * * 5"
command = "review all markdown files in docs-site/ for staleness and broken links"
timeout = "10m"
```

### Why Not the Other Options?

- **Option C (Skill):** Agents creating their own cron jobs is the opposite of reproducible
  infrastructure. Non-starter for a tool that values declarative configuration.
- **Option D (Provider integration):** Violates provider independence. GitHub Agentic
  Workflows are promising but GitHub-specific. Codex Automations are OpenAI-specific.
  aibox must work with any provider or no provider.
- **Option E (Companion container):** Architecturally clean but adds significant complexity.
  Worth revisiting if scheduling needs outgrow in-container cron -- for example, if tasks
  need to run while the main container is being rebuilt. Could be a Phase 5 evolution.

---

## 7. Open Questions

1. **Cron vs systemd timers inside the container?** Debian-based images have cron available.
   systemd timers offer better logging and dependency management but require systemd as
   PID 1 (not standard in containers). Recommendation: use cron for simplicity; consider
   a lightweight Go/Rust scheduler binary if cron proves insufficient.

2. **API key availability.** Scheduled tasks need provider API keys. These are typically
   set as environment variables in `[container.environment]` or passed via host env.
   Security consideration: keys are accessible to all scheduled tasks. Per-task key
   scoping is not practical at this stage.

3. **Cost control.** Scheduled AI tasks consume API credits. Should aibox enforce a
   per-task or per-day token budget? At minimum, the `timeout` field prevents runaway
   costs. A future `max_tokens` field could provide finer control.

4. **Notification on failure.** When a scheduled task fails (non-zero exit, timeout),
   how should the user be notified? Options: file-based log (Phase 2), desktop
   notification (macOS/Linux), webhook (Slack/Discord), or email. Start with file-based
   logging; add notification hooks later.

5. **Interaction with `aibox start`/`aibox stop`.** Should schedules run only while the
   container is active? Yes -- cron inside the container naturally stops when the container
   stops. This is the expected behavior; users who want always-on scheduling should use
   CI/CD (GitHub Actions) or the companion container pattern.

---

## 8. Sources

- [Claude Code Headless Mode](https://code.claude.com/docs/en/headless)
- [Claude Code Scheduled Tasks Guide](https://claudefa.st/blog/guide/development/scheduled-tasks)
- [Aider Watch Mode](https://aider.chat/docs/usage/watch.html)
- [GitHub Agentic Workflows](https://github.github.com/gh-aw/)
- [GitHub Blog: Automate repository tasks](https://github.blog/ai-and-ml/automate-repository-tasks-with-github-agentic-workflows/)
- [GitHub Agentic Workflows Technical Preview](https://github.blog/changelog/2026-02-13-github-agentic-workflows-are-now-in-technical-preview/)
- [Codex Automations](https://developers.openai.com/codex/app/automations)
- [Codex Non-Interactive Mode](https://developers.openai.com/codex/noninteractive)
- [MCP 2026 Roadmap](http://blog.modelcontextprotocol.io/posts/2026-mcp-roadmap/)
- [Heartbeats vs Cron for AI Agents](https://dev.to/ryancwynar/heartbeats-vs-cron-two-patterns-for-scheduling-autonomous-ai-work-1l0)
- [AI Agent Job Scheduling Patterns 2026](https://fast.io/resources/ai-agent-job-scheduling/)
- [Seven Hosting Patterns for AI Agents](https://james-carr.org/posts/2026-03-01-agent-hosting-patterns/)
- [How We Built a CRON Scheduler for AI Agents at Scale](https://blog.geta.team/how-we-built-a-cron-scheduler-for-ai-agents-at-scale/)
- [GitHub Actions Cron Timezone Support (March 2026)](https://github.blog/changelog/2026-03-19-github-actions-late-march-2026-updates/)
- [runCLAUDErun - macOS Scheduler](https://runclauderun.com/)
- [claude-code-scheduler (GitHub)](https://github.com/jshchnz/claude-code-scheduler)
