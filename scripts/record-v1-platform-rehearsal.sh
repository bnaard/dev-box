#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EVIDENCE_DIR="${PROJECT_ROOT}/.aibox/release-evidence/v1-readiness"
REHEARSAL_ROOT="${1:-}"
VERSION="${2:-}"

: "${RELEASE_CANDIDATE_SHA:?set RELEASE_CANDIDATE_SHA to the exact candidate commit}"
: "${AIBOX_RELEASE_BINARY_SHA256:?set AIBOX_RELEASE_BINARY_SHA256 to the tested binary digest}"
[[ "${RELEASE_CANDIDATE_SHA}" =~ ^[0-9a-f]{40}$ ]] || exit 2
[[ "${AIBOX_RELEASE_BINARY_SHA256}" =~ ^sha256:[0-9a-f]{64}$ ]] || exit 2
[[ -n "${REHEARSAL_ROOT}" && -n "${VERSION}" ]] || {
  echo "usage: scripts/record-v1-platform-rehearsal.sh <project-relative-directory> <version>" >&2
  exit 2
}
[[ "${REHEARSAL_ROOT}" != /* && "${REHEARSAL_ROOT}" != *..* ]] || {
  echo "rehearsal directory must be a confined project-relative path" >&2
  exit 2
}
[[ -d "${PROJECT_ROOT}/${REHEARSAL_ROOT}" && ! -L "${PROJECT_ROOT}/${REHEARSAL_ROOT}" ]] || {
  echo "rehearsal directory must be a real directory inside the project" >&2
  exit 2
}
[[ "${VERSION}" =~ ^1\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]] || {
  echo "platform rehearsal requires a v1 version" >&2
  exit 2
}
command -v jq >/dev/null || exit 2

targets=(
  aarch64-unknown-linux-gnu
  x86_64-unknown-linux-gnu
  aarch64-apple-darwin
  x86_64-apple-darwin
)
logs=(
  container-release.log
  host-release.log
  rollback.log
)
artifacts=()

for target in "${targets[@]}"; do
  archive="${PROJECT_ROOT}/${REHEARSAL_ROOT}/aibox-v${VERSION}-${target}.tar.gz"
  checksum="${archive}.sha256"
  [[ -f "${archive}" && ! -L "${archive}" && -f "${checksum}" && ! -L "${checksum}" ]] || {
    echo "missing ${target} archive or checksum in ${REHEARSAL_ROOT}" >&2
    exit 1
  }
  expected="$(awk 'NR == 1 { print $1 }' "${checksum}")"
  actual="$(sha256sum "${archive}" | awk '{print $1}')"
  [[ "${expected}" =~ ^[0-9a-f]{64}$ && "${actual}" == "${expected}" ]] || {
    echo "checksum mismatch for ${target} archive" >&2
    exit 1
  }
  tar -tzf "${archive}" |
    grep -Fqx "aibox-v${VERSION}-${target}" || {
      echo "${target} archive does not contain its expected binary" >&2
      exit 1
    }
  artifacts+=(
    "${REHEARSAL_ROOT}/$(basename "${archive}")"
    "${REHEARSAL_ROOT}/$(basename "${checksum}")"
  )
done

for log in "${logs[@]}"; do
  path="${PROJECT_ROOT}/${REHEARSAL_ROOT}/${log}"
  [[ -s "${path}" && ! -L "${path}" ]] || {
    echo "missing non-empty rehearsal log ${REHEARSAL_ROOT}/${log}" >&2
    exit 1
  }
  artifacts+=("${REHEARSAL_ROOT}/${log}")
done
grep -Fqx "release phase=container status=passed candidate=${RELEASE_CANDIDATE_SHA}" \
  "${PROJECT_ROOT}/${REHEARSAL_ROOT}/container-release.log" || {
    echo "container release log lacks the exact-candidate completion marker" >&2
    exit 1
  }
grep -Fqx "release phase=host status=passed candidate=${RELEASE_CANDIDATE_SHA}" \
  "${PROJECT_ROOT}/${REHEARSAL_ROOT}/host-release.log" || {
    echo "host release log lacks the exact-candidate completion marker" >&2
    exit 1
  }
grep -Fqx "rollback status=passed candidate=${RELEASE_CANDIDATE_SHA} version=${VERSION}" \
  "${PROJECT_ROOT}/${REHEARSAL_ROOT}/rollback.log" || {
    echo "rollback log lacks the exact-candidate and exact-version completion marker" >&2
    exit 1
  }

mkdir -p "${EVIDENCE_DIR}"
evidence_tmp="${EVIDENCE_DIR}/.four-platform-release-rollback-rehearsal.json.tmp"
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
artifacts_json="$(
  printf '%s\n' "${artifacts[@]}" |
    jq -Rsc 'split("\n")[:-1] | map({path: ., sha256: ""})'
)"
for index in $(seq 0 $((${#artifacts[@]} - 1))); do
  artifact_digest="sha256:$(sha256sum "${PROJECT_ROOT}/${artifacts[${index}]}" | awk '{print $1}')"
  artifacts_json="$(jq --argjson index "${index}" --arg digest "${artifact_digest}" \
    '.[$index].sha256 = $digest' <<< "${artifacts_json}")"
done
completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

jq -n \
  --arg commit "${RELEASE_CANDIDATE_SHA}" \
  --arg digest "${AIBOX_RELEASE_BINARY_SHA256}" \
  --arg version "${VERSION}" \
  --arg started "${started_at}" \
  --arg completed "${completed_at}" \
  --argjson artifacts "${artifacts_json}" \
  '{
    apiVersion: "aibox.projectious.work/release-evidence/v1alpha1",
    kind: "ReleaseGateEvidence",
    gate: "four-platform-release-rollback-rehearsal",
    status: "passed",
    candidateCommit: $commit,
    binarySha256: $digest,
    command: ("scripts/record-v1-platform-rehearsal.sh " + $version),
    startedAt: $started,
    completedAt: $completed,
    scenarios: [
      "linux-aarch64-artifact",
      "linux-x86_64-artifact",
      "macos-aarch64-artifact",
      "macos-x86_64-artifact",
      "checksum-verification",
      "container-release-rehearsal",
      "host-release-rehearsal",
      "rollback-reinstall-rehearsal"
    ],
    artifacts: $artifacts
  }' > "${evidence_tmp}"
mv "${evidence_tmp}" "${EVIDENCE_DIR}/four-platform-release-rollback-rehearsal.json"
echo "four-platform release and rollback rehearsal evidence recorded for ${RELEASE_CANDIDATE_SHA}"
