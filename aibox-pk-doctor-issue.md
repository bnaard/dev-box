# pk-doctor reports generated-project defects and generated-content false positives

## Summary

`aibox` created this project from scratch. The project contains no source
code and its project files, processkit context, diagnostics, plugin cache,
and authentication configuration were created or provisioned by `aibox`.

Running `/pk-doctor` immediately afterwards reports a large number of
actionable errors and warnings. This is not an acceptable baseline for a
newly generated project: the generated project should either satisfy the
doctor checks or the checks should understand which generated/local files
are outside the project health boundary.

The current run reports:

| Severity | Count |
| --- | ---: |
| ERROR | 9 |
| WARN | 56 |
| INFO | 43 |
| Actionable | 65 |

All 65 actionable findings are marked as requiring user confirmation. The
doctor reports zero safe fixes, zero tracking recommendations, and one
archive-needed item.

## Environment and reproducibility

- Repository/project was created from scratch with `aibox`.
- The project has no application source code.
- The generated project does not contain `src/` or `scripts/` at its root.
- The doctor scan walked 3,250 files and skipped 3,075 files classified as
  examples.
- The deterministic sensitive-data scan reported 63 findings.
- The findings are predominantly under ignored `.aibox/` and
  `.aibox-home/` paths, including generated diagnostics, authentication
  state, and downloaded plugin content.

## Findings

### 1. Required drift script is missing

Doctor reports:

```text
ERROR drift.script-missing
scripts/check-src-context-drift.sh not found
```

The doctor implementation expects to execute this file with
`--release-deliverable`. However, a newly created `aibox` project has no
root `scripts/` directory and no `src/` tree. This appears to be an
`aibox` generation or project-boundary bug rather than a user defect.

Please clarify and implement one of these consistent behaviors:

1. Generate the required script and the `src/context` release boundary when
   the project is expected to be a canonical source/release repository; or
2. Mark the check as not applicable for a derived/empty `aibox` project; or
3. Have `aibox` install the complete release-boundary tooling before
   enabling this doctor check.

The current behavior makes a clean doctor result impossible immediately
after project creation.

### 2. Sensitive-data scanner scans generated local state and plugin cache

The doctor reports 63 deterministic findings in generated or local-only
content. These files are ignored by Git, but they are still scanned and
turn a new project into a failed health check.

The examples below are intentionally redacted. No live credential is
included in this issue.

#### Phone-like values: 20 findings

Examples:

```text
WARN sensitive-data.phone-number
.aibox/diagnostics/latest.json:1: <redacted phone-like value>
.aibox/diagnostics/snapshot-00.json:1: <redacted phone-like value>
.aibox/diagnostics/snapshot-01.json:1: <redacted phone-like value>
```

These are generated diagnostic snapshots. They should either be sanitized
when diagnostics are written or excluded from the project-health scan when
they are known local/generated artifacts.

#### Email addresses: 20 findings

Examples:

```text
WARN sensitive-data.email-address
.aibox-home/.codex/.tmp/plugins/.agents/skills/plugin-creator/scripts/create_basic_plugin.py:50
  author@<redacted>.com

.aibox-home/.codex/.tmp/plugins/plugins/atlassian-rovo/skills/
capture-tasks-from-meeting-notes/SKILL.md:181
  sarah@<redacted>.com
```

These occur in generated plugin metadata, plugin documentation, and
example content. The scanner should distinguish synthetic/public example
addresses from user data, or `aibox` should sanitize/mark these files as
vendor-generated examples.

#### URL-embedded credentials: 3 ERROR findings

Examples:

```text
ERROR sensitive-data.url-credential
.aibox-home/.codex/.tmp/plugins/plugins/cloudflare/skills/wrangler/SKILL.md:495
  postgres://<user>:<redacted-password>@<host>/...

.aibox-home/.codex/.tmp/plugins/plugins/render/skills/render-keyvalue/SKILL.md:58
  redis://<user>:<redacted-password>@<host>/...

.aibox-home/.codex/.tmp/plugins/plugins/twilio-developer-kit/skills/
twilio-webhook-architecture/SKILL.md:353
  https://<user>:<redacted-password>@<host>/...
```

These appear in downloaded plugin skill documentation. If they are
documentation examples, the upstream/plugin import should replace them
with unmistakable placeholders such as `<password>` before installation,
or the scanner should recognize example context. If any value is real, it
must be rotated and removed before distribution.

#### Assigned high-entropy secrets: 13 findings

Examples:

