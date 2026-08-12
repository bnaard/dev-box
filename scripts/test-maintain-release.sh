#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

AIBOX_MAINTAIN_SOURCE_ONLY=1 source "${SCRIPT_DIR}/maintain.sh"

[[ "$(release_branch_for_version 0.28.4)" == "v0.x-release" ]] \
  || die "v0 release versions must resolve to v0.x-release"
[[ "$(release_branch_for_version 1.0.0-alpha.1)" == "v1.x-pre-release" ]] \
  || die "v1 prereleases must resolve to v1.x-pre-release"
[[ "$(release_branch_for_version 1.0.0)" == "v1.x-release" ]] \
  || die "v1 release versions must resolve to v1.x-release"
declare -f cmd_release_host | grep -Fq 'release-host [--dry-run] <run-dir> [--dry-run]' \
  || die "release-host must accept a prepared run directory and optional dry-run mode"
grep -Fq 'X.Y.Z or X.Y.Z-prerelease' "${SCRIPT_DIR}/build-macos.sh" \
  || die "macOS artifact builds must accept prerelease SemVer"
grep -Fq 'X.Y.Z or X.Y.Z-prerelease' "${SCRIPT_DIR}/release-runtime-smoke.sh" \
  || die "release runtime smoke must accept prerelease SemVer"
declare -F publish_release_candidate >/dev/null \
  || die "release candidate protected-branch publisher is missing"

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
RELEASE_TIMING_LOG="${RELEASE_EVIDENCE_DIR}/timing-events.tsv"
release_timing_begin "test-evidence"

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

RELEASE_CANDIDATE_SHA="different-candidate"
release_run_evidenced_step \
  evidence-probe 9.9.9 "Changed-candidate evidence probe" release_test_probe
[[ -f "${probe_output}" ]] \
  || die "evidence from a different candidate SHA was incorrectly reused"
RELEASE_CANDIDATE_SHA="test-candidate"
rm -f "${probe_output}"

release_test_fails() {
  return 17
}
if release_run_evidenced_step timing-failure 9.9.9 "Timing failure probe" release_test_fails; then
  die "failing evidenced step unexpectedly succeeded"
fi
grep -F $'step\t' "${RELEASE_TIMING_LOG}" >/dev/null \
  || die "timing event log did not record step attempts"
grep -F $'\tfailed\ttiming-failure\t' "${RELEASE_TIMING_LOG}" >/dev/null \
  || die "timing event log did not retain failed step attempt"
grep -F $'\treused\tevidence-probe\t' "${RELEASE_TIMING_LOG}" >/dev/null \
  || die "timing event log did not retain evidence reuse"

release_timing_finish 1
release_write_timing_report 9.9.9 1
timing_report="${DIST_DIR}/RELEASE-TIMINGS.md"
[[ -f "${timing_report}" ]] || die "release timing report was not written"
grep -Fq 'Cumulative attempts' "${timing_report}" \
  || die "release timing report omitted cumulative attempts"
grep -Fq 'cumulative command duration' "${timing_report}" \
  || die "release timing report omitted cumulative command duration"
grep -Fq 'timing-failure' "${timing_report}" \
  || die "release timing report omitted failed attempt"

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
  sleep 1.2
}
release_test_fast() {
  sleep 0.05
}
AIBOX_RELEASE_PARALLELISM=2
AIBOX_RELEASE_PROGRESS_INTERVAL_SECONDS=1
release_run_parallel_validation 9.9.9 \
  "scheduler-slow|Scheduler slow probe|release_test_slow" \
  "scheduler-fast|Scheduler fast probe|release_test_fast"
[[ -f "$(release_evidence_key_path scheduler-slow)" ]] \
  || die "parallel scheduler did not record the slow probe"
[[ -f "$(release_evidence_key_path scheduler-fast)" ]] \
  || die "parallel scheduler did not record the fast probe"
grep -F $'progress\t' "${RELEASE_TIMING_LOG}" | grep -F $'\trunning\tscheduler-slow\t' >/dev/null \
  || die "parallel scheduler did not emit a progress timing event"
unset AIBOX_RELEASE_PROGRESS_INTERVAL_SECONDS

host_remote="${test_root}/host-remote.git"
host_seed="${test_root}/host-seed"
host_primary="${test_root}/host-primary"
host_linked="${test_root}/host-linked"
git init --bare "${host_remote}" >/dev/null
git init -b main "${host_seed}" >/dev/null
git -C "${host_seed}" config user.name "Release Test"
git -C "${host_seed}" config user.email "release-test@example.invalid"
printf 'base\n' > "${host_seed}/tracked.txt"
git -C "${host_seed}" add tracked.txt
git -C "${host_seed}" commit -m "test: seed release repo" >/dev/null
git -C "${host_seed}" branch v0.x-release
git -C "${host_seed}" tag v0.28.13
git -C "${host_seed}" remote add origin "${host_remote}"
git -C "${host_seed}" push origin main v0.x-release v0.28.13 >/dev/null
git -C "${host_remote}" symbolic-ref HEAD refs/heads/main
git clone "${host_remote}" "${host_primary}" >/dev/null
git -C "${host_primary}" config user.name "Release Test"
git -C "${host_primary}" config user.email "release-test@example.invalid"
git -C "${host_primary}" worktree add "${host_linked}" v0.x-release >/dev/null

saved_project_root="${PROJECT_ROOT}"
PROJECT_ROOT="${host_primary}"
git -C "${host_linked}" switch --detach >/dev/null
git -C "${host_primary}" switch v0.x-release >/dev/null
ensure_release_host_checkout_current 0.28.13
[[ "$(git -C "${host_primary}" branch --show-current)" == "v0.x-release" ]] \
  || die "release-host validation changed the primary checkout"
[[ "$(git -C "${host_primary}" rev-parse HEAD)" == \
   "$(git -C "${host_primary}" rev-parse origin/v0.x-release)" ]] \
  || die "release-host validation accepted a stale release branch"
PROJECT_ROOT="${saved_project_root}"

ok "maintain.sh release evidence probe passed"
"${SCRIPT_DIR}/test-release-host-gate.sh"
