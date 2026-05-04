# Changelog

All notable changes to processkit are documented here.
Versions follow [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

_No unreleased changes yet._

---

## [v0.25.4] — 2026-05-04

v0.25.4 is a **patch release** that fixes the gateway stdio proxy
startup path used by daemon-first MCP clients. A stdio-only harness can
now point at a localhost streamable HTTP gateway URL and let processkit
start the matching daemon when no listener is present.

### Changed

- **Gateway stdio proxy now owns local daemon startup by default.**
  `stdio-proxy --url http://127.0.0.1:PORT/mcp` starts the matching
  `serve --transport streamable-http` daemon when the port is not already
  listening.
- **Added an explicit external-supervisor escape hatch.**
  `stdio-proxy --no-start-daemon` keeps the previous behavior for
  harnesses or service managers that already own daemon lifecycle.

### Fixed

- **Fixed MCP client startup timeouts for daemon-backed gateway configs.**
  A generated config that invokes `processkit-gateway` through
  `stdio-proxy` no longer fails solely because the HTTP daemon has not
  already been launched.
- **Fixed failed daemon auto-start cleanup.** If the spawned local daemon
  never becomes ready, the proxy now terminates that child process before
  reporting the startup failure.

### Verification

- `30 passed` for gateway proxy and CLI coverage in `src/context/`.
- `30 passed` for gateway proxy and CLI coverage in `context/`.
- `uv run scripts/smoke-test-servers.py` passed across the MCP smoke
  suite.
- `pk-doctor --category=mcp_gateway --no-log` passed with
  `0 ERROR / 0 WARN` for both source and shipped paths.

---

## [v0.25.3] — 2026-05-03

v0.25.3 is a **patch release** that makes role and team-member model
routing portable across harnesses. Concrete provider/model descriptions
remain timestamped `Artifact(kind=model-spec)` deliverables, while
roles, actors, team members, and default assignments now bind to
provider-neutral `Artifact(kind=model-profile)` artifacts.

### Added

- **Added provider-neutral model-profile artifacts.** processkit now
  ships nine `Artifact(kind=model-profile)` deliverables covering
  general, coding, research, writing, and governed routing profiles.
  Model-profile filenames deliberately avoid provider and model names;
  their content carries ranked candidate `Artifact(kind=model-spec)`
  references.
- **Added model-profile schema support.** Artifact schemas now recognize
  `model-profile`, binding schemas allow model-assignment targets to be
  either `ModelProfile` or `ModelSpec` artifacts, and role schemas expose
  `default_model_profile` while keeping legacy `default_model`
  compatibility.
- **Added a model-profile migration helper.**
  `migrate_model_profiles.py` creates the shipped profile artifacts and
  rewrites role/team-member model-assignment bindings from concrete
  model specs to profile targets.
- **Added runtime provider access gates to the resolver.** The resolver
  now expands model profiles into concrete model candidates only after
  applying runtime access constraints from user config, `aibox.toml`
  model providers, and native harnesses such as Codex, Claude, Gemini,
  Aider, Continue, Cursor, OpenCode, and Hermes.
- **Added `available_providers` to model-recommender configuration.**
  MCP callers can declare which providers are callable in the current
  environment without hard-coding a provider into role or team-member
  bindings.
- **Added pk-doctor checks for routing portability.** `pk-doctor` now
  flags provider/model names in model-profile filenames, missing profile
  candidate model specs, and Role/TeamMember bindings that directly pin a
  ModelSpec without `conditions.direct_model_pin: true`.

### Changed

- **Role and team-member bindings now target model profiles.** Shipped
  default bindings, including Cora's default, no longer point directly to
  OpenAI, Anthropic, Google, or other concrete model artifacts.
- **Concrete model specs remain provider/model named.** The release keeps
  the v0.25.2 decision that `ModelSpec` artifacts map directly to
  provider/model families and therefore may include provider/model names
  in their filenames.
- **Default binding documentation now describes two layers.**
  Role/team-member defaults bind to profile artifacts; model profiles
  list provider-specific model-spec candidates; project overrides may
  still use direct ModelSpec pins when intentional.
- **The release deliverable boundary accepts model-profile artifacts.**
  `scripts/check-src-context-drift.sh --release-deliverable` now permits
  timestamped `ModelProfile` artifacts in `src/context/artifacts/`.
- **Team and model-recommender docs now define harness portability.**
  The docs make aibox responsible for surfacing runtime provider access,
  while processkit owns provider-neutral defaults and local fallback
  checks.

### Removed

- **Removed dogfood-specific provider bias/veto bindings from the
  deliverable.** The old processkit-local Codex/OpenAI preference and
  Anthropic veto bindings no longer ship to derived projects.

### Fixed

- **Fixed Cora and role defaults under non-Codex harnesses.** Changing
  `aibox.toml` from Codex to Claude, Gemini, OpenCode, Hermes, or another
  supported harness no longer leaves role/team-member defaults bound to a
  provider the harness cannot call.
- **Fixed pk-doctor's blind spot around direct model pins.** The doctor
  now distinguishes portable Role/TeamMember profile bindings from
  explicit direct model pins.
- **Fixed resolver explanations for profile-expanded candidates.**
  Resolved candidates now carry `profile_id` and `profile_rank`, making
  it visible which neutral profile produced the concrete model choice.

### Verification

- `pk-doctor --no-log` passed with `0 ERROR / 0 WARN`.
- `scripts/check-src-context-drift.sh --release-deliverable` passed.
- `release_audit.py --tree=src-context` passed with `0 ERROR / 0 WARN`.
- `35 passed` for resolver and default-binding coverage tests.
- `21 passed` for model-recommender MCP filter tests.
- `20 passed` for model migration tests.
- `26 passed` for release-audit tests.
- `uv run scripts/smoke-test-servers.py` passed across the MCP smoke
  suite.

---

## [v0.25.2] — 2026-05-03

v0.25.2 is a **patch release** that settles the v0.25 model-spec
projection and release-deliverable hygiene. It keeps model descriptions
as `Artifact(kind=model-spec)` entities, but aligns their IDs and
filenames with the timestamped Artifact convention used elsewhere in
processkit.

### Added

- **Added timestamped model-spec artifacts to the shipped deliverable.**
  Model descriptions now ship under
  `ART-YYYYMMDD_HHMM-ModelSpec-<provider>-<family>.md` in both
  `context/artifacts/` and `src/context/artifacts/`.
- **Added pk-doctor context-hygiene checks for model-spec naming.**
  `pk-doctor` now flags non-timestamped model-spec artifact IDs/files
  and model-assignment bindings that still target old `ART-model-*`
  artifact IDs.
- **Added `.gitignore.example` to the release deliverable.** The example
  gitignore guidance now ships under `src/.gitignore.example` instead
  of existing only in the repository root.

### Changed

- **Model-assignment bindings now target timestamped model-spec
  Artifacts.** Shipped role/team/default model bindings now point at
  `Artifact(kind=model-spec)` IDs instead of hidden JSON records or the
  short-lived `ART-model-*` projection.
- **The model-recommender generator emits the canonical timestamped
  model-spec IDs.** Re-running `migrate_models.py` no longer recreates
  non-timestamped model artifact filenames.
- **The release boundary accepts only timestamped model-spec artifacts in
  `src/context/artifacts/`.** The drift guard now rejects other artifact
  shapes from the consumer deliverable.
- **Applied/rejected migration history is archived out of the live
  context.** The dogfood context keeps archive manifests while removing
  migrated state from the active migration directory.

### Fixed

- **Fixed pk-doctor blind spots around artifact hygiene.** HTML sidecar
  artifacts, suspicious `_0000` artifact timestamps, and legacy
  model-spec artifact names are now surfaced by `pk-doctor`.
- **Fixed resolver compatibility after the model-spec rename.** Current
  bindings use timestamped artifact IDs, while legacy `MODEL-*` and
  `ART-model-*` inputs remain accepted for derived projects during
  migration.
- **Fixed context-archiving manifest serialization for datetime values.**
  Archive creation can now serialize generated manifests that include
  datetime-like metadata.

### Verification

- `pk-doctor --no-log` passed with `0 ERROR / 0 WARN`.
- `scripts/check-src-context-drift.sh --release-deliverable` passed.
- `release_audit.py --tree=src-context` passed with `0 ERROR / 0 WARN`.
- `20 passed` for model migration tests.
- `29 passed` for resolver tests.
- `uv run scripts/smoke-test-servers.py` passed across the MCP smoke
  suite.

---

## [v0.25.1] — 2026-05-03

v0.25.1 is a **patch release** for the model-recommender after the
v0.25.0 gateway and v2 deliverable release. It updates the shipped model
market roster, makes model lifecycle explicit, and adds fine-grained task
suitability routing while preserving the existing R/E/S/B/L/G scoring
contract.

### Added

- **Added lifecycle metadata for model roster entries.** Models now carry
  `active`, `legacy`, `deprecated`, `retired`, or `unverified` status so
  processkit can distinguish “still offered but no longer top-ranked”
  from “not callable anymore.” This prevents older but useful provider
  models from disappearing from the roster merely because a newer model
  has taken the top recommendation slot.
- **Added 42 fine-grained task suitability classes.** The roster now
  scores models for concrete work types such as `architecture`,
  `debugging`, `code_review`, `summarization`, `rag`, `ocr`,
  `voice`, `bulk_generation`, `privacy_sensitive`, and
  `self_hosted_enterprise`. These classes sit on top of the stable
  Reasoning, Engineering, Speed, Breadth, Reliability, and Governance
  dimensions.
- **Added task-class-aware MCP querying.** `query_models()` now accepts
  `task_class`, `min_task_suitability`, `require_task_suitability`,
  `lifecycle`, and `include_retired`, allowing callers to ask for
  models that are suitable for a specific task without losing the
  existing dimension filters.
- **Added `list_task_classes()` to model-recommender MCP.** Callers can
  inspect the live task taxonomy and its dimension profile instead of
  hard-coding task-class names.
- **Added current-market model entries across major providers.** The
  roster now includes new or refreshed entries for OpenAI GPT-5.5,
  GPT-5.2, GPT-5.2-Codex, Anthropic Claude Opus 4.7, Google Gemini 3
  preview models, Mistral Medium 3.5 / Small 4 / Devstral 2, DeepSeek
  V4, Moonshot Kimi K2.6, Z.AI GLM-5/5.1, xAI Grok 4.20 and Grok 4.1
  Fast, Meta Llama 4 Scout/Maverick, Qwen3 Coder, Cohere Command A,
  Amazon Nova 2 Lite, and NVIDIA Nemotron 3 Super.

### Changed

- **Model routing now separates availability from ranking.** A model that
  remains available through a provider API can stay in the roster as
  `legacy` or `deprecated`; routing can still select it for cost,
  compatibility, privacy, or task-fit reasons.
- **Resolver tie-breaking now understands task classes.** `resolve_model`
  accepts `task_hints.task_class`, `task_hints.task_classes`,
  `task_hints.min_task_suitability`, and
  `task_hints.require_task_suitability`. Suitability only reorders
  otherwise comparable candidates after binding layer/rank/provider
  precedence, so project and team model bindings remain authoritative.
- **Model migration helpers preserve lifecycle and task-suitability
  metadata.** The legacy `migrate_models.py` path maps lifecycle into
  version status and carries source URLs, model classes, and task
  suitability into generated model artifacts for compatibility.
- **The model-recommender documentation now documents lifecycle and task
  suitability explicitly.** Workflow C clarifies that refreshes are
  additive and should not delete still-offered models; Workflow B now
  classifies tasks before applying the six-axis score.

### Fixed

- **Corrected the previous roster-refresh assumption that “demote” meant
  remove.** The shipped behavior now keeps offered models and lowers
  routing priority only through lifecycle/task-fit metadata.
- **Corrected stale pricing/context metadata for several existing roster
  entries.** Notable updates include GPT-5.5 API pricing/context,
  GPT-5/GPT-5 Pro legacy pricing, o3 pricing/context, Gemini 2.5 Flash
  pricing, and Kimi K2.6 API pricing.

### Notes

- This remains a patch release because processkit is still pre-1.0 and
  the change is additive for consumers: existing model IDs, the six
  dimension filters, `model_classes`, and user configuration continue to
  work.
- `unverified` entries remain visible only under the same conservative
  assumptions as `_estimated` records; production routing can exclude
  them with `include_estimated=False`.
- Provider pricing and model availability change quickly. The roster
  carries source URLs and should be refreshed again before making
  budget-critical decisions.

### Verification

- `44 passed` for model-recommender query, migration, and default-binding
  tests.
- `29 passed` for resolver tests.
- `python3 -m py_compile` passed for model-recommender server,
  resolver, and migration helper.
- `scripts/check-src-context-drift.sh --release-deliverable` passed.
- `uv run scripts/smoke-test-servers.py` passed across the MCP smoke
  suite.

---

## [v0.25.0] — 2026-05-02

v0.25.0 is a **breaking pre-1.0 minor release**. It completes the
SmoothTiger/SmoothRiver v2 deliverable boundary, adds the new
provider-neutral `processkit-gateway` MCP entry point, removes legacy
first-class primitives from the shipped `src/context/` surface, and
turns the release checks into executable gates. The release remains below
1.0 because processkit is still evolving its public compatibility
contract; consumers should read the breaking-change notes before
upgrading.

### Breaking Changes

- **`src/context/` is now the release deliverable boundary.** The shipped
  tree is validated as a consumer-facing package, not as a mirror of this
  repository's live dogfood `context/`. Runtime project memory,
  generated logs, local decisions, and migration history remain in the
  consuming project's `context/`; only the versioned processkit
  primitives, skills, schemas, state machines, and release metadata ship
  from `src/context/`.
- **`Metric`, `Model`, `Process`, `Schedule`, and `StateMachine` are no
  longer shipped as first-class v2 entity primitives.** Their schemas,
  shipped entity directories, and primitive asset schemas were removed
  from `src/context/`. Existing projects may still keep those files as
  migration-source or project-local memory, but new processkit releases
  do not promise these directories or schemas as part of the v2 package
  contract.
- **Model assignments are no longer typed as `Model` entity bindings.**
  Shipped model-assignment Binding files now reference model-recommender
  roster/profile identifiers directly and no longer carry
  `target_kind: Model`. Consumers that validated these bindings against
  the old first-class `Model` primitive must update that logic.
- **Task-router process overrides are legacy-v1 compatibility data.**
  The `process_override` field is retained for older derived projects,
  but the server now identifies it as `legacy-v1` metadata. New v2
  routing should rely on skill and tool routing rather than first-class
  `Process` entities.
- **The release gate now rejects dogfood-only content in `src/context/`.**
  `scripts/check-src-context-drift.sh --release-deliverable` fails if
  the shipped tree contains local project memory, demoted primitive
  schemas, demoted entity directories, or `target_kind` references to
  removed primitives. This can break downstream packaging flows that
  assumed `context/` and `src/context/` were interchangeable.

### Added

- **Added `processkit-gateway`, a provider-neutral MCP gateway.** The new
  gateway speaks standard MCP on the harness side and processkit's
  skill/entity/tool layer on the implementation side. It does not call
  Anthropic, OpenAI, or other provider APIs; harness-specific behavior
  remains limited to installer/config adapters.
- **Added gateway runtime helpers under
  `context/skills/_lib/processkit/gateway/`.** The library covers tool
  catalog loading, module loading, registry construction, session and
  permission helpers, transport helpers, daemon runtime wiring, lazy
  registration, and stdio proxy support.
- **Added gateway transports and modes.** The gateway can run as direct
  stdio (`serve --transport stdio`), as a localhost streamable HTTP
  daemon (`serve --transport streamable-http --host 127.0.0.1 --port
  8000 --path /mcp`), or through `stdio-proxy --url ...` for harnesses
  that only support command-launched MCP servers.
- **Added lazy catalog support for lower idle memory.** The gateway can
  generate and consume `mcp/tool-catalog.json` so it can list tool
  metadata without importing every backing MCP server at startup. Hot
  modules are loaded on first use and remain available through the
  gateway runtime.
- **Added gateway health and discovery tools.** `list_gateway_tools` and
  `gateway_health` expose the gateway's registered source servers,
  imported tools, duplicate-name handling, and runtime mode for harness
  diagnostics.
- **Added gateway verification coverage.** New tests cover direct
  gateway registration, daemon runtime behavior, stdio proxy behavior,
  and aggregate/gateway compatibility. `scripts/measure-mcp-gateway.py`
  measures process count and tool coverage across granular, aggregate,
  gateway, and daemon-proxy modes.
- **Added v2 workflow and projection MCP surfaces.** The release includes
  hook-inbox note-management helpers, process-instance and SEP handoff
  helpers, schedule and budget binding helpers, agent-card projection,
  eval-gate authoring, security policy projections, and v2 contract
  doctor checks.
- **Added release-audit target selection.** The release-audit script now
  supports `--tree=context`, `--tree=src-context`, and `--tree=both` so
  local dogfood context and shipped release content can be audited under
  the right expectations.
- **Captured aibox follow-up responsibilities as internal processkit
  context.** The handover covers installing the pinned release,
  supervising the gateway daemon, generating harness MCP config,
  exposing user-facing daemon/reset knobs, preserving provider
  neutrality, and implementing the optional harder reset path.

### Changed

- **Release packaging now enforces the deliverable boundary.**
  `scripts/build-release-tarball.sh` runs the release-boundary guard and
  the `src-context` release audit before building a tarball. A release
  artifact is no longer just a copy operation; it must pass the shipped
  package contract first.
- **pk-doctor drift checks now wrap the release-boundary guard.** The
  drift category reports whether `src/context` is a valid release
  deliverable, making the same invariant available during normal project
  health checks.
- **MCP manifest generation and drift checks were tightened.** The MCP
  config manifest remains the installer-facing signal that aibox should
  re-merge generated harness MCP config even when the processkit version
  number has not changed.
- **`aggregate-mcp` remains as a compatibility bridge.** It was updated
  to coexist with `processkit-gateway` and keep unique tool names stable
  while qualifying duplicate helper names as `<skill_slug>__<tool_name>`.
- **Docs now describe the 25-server MCP surface.** The MCP server
  overview documents per-skill servers, `aggregate-mcp`, the new
  gateway, direct stdio, streamable HTTP daemon mode, and stdio proxy
  mode.
- **Harness guidance now covers Claude Code, Codex, OpenCode, Hermes,
  Aider, and other MCP-capable clients.** The guidance keeps processkit
  usable without aibox while documenting the integration points aibox can
  automate in managed devcontainers.
- **v2 contract docs now explain demoted primitives and migration
  expectations.** The docs make clear which old primitives are
  migration-source only and which v2 primitives remain durable shipped
  entities.
- **SKILL documentation was brought up to the release-audit contract.**
  `agent-card`, `eval-gate-authoring`, `security-projections`,
  `processkit-gateway`, and release-audit-facing docs now include the
  required sections and clearer operational guidance.
- **The docs site now has a root route and modern Docusaurus markdown
  hook configuration.** This removes the `/processkit/` navbar broken
  link warning and the deprecated `siteConfig.onBrokenMarkdownLinks`
  warning during docs builds.

### Removed

- Removed shipped `src/context/models/` and `src/context/processes/`.
  These remain valid as project-local migration-source memory in a live
  consuming context, but they are no longer part of the released
  processkit v2 package.
- Removed shipped demoted primitive schemas from `src/context/schemas/`:
  `metric.yaml`, `model.yaml`, `process.yaml`, `schedule.yaml`, and
  `statemachine.yaml`.
- Removed shipped demoted primitive asset schemas from
  metrics-management, process-management, schedule-management, and
  state-machine-management.
- Removed `target_kind: Model` from shipped model-assignment Binding
  files.
- Removed the docs primitive page for `Metric` as a first-class shipped
  primitive.

### Migration Notes

- The normal upgrade path is still the existing migration flow. aibox
  should continue to generate and apply migrations as it does today.
- The proposed harder reset path is optional and belongs in aibox as an
  explicit user-selected mode of `aibox apply` or `aibox reset`. That
  flow should export project memory, bootstrap a fresh `context/` tree
  from the new processkit release, re-import preserved entities with
  updated references/schema fields, and write an auditable migration
  record.
- aibox should not re-create removed `Model`, `Process`, `Metric`,
  `Schedule`, or `StateMachine` release schemas during sync. Existing
  local project files of those kinds should be treated as preserved
  migration-source memory or project-local content.
- Harnesses can still use per-skill MCP servers directly. aibox is not a
  runtime dependency of processkit; it is the recommended installer,
  supervisor, and harness-config manager for managed devcontainers.

### Verification

- Release-audit tests passed for both dogfood and shipped trees:
  `26 passed`.
- Gateway and aggregate MCP tests passed: `30 passed`.
- `scripts/check-src-context-drift.sh --release-deliverable` passed.
- `release_audit.py --tree=src-context` passed with
  `0 ERROR / 0 WARN / 485 INFO`.
- `pk-doctor --category=drift --no-log` passed with
  `0 ERROR / 0 WARN / 1 INFO`.
- `uv run scripts/smoke-test-servers.py` passed across the MCP server
  smoke suite.
- `scripts/measure-mcp-gateway.py --format text --include-metadata`
  reported the expected one-process gateway and daemon-proxy modes with
  `130` tools across `23` source servers.

---

## [v0.24.0] — 2026-04-30

A minor release focused on model-routing depth, context lifecycle
management, and operator diagnostics. It adds richer model/provider
metadata, local semantic routing support, a read-safe context archiving
skill, context-consumption reporting, and team/model assignment
infrastructure. No breaking changes are intended for existing consumers.

### Added

- Added model-class assignment and richer model/provider characteristics:
  provider, model, version, effort, architecture, license, deployment,
  latency/speed, context length, token accounting, access, rate limits,
  and supported runtime parameters.
- Added `get_model_for_class()` and class-based routing metadata to the
  model recommender so callers can resolve fast, balanced, deep, and
  specialized model classes consistently.
- Added the `context-archiving` skill and MCP server for planning and
  creating archive tarballs, extracting archived payloads, and keeping
  archived entities queryable through the index.
- Added archive-aware index metadata via `storage_location`, including
  archived LogEntry event-query support.
- Added task-router v0.2 scoring with local n-gram semantic matching,
  scoring basis details, and low-confidence LLM-escalation metadata.
- Added context-consumption checkpoint reports and team identity /
  model-assignment bindings for role-based routing.
- Added optional sqlite-vec semantic search and FTS5-backed index search.
- Added a library-expert skill template plus skill-builder and
  skill-reviewer guidance for version-scoped library expertise.

### Changed

- Improved processkit routing and MCP ergonomics across task-router,
  skill-finder, model-recommender, and related docs.
- Refined harness diagnostics and pk-doctor drift reporting, including
  server-header drift coverage.
- Updated model schemas in both `context/` and `src/context/` to carry
  the broader model-characteristics surface.

### Fixed

- Fixed artifact metadata updates so callers can avoid restamping
  freshness-sensitive `metadata.updated` values.
- Restored the docs-site build after the v0.23.1 release line.

### Notes

- v0.24.0 is a **minor** release because it introduces new
  backwards-compatible capabilities. The context-archiving MCP server is
  implemented, but harness registration remains gated on installer sync
  support for new per-skill MCP config entries.
- Pre-release validation on the release branch retains the known optional
  release-audit warnings for unused entity directories (`actors/`,
  `scopes/`, `gates/`) and the known pk-doctor preauth warning gated by
  aibox#55.

---

## [v0.23.1] — 2026-04-26

A focused content+code patch resolving the 106 ERRORs surfaced by the
first-run release-audit on v0.23.0. Hybrid validator + content fixes
per `DEC-20260426_1859-MightyRobin` (`BACK-FierceOwl`): three documented
validator relaxations close 96 false-positives, and ten real content
findings get backfilled. Net: 106 ERROR → 0 ERROR; 3 WARN unchanged
(actors / scopes / gates entity-dirs are deliberately unused — flagged
for v0.24.0).

### Changed

- **change(release-audit): excludelist aibox CLI migration prose docs.**
  `entity_files` walker now skips `*-to-*.md` and `INDEX.md` under
  `context/migrations/`. Closes 16 A1 false-positives — these are aibox
  CLI prose, not Migration entities. Real Migration entities under
  `pending/` and `applied/` continue to be validated.
- **change(release-audit): model the team-member directory layout.**
  `entity_files` walker now understands the team-manager skill's
  alt schema: registers `Persona` as a known kind, special-cases the
  `team-member.md` filename stem to map onto `TEAMMEMBER-{slug}`, and
  walks `relations/` subdirs as Persona-related sub-files. Closes 11
  A2 false-positives.
- **change(release-audit): make `metadata.processkit.layer` optional
  for non-processkit-category skills.**
  Layer is a processkit-DAG concept; library-style skills
  (`category != "processkit"`) participate in no compositional layer
  graph. Strict layer enforcement preserved for the processkit DAG
  where it matters. Closes 69 C false-positives — across engineering
  (39), devops (11), data-ai (11), design (5), documents (3).

### Added

- **add(release-audit): 24 new pytest cases.** `test_release_audit.py`
  gains `TestMigrationProseSkipped` (4 cases), `TestTeamMemberLayout`
  (3 cases), and `TestLayerConditional` (2 cases) covering the three
  validator changes above. Plus `+203` lines total of fixture and
  helper plumbing.

### Fixed

- **fix(skills): backfill `metadata.processkit.layer` on 7
  processkit-category SKILL.md files.** Each value computed from the
  skill's declared `uses[]` deps:
  - `model-recommender`: layer 2
  - `skill-builder`: layer 4
  - `skill-finder`: layer 4
  - `skill-gate`: layer 4
  - `skill-reviewer`: layer 4
  - `task-router`: layer 4
  - `team-creator`: layer 4
- **fix(team-creator): author missing `## Full reference` section.**
  +213 lines reference content covering all team-creator commands,
  flags, and recipes.
- **fix(team-manager): author missing `## Overview` and
  `## Full reference` sections.** +316 lines covering the team-member
  directory layout, persona files, relations, and consistency checks.

### Notes

- v0.23.1 is a **patch** release — additive validator allowances and
  documentation backfills only, no breaking changes. Consumers on
  v0.23.0 can adopt with no migration steps.
- pk-release-audit on this release: **0 ERROR / 3 WARN / 999 INFO**
  (down from 106 ERROR / 3 WARN / 903 INFO on v0.23.0). The 3
  remaining WARNs are `entity.missing-dir` for `actors/`, `scopes/`,
  and `gates/` — the project doesn't use those entity kinds yet.
- Incidentals rolled into the same release per FierceOwl's transition
  note: pk-doctor `drift.py` parser fix follow-up to processkit#13,
  drift-script provenance allowlist updates, and a refreshed aibox CLI
  migration prose for v0.20.0 → v0.21.1.
- `DEC-20260426_1627-CuriousButter` was corrupted at write-time
  (XML-style fields embedded inside the `decision` string plus
  trailing tool-call markup) and supersedes-replaced by
  `DEC-20260426_1859-MightyRobin`.

---

## [v0.23.0] — 2026-04-26

A focused batch release: governance filter implementation that gives
the v0.22.1 RoyalFern schema fields runtime teeth, plus four scaffolding
bug-fixes (CalmArch / RapidDaisy / VastLark) and four new validators &
tools (TidyGrove / NobleBrook / AmberCliff). Eight WorkItems in total,
all detect-or-fix improvements with no breaking changes.

Implementation followed two waves: wave one (CalmArch + RapidDaisy +
VastLark + SolidWolf) shipped sequentially; wave two (TidyGrove +
NobleBrook + AmberCliff + BraveMeadow) ran as four parallel
sonnet-tier subagents, one per WI, against disjoint target dirs.

### Added

- **feat(model-recommender): query_models() RoyalFern filters (SolidWolf).**
  `BACK-20260426_1214` — query_models() gains 9 governance / data-privacy
  filter kwargs sourced from the RoyalFern fields landed in v0.22.1:
  `phi_hipaa_eligible`, `pii_eligible`, `gdpr_eligible`,
  `training_on_customer_data` (enum match), `data_retention_days_max`
  (numeric ceiling; accepts the literal `"zero"`, rejects `"unknown"`),
  `jurisdiction_country_in` (HQ membership), `legal_regime_in` (any-match
  on `applicable_legal_regimes`), `data_residency_in` (any-match on
  `data_residency_regions`), and `max_latency_p50_ms`. Missing-field
  semantics: a model lacking the relevant field is REJECTED — conservative
  governance default. `_profile_block` surfaces the new fields on every
  result so callers can verify what filter they got hits on. The four
  query_models() examples in the model-recommender SKILL.md (HIPAA,
  jurisdiction, training, retention) now return non-empty result sets
  against the live 34-model roster (16/20/10/15 hits respectively).
- **feat(release-audit): new skill (TidyGrove).**
  `BACK-20260409_1830` — `context/skills/processkit/release-audit/`
  ships a detect-only validator that runs four passes before a release
  tag: entity-file frontmatter compliance (apiVersion / kind / id-vs-
  filename), SKILL.md structural compliance (required frontmatter
  fields + four required body sections), MCP-server tool annotation
  presence, and skill cross-reference resolution. Single Markdown
  report, exit 0 / 1, command `pk-release-audit`. First live run
  surfaces 105 pre-existing ERRORs (legacy aibox migration prose,
  team-member sub-files using an alt schema, non-processkit skills
  missing the `layer` field) — flagged for separate triage and not
  blocking this release.
- **feat(pk-doctor): `skill_dag` check (NobleBrook).**
  `BACK-20260409_1738` — new pk-doctor check walks every SKILL.md,
  builds the dependency graph from each `metadata.processkit.uses[]`,
  and validates: missing-references (every used skill exists), cycles
  (iterative DFS with 3-colour marking), and layer constraints (a
  skill at layer N may only reference skills at layer ≤ N). On the
  live tree the first run found 0 cycles, 0 missing refs, and 5 layer
  violations — all five resolved in this release as a content-only
  layer bump (see *Fixed* below).
- **feat(skill-finder): user-facing `catalog` MCP tool (AmberCliff).**
  `BACK-20260410_1840` — new `catalog()` tool on
  `processkit-skill-finder`. Signature:
  `catalog(category, tag, keyword, columns, sort_by, output)` with
  `output ∈ {"markdown", "json", "yaml"}` and column projection on
  any frontmatter field. Defaults to a Markdown table of every skill
  sorted by name. Reuses `_all_skills()` — no new SQLite plumbing.
  SKILL.md gains a "Catalog queries" section with trigger phrases
  ("list skills", "show me all skills in X", "skills as JSON") and
  five example invocations.

### Changed

- **change(skill-gate): prefix-based PreToolUse matcher (VastLark).**
  `BACK-20260425_1235` — `check_route_task_called.py` switches from an
  explicit allowlist to a prefix-based matcher. Any tool whose name
  starts with `create_`, `transition_`, `link_`, `record_`, `open_`,
  `update_`, `apply_`, `reject_`, `add_`, `end_`, `import_`,
  `reactivate_`, `deactivate_`, `release_`, `reserve_`, `supersede_`,
  `start_`, `evaluate_`, or `skip_` is now gate-locked, plus an
  explicit allowlist for `log_event` (verb+noun shape). Closes the
  auto-renew gap that surfaced in the 2026-04-25 session when only
  `create_binding` traffic ran across a midnight boundary and the
  marker expired despite continuous compliant writes. 40 new test
  cases in `test_hooks.py`.
- **change(workitem-management + all entity-writing MCPs): YAML
  literal-block-scalar serialization (CalmArch).**
  `BACK-20260425_1755` — `context/skills/_lib/processkit/frontmatter.py`
  adopts a custom `SafeDumper` subclass with a `str` representer that
  emits `|` block-scalar style for any string containing a newline,
  normalizing trailing whitespace to a single `\n` (canonical clip-
  chomp shape). `parse()` re-attaches the trailing `\n` the regex
  consumed at the `\n---` boundary. Markdown bodies with pipe-tables,
  fenced code blocks, and backslashes now round-trip safely instead
  of failing PyYAML re-parse. The fix lives in the shared lib so
  every entity-writing MCP (workitem-management, decision-record,
  event-log, …) benefits. 8 new tests cover the QuickBison repro
  shape.
- **change(model-recommender): `_entity_to_legacy()` propagates RoyalFern fields.**
  Pass-through for `vendor_model_id`, `knowledge_cutoff`,
  `latency_p50_ms`, `jurisdiction`, and `data_privacy` from
  `Model.versions[]` into the legacy-shape entry that `query_models()`
  iterates over. Required to make the SolidWolf filters above work
  end-to-end against the real model roster.
- **change(skill-finder): SKILL.md "Catalog queries" section.**
  Documents the new `catalog()` tool, trigger phrases, and example
  invocations.

### Fixed

- **fix(event-log): log_event MCP now validates required `actor` field
  (RapidDaisy).**
  `BACK-20260425_1755` — `log_event()` calls
  `schema.validate_spec("LogEntry", spec)` before write. Calls
  missing or with empty `actor` return `{"error": "..."}` and write
  no file. Previously the schema-required field was silently omitted,
  producing LogEntry files that pk-doctor's `schema_filename` check
  flagged as ERRORs after the fact (and required a hand-fix). 3 new
  tests cover missing-actor, valid-actor, empty-actor cases. Docstring
  + module preamble updated to mark `actor` as required.
- **fix(skills): layer-value bumps to satisfy `uses[]` declarations.**
  Surfaced by the new NobleBrook skill_dag check on its first live
  run. Five layer violations resolved by promoting the affected skills
  to a layer ≥ their highest declared dependency:
  - `engineering/changelog`: layer 2 → 3 (uses `release-semver` at 3).
  - `processkit/status-briefing`: layer 2 → 4 (uses `agent-management` at 4).
  - `processkit/status-update-writer`: layer 2 → 4 (uses `agent-management` at 4).
  - `product/onboarding-guide`: layer 2 → 4 (uses `agent-management` at 4).
  Behavior unchanged; only the frontmatter `metadata.processkit.layer`
  values change.

### Verified (no changes needed)

- **chore(owner-profiling): reference files complete (BraveMeadow).**
  `BACK-20260410_1049` — verified
  `owner-profiling/references/observable-signals.md` (105 lines, 37
  signals × 6 categories) and `interview-protocol.md` (128 lines)
  are present, complete, spec-compliant against
  `NOTE-20260410_1046-StoutSwan`, and mirror-clean. No file changes.

### Notes

- v0.23.0 is a **minor** release — additive features and pure bug
  fixes, no breaking changes. Consumers on v0.22.1 can adopt with no
  migration steps.
- The release-audit skill's first live run reports 105 ERRORs of
  pre-existing tree issues; these are NOT blockers for v0.23.0 and
  will be triaged in a separate v0.23.x content-cleanup pass.
- Two cross-project bugs were filed upstream during the prep window
  (no processkit-side fix): aibox#56 (CleverRiver — same-version
  migrations) and aibox#57 (FierceWren — content-diff overwrites
  locally-added entity content). Workaround in place: run `git status`
  after every `aibox sync`.
