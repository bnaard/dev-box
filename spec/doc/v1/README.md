# aibox v1 product and engineering specification

| Field | Value |
|---|---|
| Status | Draft for owner review |
| Specification version | 1.0.0-draft.1 |
| Intended product line | aibox v1.x |
| Primary implementation | New Go CLI and optional Go `aiboxctl` |
| Last updated | 2026-08-24 |

This directory is the canonical draft specification for the from-scratch aibox
v1 implementation. The existing Rust implementation and v0 configuration are
evidence and migration inputs, not implementation baselines or compatibility
constraints.

The words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are
normative requirement levels. Requirement identifiers are stable references
for implementation plans, tests, reviews, documentation, and release evidence.

## Product statement

aibox is the agent-native environment execution boundary for reproducible AI
workspaces. Its MCP and CLI adapters validate standard Dev Container
definitions, bind them to authorized targets through saved deployment plans,
and delegate build and lifecycle mechanics to established tools through
internal Go adapters. It supports local
and remote Docker-compatible hosts, Kubernetes targets, interactive and
headless workloads, and an optional in-container `aiboxctl` for bounded runtime
customization of the current environment through both CLI and bounded MCP.

Standard Dev Container Templates, Features, Dockerfiles, Compose, Kubernetes
material, and native configuration own environment content. Aibox defines no
parallel addon/component/value language. Processkit and harnesses are ordinary
Features or project tooling rather than aibox subsystems.

## Specification map

1. [Product boundary](01-product-boundary.md)
2. [Concepts, dimensions, and layouts](02-concepts-and-layouts.md)
3. [Deployment intent, planning, and runtime adapters](03-contracts-and-compilation.md)
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
- [Illustrative Dev Container Template metadata](../../examples/v1/devcontainer-template.json)
- [Illustrative Dev Container definition](../../examples/v1/devcontainer.json)

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

The standard-artifact and runtime-adoption amendment incorporates
`DEC-20260824_0946-AbleDew-base-aibox-v1-on-standard-dev`,
`DEC-20260824_0946-ShinyLily-compile-runtime-adapters-into-the-aibox`, and
`DEC-20260824_0946-CrispPlum-align-aibox-lifecycle-ux-with-ainfra`.

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
  planning, adapter, security, output, and compatibility requirements.
- **MCP conformance:** `aibox mcp serve` exposes guarded agent operations over
  the same application core with CLI-equivalent lifecycle and recovery.
- **Definition conformance:** standard Templates, Features, native sources,
  documentation, target material, and fixtures satisfy upstream and aibox
  policy requirements.
- **Plan conformance:** a saved plan is secret-free, bound to declared inputs,
  target, adapter/tool versions, policy and authorization, and exactly applied.
- **Runtime-controller conformance:** optional `aiboxctl` and its stdio MCP
  projection route only policy-allowed current-environment providers and do not
  become a secret manager or deployment engine.

Schemas validate owned document structure. They do not replace behavioral,
security, lifecycle, or target-tool requirements.

## Publishing format

The numbered chapter files are portable Markdown fragments for repository
review and later inclusion in the public documentation site. They contain no
front matter or page-level H1, begin headings at H2, and avoid site-specific
shortcodes.
