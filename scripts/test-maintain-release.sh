#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

AIBOX_MAINTAIN_SOURCE_ONLY=1 source "${SCRIPT_DIR}/maintain.sh"

[[ "$(release_branch_for_version 0.28.4)" == "v0.x-release" ]] \
  || die "v0 release versions must resolve to v0.x-release"
[[ "$(release_branch_for_version 1.0.0)" == "v1.x-release" ]] \
  || die "v1 release versions must resolve to v1.x-release"

test_root="$(mktemp -d)"
trap 'rm -rf "${test_root}"' EXIT

DIST_DIR="${test_root}/dist"
RELEASE_EVIDENCE_DIR="${DIST_DIR}/evidence"
RELEASE_LOG_DIR="${RELEASE_EVIDENCE_DIR}/logs"
RELEASE_CANDIDATE_SHA="test-candidate"
RELEASE_TOOLCHAIN_FINGERPRINT="test-toolchain"
RELEASE_TREE_STATE="clean"
RELEASE_PHASE="container"
AIBOX_RELEASE_REUSE_EVIDENCE=0
mkdir -p "${RELEASE_LOG_DIR}"

probe_output="${test_root}/probe-ran"
release_test_probe() {
  printf 'ran\n' > "${probe_output}"
}

release_run_evidenced_step \
  evidence-probe 9.9.9 "Evidence probe" release_test_probe

marker="$(release_evidence_key_path evidence-probe)"
[[ -f "${probe_output}" ]] || die "evidenced step did not invoke its command"
[[ -f "${marker}" ]] || die "evidenced step did not write its marker"
grep -Fqx 'commit=test-candidate' "${marker}" \
  || die "evidence marker is not candidate-bound"

rm -f "${probe_output}"
AIBOX_RELEASE_REUSE_EVIDENCE=1
release_run_evidenced_step \
  evidence-probe 9.9.9 "Evidence probe" release_test_probe
[[ ! -e "${probe_output}" ]] || die "valid evidence did not skip command execution"

RELEASE_TREE_STATE="dirty"
release_run_evidenced_step \
  evidence-probe 9.9.9 "Dirty-tree evidence probe" release_test_probe
[[ -f "${probe_output}" ]] || die "dirty tree incorrectly reused clean-tree evidence"

RELEASE_TREE_STATE="clean"
mkdir -p "${DIST_DIR}"
for target in aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu; do
  archive="${DIST_DIR}/aibox-v9.9.9-${target}.tar.gz"
  printf 'archive-%s\n' "${target}" > "${archive}"
  sha256_file "${archive}" > "${archive}.sha256"
done
release_record_evidence build-linux 9.9.9 1
release_evidence_valid build-linux 9.9.9 \
  || die "fresh Linux artifact evidence was rejected"
printf 'tampered\n' >> "${DIST_DIR}/aibox-v9.9.9-aarch64-unknown-linux-gnu.tar.gz"
if release_evidence_valid build-linux 9.9.9; then
  die "tampered Linux artifact incorrectly reused cached evidence"
fi

release_test_slow() {
  sleep 0.2
}
release_test_fast() {
  sleep 0.05
}
AIBOX_RELEASE_PARALLELISM=2
release_run_parallel_validation 9.9.9 \
  "scheduler-slow|Scheduler slow probe|release_test_slow" \
  "scheduler-fast|Scheduler fast probe|release_test_fast"
[[ -f "$(release_evidence_key_path scheduler-slow)" ]] \
  || die "parallel scheduler did not record the slow probe"
[[ -f "$(release_evidence_key_path scheduler-fast)" ]] \
  || die "parallel scheduler did not record the fast probe"

ok "maintain.sh release evidence probe passed"