- pk-doctor green at release time: 0 ERROR / 2 WARN / 11 INFO. The
  two WARNs are the unchanged `context/.processkit-provenance.toml`
  drift (known v0.22.0 stamping bug, acknowledged in aibox v0.20.0
  release notes and tolerated). One additional INFO comes from the
  new skill_dag check.

---

## [v0.22.1] — 2026-04-26

A focused point release shipping the RoyalFern Model schema enhancement
(`DEC-20260425_2256`): five additive optional fields on
`Model.spec_schema.versions[]` — `jurisdiction`, `data_privacy`,
`knowledge_cutoff`, `vendor_model_id`, and `latency_p50_ms` — backfilled
across all 34 model entities and documented under the Governance
dimension in the model-recommender skill. No breaking changes; existing
callers are unaffected because every new field is optional.

The `query_models()` MCP tool does not yet implement filtering on these
fields — that work is tracked as `BACK-20260426_1214-SolidWolf` and
targeted for v0.23.0. Until that lands, the new fields are read via
`get_profile()` / direct file inspection.

### Added

- **feat(schema): RoyalFern Model schema fields.**
  `context/schemas/model.yaml` gains five optional fields on
  `spec_schema.versions[]`:
  - `jurisdiction` — vendor HQ country (ISO-3166-1 alpha-2),
    `applicable_legal_regimes` (slugs: EU-GDPR, US-CLOUD-Act,
    US-HIPAA, CN-DSL, CN-PIPL, UK-DPA, CA-PIPEDA), and
    `data_residency_regions` (e.g. `["us-east","eu-west"]`). Lossless
    replacement for ad-hoc `governance_warning` free-text callouts.
  - `data_privacy` — vendor data-handling posture: `dpa_available`
    (boolean or DPA URL), `data_retention_days` (integer or
    `zero`/`unknown`), `training_on_customer_data` (`never`/`opt-in`/
    `opt-out`/`always`/`unknown`), `pii_eligible`, `phi_hipaa_eligible`,
    `gdpr_eligible`, and `sub_processors_url`.
  - `knowledge_cutoff` — vendor-published training-data cutoff date.
  - `vendor_model_id` — exact provider API model identifier
    (e.g. `claude-opus-4-7-20251031`), preferred dated/pinned form.
  - `latency_p50_ms` — approximate p50 first-token latency in
    milliseconds; quantitative companion to `dimensions.speed`.
  All fields are optional and additive; pre-RoyalFern Model entities
  remain valid without modification.

