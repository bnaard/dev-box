#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EVIDENCE_DIR="${PROJECT_ROOT}/.aibox/release-evidence/v1-readiness"
FEEDBACK_ROOT="${1:-}"

: "${RELEASE_CANDIDATE_SHA:?set RELEASE_CANDIDATE_SHA to the exact candidate commit}"
: "${AIBOX_RELEASE_BINARY_SHA256:?set AIBOX_RELEASE_BINARY_SHA256 to the tested binary digest}"
[[ "${RELEASE_CANDIDATE_SHA}" =~ ^[0-9a-f]{40}$ ]] || exit 2
[[ "${AIBOX_RELEASE_BINARY_SHA256}" =~ ^sha256:[0-9a-f]{64}$ ]] || exit 2
[[ -n "${FEEDBACK_ROOT}" && "${FEEDBACK_ROOT}" != /* && "${FEEDBACK_ROOT}" != *..* ]] || {
  echo "usage: scripts/record-v1-external-pilot-feedback.sh <project-relative-directory>" >&2
  exit 2
}
[[ -d "${PROJECT_ROOT}/${FEEDBACK_ROOT}" && ! -L "${PROJECT_ROOT}/${FEEDBACK_ROOT}" ]] || {
  echo "feedback directory must be a real directory inside the project" >&2
  exit 2
}
command -v jq >/dev/null || exit 2

journeys=(
  aibox-self-host
  migrated-v0
  new-compose-without-processkit
  existing-kubernetes
  direct-processkit
)
artifacts_json='[]'

for journey in "${journeys[@]}"; do
  relative_path="${FEEDBACK_ROOT}/${journey}.json"
  path="${PROJECT_ROOT}/${relative_path}"
  [[ -f "${path}" && ! -L "${path}" ]] || {
    echo "missing external pilot feedback for ${journey}" >&2
    exit 1
  }
  jq -e \
    --arg journey "${journey}" \
    --arg candidate "${RELEASE_CANDIDATE_SHA}" \
    '
      .apiVersion == "aibox.projectious.work/pilot-feedback/v1alpha1"
      and .kind == "ExternalPilotFeedback"
      and .journey == $journey
      and .candidateCommit == $candidate
      and .status == "completed"
      and (.operatorFeedback | type == "string" and length > 0)
      and (.configurationFriction | type == "array")
      and (.recoverySteps | type == "array")
      and (.migrationDecisions | type == "array")
      and (.runtimeErrors | type == "array")
      and (.documentationGaps | type == "array")
      and (.terminologyConfusion | type == "array")
    ' "${path}" >/dev/null || {
      echo "invalid or candidate-mismatched external pilot feedback for ${journey}" >&2
      exit 1
    }
  artifact_digest="sha256:$(sha256sum "${path}" | awk '{print $1}')"
  artifacts_json="$(
    jq \
      --arg path "${relative_path}" \
      --arg digest "${artifact_digest}" \
      '. + [{path: $path, sha256: $digest}]' <<< "${artifacts_json}"
  )"
done

mkdir -p "${EVIDENCE_DIR}"
evidence_tmp="${EVIDENCE_DIR}/.external-pilot-feedback.json.tmp"
recorded_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
scenarios_json="$(printf '%s\n' "${journeys[@]}" | jq -Rsc 'split("\n")[:-1]')"

jq -n \
  --arg commit "${RELEASE_CANDIDATE_SHA}" \
  --arg digest "${AIBOX_RELEASE_BINARY_SHA256}" \
  --arg recorded "${recorded_at}" \
  --argjson scenarios "${scenarios_json}" \
  --argjson artifacts "${artifacts_json}" \
  '{
    apiVersion: "aibox.projectious.work/release-evidence/v1alpha1",
    kind: "ReleaseGateEvidence",
    gate: "external-pilot-feedback",
    status: "passed",
    candidateCommit: $commit,
    binarySha256: $digest,
    command: "scripts/record-v1-external-pilot-feedback.sh",
    startedAt: $recorded,
    completedAt: $recorded,
    scenarios: $scenarios,
    artifacts: $artifacts
  }' > "${evidence_tmp}"
mv "${evidence_tmp}" "${EVIDENCE_DIR}/external-pilot-feedback.json"
echo "external pilot feedback evidence recorded for ${RELEASE_CANDIDATE_SHA}"
