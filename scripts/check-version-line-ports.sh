#!/usr/bin/env bash
# Gate releases on traceable ports between the maintained v0.x and v1.x lines.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BASELINES="${PROJECT_ROOT}/.github/version-line-port-baselines.toml"

die() {
  printf 'ERR: %s\n' "$*" >&2
  exit 1
}

baseline_for() {
  local line="$1"
  sed -nE "s/^${line}[[:space:]]*=[[:space:]]*\"([0-9a-fA-F]+)\"[[:space:]]*$/\\1/p" \
    "${BASELINES}" | head -n 1
}

branch_for_line() {
  local branch
  case "$1" in
    v0) branch='v0.x-release' ;;
    v1) branch='v1.x-dev' ;;
    *) return 1 ;;
  esac
  if git -C "${PROJECT_ROOT}" show-ref --verify --quiet "refs/heads/${branch}"; then
    printf '%s\n' "${branch}"
  else
    printf 'origin/%s\n' "${branch}"
  fi
}

opposite_line() {
  case "$1" in
    v0) printf 'v1\n' ;;
    v1) printf 'v0\n' ;;
    *) return 1 ;;
  esac
}

target_ref_for() {
  local target="$1" current
  current="$(git -C "${PROJECT_ROOT}" branch --show-current)"
  case "${target}:${current}" in
    v0:v0.x-*|v0:codex/v0.*) printf 'HEAD\n' ;;
    v1:v1.x-*|v1:codex/v1.*) printf 'HEAD\n' ;;
    *) branch_for_line "${target}" ;;
  esac
}

has_trailer() {
  local message="$1" pattern="$2"
  grep -Eiq "^Version-Line-Port:[[:space:]]*${pattern}[[:space:]]*$" <<<"${message}"
}

check_target() {
  local target="$1" source source_ref source_baseline target_ref
  local sha message subject parents failures=0
  [[ "${target}" =~ ^v[01]$ ]] || die "target must be v0 or v1"
  [[ -f "${BASELINES}" ]] || die "missing ${BASELINES}"

  source="$(opposite_line "${target}")"
  source_ref="${AIBOX_PORT_SOURCE_REF:-$(branch_for_line "${source}")}"
  source_baseline="$(baseline_for "${source}")"
  target_ref="${AIBOX_PORT_TARGET_REF:-$(target_ref_for "${target}")}"

  [[ -n "${source_baseline}" ]] || die "missing ${source} baseline in ${BASELINES}"
  git -C "${PROJECT_ROOT}" cat-file -e "${source_baseline}^{commit}" 2>/dev/null \
    || die "baseline ${source_baseline} is not available locally"
  git -C "${PROJECT_ROOT}" cat-file -e "${source_ref}^{commit}" 2>/dev/null \
    || die "source ref ${source_ref} is not available; fetch maintained branches first"
  git -C "${PROJECT_ROOT}" cat-file -e "${target_ref}^{commit}" 2>/dev/null \
    || die "target ref ${target_ref} is not available; fetch maintained branches first"
  git -C "${PROJECT_ROOT}" merge-base --is-ancestor "${source_baseline}" "${source_ref}" \
    || die "${source_baseline} is not an ancestor of ${source_ref}"

  while IFS= read -r sha; do
    [[ -n "${sha}" ]] || continue
    parents="$(git -C "${PROJECT_ROOT}" rev-list --parents -n 1 "${sha}")"
    # Merge commits carry individual commits; they are not separate obligations.
    [[ "$(wc -w <<<"${parents}")" -le 2 ]] || continue
    message="$(git -C "${PROJECT_ROOT}" show -s --format=%B "${sha}")"
    subject="${message%%$'\n'*}"
    if has_trailer "${message}" 'not-applicable'; then
      continue
    fi
    # A port commit settles an obligation; it does not create the reverse one.
    if has_trailer "${message}" 'ported-from=[0-9a-fA-F]+'; then
      continue
    fi
    if [[ -n "$(git -C "${PROJECT_ROOT}" log -1 --format=%H \
        --extended-regexp --regexp-ignore-case \
        --grep="^Version-Line-Port:[[:space:]]*ported-from=${sha}[[:space:]]*$" \
        "${target_ref}")" ]]; then
      continue
    fi
    printf 'MISSING: %s %s\n' "${sha}" "${subject}" >&2
    failures=$((failures + 1))
  done < <(git -C "${PROJECT_ROOT}" rev-list --reverse "${source_baseline}..${source_ref}")

  if [[ "${failures}" -gt 0 ]]; then
    die "${failures} commit(s) from ${source}.x lack a traceable port or not-applicable trailer on ${target}.x"
  fi
  printf 'Version-line port gate is clear for %s.x against %s.\n' "${target}" "${source_ref}"
}

usage() {
  cat <<'EOF'
Usage:
  scripts/check-version-line-ports.sh check v0|v1

Commit trailers:
  Version-Line-Port: ported-from=<full-source-sha>
  Version-Line-Port: not-applicable

The gate checks every non-merge commit on the opposite maintained line after
the recorded baseline. A source commit must be explicitly not applicable,
itself be a port, or have a matching ported-from trailer in the target history.
EOF
}

case "${1:-}" in
  check)
    [[ "$#" -eq 2 ]] || { usage >&2; exit 2; }
    check_target "$2"
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