### Changed

- **chore(models): backfill RoyalFern fields across all 34 model entities.**
  Every `MODEL-*.md` under `context/models/` (Anthropic, OpenAI, Google,
  xAI, Meta, Mistral, DeepSeek, Cohere, Microsoft, Alibaba, MiniMax)
  now carries the new fields where vendor-published values exist;
  `unknown` / omitted where they don't. Backfill done by two parallel
  shard subagents to bound context cost; both passes verified against
  the schema.
- **docs(model-recommender): Governance dimension expanded.**
  `context/skills/processkit/model-recommender/SKILL.md` adds a
  structured-fields block under the Governance dimension and four
  example `query_models()` filter usages
  (`phi_hipaa_eligible=true`,
  `jurisdiction.vendor_hq_country in ["US","EU"]`,
  `data_privacy.training_on_customer_data="never"`,
  `data_privacy.data_retention_days=0`). Note: the SKILL.md docs are
  the spec; the corresponding `query_models()` filter implementation
  ships in v0.23.0 (tracked as BACK-20260426_1214-SolidWolf).

### Notes

- v0.22.1 is a **patch** release — five additive optional schema
  fields with no behavioural changes to any existing tool. Consumers
  on v0.22.0 can adopt with no migration steps.
- Two upstream aibox bugs surfaced during this session and were
  filed for the aibox project: aibox#56 (CleverRiver — same-version
  migrations) and aibox#57 (FierceWren — content-diff overwrites
  locally-added entity content). Neither has any processkit-side
  fix in this release; both are tracked as cross-project WorkItems
  (BACK-20260425_1711-CleverRiver, BACK-20260426_1205-FierceWren).
