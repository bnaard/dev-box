# AGENTS.md

<!-- pk-managed:pk-compliance-contract-v2 BEGIN -->
<!-- pk-compliance-contract v2 BEGIN -->
<!-- pk-compliance v2 -->

## processkit Compliance Contract

<!-- BEGIN HOOK -->

### processkit per-turn checklist

- On session start, call `acknowledge_contract(version="v2")` once
  before any write-side processkit tool call.
- Before any sub-agent / `Task` dispatch or write-side MCP call
  (`create_*`, `transition_*`, `link_*`, `record_*`, `open_*`),
  call `route_task(task_description)` and use its recommendations.
- On any domain-relevant request, consult `skill-finder` (or call
  `find_skill(task_description)`) before acting.
- Do not hand-edit files under `context/` — use MCP tools.
- Do not browse `context/` with `ls` or `grep` — use `index-management`.
- Do not edit `context/templates/` (read-only upstream mirror).
- Full positive actions and prohibitions: see
  `context/skills/processkit/skill-gate/assets/compliance-contract.md`.

<!-- END HOOK -->

## On session start

- Call `acknowledge_contract(version="v2")` once before any write-side
  processkit tool call. This unblocks the skill-gate for the session.
- Treat each new domain-relevant request as a routing checkpoint (see
  *Tool routing*).

## Sub-agent dispatch

- Call `route_task(task_description)` before any sub-agent / `Task` /
  `Agent` dispatch; read `recommended_team_member_slug` and
  `recommended_model_class` from the response.
- Pass the recommended TeamMember slug as the sub-agent's identity where
  the harness supports it, and pick the cheapest model in the recommended
  class (Haiku < Sonnet < Opus).
- Bare-model sub-agent dispatch without a prior `route_task` call is a
  compliance miss.

## Tool routing

- Consult `skill-finder` (or call `find_skill(task_description)`) before
  acting whenever there is even a 1% chance a processkit skill applies.
- Call `route_task(task_description)` before any `create_*`,
  `transition_*`, `link_*`, `record_*`, or `open_*` tool call.
- Read entities through `index-management` (`query_entities`,
  `get_entity`, `search_entities`) when looking up entity content.

## Entity writes

- Write entities through MCP tools so schema validation, state-machine
  enforcement, and event-log auto-entry all run.
- Create the WorkItem, DecisionRecord, Note, or Artifact in the same
  turn you decide on it — deferred entity creation is lost.
- Log an event after any state change that an MCP write did not already
  produce automatically.

## Decisions

- Call `record_decision` in the same turn a cross-cutting recommendation
  is accepted.
