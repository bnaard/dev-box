## Governing company standards

The specification adopts the current projectious.work standards:

- Git branching and release promotion;
- software verification and release engineering;
- security and software supply chain;
- application configuration;
- application output, logging, and evidence;
- application profiles;
- compatibility and machine interfaces;
- product roadmap and development evidence;
- spec-driven development cycle;
- human-controlled host-phase execution and host-gated conformance;
- open-source documentation strategy; and
- AI-agent accessibility and generative discovery; and
- agent-native product interfaces.

The company coordination repository remains the canonical source of those
standards. This chapter records their aibox-specific application so the product
specification is actionable without hidden requirements.

## Branching and version line

Development uses short-lived `feat/*` and `fix/*` branches into `v1.x-dev`.
`v1.x-pre-release` and `v1.x-release` are fast-forward-only promotion pointers
and receive no unique commits. Alpha/beta tags come from pre-release; RC/final
tags come from release; the stable commit fast-forwards to `main`.

Specification, schemas, examples, documentation, code, and release metadata
follow the same reviewed path. Promotions preserve commit identity and stop on
divergence. Tags are immutable, annotated, and should be signed.

## Versioning and compatibility

Product releases use SemVer. Owned contract versions evolve independently
when necessary. Every public or persisted contract defines compatibility,
unknown fields/versions, support window, deprecation, migration, rollback, and
fixtures. Security-default changes and validation tightening receive semantic
compatibility review rather than being called compatible based only on schema
shape.

The v0-to-v1 transition is a major migration. V1 may read v0 intent for
diagnosis and deterministic migration, but it does not preserve deprecated
engine-owned features. Their replacements are template selections or native
overrides.

## Documentation system

The repository remains the source of truth. User-visible implementation phases
update user documentation in the same change; planned specification text is
clearly labeled and must not leak into current-version instructions.

Required public surfaces include README, documentation site, installation and
quick start, concepts, template authoring, configuration/schema reference,
CLI/machine contract, target compatibility, security/trust model, secrets
guidance, remote operation, troubleshooting/recovery, migration, roadmap,
releases/changelog, contribution, security reporting, license, and community
files.

Examples are executable promises. The docs build, links, generated reference,
and deployment commands are validated locally; the repository adds no
project-authored GitHub Actions.

## AI-agent accessibility

Released product documentation has a canonical Markdown projection, semantic
HTML, `/llms.txt`, bounded `/llms-full.txt` when useful, sitemap, crawl policy,
and accurate version/provenance signals. A documented read-only MCP resource
server exposes released public product material and owned schemas but not
processkit entities, private context, credentials, unpublished plans, or
arbitrary repository files.

Optional product tools may validate intent/template content or return proposed
starter content. Mutating deployment tools are not required to satisfy the
discovery standard and need an independent authority/security specification
before inclusion.

The guarded operational MCP surface defined in
[Agent-native posture and guarded MCP](13-agent-native-mcp.md) is separate from
this read-only discovery baseline. Public documentation leads with the
agent-native environment-execution posture, presents an MCP journey beside the
equivalent CLI journey, and explains the executor/target and aibox/aiboxctl
boundaries.

- **AIBOX-DOC-001:** documentation MUST NOT describe a host bridge, creator
  callback, mounted engine socket, or `aiboxctl` lifecycle proxy as supported.
- **AIBOX-DOC-002:** documentation MUST state that most managed environments do
  not install aibox; template/environment engineering workloads may install it
  to manage distinct downstream targets.
- **AIBOX-DOC-003:** documentation MUST distinguish probabilistic intent and
  template authoring from deterministic validation, authorization, execution,
  evidence, cleanup, and recovery.

## Spec-driven development

Every roadmap phase begins from an accepted spec commit and records applicable
requirements, schemas, examples, decisions, standards, risks, dependencies,
tests, documentation, and evidence. The implementation plan maps requirement
IDs to work and receives independent review. Implementation drift does not
silently amend the spec. Completion requires code, tests, specification,
documentation, development note, roadmap, and release identity to agree.

## Release artifacts

The intended host CLI matrix is Linux/macOS `amd64` and `arm64`. Release
artifacts contain binaries, checksums, SBOMs, licenses/notices, provenance,
signatures/attestations, changelog/release notes, and documented verification.
If `aiboxctl` is shipped separately, its platform/image artifact identity is
equally bound. Published template packages carry independent identity, digest,
compatibility, license, and provenance.

- **AIBOX-REL-001:** no release is built from a dirty worktree or unaccepted
  specification state.
- **AIBOX-REL-002:** release artifacts MUST derive from the exact immutable
  candidate and identify source/toolchain.
- **AIBOX-REL-003:** release validation MUST fail on required unexpected skips,
  stale evidence, changed subjects, or incomplete cleanup.
- **AIBOX-REL-004:** documentation and AI-facing surfaces MUST describe the
  same shipped contract and version.