```text
WARN sensitive-data.generic-assigned-secret
.aibox-home/.codex/.tmp/plugins/plugins/codex-security/scripts/rank_preview.py:335
  token = <redacted>

.aibox-home/.codex/.tmp/plugins/plugins/vercel/commands/status.md:148
  secret = <redacted>

.aibox-home/.codex/.tmp/plugins/plugins/vercel/skills/chat-sdk/SKILL.md:467
  client_secret = <redacted>

.aibox-home/.codex/.tmp/plugins/plugins/vercel/skills/v0-dev/SKILL.md:116
  apiKey = <redacted>

.aibox-home/.codex/.tmp/plugins/plugins/zoom/skills/general/use-cases/
retrieve-meeting-and-subscribe-events.md:558
  password = <redacted>
```

These are in generated/vendor plugin content. The installer should either
sanitize example assignments, preserve explicit example markers, or keep
vendor cache content outside the default doctor scan. A real value must be
rotated; this report does not assert that these values are live.

#### Credit-card-like values: 5 ERROR findings

Examples:

```text
ERROR sensitive-data.credit-card
.aibox-home/.codex/.tmp/plugins/plugins/replayio/skills/replay-qa-api/SKILL.md:71
  000000...0000

.aibox-home/.codex/.tmp/plugins/plugins/superhuman/.codex-plugin/plugin.json:9
  498107...2067

.aibox-home/.codex/.tmp/plugins/plugins/twilio-developer-kit/skills/
twilio-compliance-onboarding/SKILL.md:31
  833862...5147
```

These appear to be test/example numbers, including an all-zero value, but
the Luhn-based detector treats them as card-like. Example/test fixtures
should be excluded or the detector should require stronger contextual
signals before raising an ERROR.

#### JWT-like values: 2 findings

Examples:

```text
WARN sensitive-data.jwt
.aibox-home/.codex/auth.json:5: eyJhbG...nR24
.aibox-home/.codex/auth.json:6: eyJhbG...lvuQ
```

This file is generated authentication state and is ignored by Git. The
doctor should not report local auth state as a project defect by default,
but it must still protect users if the file is ever copied into a tracked
or distributable tree. The generated auth path should be explicitly
classified as local secret material and handled by a dedicated credential
check rather than the project-content check.

### 3. Archive warning is raised before the policy threshold

Doctor reports:

```text
WARN archive.applied-migrations
1 applied migration(s) are archive candidates
```

The migration is:

```text
MIG-LOCK-20260720T113122
summary: Backfilled previous_selection: 7 addon(s), 22 tool(s), 1 harness(es)
applied_at: 2026-07-20T11:55:52+00:00
```

The configured migration archive policy is 30 days. A policy-aware archive
plan for `Migration / applied / older_than_days=30` returns zero candidates.
This warning is therefore premature and should not be emitted until the
entity is actually eligible for archival.

### 4. Previously reported agent-only projection findings

An earlier run reported five agent-only skill projections without canonical
commands. The current rerun does not reproduce those findings:

```text
INFO commands_consistency.ok
all skill commands: metadata entries have matching commands/*.md files

INFO team_member_exports.in-sync
1 active TeamMember(s) match 1 export file(s) under .claude/agents/
```

Please still verify that the generation path is deterministic. A fresh
project should not intermittently produce projections that fail the doctor
depending on installation order or cache state.

## Expected behavior

For a newly created empty project, `aibox` should produce either:

- a doctor-clean project; or
- a clearly scoped doctor configuration in which generated local state,
  vendor/plugin caches, and not-applicable release checks are excluded
  without suppressing checks over actual project content.

The doctor should also apply age thresholds consistently: an archive warning
must not identify an applied migration as eligible before the configured
30-day threshold.

## Suggested acceptance criteria

- A fresh `aibox` project does not fail because
  `scripts/check-src-context-drift.sh` is absent when no source/release
  tree is expected.
- The generated project and the doctor agree on the project boundary.
- Generated diagnostics, local auth state, and downloaded plugin caches are
  either sanitized or excluded from the default project-content scan.
- Synthetic examples do not trigger credential, phone, email, or payment
  card errors without appropriate context.
- Real credentials remain detectable when they occur in tracked or
  distributable project content.
- Archive candidates honor the configured age threshold.
- Repeated fresh-project generation produces the same doctor result.

## Reproduction

```sh
aibox <create-project-command>
cd <generated-project>
/pk-doctor
```

Observed result: 9 ERROR, 56 WARN, 43 INFO, and 65 actionable findings.

