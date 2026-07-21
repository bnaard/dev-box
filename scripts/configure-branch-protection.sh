#!/usr/bin/env bash
# Configure long-lived aibox branches without GitHub Actions.
set -euo pipefail

repo="${AIBOX_GITHUB_REPO:-projectious-work/aibox}"
branches=(main v0.x-dev v0.x-release v1.x-dev v1.x-pre-release)

command -v gh >/dev/null 2>&1 || { echo "gh CLI is required" >&2; exit 1; }

# PR-only with resolved conversations, but no hosted status checks or mandatory
# second reviewer: releases are maintainer-run locally.
payload='{"required_status_checks":null,"enforce_admins":true,"required_pull_request_reviews":{"dismissal_restrictions":{"users":[],"teams":[],"apps":[]},"dismiss_stale_reviews":true,"require_code_owner_reviews":false,"required_approving_review_count":0,"require_last_push_approval":false},"restrictions":null,"required_linear_history":false,"allow_force_pushes":false,"allow_deletions":false,"block_creations":false,"required_conversation_resolution":true,"lock_branch":false,"allow_fork_syncing":false}'

for branch in "${branches[@]}"; do
  printf 'Protecting %s\n' "${branch}"
  gh api --method PUT "repos/${repo}/branches/${branch}/protection" --input - \
    <<<"${payload}" >/dev/null
done

echo "Protected ${#branches[@]} branches; no GitHub Actions or status checks configured."