- When the last five user messages contain explicit decision language
  (approved / decided / ship it / let's go / ok / yes / confirmed),
  either call `record_decision` in the same turn or call
  `skip_decision_record(reason=...)` to acknowledge the skip.

## Prohibitions

- Do not hand-edit files under `context/` to create or mutate entities
  (use MCP tools).
- Do not browse `context/` with `ls`, `grep`, or raw filesystem walks
  (use `index-management`).
- Do not edit any file under `context/templates/` (read-only upstream
  mirror used as a diff baseline).
- Do not hand-edit the generated harness MCP config — edit the
  per-skill `mcp-config.json` and let the installer re-merge.

## Preferred MCP entry points by task type

| Task type | Preferred MCP entry point |
|-----------|--------------------------|
| Read a single entity by ID | `get_entity(id=...)` or kind-specific `get_workitem` / `get_decision` / `get_team_member` |
| Read an entity by filesystem path | `get_entity_by_path(path=...)` |
| List entities across kinds | `list_entities(kind?, state?, limit?)` |
| Search entities by text | `search_entities(text)` or `hybrid_search_entities(text)` |
| Create / mutate an entity | `create_*` / `transition_*` / `record_*` / `open_*` tools (always route_task first) |
| Run the aggregator health check | `run_pk_doctor(check?, fix?)` |
| Run the pre-release validation sweep | `run_pk_release_audit(tree?)` |
| Dispatch a sub-agent | `route_task(task_description=...)` first → then `Agent` or `Task` with recommended model |

## Read is OK for non-entity files

The Read tool is **blocked** on canonical entity files (paths matching
`context/{workitems,decisions,artifacts,team-members,scopes,gates,actors,
roles,bindings}/*.md`). A PreToolUse hook enforces this at runtime.

Read is **allowed** (no hook block) for:
- Skill source code under `context/skills/<skill>/` — scripts, SKILL.md,
  configs, assets are all readable directly.
- Schema definitions under `context/schemas/` (reading is fine; writes
  require a Migration + DEC).
- Log entries under `context/logs/` (append-only, safe to scan).
- Applied migrations under `context/migrations/applied/`.
- TeamMember sub-files: `persona.md`, `card.json`, and everything under
  `knowledge/`, `journal/`, `skills/`, `relations/`, `lessons/`,
  `private/`, `working/`.
- Any file outside `context/` entirely (docs/, src/, README.md, etc.).
<!-- pk-compliance-contract v2 END -->
<!-- pk-managed:pk-compliance-contract-v2 END -->

## About & session start

**aibox** is a Rust CLI that manages reproducible, AI-ready dev containers.
Since v0.16.0 it has a strict two-part scope:

1. **Containers** — generates `.devcontainer/Dockerfile`, `docker-compose.yml`,
   and `devcontainer.json` from `aibox.toml`, plus a tool-bundle addon system
   (`addons/`) and themed `.aibox-home/` runtime config seed.
2. **processkit installer** — fetches a pinned release of
   [`projectious-work/processkit`](https://github.com/projectious-work/processkit)
   and installs its skills, primitives, processes, and the canonical `AGENTS.md`
   template into the consuming project under `context/`.

**MCP Permissions** — Since v0.18.7, `aibox sync` auto-generates harness-specific
permission files for all MCP servers. Configure `[mcp.permissions]` in `aibox.toml`
to eliminate repetitive permission prompts. Glob patterns expand into concrete
server names; deny patterns take precedence over allow for security. See
[Configuration / MCP Permissions](./docs-site/docs/reference/configuration.md#permission-configuration-mcppermissions).

Target users: solo developers, small teams, and consultants who want
reproducible AI-ready dev environments without manual Docker/devcontainer setup.
Success looks like: `aibox init` → working themed tmux session with processkit
content in place in under 5 minutes.

Run `pk-resume` before acting. Provider-specific files (`CLAUDE.md`,
`CODEX.md`, `.cursor/rules`, …) are thin pointers — edit **this** file.

**For structured workflows** (releases, migrations, design reviews, decision records):
Always check `context/notes/` and `context/work-instructions/` for canonical documented
procedures before acting. These take precedence over general knowledge or tool defaults.

## Setup

```sh
# build the CLI binary
cd cli && cargo build

# run all tests (unit + integration + E2E tier 1)
cd cli && cargo test

# lint (zero warnings required)
cd cli && cargo clippy --all-targets -- -D warnings

# format check
cd cli && cargo fmt -- --check
```

For E2E tier 2 (full container lifecycle tests, requires the `aibox-e2e-testrunner`
companion service to be running alongside the devcontainer):

```sh
cd cli && cargo test --features e2e
```

The E2E companion is reached from this devcontainer over SSH/SCP:
`ssh -i /workspace/.aibox-e2e-runner-home/.ssh/id_ed25519 testuser@aibox-e2e-testrunner`.
Do not treat missing local `docker` or `podman` in the main devcontainer as
evidence that the companion is unavailable; use SSH reachability first. The
companion itself owns the container runtime used by Tier 2 tests.

### aibox CLI commands inside this devcontainer

Normal dogfood/self-management inside the workspace container uses processkit
tools: run `pk-doctor` for processkit/runtime diagnostics. Do not run
`aibox doctor` from inside the container to judge this live project; it checks
aibox-managed host/project posture, not processkit runtime health.

There are narrow exceptions when developing or releasing the aibox CLI itself.
These are host-context simulations, not dogfood operations:

- Unit, integration, and Tier 1 E2E tests may run `aibox init`, `aibox apply`,
  `aibox doctor`, and related commands in temporary projects, often with
  `AIBOX_NO_CONTAINER=1` or mocked runtimes.
- Tier 2 E2E tests may run `aibox apply`, `aibox up`, and `aibox doctor`
  against projects deployed to the `aibox-e2e-testrunner` companion; verify the
  companion over SSH first.
- Release Phase 0 may run `aibox doctor` through
  `./scripts/maintain.sh release-doctors` to exercise host-side doctor behavior
  even when launched from this devcontainer.
- Local test-installs may run the freshly built `aibox` binary against a scratch
  project to simulate what a host user would do.

Outside those explicit test/release contexts, prefer `pk-doctor` inside the
container and reserve `aibox doctor` for the host.

See `context/work-instructions/DEVELOPMENT.md` (or `CONTRIBUTING.md`) for
the full development workflow, E2E test architecture, and cross-compile steps.

<!-- pk-managed:pk-commands BEGIN -->
<!-- pk-commands BEGIN -->
<!--
build: "cd cli && cargo build"
test: "cd cli && cargo test"
lint: "cd cli && cargo clippy --all-targets -- -D warnings"
fmt: "cd cli && cargo fmt"
typecheck: ""
-->
<!-- pk-commands END -->
<!-- pk-managed:pk-commands END -->

## Code style & PRs

### Code style and conventions

- **Zero clippy warnings** — always run with `-D warnings`; CI rejects any warning.
- **All tests must pass** before committing; run `cargo test` and
  `cargo clippy --all-targets -- -D warnings` before every commit.
- **`cargo audit` must be clean** before tagging a release.
- **Conventional commits** — `feat:`, `fix:`, `chore:`, `docs:`. For releases that
  include real changes beyond the version bump, use `fix(vX.Y.Z): <summary>` with a
  section-by-section body and `Refs: DEC-NNN, BACK-NNN` + `Co-Authored-By:` trailer.
- **Reference GitHub issue numbers** in commits: `fixes #N`, `refs #N`.
- **Include `Cargo.lock`** in version bump commits.
- **Never force-push to `main`.**
- **No hardcoded processkit vocabulary** in production Rust code — add constants to
  `cli/src/processkit_vocab.rs` instead.
- **No trailing summaries** in agent responses — the user can read the diff.

### Pull requests

- **Direct commits to `main`** — no PR ceremony on this repo.
- **"Ship it" means the full release ritual** end-to-end: build, test, commit, tag,
  push, GitHub release, deploy docs. Do not ask permission at each step.
- **Phase 0 runs three steps before any version bump:** (1) `./scripts/maintain.sh release-check-state`
  → `dist/RELEASE-STATE.md`; (2) `./scripts/maintain.sh release-doctors` → `dist/RELEASE-DOCTORS.md`
  (runs pk-doctor + aibox doctor; ERRORs from either block the release, WARNs surface in the report
  but don't block); (3) review both reports. `./scripts/maintain.sh release <version>` calls all
  three automatically.
- **Phase 2 is always the user's job** — macOS host builds and GHCR image pushes
  run via `./scripts/maintain.sh release-host X.Y.Z` on the host, never from the container.
  Before Phase 2, sync the host checkout to the matching version-line release branch
  (for example, `git fetch origin v0.x-release --tags && git switch v0.x-release && git reset --keep origin/v0.x-release`).
- **Detailed release procedures:** See [`context/notes/NOTE-20260411_0000-LoyalSpruce-aibox-release-process.md`](./context/notes/NOTE-20260411_0000-LoyalSpruce-aibox-release-process.md) for step-by-step Phase 1 and Phase 2 instructions, including exact commands and prerequisites.
- **One big change preferred over many small PRs** for breaking releases. The user
  explicitly said: "make one big change, I'll handle derived project dependencies."
- **Hold uncommitted changes when the user says "hold"** — leave in working tree,
  do not commit, do not bump version until the user says "ship".

## processkit preferences

Runtime configuration lives in per-skill config files under
`context/skills/<name>/config/settings.toml`. The agent edits these
directly; MCP servers read them on every call — no restart needed.

Schema and storage policy is strict. When processkit changes entity
vocabularies, filenames, IDs, or directory layouts, migrate existing entities
forward and update references. Do not keep local grandfathering lists, mixed
filename policies, or pk-doctor suppressions as the final state.

Config overrides in `context/skills/id-management/config/settings.toml`:

- **ID format:** `word` + `camel` + datetime prefix + slug — e.g. `BACK-20260411_0109-CalmFox-adapt-aibox-self-hosted`.
- **Directory names:** all processkit defaults (`workitems/`, `decisions/`, etc.).
- **Log sharding:** processkit default (no date-based subdirectories).

## AI agents on this project

Configured providers: **claude**, **openai**. Other agents may be working
on this project — coordinate through the entity layer
(`workitem-management`, `event-log`, `discussion-management`) rather than
assuming you are alone.

### Team

This project operates with a permanent multi-role AI-agent team backed
by processkit primitives. The owner is the sole human approver; the
TeamMember whose role is marked `primary_contact: true` is the single
agent that speaks to the owner and routes work to the rest of the team.

Roles, seniorities, and TeamMember identities are provider-neutral:

- `context/roles/` — Role responsibilities (`junior → specialist → expert
  → senior → principal` seniority ladder).
- `context/team-members/<slug>/` — Persistent TeamMember identities
  (persona + A2A card + tiered memory). These bind to
  `Artifact(kind=model-profile)` capability profiles, not to provider
  names or model IDs.
- `context/bindings/` — Role and TeamMember model-assignment bindings.
  Resolution: `model-recommender.resolve_model` selects a concrete
  `Artifact(kind=model-spec)` candidate (which may encode a provider
  name) at dispatch time, gated by runtime access.
- `context/decisions/DEC-20260422_0234-BraveFalcon` — role catalog +
  seniority charter.
- `context/decisions/DEC-20260422_0234-LoyalComet` and
  `DEC-20260503_1829-LoyalComet` — model artifacts + binding routing.
- Active interlocutor: `team-manager.get_active_interlocutor`. If
  configured, show the TeamMember identity at session start; otherwise
  state that the current speaker is an ephemeral harness agent.
- TeamMembers are cloneable on demand (default cap 5; owner approves
  beyond). Clones get fresh IDs; processkit reserves the original
  TeamMember slug. Do not reuse retired TeamMember IDs.

Model-spec filenames may encode provider/model names; model profiles,
roles, and TeamMember identities must not. Never hardcode a model tier
or provider directly in a role definition — always go through the
binding layer.

**Commit to actions immediately.** If you decide to create an entity
(WorkItem, DecisionRecord, etc.), call the tool in the same turn. Do
not say "I'll track that" and move on — deferred commitments are
routinely dropped and leave the entity layer out of sync with what was
discussed.

**Check the skill catalog before acting on domain tasks.** When a
domain-specific task arrives — writing a PRD, creating a release,
reviewing a skill, designing a schema — search the processkit skill
catalog first. Use `search_entities` via index-management or check
`skill-finder` before falling back to general knowledge. A matching
skill may exist with processkit-specific conventions (entity storage
paths, workitem linking, output formats) that general knowledge does
not know. Missing a skill wastes work and produces non-standard
output.

## Project-specific notes

### Critical: `.devcontainer/` vs `images/`

**We are in a dev-container building dev-containers.** Never confuse:

- **`.devcontainer/`** — THIS project's own dev environment (Rust + Python/uv + Docusaurus).
- **`images/`** — Published images for OTHER projects (pushed to GHCR). They do NOT include Rust toolchain or MkDocs.

Changes to `.devcontainer/` affect our development. Changes to `images/` affect downstream projects.

### Project structure

| Path | Owns |
|---|---|
| `cli/` | The Rust CLI (`aibox` binary) — the only shipped artifact besides addon YAMLs |
| `addons/` | YAML addon definitions (python, rust, node, latex, …) |
| `images/` | Container image build recipes published to GHCR |
| `docs-site/` | Docusaurus documentation site |
| `context/` | This project's context (backlog, decisions, research, …) |
| `scripts/` | Release and maintenance tooling (`maintain.sh`, `record-asciinema.sh`, …) |

**Key Rust module:** `cli/src/processkit_vocab.rs` — central constants module.
All processkit-related compile-time vocabulary (path prefixes, filenames, category order,
frontmatter types) lives here. **Never hardcode processkit strings in production code.**

### aibox ⇄ processkit boundary

- **aibox owns:** containers, addons, the `[processkit]` config section, the
  install/diff/migrate machinery, the slim project skeleton at init time
  (`.aibox-version`, `.gitignore`, empty `context/`, thin provider pointer files
  like `CLAUDE.md`), and the docs site.
- **processkit owns:** every skill (`SKILL.md`), every primitive schema, every state
  machine, the canonical `AGENTS.md` template, processes, and the package YAMLs.
- **The `context/` directory** is shared territory: aibox creates it, processkit fills
  it. An immutable upstream reference lives under `context/templates/processkit/<version>/`
  for the three-way diff.

**If something process-related is missing, add it to processkit, not aibox.**
Do not re-introduce skills, processes, or primitives into aibox — that all belongs in
processkit now (DEC-027).

### Anti-patterns — stop and reconsider if you find yourself doing these

- Writing to `.claude/`, `.gemini/`, or any other provider directory from aibox
  code (off-perimeter per DEC-029)
- Hardcoding path strings, filenames, or processkit vocabulary in Rust source
  (add to `processkit_vocab.rs` instead)
- Trying to do Phase 2 of a release from inside the container (needs macOS host)
- Skipping `cargo audit`, `cargo clippy --all-targets -- -D warnings`, or `cargo test`
  before tagging a release
- Pointing users at the removed `aibox` skills subcommand — that surface was removed in v0.16.0 (DEC-027); processkit skills are now the only home for skill content
- **Creating GitHub releases directly** with `gh release create` — always use
  `./scripts/maintain.sh release <version>` inside the container instead. It runs
  tests, cargo audit, builds Linux binaries, creates the release with assets attached,
  deploys docs, and prints the Phase 2 prompt. A bare `gh release create` produces
  a release with no binary assets.
- **Releasing when `cargo build --release` is broken** — if the build fails (e.g.
  missing linker after a container config change), stop and tell the user rather
  than creating an empty release. A release without binaries is worse than no release.
  The precondition is: `cargo build --release --target aarch64-unknown-linux-gnu`
  must succeed inside the container before `maintain.sh release` is invoked.

### Design principles (non-negotiable)

These are load-bearing. When a design call comes up, the answer is whichever option
respects more of these principles.

1. **Provider neutrality.** No file path, config field, binary, or API surface is bound
   to a specific AI provider. Skills live under `context/skills/`, never `.claude/skills/`.
   Provider-specific files (CLAUDE.md, etc.) are thin pointers to AGENTS.md.
2. **Reproducibility.** Every consumed processkit release is pinned by `(source, version,
   sha256)` in `aibox.lock`. Moving-branch consumption is a dev fallback, not production.
3. **Locality.** Everything a project needs lives inside the project directory. A fresh
   `git clone` + `aibox sync` reproduces the environment exactly.
4. **Edit-in-place.** Installed processkit content lives at editable, top-level paths.
   The immutable upstream reference under `context/templates/processkit/<version>/` is the
   diff baseline only — not a restriction on editing.
5. **Forkability.** Every reference to processkit goes through `[processkit].source`.
   Companies can fork processkit and consume the fork by changing one line.
6. **Single source of truth.** Each piece of content lives in exactly one project.
   Skills/primitives/processes/AGENTS.md template → processkit only. Container generation/
   addon management/install pipeline → aibox only. DEC-20260411_0000-JollyClover-rip-bundled-process-layer made this strict.
7. **Generic content-source machinery.** The fetcher in `content_source.rs` is content-
   source-neutral by construction. It doesn't know "processkit" specifically — it knows
   how to fetch a release-asset tarball from any GitHub-shaped source, verify it, and
   extract it. Processkit-compatible alternatives consume the same machinery with no code change.

### Provider independence

All project state must be stored in `./context/` — never in provider-specific locations
(e.g. `.claude/memory/`, `.aider/`). This ensures any AI agent (Claude, Aider, Gemini,
etc.) can pick up where another left off, and session handovers are committed to git.
Do not write to `.claude/`, `.gemini/`, or any other provider directory from aibox code.

### MCP Permissions Troubleshooting

If you're still seeing permission prompts for aibox-shipped MCP servers:

1. **Verify `[mcp.permissions]` is configured** in `aibox.toml`:
   ```toml
   [mcp.permissions]
   default_mode = "allow"
   allow_patterns = ["mcp__processkit-*"]
   ```
   
2. **Run `aibox sync`** to regenerate harness permission files. Permission configuration is applied during sync, not during runtime.

3. **Check harness-specific behavior:**
   - **Claude Code**: Checks `.claude/settings.local.json` → run `aibox sync` to populate `permissions.allow[]`
   - **OpenCode**: Reads `.opencode/config.toml [mcp]` section → verify `mode = "allow"` and `allow[]` array
   - **Continue IDE**: Per-tool `mode` in `continue/config.json` → defaults to "Ask" for safety; override with `[mcp.permissions.harness.continue] mode = "allow"`
   - **Cursor IDE**: Checks `.cursor/settings.json allowedMcpServers[]` → verify entries match expanded server names
   - **Gemini CLI**: Dual `includeTools`/`excludeTools` in `.gemini/settings.json` → intersection semantics (both must match)
   - **GitHub Copilot**: Reads environment variables `COPILOT_MCP_ALLOW_TOOLS`, `COPILOT_MCP_DENY_TOOLS` → created as `.copilot-env`
   - **Aider**: Checks `.aider/mcp-permissions.json allowed_tools` → fallback for harnesses without native MCP permission support
   - **Codex**: Uses project-level `trust_level = "trusted"` in `.codex/config.toml` → applies to all tools

4. **Verify pattern matching:**
   - `"mcp__processkit-*"` matches all processkit MCP servers (e.g., `mcp__processkit-workitem-management__create_workitem`)
   - `"bash"` matches the Bash tool fallback
   - First-match-wins: if a tool matches both allow and deny patterns, deny takes precedence
   - Check `/workspace/.aibox/aibox.log` for pattern expansion details

5. **Check for typos in `allow_patterns`** — misspelled patterns expand to zero tools. `aibox sync` logs warnings for patterns that match no servers.

### Operational gotchas

- **Podman compose** output format varies by version — always use `inspect`, never parse `ps` output.
- **Stale image cache**: if the container exits immediately after start, rebuild with `--no-cache`.
- **`.aibox-home/` must be in `.gitignore`** — it contains SSH keys and personal config.
- **tmux version pin**: tmux is installed from the base Debian image package set; upgrade it through `images/base-debian/Dockerfile` package inputs or the base image.
- **`host.docker.internal`**: works on Docker Desktop and Podman pasta; bare Linux Docker may need `--add-host`.
- **OrbStack virtiofs**: files mounted from macOS may lose execute permissions — workaround: `chmod +x` inside container.
- **Claude Code OAuth in containers**: use `claude setup-token` or authenticate on host (credentials shared via `.claude` mount). See anthropics/claude-code#14528. Do NOT use `network_mode: host`.
- **OrbStack network dropout**: after ~20 minutes idle, OrbStack's VM NAT can drop connections. Fix: set `keepalive = true` in `[container]` of `aibox.toml` (adds a lightweight DNS keepalive every 2 minutes via `postStartCommand`).

### Tmux session scripts — two files, two roles

Two files share the name `aibox-tmux-session.sh` (or `aibox-session.sh`) but serve different purposes. Editing the wrong one is a common source of confusion:

| File | Role |
|---|---|
| `images/base-debian/config/bin/aibox-tmux-session.sh` | **IMAGE/RUNTIME variant.** Baked into `/usr/local/bin` inside the container image. Acts as the authoritative fallback launcher used by the container entrypoint before any apply-time managed files are present. No per-project customisation. |
| `cli/src/templates/aibox-home/.config/tmux/aibox-session.sh` | **GENERATED template variant.** Source template copied into `.aibox-home/.config/tmux/` at `aibox apply` time. Reflects the project owner's configured layout, session name, etc. from `aibox.toml`. |

At runtime the GENERATED variant takes precedence (via `AIBOX_TMUX_MANAGED_SESSION`); the IMAGE variant is the fallback when no managed session script is present yet. Both files carry a header banner that identifies which role they play. When changing either file, review the other for behavioural parity.

The generated layout scripts in `.aibox-home/.config/tmux/layouts/` (one per `ConfigLayout`) are rendered by `cli/src/tmux/layouts.rs::tmux_layout_script`.

### Runtime artifacts for agents (in derived projects)

When an AI agent is working inside a project that uses aibox:

| Path | Contents |
|---|---|
| `.aibox/aibox.log` | NDJSON structured log of every `aibox` command. Read to understand what aibox did recently. Rotates at 1 MB. |
| `aibox.lock` | Pinned versions of the aibox CLI and processkit last synced. |
| `context/migrations/pending/` | Pending Migration entities awaiting review; resolve through `migration-management` MCP (`apply_migration` / `reject_migration`). Never hand-move migration files. |
| `context/migrations/applied/` | Applied or rejected Migration entities (state-bucketed by processkit convention). |

### MCP config topology

Each MCP server ships a `mcp-config.json` next to its `server.py` under
`context/skills/<category>/<skill>/mcp/`. `aibox sync` merges these
into the harness-specific roots (`.mcp.json`, `.codex/config.toml`,
`.opencode/config.toml`, …) and records a SHA256 aggregate in
`context/.processkit-mcp-manifest.json`. Downstream installers compare
the aggregate hash against their last-merged state and re-merge when
they differ — independent of whether the processkit version changed.
Never hand-edit the generated harness MCP config; edit the per-skill
`mcp-config.json` and let the installer re-merge. The
`mcp_config_drift` pk-doctor check validates the manifest locally.

### GitHub organization

- **Repo:** `projectious-work/aibox`
- **GHCR:** `ghcr.io/projectious-work/aibox`
- **Docs:** `https://projectious-work.github.io/aibox/`
- **processkit upstream:** `https://github.com/projectious-work/processkit`
- **processkit releases:** `https://github.com/projectious-work/processkit/releases`

---

<sub>Scaffolded by processkit `v0.25.1` on `2026-05-03`. Re-rendered on each installer sync.</sub>
