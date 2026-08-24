# aibox v1 product and engineering specification

| Field | Value |
|---|---|
| Status | Draft for owner review |
| Specification version | 1.0.0-draft.1 |
| Intended product line | aibox v1.x |
| Primary implementation | New Go CLI and optional Go `aiboxctl` |
| Last updated | 2026-08-23 |

This directory is the canonical draft specification for the from-scratch aibox
v1 implementation. The existing Rust implementation and v0 configuration are
evidence and migration inputs, not implementation baselines or compatibility
constraints.

The words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are
normative requirement levels. Requirement identifiers are stable references
for implementation plans, tests, reviews, documentation, and release evidence.

## Product statement

aibox is the agent-native environment execution boundary for reproducible AI
workspaces. Its MCP and CLI adapters compile versioned project intent and
standards-based templates into inspectable container-image and deployment
bundles, then delegate build and deployment to native tools. It supports local
and remote Docker-compatible hosts, Kubernetes targets, interactive and
headless workloads, and an optional in-container `aiboxctl` for bounded runtime
customization of the current environment.

Templates own image content, addons, deployment topology, and runtime features.
The engine knows their contracts, not their installed tools. Processkit is an
ordinary template-selected addon rather than an aibox subsystem.

## Specification map

1. [Product boundary](01-product-boundary.md)
2. [Concepts, dimensions, and layouts](02-concepts-and-layouts.md)
3. [Intent, templates, and compilation contracts](03-contracts-and-compilation.md)
4. [CLI and lifecycle](04-cli-and-lifecycle.md)
5. [Targets and remote execution](05-targets-and-remote-execution.md)
6. [Security and secrets](06-security-and-secrets.md)
7. [Software architecture](07-software-architecture.md)
8. [Template authoring and runtime customization](08-template-authoring.md)
9. [Configuration, migration, output, and evidence](09-configuration-output-and-migration.md)
10. [Build, testing, and quality](10-build-testing-and-quality.md)
11. [Documentation, releases, and company standards](11-documentation-release-and-standards.md)
12. [Acceptance and implementation roadmap](12-acceptance-and-roadmap.md)
13. [Agent-native posture and guarded MCP](13-agent-native-mcp.md)

Supporting material:

- [Implementation roadmap](roadmap.yaml)
- [Illustrative project intent](../../examples/v1/aibox.toml)
- [Illustrative local operational configuration](../../examples/v1/aibox-local.toml.example)
- [Illustrative template manifest](../../examples/v1/aibox-template.toml)

The examples are design fixtures. They become conformance fixtures when their
schemas are introduced during the contracts phase.

## Decision provenance and specification authority

This specification incorporates the accepted company decision
`DEC-20260823_1428-CalmWolf-define-aibox-v1-x-rewrite-architecture`: rewrite
aibox and optional aiboxctl in Go; separate engine and templates; preserve
native deployment escape hatches; model targets and secret delivery as
orthogonal dimensions; support SOPS, OpenBao, and future attestation-gated KBS.

It also incorporates
`DEC-20260824_0139-TenderFlame-separate-aibox-external-environment-management-from`:
aibox manages only other environments, `aiboxctl` manages only the current
environment, and no bridge gives a managed environment authority over its
creator.

The checked-in specification is self-contained. External discussion or a
decision record is provenance, not hidden normative authority. A changed
outcome affects implementation only after a reviewed specification amendment
updates every affected requirement, schema, example, acceptance journey, and
roadmap entry.

- **AIBOX-SPEC-001:** implementation and conformance review MUST use the
  accepted checked-in specification baseline as normative authority.
- **AIBOX-SPEC-002:** provenance references MUST NOT independently override the
  checked-in specification.
- **AIBOX-SPEC-003:** planned behavior MUST NOT be presented as shipped product
  behavior before its roadmap phase has conformance evidence.

## Conformance model

There are four independent claims:

- **CLI conformance:** the host `aibox` binary satisfies its lifecycle,
  compilation, security, output, and compatibility requirements.
- **MCP conformance:** `aibox mcp serve` exposes guarded agent operations over
  the same application core with CLI-equivalent lifecycle and recovery.
- **Template conformance:** a template satisfies its manifest, source,
  documentation, feature, target, and fixture contracts.
- **Bundle conformance:** a rendered bundle is secret-free, deterministic for
  declared inputs, attributable to its template and intent, and consumable by
  the declared standard tools.
- **Runtime-controller conformance:** optional `aiboxctl` satisfies only its
  declared in-container customization contract and does not become a secret
  manager or deployment engine.

Schemas validate owned document structure. They do not replace behavioral,
security, lifecycle, or target-tool requirements.

## Publishing format

The numbered chapter files are portable Markdown fragments for repository
review and later inclusion in the public documentation site. They contain no
front matter or page-level H1, begin headings at H2, and avoid site-specific
shortcodes.
