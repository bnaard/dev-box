#!/usr/bin/env bash
# Verify the E2E companion's local contract and, when available, that the
# supported v0 and v1 lines use byte-identical companion sources.
#
# The reference is optional on purpose: ordinary branch CI and source tarballs
# do not need to fetch another release line. Release-line integration invokes
# this with a locally available v0.x-release ref to catch source divergence.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
reference="${AIBOX_E2E_COMPANION_REFERENCE:-v0.x-release}"
candidate="HEAD"

usage() {
  echo "Usage: $0 [--reference <git-ref>] [--candidate <git-ref>] [--require-reference]" >&2
}

require_reference=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --reference)
      [[ $# -ge 2 ]] || { usage; exit 2; }
      reference="$2"
      shift 2
      ;;
    --require-reference)
      require_reference=1
      shift
      ;;
    --candidate)
      [[ $# -ge 2 ]] || { usage; exit 2; }
      candidate="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      exit 2
      ;;
  esac
done

dockerfile="${project_root}/.devcontainer/Dockerfile.e2e"
compose="${project_root}/.devcontainer/docker-compose.override.yml"
marker='aibox-e2e-companion-contract=2'

grep -Fqx "    && printf '%s\\n' 'aibox-e2e-companion-contract=2' \\" "${dockerfile}"
grep -Fq "${marker}" "${dockerfile}"
grep -Fq '/usr/local/share/aibox/e2e-companion-contract' "${dockerfile}"
grep -Fq 'CMD ["/sbin/init"]' "${dockerfile}"
grep -Fq 'cgroup_manager = "systemd"' "${dockerfile}"
grep -Fq 'cgroup: private' "${compose}"
grep -Fq '/lib/modules:/lib/modules:ro' "${compose}"

if ! git -C "${project_root}" rev-parse --verify --quiet "${reference}^{commit}" >/dev/null; then
  if [[ "${require_reference}" -eq 1 ]]; then
    echo "E2E companion parity reference '${reference}' is unavailable locally." >&2
    exit 1
  fi
  echo "E2E companion local contract is valid; parity skipped because '${reference}' is unavailable locally." >&2
  exit 0
fi

if ! git -C "${project_root}" rev-parse --verify --quiet "${candidate}^{commit}" >/dev/null; then
  echo "E2E companion parity candidate '${candidate}' is unavailable locally." >&2
  exit 1
fi

for path in .devcontainer/Dockerfile.e2e .devcontainer/docker-compose.override.yml; do
  if ! cmp -s \
      <(git -C "${project_root}" show "${reference}:${path}") \
      <(git -C "${project_root}" show "${candidate}:${path}"); then
    echo "E2E companion source diverges: ${candidate} does not match ${reference}: ${path}" >&2
    exit 1
  fi
done

echo "E2E companion local contract and ${candidate}/${reference} parity are valid."
