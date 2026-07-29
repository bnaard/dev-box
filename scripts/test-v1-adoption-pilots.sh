#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI_MANIFEST="${PROJECT_ROOT}/cli/Cargo.toml"
EVIDENCE_DIR="${PROJECT_ROOT}/.aibox/release-evidence/v1-readiness"
LOG_DIR="${EVIDENCE_DIR}/logs"
M7C_EVIDENCE="${PROJECT_ROOT}/.aibox/release-evidence/m7c-live.json"
M5_EVIDENCE="${EVIDENCE_DIR}/m5-alpha3-exact-lifecycle.json"

: "${RELEASE_CANDIDATE_SHA:?set RELEASE_CANDIDATE_SHA to the exact candidate commit}"
: "${AIBOX_RELEASE_BINARY_SHA256:?set AIBOX_RELEASE_BINARY_SHA256 to the tested binary digest}"
[[ "${RELEASE_CANDIDATE_SHA}" =~ ^[0-9a-f]{40}$ ]] || exit 2
[[ "${AIBOX_RELEASE_BINARY_SHA256}" =~ ^sha256:[0-9a-f]{64}$ ]] || exit 2
command -v jq >/dev/null || {
  echo "jq is required for adoption-pilot evidence" >&2
  exit 2
}

for evidence in "${M7C_EVIDENCE}" "${M5_EVIDENCE}"; do
  [[ -f "${evidence}" ]] || {
    echo "missing prerequisite pilot evidence: ${evidence}" >&2
    exit 1
  }
  jq -e \
    --arg commit "${RELEASE_CANDIDATE_SHA}" \
    --arg digest "${AIBOX_RELEASE_BINARY_SHA256}" \
    '.candidateCommit == $commit and .binarySha256 == $digest and .status == "passed"' \
    "${evidence}" >/dev/null || {
      echo "pilot prerequisite is not bound to the exact candidate: ${evidence}" >&2
      exit 1
    }
done

mkdir -p "${LOG_DIR}"
log_tmp="${LOG_DIR}/.adoption-pilots.log.tmp"
log_path="${LOG_DIR}/adoption-pilots.log"
evidence_tmp="${EVIDENCE_DIR}/.adoption-pilots.json.tmp"
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

run_pilots() {
  echo "journey=new-compose"
  cargo test --manifest-path "${CLI_MANIFEST}" \
    deploy_plan_renders_compose_artifacts_without_writing_project_files -- --nocapture
  cargo test --manifest-path "${CLI_MANIFEST}" \
    compose_plan::tests:: -- --nocapture

  echo "journey=migrated-v0"
  cargo test --manifest-path "${CLI_MANIFEST}" \
    v1_config_migration -- --nocapture

  echo "journey=existing-kubernetes-target"
  jq -e '
    [.scenarios[].id] as $ids
    | ["first-apply","unchanged-apply","changed-apply","drift-recovery",
       "status-logs","exec-port-forward","ingress","foreign-destroy-refusal"]
      | all(. as $required | $ids | index($required))
  ' "${M7C_EVIDENCE}" >/dev/null

  echo "journey=direct-processkit"
  jq -e '
    .gate == "m5-alpha3-exact-lifecycle"
    and (.scenarios | index("signed-install"))
    and (.scenarios | index("verify"))
    and (.scenarios | index("no-op-update"))
    and (.scenarios | index("uninstall"))
  ' "${M5_EVIDENCE}" >/dev/null
}

if ! run_pilots > "${log_tmp}" 2>&1; then
  cat "${log_tmp}" >&2
  rm -f "${log_tmp}" "${EVIDENCE_DIR}/adoption-pilots.json"
  exit 1
fi
completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
cat "${log_tmp}"
mv "${log_tmp}" "${log_path}"
log_sha256="sha256:$(sha256sum "${log_path}" | awk '{print $1}')"

jq -n \
  --arg commit "${RELEASE_CANDIDATE_SHA}" \
  --arg digest "${AIBOX_RELEASE_BINARY_SHA256}" \
  --arg started "${started_at}" \
  --arg completed "${completed_at}" \
  --arg log_sha256 "${log_sha256}" \
  '{
    apiVersion: "aibox.projectious.work/release-evidence/v1alpha1",
    kind: "ReleaseGateEvidence",
    gate: "adoption-pilots",
    status: "passed",
    candidateCommit: $commit,
    binarySha256: $digest,
    command: "scripts/test-v1-adoption-pilots.sh",
    startedAt: $started,
    completedAt: $completed,
    scenarios: [
      "new-compose",
      "migrated-v0",
      "existing-kubernetes-target",
      "direct-processkit"
    ],
    artifacts: [{
      path: ".aibox/release-evidence/v1-readiness/logs/adoption-pilots.log",
      sha256: $log_sha256
    }]
  }' > "${evidence_tmp}"
mv "${evidence_tmp}" "${EVIDENCE_DIR}/adoption-pilots.json"
echo "adoption pilot evidence written to ${EVIDENCE_DIR}/adoption-pilots.json"