- pk-doctor green at release time: 0 ERROR / 2 WARN (both expected
  drift WARNs for `context/.processkit-provenance.toml` — known
  v0.22.0 stamping bug acknowledged in aibox v0.20.0 release notes
  and tolerated by the host).

---

## [v0.22.0] — 2026-04-25

Two new pk-doctor checks (`migration_integrity`, `server_header_drift`)
and the processkit-side half of preauth-spec shipping (WildGrove
Phase A) — plus a host CLI bump to aibox 0.19.2 with `mcp_config_hash`
recorded in the lock file. Closes the v0.21.0-deferred RapidSwan
pk-doctor check, lands the schema-malformed-migration detection that
motivated CleverRiver, and exports an 18-pattern preauth bundle for
derived projects (consumed by aibox#55).

A small one-shot cleanup pass also clears 3 pre-existing pk-doctor
schema ERRORs (1 WorkItem with mangled YAML quoting, 2 LogEntries
missing `actor`); the producer-side MCP fixes are tracked as
RapidDaisy and CalmArch for a future release.

### Added

- **feat(pk-doctor): `migration_integrity` check (CleverRiver).**  New
  check flags two malformed-migration patterns: `same-version-with-content`
  (a Migration whose `from_version == to_version` but whose
  `affected_groups` / `affected_files` are non-empty) and
  `affected-files-empty` (table rows present in the body but the
  `affected_files` array empty).  Both fire as WARN, cite CleverRiver,
  and recommend `reject_migration` on the offending file.  Surfaced the
  malformed `MIG-20260425T164303` that motivated the WI.  CleverRiver
  itself stays open as a cross-project tracker for the upstream
  aibox-sync diff-generator fix.
- **feat(pk-doctor): `server_header_drift` check (RapidSwan).**  New
  WARN-level check that hashes the PEP 723 inline metadata block
  (`# /// script` … `# ///`) of every MCP `server.py` and compares
  against a manifest baseline.  On drift, lists the affected skill
  slugs and recommends restarting the harness so `uv` re-resolves the
  pinned venv.  `context/.processkit-mcp-manifest.json` is extended
  with a per-skill `per_server_header` field; `aggregate_sha256`
  semantics are unchanged (still computed over `mcp-config.json`
  files only — aibox#54 contract preserved).  Per
  `DEC-20260424_0127-QuickPine` (SharpBrook split: BraveBird shipped
  the schema-reload half in v0.21.0; RapidSwan ships the dep-drift
  half here).  Closes
  `BACK-20260424_0128-RapidSwan-pk-doctor-server-header`.
- **feat(skill-gate): preauth spec asset (WildGrove Phase A).**
  `context/skills/processkit/skill-gate/assets/preauth.json` ships an
  18-pattern bundle of server-wildcard permission entries and 18
  matching `enabledMcpjsonServers` entries that derived projects merge
  into their harness settings to pre-authorise every processkit MCP
  server.  Phase A is processkit-side only — aibox#55 picks up Phase
  B (the merge logic on `aibox sync`).
- **feat(pk-doctor): `preauth_applied` check (WildGrove Phase A).**
  WARNs in derived-project context when the preauth spec exists in
  the processkit tree but the corresponding entries are missing from
  the harness settings file, with a "run `aibox sync` once aibox#55
  ships" hint.  No-op in the processkit dogfood repo.

### Changed

- **chore(aibox): host CLI 0.18.7 → 0.19.2 + record `mcp_config_hash`
  in lock file.**  Bumped the harness CLI; `aibox.lock` now carries an
  `mcp_config_hash` field recording the sha256 of the merged
  `.mcp.json` at sync time, giving pk-doctor and aibox-sync a stable
  reference for drift detection independent of per-skill
  manifest hashes.

### Fixed

- **chore(repair): clear 3 pre-existing pk-doctor schema ERRORs.**
  One-shot direct edit on a WorkItem whose YAML quoting was mangled
  by a fragile MCP serializer, plus two LogEntries missing the
  required `actor` field (written by a broken `log_event` MCP path
  that didn't populate the field).  No tooling changes here — the
  producer-side MCP fixes are tracked as RapidDaisy
  (log_event missing actor validation) and CalmArch
  (workitem MCP description quoting) for a future release.
- **chore(migrations): reject `MIG-20260425T164303`.**  Malformed
  migration (rejected via `reject_migration` per the
  `migration_integrity` finding above).  Filed CleverRiver as the
  upstream-side aibox-sync diff defect that produced it.

### External

- `projectious-work/aibox#55` — aibox-side preauth merge (WildGrove
  Phase B).  The processkit-side spec stays in-tree; `preauth_applied`
  WARNs in derived projects until the matching `aibox sync` logic
  ships.

---

## [v0.21.0] — 2026-04-25

Five fixes spanning the v0.19.2 retro action items and two
correctness bugs in pk-doctor that surfaced when the owner ran
`/pk-doctor` inside an aibox-derived project and got a falsely-clean
report.  Both pk-doctor bugs were silent-zero failures: the checks
silently no-op'd in derived projects and hid every entity-hygiene
problem they were supposed to catch.

The two heavier retro items (SharpBrook MCP schema hot-reload,
SnappyBird append-only LogEntry repair) were worked through design
discussions (DISC-DaringBird, DISC-WiseLily) and split or narrowed
per DEC-QuickPine and DEC-BrightHawk.

### Added

- **feat(_lib + 4 servers): `reload_schemas` MCP tool.**  Each of
  the schema-active servers (workitem-management, decision-record,
  event-log, artifact-management) gains a thin tool that calls a
  shared helper in `_lib/processkit/schema.py::reload_caches()`,
  clearing the in-process `load_schema` and
  `state_machine.load` `lru_cache`es so a disk-level schema or
  state-machine edit becomes visible without a server restart.
  Returns `{ok, cleared: {schemas, state_machines}}`.  PEP 723
  dep-header edits are NOT addressed (the `uv`-resolved venv is
  pinned at process start — tracked separately by
  `BACK-RapidSwan` for v0.22.0+).  Closes
  `BACK-20260424_0128-BraveBird-reload-schemas-mcp-tool` per
  `DEC-20260424_0127-QuickPine`.
- **feat(model-recommender): 20 specialist + expert seed bindings.**
  The v0.19.0 roster declared a 5-level seniority ladder
  (junior → specialist → expert → senior → principal), but the
  default-bindings pack seeded only 3 levels.  As a result,
  `resolve_model(<role>, "specialist"|"expert")` returned
  `no viable model` across all 10 seeded roles — 20 missing
  combinations.  KeenFern adds seeds inheriting from the nearest
  covered neighbour: `specialist` → junior's (model, effort band),
  `expert` → senior's (model, effort band).  MANIFEST.yaml grew from
  30 → 50 seeds.  New regression test
  `test_default_bindings_coverage.py` asserts every seeded role
  covers all 5 ladder levels and fires loudly on any future gap.
  Closes `BACK-20260424_0134-KeenFern-fill-model-assignment-seniority`.
- **docs(AGENTS.md): "Sub-agent delegation" section.**  Codifies
  the read-only / mutating split for harness `Agent`-tool sub-agents
  (delegate read-only; keep mutating writes on the main session).
  Closes `BACK-20260424_0038-ToughAnt-ephemeral-sub-agent-defaults`.
- **docs(AGENTS.md): "LogEntry repair" Skill-guards entry.**
  Documents the narrow hand-edit escape hatch for schema-invalid
  LogEntries and points to `pk-doctor --fix=schema_filename` as
  the preferred path.  Part of MightyFjord.

### Changed

- **feat(skill-gate): compliance-ack TTL is now an idle timeout.**
  `_any_valid_marker()` rewrites the matching marker's
  `acknowledged_at` to `now` on every successful gate check.
  Sessions actively making compliant writes never expire mid-flow
  (the v0.19.2 midnight-span pain pattern); idle sessions
  (> `_ACK_LIFETIME_HOURS`, default 12 h) still must re-acknowledge.
  Touch failures are non-fatal.  Closes
  `BACK-20260424_0038-SwiftLynx-compliance-contract-acknowledgement-ttl`.

### Fixed

- **fix(pk-doctor): `schema_filename` schemas-dir fallback (HappyReef).**
  `schema_filename.py` resolved the schemas dir as
  `<repo_root>/src/context/schemas/` only — a path that exists
  exclusively in the processkit dogfood repo.  In every derived
  project (aibox-installed) schemas live at `<repo_root>/context/
  schemas/`, so the check silently walked 0 entity files and
  reported 0 ERROR / 0 WARN regardless of how many malformed
  entities were on disk.  The new `_resolve_schemas_dir()` tries
  the dogfood path first, then falls back to `context/schemas/`,
  and surfaces a single WARN when neither exists.  Verified
  against an aibox checkout: the check now walks 186 entity files
  and reports the real findings (filename-date mismatches on
  `_0000` placeholder filenames, missing-`actor` ERRORs, etc.).
  Closes `BACK-20260425_1041-HappyReef-pk-doctor-schema-filename`.
- **fix(pk-doctor): `migrations` layout fallback (DeepMoss).**
  Companion to HappyReef.  The `migrations` check looked only in
  `context/migrations/pending/` (processkit dogfood convention).
  Derived projects (aibox) keep pending migrations at the top
  level of `context/migrations/` and only move *applied* ones into
  `applied/`.  The new `_candidate_pending_paths()` probes
  `pending/` first; if absent, walks `context/migrations/*.md`
  minus `applied/`, `INDEX.md`, and aibox-CLI upgrade-doc
  filenames (`<YYYYMMDD>_<HHMM>_<from>-to-<to>.md`).  `_list_pending`
  also filters by `kind: Migration` so non-Migration markdown
  alongside migrations is ignored.  Closes
  `BACK-20260425_1041-DeepMoss-pk-doctor-migrations-detect`.
- **fix(pk-doctor): `--fix=schema_filename` for the CalmAnt-class
  pattern (MightyFjord).**  New `run_fix` on
  `schema_filename.py` patches LogEntries missing the required
  `actor` field by inserting `actor: system`, validates
  post-patch, and rolls back on failure.  Behind `--fix=schema_filename
  --yes`, never default.  Closes
  `BACK-20260424_0128-MightyFjord-pk-doctor-fix-schema` per
  `DEC-20260424_0128-BrightHawk`.
- **fix(skill-gate): remediation text uses on-disk contract version.**
  `_remediation_msg()` parses the leading `<!-- pk-compliance vN -->`
  marker rather than hard-coding `v1`.  Part of SwiftLynx.
- **fix(retrospective): `/pk-retro --auto-workitems` no longer fails
  with `ModuleNotFoundError: No module named 'mcp'`.**  Added
  `mcp[cli]>=1.0` and `jsonschema>=4.0` to `pk_retro.py`'s PEP 723
  header so `uv run --script pk_retro.py` resolves the in-process
  MCP loader's transitive imports automatically.  Closes
  `BACK-20260424_0038-WildLake-pk-retro-auto-workitems`.
- **fix(skills): resolve cross-category `retrospective` skill basename
  collision.**  `product/retrospective` is renamed to
  `product/sprint-retrospective`; the processkit-category
  `retrospective` (release-scope, `/pk-retro`) keeps the bare name.
  Closes [#11].

### Deferred

- `BACK-20260424_0128-RapidSwan-pk-doctor-server-header` — pk-doctor
  `server_header_drift` check (PEP 723 dep drift detection,
  WARN-only).  Targeted v0.22.0+.
- `BACK-20260424_0037-SharpBrook-mcp-servers-cache-schemas`
  (cancelled; superseded by BraveBird + RapidSwan per
  `DEC-20260424_0127-QuickPine`).
- `BACK-20260424_0038-SnappyBird-data-repair-path-for`
  (cancelled; replaced by the narrow MightyFjord run_fix per
  `DEC-20260424_0128-BrightHawk`; general data-fix migration kind
  not built — revisit only if >2 more recurrences in a calendar
  quarter).

[#11]: https://github.com/projectious-work/processkit/issues/11

---

## [v0.20.0] — 2026-04-24

Retro follow-up batch: fixes the three lightest action items from the
v0.19.2 retrospective (WildLake, ToughAnt, SwiftLynx).  The two heavier
retro items — SharpBrook (MCP schema hot-reload) and SnappyBird
(data-repair path for append-only LogEntries) — are deferred to
v0.21.0 pending the design discussions opened as DISC-DaringBird and
DISC-WiseLily.  See `DEC-20260424_0101-SolidBadger-split-v0-20-0`.

### Added

- **docs(AGENTS.md): "Sub-agent delegation" section.**  Pairs with the
  existing "Skill guards" list and codifies the read-only / mutating
  split for harness `Agent`-tool sub-agents: delegate Read, search
  Bash, and MCP `query_*` / `get_*` / `search_*` / `list_*`; keep
  Write, Edit, new `mkdir`, MCP `create_*` / `transition_*` /
  `record_*` / `link_*` / `open_*` / `log_event`, and git mutations
  on the main session.  Closes
  `BACK-20260424_0038-ToughAnt-ephemeral-sub-agent-defaults`.

### Changed

- **feat(skill-gate): compliance-ack TTL is now an idle timeout.**
  `_any_valid_marker()` in `check_route_task_called.py` now rewrites
  the matching marker's `acknowledged_at` to `now` on every
  successful gate check.  A session that is actively making
  compliant writes no longer expires mid-flow (the v0.19.2
  midnight-span pain pattern); a session that goes idle for longer
  than `_ACK_LIFETIME_HOURS` (12h) still must re-acknowledge.  Touch
  failures are non-fatal.  Closes
  `BACK-20260424_0038-SwiftLynx-compliance-contract-acknowledgement-ttl`.

### Fixed

- **fix(skill-gate): remediation text uses the on-disk contract
  version.**  `_remediation_msg()` parses the leading
  `<!-- pk-compliance vN -->` marker in
  `assets/compliance-contract.md` rather than hard-coding `v1`.  The
  v0.19.2 hard-code told callers to acknowledge `v1` even though the
  on-disk contract was already `v2`.  Part of SwiftLynx.
- **fix(retrospective): `/pk-retro --auto-workitems` no longer fails
  with `ModuleNotFoundError: No module named 'mcp'`.**  `pk_retro.py`
  declared only `pyyaml` in its PEP 723 header, but its in-process
  MCP loader imports `server.py` modules from artifact-management,
  event-log, and workitem-management — all of which require
  `mcp[cli]` and `jsonschema`.  Added both to the PEP 723 header so
  `uv run --script pk_retro.py` resolves them automatically.  Closes
  `BACK-20260424_0038-WildLake-pk-retro-auto-workitems`.
- **fix(skills): resolve cross-category `retrospective` skill
  basename collision.**  `product/retrospective` is renamed to
  `product/sprint-retrospective` (directory, `name:`, and `id:
  SKILL-sprint-retrospective`); the processkit-category
  `retrospective` (release scope, `/pk-retro`) keeps the bare name.
  Closes [#11].  The missing `pk-doctor/commands/` directory
  ([#10]) and the duplicate `pk-resume.md` command
  ([#12]) were already resolved in v0.19.2 (90c980f and 87e185e
  respectively) and are closed here for bookkeeping.

### Deferred to v0.21.0

- `BACK-20260424_0037-SharpBrook-mcp-servers-cache-schemas` — MCP
  schema hot-reload / `reload_schemas` tool. Design discussion:
  `DISC-20260424_0101-DaringBird-how-should-mcp-servers`.
- `BACK-20260424_0038-SnappyBird-data-repair-path-for` — data-repair
  path for malformed append-only LogEntries. Design discussion:
  `DISC-20260424_0101-WiseLily-how-do-we-allow`.

[#10]: https://github.com/projectious-work/processkit/issues/10
[#11]: https://github.com/projectious-work/processkit/issues/11
[#12]: https://github.com/projectious-work/processkit/issues/12

---

## [v0.19.2] — 2026-04-24

Release-hygiene and derived-project-install bulletproofing.  Closes
four v0.19.2 WorkItems (SteadyCedar, BraveDove, ToughMeadow,
HappyFinch) and ships the processkit-side half of TrueQuail; the
aibox-side reconcile is tracked at
[projectious-work/aibox#54](https://github.com/projectious-work/aibox/issues/54).

### Added

- **feat(pk-doctor): `commands_consistency` check.** Walks every
  `context/skills/processkit/*/SKILL.md` and ERRORs when a declared
  `commands[].name` has no matching `commands/<name>.md` file (WARNs
  on the inverse — stray files not declared in metadata).  Prevents
  recurrence of the v0.19.1 slip where pk-doctor shipped without its
  own `commands/pk-doctor.md`.  Closes
  `BACK-20260423_1103-ToughMeadow-pk-doctor-skill-missing`.
- **feat(pk-doctor): `mcp_config_drift` check.** Reads
  `context/.processkit-mcp-manifest.json`, recomputes per-skill
  `mcp-config.json` sha256es, and reports manifest staleness (WARN)
  or — in a derived-project context (`aibox.lock` + `.mcp.json` at
  repo root) — missing processkit servers in `.mcp.json`'s
  `mcpServers` map (ERROR with a "run `aibox sync`" hint).  Surfaces
  the exact failure mode that motivated TrueQuail.
- **feat(release-semver): `scripts/generate-mcp-manifest.py`.** Writes
  `context/.processkit-mcp-manifest.json` + src/ mirror with a sha256
  per-skill-config and an aggregate hash.  Wired into
  `scripts/build-release-tarball.sh` so every release tarball ships a
  fresh manifest.  Stable contract for downstream installers (aibox)
  to detect per-skill-config drift independent of the processkit
  version delta — see
  `DEC-20260423_2049-VastLake-truequail-split-processkit-ships` and
  `projectious-work/aibox#54`.  Closes the processkit-side of
  `BACK-20260423_0829-TrueQuail-aibox-installer-reconcile-mcp`.
- **feat(pk-doctor): ship `commands/pk-doctor.md`.** v0.19.1 declared
  the slash command in SKILL.md metadata but forgot the file, so
  `/pk-doctor` was never registered in derived projects.  This release
  ships the file in both trees and adds `commands_consistency` above
  as the prevention mechanism.
- **docs(AGENTS.md): "MCP config manifest" contract section.**
  Documents the manifest shape + path so aibox (and any other
  processkit installer) can implement against a stable interface.

### Changed

- **chore(skills): reconcile SKILL.md `commands:` metadata to `/pk-`
  namespace.** 14 skills had stale `<skill>-<verb>` names declared in
  metadata while actually shipping `/pk-<verb>.md` files.  This
  release renames the metadata to match shipped filenames (and for
  team-creator, promotes the `commands:` block out of `provides:` to
  `metadata.processkit.commands:` so the new
  `commands_consistency` check can see it).  Drops three entries for
  commands that were declared but never shipped
  (`model-recommender-profile`, `owner-profiling-refine`,
  `skill-reviewer-bulk-gotchas`).  No behavioural changes to any
  command.  Closes
  `BACK-20260423_2055-HappyFinch-skill-md-commands-metadata`.

### Fixed

- **fix(model-recommender): add missing `pyyaml>=6.0` to MCP server's
  PEP 723 header.** The `resolve_model` call failed with
  `ModuleNotFoundError: No module named 'yaml'` when the harness
  invoked `uv run server.py` (no `--script`).  Smoke-tested live
  returning an 8-layer routing trace for
  `ROLE-product-manager@senior`.  Closes
  `BACK-20260423_0829-SteadyCedar-model-recommender-mcp-server`.
- **fix(schemas): widen identity-class ID pattern to
  `^(ACTOR|TEAMMEMBER)-[a-zA-Z0-9_-]+$` across five schemas.**  The
  v0.19.0 TeamMember rollout left `workitem.assignee`,
  `decisionrecord.deciders[]`, `discussion.participants[]`,
  `artifact.owner`, and `metric.owner` still requiring `ACTOR-*`
  only, so MCP writes carrying a `TEAMMEMBER-*` subject failed
  jsonschema validation.  Chose alternation over a clean flip to
  avoid retroactively invalidating ~90 residual `ACTOR-*` references
  in 44 entity files.  Smoke-tested live post-harness-restart with
  both a `create_workitem(assignee="TEAMMEMBER-cora")` and a
  `record_decision(deciders=["TEAMMEMBER-cora"])`.  Closes
  `BACK-20260422_1643-BraveDove-schema-drift-workitem-assignee`.
- **fix(log): add missing required `actor` field to
  `LOG-20260422_1643-CalmAnt-workitem-created`.**  Pre-TeamMember MCP
  server wrote a LogEntry without populating `actor`, so
  `pk-doctor --category=schema_filename` reported a schema ERROR.
  `actor: system` accurately reflects the unattributed machine-origin
  of the event.  One-off direct edit; no MCP tool exists to patch
  append-only LogEntries.

### External

- `projectious-work/aibox#54` — aibox-side reconcile-on-manifest-drift
  tracking issue.  `BACK-20260423_0829-TrueQuail` stays open in the
  processkit backlog until that PR lands.

---

## [v0.19.1] — 2026-04-22

Local-only release bulletproofing — no CI workflows. Addresses the
v0.19.0 post-mortem finding that a tag push does not create a GitHub
Release. Supersedes `DEC-20260422_0926-MerryArch` (CI workflow
approach, rejected on vendor-lock + cost grounds) with
`DEC-20260422_1348-SnowyWolf` (skill flow + doctor detection).

### Added

- **feat(pk-doctor): 6th check category `release_integrity`** — walks every local `v*` git tag, probes GitHub via `gh release view` for a matching Release, and WARNs on any tag without one. INFO when `gh` is unavailable or the tag set is empty. Opt-out via `PK_DOCTOR_SKIP_RELEASE_INTEGRITY=1`; tag-scan cap via `PK_DOCTOR_RELEASE_INTEGRITY_MAX` (default 50). Each WARN carries a ready-to-paste `gh release create <TAG>` command with CHANGELOG extraction inlined. See `DEC-20260422_1348-SnowyWolf-local-only-release-bulletproofing`.

### Changed

- **chore(release-semver): collapse /pk-release into a single bulletproof flow.** `SKILL.md` (both trees) rewritten with a 9-step recipe that prepares, publishes, and **verifies** in one turn — the release is not considered complete until `gh release view vX.Y.Z` succeeds (step 8). `/pk-publish` retained as a recovery alias for historical tags that are missing a Release. New Gotcha: "`git push --tags` is not a GitHub Release." Closes `BACK-20260422_0925-MightyOtter-v0-19-1-release`.

### Fixed

- **fix(release flow): v0.19.0 tag was pushed without a corresponding GitHub Release** — the prepare phase of release-semver completed but the publish phase was skipped. v0.19.0 Release was created manually; v0.19.1 ships the prevention (collapsed flow) and detection (`release_integrity` check) mechanisms.

---

## [v0.19.0] — 2026-04-22

### Added (v0.19.0 architecture refactor)

- **feat(team-manager): new skill replacing `actor-profile`**. Persistent
  participants are now **TeamMembers** (humans, named AI personas,
  services), each living as a directory tree:
  `context/team-members/<slug>/` with `team-member.md` (entity), `persona.md`
  (loaded as persona prompt), `card.json` (A2A v0.3 Agent Card), and six
  memory tiers (`knowledge/`, `journal/`, `skills/`, `relations/`,
  `lessons/`, `private/`). Ad-hoc worker invocations are **not**
  persisted — they are ephemeral `(role, seniority)` dispatches. Skill
  ships 17 MCP tools (lifecycle, name pool, memory tree, export/import,
  consistency), a curated 59-name international name pool, and 10
  consistency check categories. **Replaces actor-profile** (clean break,
  no backward compat). See `DEC-20260422_0233-SpryTulip`.

- **feat(team-member memory): file-based tiered memory**. Six tiers
  (working, episodic, semantic, procedural, relational, lessons) all
  stored as Markdown with YAML frontmatter (`tier`, `source`,
  `sensitivity`, `confidence`, `importance`, `created`, `last_reinforced`,
  `scope`). Default consolidation cadence: per-task + daily journal +
  weekly importance-triggered promotion (Park-style). Each team-member
  has a `private/` subdirectory for developer-local notes
  (gitignored via `.gitignore.example` shipped at repo root).

- **feat(team-member export/import)**: `export_team_member` produces a
  versioned tarball excluding `journal/`, `relations/`, and
  `private/` by default; sensitivity-tagged files (`pii`, `confidential`)
  are redacted. `import_team_member` validates the A2A card signature
  field and creates the entity tree on import.

- **feat(role catalog): expanded from 8 to 51 curated roles** under
  `context/roles/`. Function-grouped (engineering-software, platform-infra,
  data-ml, security, architecture, design-ux, marketing, sales-customer,
  finance, legal-compliance, executive, etc.). **Seniority no longer
  baked into slugs** — pure ordinal attribute with ladder
  `junior → specialist → expert → senior → principal`.
  Bindings (not role files) decide what each rung maps to in
  (model, effort). See `DEC-20260422_0234-BraveFalcon`.

- **feat(model artifacts): models become first-class entities** under
  `context/models/`, one file per `(provider, family)` with
  versions[] nested. 34 model artifacts replace the monolithic
  `model_scores.json` registry (which becomes a compiled cache).
  Each model carries `equivalent_tier` in the **provider-neutral
  T-shirt capacity ladder** (`xs / s / m / l / xl / xxl`, extensible
  in both directions). Effort enum normalised to
  `[none, low, medium, high, extra-high, max]` (aliases `extra-high → xhigh`
  at the Anthropic adapter boundary). See `DEC-20260422_0234-LoyalComet`.

- **feat(bindings): `model-assignment` binding type + 8-layer resolver**.
  `model-recommender.resolve_model(role, seniority?, team_member?, scope?, task_hints?)`
  returns ranked `(model, version, effort)` candidates via the precedence
  ladder: task-pin → team-member preference → project veto → capability
  filter → role+seniority → role default → project bias → shim fallback.
  Tie-breakers: project-preferred provider → cost → recency → reliability.
  Effort clamping, version pinning, stale-binding skip, in-module result
  caching, explain-mode trace. New `/pk-explain-routing` slash command
  for debugging. **Default binding pack** at
  `context/skills/processkit/model-recommender/default-bindings/MANIFEST.yaml`
  ships 30 starter bindings (10 roles × 3 seniorities) materialised into
  `context/bindings/`.

- **feat(pk-doctor): 5th check category `team_consistency`**. Wraps
  `team-manager.check_all()` and surfaces the 10 team-consistency check
  codes in pk-doctor's standard report (schema drift, tier-missing,
  dangling refs, name collision, name-pool compliance, orphan files,
  sensitivity placement, private-dir gitignore, memory file headers,
  card staleness).

- **chore(.gitignore.example)**: bundle of all proposed processkit
  ignores (cache/state, `context/**/private/`, model cache, harness
  configs, OS/IDE/Python noise) shipped at repo root for adoption by
  new processkit projects.

### Removed

- **actor-profile skill superseded** by team-manager. 8 role-class
  actors removed (`ACTOR-developer`, `ACTOR-assistant`, `ACTOR-pm-claude`,
  `ACTOR-{sr,jr}-{architect,researcher,developer}`); identity-class
  `ACTOR-20260421_0144-AmberDawn-legacy-historical-backfill` removed
  (not meaningful); `ACTOR-20260421_0144-ThriftyOtter-owner` migrated
  to `TEAMMEMBER-thrifty-otter`. 7 legacy role files
  (`ROLE-{developer,project-manager,senior-architect,senior-researcher,
  junior-architect,junior-developer,junior-researcher}`) removed,
  superseded by entries in the curated 51-role catalog. 8 legacy
  role-assignment bindings removed.

### Fixed

- fix(mcp): systemic auto-log actor fix — every entity-mutating MCP tool (create_*, transition_*, link_*, update_*, apply_*, reject_*, start_*, end_*, open_*, record_*, supersede_*, deactivate_*) now passes actor=<subject-id> to log helpers, producing schema-valid LogEntries. Covers the bug pattern previously fixed for create_actor in WarmGrove. Includes backfill of 6 pre-fix-emission LogEntries from this session and drift-allowlist entry for template-only scripts/ subdirs.
- fix(actor-profile): create_actor MCP tool now emits actor.created LogEntry with spec.actor = new actor id (self-attribution); previously the emitted LogEntry was schema-invalid (missing required field). Closes BACK-20260421_0156.

### Changed

- chore(mcp): skill-consultation prompt added to every entity-mutating MCP tool description across all processkit servers (provider-neutral replacement for pre-tool-use hooks). Closes BACK-20260411_0802-SolidCrow.
- chore(AGENTS): slim root AGENTS.md to <60 lines of core content; domain-specific instructions moved into their owning skills; explicit if/then skill guards added. Closes BACK-20260411_0802-EagerSpruce.
- chore(grooming): rename 4 process files and 1 artifact to match canonical metadata.id prefixes (PROC-/ART-). pk-doctor WARN residual eliminated (5 → 0). Closes pk-doctor Phase 2 filename-rename intent.

### Added

- **Renamed skill**: morning-briefing → status-briefing (provider-neutral; time-of-day agnostic). Content migration emitted for downstream projects.

- **New skill: retrospective** — /pk-retro generates post-release blameless retrospectives (4 signals: release_summary, timeline, workitems, drift; dual-emit Artifact+LogEntry; --auto-workitems for proposed follow-ups; --verbose for full narrative including Appendix A raw signal dumps). See BACK-20260420_1340-LoyalFrog-add-pk-retro-skill.

- **feat(pk-doctor): new health-check aggregator skill + /pk-doctor slash command** (Phase 1). Detect-only by default; 4 checks: schema+filename validation against src/context/schemas/, sharding (logs YYYY/MM, migrations state-bucket), stale pending migrations, src/context drift. --fix/--fix-all opt-in; fixes route through existing MCP write tools (no hand-edits). Each run emits a doctor.report LogEntry via event-log MCP. See DEC-20260420_1631-WiseGarnet and BACK-20260420_1631-ProudGlade.
- **feat(migration-management): new MCP server exposing 5 tools**
  (fixes [#9](https://github.com/projectious-work/processkit/issues/9)).
  `list_migrations`, `get_migration`, `start_migration`,
  `apply_migration` (with implicit-start when called from pending),
  and `reject_migration`. Each write-side tool stamps the appropriate
  timestamp (`started_at` / `applied_at` / `rejected_at`), moves the
  Migration file between `pending/`, `in-progress/`, and `applied/`
  subdirectories, refreshes `context/migrations/INDEX.md` (preserving
  the Applied-table Notes column and the `## CLI Migrations` tail
  section), and writes a `migration.*` event via the side-effect log
  helper. Rejected migrations park under `applied/` and are listed in
  a dedicated `## Rejected` INDEX.md section. Adds
  `spec.started_at` to the Migration schema (optional,
  backward-compatible). See
  `DEC-20260420_1342-WarmClover` (shape) and
  `DEC-20260420_1353-ProudReef` (5 refinements).

---

## [v0.18.2] — 2026-04-18

### Fixed

- **Stale `mcp-config.json` paths block MCP server startup in derived
  projects** ([#8](https://github.com/projectious-work/processkit/issues/8)).
  12 of the 16 per-skill `mcp-config.json` files under
  `context/skills/processkit/<skill>/mcp/` shipped a `script` path that
  omitted the `processkit/` category segment introduced in v0.17.0's
  nested layout (`context/skills/<skill>/mcp/server.py` instead of
  `context/skills/processkit/<skill>/mcp/server.py`). Harnesses merged
  the broken path verbatim and then failed to launch the affected MCP
  servers on startup, forcing agents back to bash/python workarounds
  for `context/` writes. Latent since v0.17.0; masked until v0.18.6 of
  aibox because earlier aibox walker bugs prevented `.mcp.json` from
  being written at all. Mechanical fix — prepend `processkit/` to the
  script arg for: `actor-profile`, `binding-management`,
  `decision-record`, `discussion-management`, `event-log`,
  `gate-management`, `id-management`, `index-management`,
  `model-recommender`, `role-management`, `scope-management`,
  `workitem-management`. Mirrored into `src/context/`.
- **`AGENTS.md` Session start block points at `pk-resume`** (was stale
  `morning-briefing-generate`, renamed in v0.17.0).

### Added

- **`skip_decision_record(reason, session_id?)` MCP tool on
  `skill-gate`** — acknowledges the v2 contract's decision-language
  window without writing a record, when the agent concludes no decision
  needs recording (e.g. the user withdrew a proposal). Writes a
  `session-<id>.decision-skip` marker next to the existing `.ack` /
  `.decision-observed` markers (24 h TTL, empty reason rejected).
  `check_decision_captured.py` honours skip markers in block mode.
- **`compliance-contract.md` asset bumped to v2.** Adds the clause:
  "When the last five user messages contain explicit decision language
  (approved / decided / ship it / let's go / ok / yes / confirmed),
  either call `record_decision` in the same turn or call
  `skip_decision_record(reason=...)` to acknowledge the skip." Resolves
  the v1 asset vs v2 `AGENTS.md` inconsistency called out in the
  v0.18.1 known-issues block. Existing session markers (which embed
  the old contract hash) automatically invalidate; callers re-ack on
  next hook invocation.
- **`scripts/check-src-context-drift.sh` — release-time drift guard.**
  Runs `diff -rq context/ src/context/` with an allowlist for
  dogfood-only directories (`actors/`, `artifacts/`, `bindings/`,
  `decisions/`, `discussions/`, `logs/`, `migrations/`, `notes/`,
  `roles/`, `team/`, `workitems/`, root `INDEX.md`), runtime paths
  (`.cache/`, `.state/`, `__pycache__/`, `.gitkeep`), and `templates/`.
  Wired into `scripts/build-release-tarball.sh` as a pre-build step;
  non-zero exit fails the release before the tarball is built.
  Structural guard against the v0.15.0–v0.18.0 drift class.
- **Session-start skill-check checklist (SnappyTrout).** `AGENTS.md`
  and `session-handover/SKILL.md` now name six task classes (research
  ingestion, artifact creation, discussion management, decision
  recording, backlog item creation, quality audits) that must be
  preceded by a `search_entities(kind=skill, ...)` / `find_skill(...)`
  / `list_skills(...)` call. Extends, does not duplicate, the
  compliance-contract 1 % rule.
- **`/pk-standup` and `/pk-status` differentiated.** `/pk-standup` is
  now the brief daily standup (done / doing / next / blockers);
  `/pk-status` is the fuller status snapshot. Both invoke
  `standup-context`, different argument framing. Prior identical
  bodies were a v0.17.0 authoring glitch.

### Closed work

- `BACK-20260410_1049-SnappyTrout` — session-start skill-check
  (shipped here).
- `BACK-20260411_0802-LivelyTulip` — skill-finder MCP server
  (`find_skill`, `list_skills` already ship; verified by
  `scripts/smoke-test-servers.py`).

---

## [v0.18.1] — 2026-04-17

### Fixed

- **Release tarball now actually contains v0.15.0–v0.18.0 content**
  ([#7](https://github.com/projectious-work/processkit/issues/7)). The
  build script (`scripts/build-release-tarball.sh`) packages from
  `src/context/`, but content added since v0.14.0 had been landing only
  in the dogfooded `context/` tree. v0.18.0 and earlier tarballs were
  therefore missing features announced in their own CHANGELOG entries.
  v0.18.1 is a pure-sync release: no behavioural changes beyond bringing
  `src/context/` up to parity with what was already running in the
  dogfood.
- **`skill-gate/scripts/emit_compliance_contract.py` now emits
  `hookEventName`** in the `hookSpecificOutput` envelope, as required by
  Claude Code 2.1+. Reads `hook_event_name` from stdin (sent by every
  Claude Code hook invocation) and echoes it back as `hookEventName`.
  Falls back to `"UserPromptSubmit"` if stdin is empty or malformed. The
  Codex / plain-stdout path is unchanged. Resolves the
  `UserPromptSubmit hook error: invalid hookSpecificOutput` that was
  blocking every prompt in Claude Code 2.1+ consumer projects.
- **`check_route_task_called.py`** — marker-lookup decoupled from
  `session_id` (scans `.state/skill-gate/` for any valid marker with
  matching contract_hash + 12h TTL) so Claude Code sessions whose id
  shifts mid-session no longer lock out of MCP tools.
- **`test_hooks.py`** — adds coverage for SessionStart / UserPromptSubmit
  event-name echo, stale marker (> 12h), and hash-mismatch cases.

### Added (now actually shipped — were missing from v0.15.0–v0.18.0 tarballs)

- `team-creator` skill (SKILL.md + commands + references) — entire
  directory tree. Claimed in v0.15.0 CHANGELOG, previously only in the
  dogfood tree.
- All 26 `/pk-*` slash-command files across 13 skills — the rename from
  `<skill>-<verb>.md` to `pk-<verb>.md` happened in v0.17.0 and was
  extended in v0.18.0, but legacy names lingered in `src/context/`.
- Skill-gate Rail 5 scripts and fixtures:
  `check_decision_captured.py`, `decision_markers.py`,
  `decision_sweeper.py`, `record_decision_observer.py`,
  plus transcript and sessionend fixtures.
- `discussion-management/commands/`, `id-management/config/`,
  `index-management/config/` directories.
- Three `owner-profiling` assets (`OWNER-20260409_2054-*`).
- `actor.yaml` / `role.yaml` schema updates for CapabilityProfileRouting
  (`Role.model_profiles`, `Actor.model_profile_override`).
- Updated `AGENTS.md` template: compliance contract v2 marker + decision-
  language clause + Session start block.

### Known non-blocking inconsistency

- `AGENTS.md` template carries `pk-compliance v2` marker and the
  decision-language clause, but the `skill-gate/assets/compliance-
  contract.md` asset (printed by the hook) is still v1. Clauses remain
  a strict superset in v2; a follow-up release will reconcile both to v2
  and ship the `skip_decision_record` MCP tool referenced by the clause.
  Tracked in the dogfood backlog.

## [v0.18.0] — 2026-04-17

### Added

- **CapabilityProfileRouting — three-layer model-selection architecture**
  (DEC-20260417_1800). Replaces the single `preferences.model` string
  pin per Actor with:
  - **Layer A — Model catalog** (*what exists*): curated model registry
    inside `model-recommender` skill, refreshed via `/pk-model-refresh`.
  - **Layer B — Project/owner preferences** (*what this project can use*):
    subscription tier, API keys, cost bias, preferred ordering, owned by
    `model-recommender` skill's `set_config` / `get_config` MCP tools
    (`/pk-model-setup`).
  - **Layer C — Role standard sets** (*what this role needs*): each
    `Role` now carries a ranked `model_profiles` array (provider +
    family + default_version + default_effort + rationale, primary +
    fallbacks). `Actor` gains optional `model_profile_override` for rare
    per-actor deviations.
  - All 8 permanent team roles populated with ranked profiles covering
    Anthropic / OpenAI / Google / xAI / DeepSeek fallbacks. PM routing
    algorithm wired in `context/team/roster.md`.
- **13 additional `/pk-*` slash-command promotions** (total 26 /pk-*
  commands, 0 legacy processkit commands): `/pk-work`, `/pk-dec`,
  `/pk-dec-find`, `/pk-route`, `/pk-model-setup`, `/pk-model-refresh`,
  `/pk-groom`, `/pk-note-promote`, `/pk-note-review`,
  `/pk-owner-bootstrap`, `/pk-observe`, `/pk-skill-new`,
  `/pk-skill-audit`.
- **Actor schema**: optional `model_profile_override` field — identical
  shape to a single `model_profiles` entry, used only when a specific
  actor deviates from its role's primary.
- **Role schema**: required-on-new-roles `model_profiles` array with
  `rank`, `provider`, `family`, `default_version`, `default_effort`
  (enum: minimal / low / medium / high / xhigh / none), `rationale`.

### Fixed

- **Skill-gate PreToolUse hook decoupled from `session_id`** — the hook
  used to key marker-file matching on the harness-provided session id,
  but the processkit MCP server wrote markers under its own pid, so the
  two identifiers never matched and file edits silently blocked once
  the first legacy marker drifted. Hook now scans the marker directory
  for any marker whose `contract_hash` matches the current contract
  hash and whose `acknowledged_at` is within the 12 h TTL.
- **`emit_compliance_contract.py` echoes `hookEventName`** in its
  `hookSpecificOutput` JSON envelope — Claude Code 2.1+ rejects the
  envelope without it (`Hook JSON output validation failed —
  hookSpecificOutput is missing required field 'hookEventName'`). The
  script now reads `hook_event_name` from stdin and echoes it back.
- **`test_hooks.py`** — two new failure-mode tests (stale marker beyond
  TTL, marker with mismatched `contract_hash`). 13 cases total, all
  green.

### Changed

- **Team roster** rewritten with three-layer architecture section, PM
  routing algorithm (5 steps), Primary default + Fallbacks columns,
  effort-tier attribution for session handovers.
- **`preferences.model` string on Actor is deprecated** (no breaking
  migration — roles carry the routing intent now; actors stay identity-
  only unless overriding via `model_profile_override`).

### Notes

- `DEC-20260417_1800-CapabilityProfileRouting` was hand-written via a
  file Write (not the `record_decision` MCP tool) because the installer
  did not merge the per-skill `mcp-config.json` files into a harness-
  readable `.mcp.json` (filed upstream as aibox#53). The file carries a
  `CLEANUP-REQUIRED` marker; re-record through the MCP tool once the
  wiring ships.

---

## [v0.17.0] — 2026-04-17

### Added

- **13 `/pk-*` ergonomic slash commands** — the `/pk-<verb>` namespace
  is processkit's canonical command surface, provider-neutral across
  Claude Code, Codex CLI, Cursor, OpenCode (Aider falls back to
  AGENTS.md instructions). Three delivery surfaces:
  - **Skill-driven (9):** `/pk-resume` (session start),
    `/pk-status` + alias `/pk-standup` (mid-session status),
    `/pk-wrapup` (session end), `/pk-note` (fleeting capture with
    Zettelkasten qualified-link suggestion at capture time),
    `/pk-discuss` (structured multi-turn engagement),
    `/pk-research` (investigate with confidence labels),
    `/pk-release` (prepare a release), `/pk-publish` (push + publish).
  - **AGENTS.md-driven (3):** `/pk-test`, `/pk-build`, `/pk-lint` —
    read a structured `<!-- pk-commands -->` YAML block in AGENTS.md
    and execute the project's declared command. Fills the cross-harness
    gap (4 of 5 harnesses ship zero build/test coverage).
  - **Cross-cutting skill-driven (1):** `/pk-review` — processkit-
    opinionated code review against AGENTS.md conventions + compliance
    contract.
- **`<!-- pk-commands -->` YAML block in AGENTS.md** — machine-readable
  project-specific build/test/lint/fmt/typecheck declarations.
- **OpenWeave: team-creator 4-layer override surface** (FEAT-OpenWeave)
  — landscape artifact override (3-level precedence), DEC-*-TeamWeights
  (tag-based weight + threshold override), `role-archetypes.yaml`
  (project-level role→class pin override with delta/replace semantics
  and eager validation). team-creator bumped v1.1.0 → v1.2.0.
- **QuietLedger Rail 5: auto-capture of decisions** (FEAT-QuietLedger)
  — Lever 1 PreToolUse decision-language gate (shadow-mode-ON default;
  `--mode=block` available but NOT recommended until calibrated) +
  Lever 2 SessionEnd sweeper (writes Note artifact tagged
  `decision-candidates` for owner async review). New MCP tool:
  `skip_decision_record(reason, session_id)` on skill-gate.
  Compliance contract bumped v1 → v2 (new Rail 5 clause).
- **ShadowCount calibration** (RES-ShadowCount) — 9-session corpus,
  precision 6/19 = 0.316, recall 6/6 = 1.00. Verdict: NO-GO on
  `--mode=block` with current marker list. Marker tightening proposed.
- **DEC-CommandNexus** — strategic decision: `/pk-<verb>` namespace,
  processkit ships commands for any lifecycle phase not uniformly
  built-in across all major harnesses. Build/test/lint exclusion
  reversed.
- **CommandCompass research artifact** — cross-harness built-in command
  matrix + external landscape scan + gap analysis + proposal.

### Fixed

- **tool_use transcript filter** — PreToolUse gate + SessionEnd sweeper
  now filter `tool_use`, `tool_result`, `isCompactSummary`,
  `isSidechain`, and `<local-command-*>` entries from transcript reads.
  Prevents false-positive gate fires on agent write payloads that
  contain decision-language text (e.g., "ship it" in WorkItem YAML).

### Changed

- **Compliance contract v1 → v2** — new clause: "When the last five
  user messages contain explicit decision language, either call
  `record_decision` in the same turn or call `skip_decision_record`."
  Existing session acknowledgements re-prompt once on upgrade (contract
  hash changes).
- **team-creator v1.1.0 → v1.2.0** — 4-layer override docs + new
  `--threshold-overrides` CLI flag + agent-driven discovery section
  with trigger phrases for all 4 layers.
- **skill-finder** — 13 new trigger-phrase entries for `/pk-*` commands
  + v0.17.0 ergonomic shortcuts category section.

---

## [v0.16.0] — 2026-04-15

### Added

- **Canonical team-composition schema fields** — closes the aibox field
  report in issue #6, which requested that processkit land a canonical
  team primitive in place of their temporary `x_aibox.*` namespaced
  extension. Three new optional fields on `Role` (`primary_contact`,
  `clone_cap`, `cap_escalation`) and two on `Actor` (`is_template`,
  `templated_from`):
  - `Role.primary_contact` (bool, default `false`) — true for the role
    that is the single point of contact with the human owner. Exactly
    zero or one role per team may set this.
  - `Role.clone_cap` (int, default `5`) — per-role parallelism
    ceiling. The role with `primary_contact: true` must set
    `clone_cap: 1` (never cloned).
  - `Role.cap_escalation` (string, default `"owner"`) — who must
    approve exceeding `clone_cap`. Literal `"owner"` or an actor-ref.
  - `Actor.is_template` (bool, default `false`) — true for the
    canonical seed Actor that defines a role binding; clones derive
    from it. Enables index queries that separate seed team members
    from task-specific clones.
  - `Actor.templated_from` (Actor-ref, default `null`) — back-
    reference from a clone to its template Actor.
- **`team-creator` skill bumped 1.0.0 → 1.1.0** — emits the five new
  canonical fields on every `team-create` / `team-rebalance` run.
  Deactivation logic now uses `is_template: true` as the canonical
  seed identifier instead of the prior same-model-same-role heuristic.
- **Two new applied migrations** — `MIG-20260415T085311` (Role schema
  + data) and `MIG-20260415T095000` (Actor schema + data). Back-fill
  every existing Role/Actor entity so v0.15.0-installed teams carry
  the new fields automatically on upgrade.

### Changed

- **`role-management` skill bumped to v1.0.1** — documents the three
  new Role fields and their validation invariants.
- **`actor-profile` skill bumped to v1.0.1** — documents the two new
  Actor fields and their validation invariants.

### Upstream migration note

Consumers carrying the aibox-style `spec.x_aibox.*` extension can lift
those fields into the canonical names with a one-to-one mapping:
`x_aibox.is_template` → `is_template`, `x_aibox.clone_of` →
`templated_from`, `x_aibox.default_clone_cap` → `clone_cap`,
`x_aibox.escalate_cap_to` → `cap_escalation`, plus a new field
`primary_contact` set true on the PM-equivalent role.

---

## [v0.15.0] — 2026-04-15

### Added

- **`team-creator` skill** — provider-neutral team composition. Tiers
  available models into heavy / medium / light using a weighted
  formula over Capability (0.60), Cost-efficiency (0.20), Latency
  (0.10), and Governance (0.10), then maps eight role archetypes
  (PM, sr-architect, jr-architect, developer, sr-researcher,
  jr-researcher, jr-developer, assistant) onto the tiered models.
  Three commands: `team-create` (full derivation, writes 24 entities
  + roster + DecisionRecord), `team-review` (read-only diff vs latest
  landscape), `team-rebalance` (targeted re-tier of named roles).
  Composes `model-recommender`, `role-management`, `actor-profile`,
  `binding-management`, `decision-record-write` — no new primitives,
  no MCP server. Defaults reproduce the v0.14.0 manually-composed
  team 8/8 on Anthropic Max 5×. Validated by Phase 3 dogfood
  (`ART-20260415_1545-TeamWeaver-team-creator-dogfood-diff`).
- **Session-orientation wiring** — AGENTS.md gains a "Session start"
  section under the compliance-contract block: agents run
  `morning-briefing-generate` before acting (provider-neutral, every
  AGENTS.md-aware harness picks it up). `emit_compliance_contract.py`
  hook script extended with `--include-session-start` flag for
  Claude Code / Cursor reinforcement.
- **6 new artifacts under `context/artifacts/`** — TeamWeaver Phase 1
  design, OpenWeave Phase 1 design, landscape snapshot (HTML +
  structured summary), Phase 3 dogfood diff, Rail 5 research.
- **3 new follow-up WorkItems filed** — `FEAT-OpenWeave` (4-layer
  override design done; implementation queued), `FEAT-QuietLedger`
  (Rail 5 L1+L2 implementation), `RES-GapScout` (research closed).

### Changed

- **`morning-briefing` skill bumped 1.0.0 → 1.1.0** — adds
  `context/migrations/pending/` to "Sources to read" and emits a
  one-line token-budget-share snapshot (Opus / Sonnet / Haiku
  actuals vs ≈5/85/10 target, flagging drift > ±10pp per
  `DEC-20260414_0900-TeamRoster`).

### Closed

- **`ARCH-20260414_1245-FirmFoundation`** — enforcement
  implementation plan closed. All four originally-blocking FEAT
  items (Rails 1–4) plus the two reconstructed follow-ups
  (PathFinder, TeamWeaver) shipped.

---

## [v0.14.0] — 2026-04-14

### Added

- **Compliance contract (Rail 1)** — canonical
  `context/skills/processkit/skill-gate/assets/compliance-contract.md`
  is the single source of truth for the eight enforceable rules. AGENTS.md
  carries a verbatim copy between `<!-- pk-compliance-contract v1 BEGIN -->`
  and `<!-- pk-compliance-contract v1 END -->` markers at the top of the
  file (primacy header), so every harness that reads AGENTS.md surfaces
  the contract before the rest of the project narrative.
- **`acknowledge_contract()` MCP tool (Rail 3)** — added to the
  `skill-gate` MCP server. Records a per-session acknowledgement marker
  under `context/.state/skill-gate/session-<id>.ack`. Paired with the
  PreToolUse hook script (Rail 2) so writes under `context/` block until
  the contract has been acknowledged once per session. The server also
  ships `read_contract()` and `check_acknowledged()`.
- **Provider-neutral hook scripts (Rail 2)** — stdlib-only Python scripts
  shipped under `context/skills/processkit/skill-gate/scripts/`:
  `emit_compliance_contract.py` (SessionStart / UserPromptSubmit) and
  `check_route_task_called.py` (PreToolUse). Harness wiring (Claude Code
  `settings.json`, Codex CLI `hooks.json`, etc.) is aibox's responsibility;
  fixtures captured from a real Claude Code 2.1+ session ship as golden
  files alongside the scripts. Includes `test_hooks.py` for CI.

### Changed

- **1% rule sentence in 8 MCP tool descriptions (Rail 4)** — the
  entity-mutating tools on `artifact-management`, `decision-record`,
  `discussion-management`, `event-log`, `skill-finder`, and
  `workitem-management` MCP servers now carry a one-line reminder in
  their docstring: `route_task()` first if there is even a 1% chance a
  processkit skill applies. Always-on, zero-dependency rail visible in
  the tool schema every turn.

### Notes

- v0.14.0 is the **processkit (Bucket A) half** of the enforcement
  rollout. The matching aibox-side wiring (merged MCP config per
  harness, hook installation, drift report) is tracked as upstream
  issues #43–#51 on `projectious-work/aibox`. Until aibox ships those,
  the rails are present in the source tree but only Rail 1 (prose) and
  Rail 4 (tool descriptions) are visible to derived projects.

---

## [v0.13.0] — 2026-04-11

### Added

- **`task-router` skill and MCP server** — primary routing entry point
  for processkit agents. `route_task(task_description)` returns the
  matching skill, project-specific process override (from
  `context/processes/`), and recommended MCP tool in a single call
  without an LLM call. Two-phase heuristic routing: keyword match
  against 13 domain groups (Phase 1), token-overlap scoring within
  the group's tools (Phase 2), skill-finder trigger-table fallback for
  cross-domain tasks. Return shape includes `skill_description_excerpt`
  (first 150 chars), `tool_qualified` (`{server}__{tool}` collision-safe
  naming), `confidence`, `routing_basis`, and `candidate_tools[]`.
- **`skill-finder` MCP server** — new MCP server for the existing
  skill-finder skill. Tools: `find_skill(task_description)` (trigger-
  phrase table lookup + token-overlap scoring) and `list_skills(category?)`
  (catalog browser). Both tools are read-only with `readOnlyHint=true`.
- **`skill-gate` meta-skill** — provider-neutral prose skill that
  enforces the 1% rule: if there is even a 1% chance a processkit skill
  covers the task, check the router first. Includes decision graph,
  rationalization pre-emption table (5 entries), and escape hatch for
  agents already operating inside a named skill workflow.

### Changed

- **MCP tool prerequisite prompts (Track C)** — all 20 entity-mutating
  MCP tools (`create_*`, `transition_*`, `link_*`, `record_*`, `open_*`)
  now carry a prerequisite sentence in their docstring: call
  `route_task()` or confirm you are inside a named skill workflow before
  using the tool. These prompts appear in the tool schema every turn.
- **`skill-builder` and `skill-reviewer` updated (Track E)** — skill
  `description:` field convention changed from "Use when…" summaries to
  one-sentence imperatives (verb-noun, ≤150 chars). Both skills enforce
  this: skill-builder's step 4 template uses the new format; skill-
  reviewer's Category 6 and Skill Killer #1 check for violations.
- **`AGENTS.md` routing reference** — `find_skill()` replaced by
  `route_task()` as the primary 1% rule entry point. `task-router` added
  to the mandatory MCP server table.
- **`context/processes/release.md`** — fleshed out from generic stub to
  processkit-specific: added `breaking-change-audit`, `update-docs-site`,
  `stamp-provenance`, `push`, `build-and-upload-release`, `deploy-docs`
  steps; docs-deploy WildButter/aibox#42 blocker documented.

---

## [v0.12.0] — 2026-04-11

### Added

- **`artifact-management` skill and MCP server** — new Layer 2
  processkit primitive for registering and retrieving completed
  deliverables (documents, datasets, builds, diagrams, URLs, etc.).
  Supports two usage patterns: self-hosted (Markdown body in the
  entity file) and pointer (external URL or path via `location`).
  MCP tools: `create_artifact`, `get_artifact`, `query_artifacts`,
  `update_artifact`. Artifact has no state machine (`state_machine:
  null`) — it is a catalogue record, not a work-tracking entity.
- **`skill-finder` updated** — new trigger phrases for
  `artifact-management` (`"register an artifact"`, `"catalog this
  document"`, `"store this deliverable"`, `"link this design file"`)
  and a one-liner in the Process category.
- **`context/skills/INDEX.md`** — `processkit/` skill count updated
  to 31; Layer 2 entry updated to include `artifact-management **MCP**`.

---

## [v0.11.1] — 2026-04-11

### Fixed

- **`ids.py`: `pascal` and `camel` are now distinct word styles** —
  `pascal` produces PascalCase (`BoldVale`, every word capitalised);
  `camel` now correctly produces true camelCase (`boldVale`, first word
  lowercase). Previously `camel` incorrectly generated PascalCase.
  `id-management/config/settings.toml` updated to `word_style = "pascal"`.

### Changed

- **`context/` dogfood mirror synced** — `resolve_entity()` and
  partial-ID lookup now live in the installed `context/` copies of
  `index.py` and all MCP servers, consistent with the `src/`
  implementation shipped in v0.10.0.
- **`session-handover` SKILL.md** — log-entry writing steps updated
  with `generate_id` call, date-sharded path derivation, and word-pair
  ID template.
- **`context/skills/INDEX.md`** — expanded with skill package layout
  section and `_lib/` note.
- **`aibox.lock`** — updated to aibox v0.17.12, processkit v0.10.0.

---

## [v0.11.0] — 2026-04-11

### Added

- **Note schema: `links` field** — qualified Zettelkasten note-to-note
  links with `target`, `relation` (enum: elaborates, contradicts,
  supports, is-example-of, see-also, refines, sourced-from), and a
  required `context` sentence explaining *why* the connection matters.
  Tags group notes by topic; links build arguments.
- **note-management SKILL.md: linking section** — new "Linking notes"
  section with a full relation table and usage guidance; note template
  updated with links example.

### Changed

- **Note schema: type descriptions aligned with Luhmann/Ahrens taxonomy**
  — `insight` = permanent note (never discarded, part of the knowledge
  base); `reference` = literature note; `fleeting` = fleeting note.
  Schema description clarifies that permanent notes are not ephemeral.
  `type` descriptions and note-management SKILL.md type table updated
  accordingly.
- **Artifact schema: self-hosted and pointer patterns both documented** —
  description updated to acknowledge two valid usage patterns:
  self-hosted (Markdown body in the file, `location` optional) and
  pointer (external URL/path, `location` required). `location` removed
  from the `required` array. Note vs Artifact distinction added.
- **`config.py` `_skill_config_dir`** — now tries the `processkit/`
  category subdirectory before falling back to the flat layout, fixing
  ID generation and config loading after the v0.10.0 skills
  reorganization.
- **`.mcp.json`** — all 12 MCP server paths corrected from
  `context/skills/<name>/` to `context/skills/processkit/<name>/` after
  the v0.10.0 SteadyLeaf reorganization.
- **`AGENTS.md`** — added AGENTS.md lean-scope principle; expanded
  `context/` layout table to all 15 directories including `artifacts/`,
  `decisions/`, `discussions/`, `notes/`, `workitems/`, `logs/`,
  `migrations/`, `owner/`, `actors/`, `roles/`.

---

## [v0.10.0] — 2026-04-10

### Added

- **`index.resolve_entity()`** — new shared library function with 3-stage ID
  resolution: exact match → prefix match (missing slug) → word-pair match (bare
  word-pair like `StoutCrow`). Enables agents and humans to look up entities by
  the colloquial word-pair shorthand rather than needing the full canonical ID.

### Changed

- **Skills directory reorganized into 7 category subdirectories** (SteadyLeaf):
  `processkit/` (30), `engineering/` (46), `devops/` (15), `data-ai/` (11),
  `product/` (11), `documents/` (8), `design/` (5). All 126 skills moved in
  both `src/context/skills/` and `context/skills/`. All SKILL.md files updated
  with a consistent `category:` frontmatter field.
- **`skill-finder`** updated with a "By directory" table mapping the 7 categories
  to their on-disk subdirectory names.
- **`get_workitem`**, **`get_decision`**, **`get_entity`** (index-management) now
  use `resolve_entity()` and return `{"error": "..."}` instead of `null`/`None`
  when the entity is not found, and `{"error": "ambiguous..."}` with a candidate
  list when multiple entities match.
- **All `_load_*` helpers** (`_load_scope`, `_load_gate`, `_load_actor`,
  `_load_role`, `_load_discussion`, `_load_decision`, `_load_workitem`) updated
  to use `resolve_entity()` so word-pair lookup also works in write tools
  (transition, link, etc.).
- **`smoke-test-servers.py`** updated to find MCP server files in the new
  category subdirectory layout.

### Fixed

- `get_workitem` and `get_decision` previously returned `null` for not-found
  IDs, making it impossible for callers to distinguish "not found" from
  "found but empty". Both now return a structured error dict (SnappyCrane).
- Agents looking up entities by word-pair shorthand (e.g. `StoutCrow` instead
  of the full `BACK-20260410_1050-StoutCrow-create-brand-design-skill`) now
  resolve correctly via the new `resolve_entity()` fallback chain (SteadyPeak).

---

## [v0.9.0] — 2026-04-09

`src/` → `context/` mirror restructure (GrandLily). All processkit content
now lives under `src/context/` in the source tree and is mirrored to
`context/` in the target project root on `aibox sync`.

---

## [v0.8.0] — 2026-04-09

Auto-log side effects in all entity-mutating MCP servers. Every
`create_*`, `transition_*`, and `link_*` call now appends a `LogEntry`
without the caller doing anything extra.

---

## [v0.7.0] — 2026-04-09

Initial processkit v0.7.0 release. Core entity MCP servers, state machines,
schemas, and 85+ skills migrated from the original aibox templates.
