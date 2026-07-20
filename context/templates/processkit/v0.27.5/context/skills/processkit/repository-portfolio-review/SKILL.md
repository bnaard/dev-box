---
name: repository-portfolio-review
description: >
  Review one or more GitHub repositories for governance, documentation,
  licensing, privacy, security, and portfolio-boundary hygiene; deduplicate
  and create repository-local issues for verified remediation work. Use for
  portfolio reviews, repository standards audits, or baseline governance work.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v2
    id: SKILL-repository-portfolio-review
    version: "1.0.0"
    category: processkit
    layer: 4
---

# Repository Portfolio Review

## Intro

Review only repositories visible to the authenticated account. Treat the
result as engineering evidence, not a legal, privacy, or security guarantee.

## Workflow

1. Inventory each repository's visibility, lifecycle, default branch,
   description, homepage, license metadata, and accessible branches.
2. Check README, contributing and security guidance, release/rollback steps,
   local identity policy, hosted-automation policy, dependency provenance, and
   sensitive-data handling.
3. Search open and recently closed local issues before recording a finding.
4. Create repository-local issues only for verified, actionable gaps. Keep
   security-sensitive details out of public issues and use the private
   reporting route instead.
5. Separate completed remediation from findings, external decisions, and
   access limitations in the final report.

## Overview

### Required issue content

Every created issue states the evidence, concrete proposed change, observable
acceptance criteria, legal/IP/privacy/security notes, and ownership or
dependencies. Do not assign a lifecycle state, change Actions settings, or
rewrite history without explicit owner authorization.

### Validation

Confirm the intended branch and repository before changing settings. For each
finding, preserve a direct evidence link or command output. Re-run the local
build/test commands after repository-content changes and report any checks
that were unavailable.

## Gotchas

- Do not represent visible repositories as a complete organization inventory.
- Never place alleged secrets, exploit details, or personal data in a public issue.
- Do not treat a new issue as completed remediation.

## Full reference

The review distinguishes verified remediation, repository-local follow-up,
and owner decisions across governance, documentation, provenance, privacy,
security, and portfolio boundaries.
