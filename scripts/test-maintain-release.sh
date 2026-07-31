#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

AIBOX_MAINTAIN_SOURCE_ONLY=1 source "${SCRIPT_DIR}/maintain.sh"

[[ "$(release_branch_for_version 0.28.5)" == "v0.x-release" ]] \
  || die "v0 release versions must resolve to v0.x-release"
[[ "$(release_branch_for_version 1.0.0-alpha.1)" == "v1.x-pre-release" ]] \
  || die "v1 prereleases must resolve to v1.x-pre-release"
[[ "$(release_branch_for_version 1.0.0)" == "v1.x-release" ]] \
  || die "v1 GA versions must resolve to v1.x-release"
declare -f cmd_release_host | grep -Fq 'X.Y.Z or X.Y.Z-prerelease' \
  || die "release-host must accept prerelease SemVer"
grep -Fq 'X.Y.Z or X.Y.Z-prerelease' "${SCRIPT_DIR}/build-macos.sh" \
  || die "macOS artifact builds must accept prerelease SemVer"
grep -Fq 'X.Y.Z or X.Y.Z-prerelease' "${SCRIPT_DIR}/release-runtime-smoke.sh" \
  || die "release runtime smoke must accept prerelease SemVer"
declare -F publish_release_candidate >/dev/null \
  || die "release candidate protected-branch publisher is missing"
declare -F release_docs_gate >/dev/null \
  || die "release documentation gate is missing"
declare -F release_v1_stable_evidence_gate >/dev/null \
  || die "stable-v1 producer evidence gate is missing"
[[ -x "${SCRIPT_DIR}/test-v1-adoption-pilots.sh" ]] \
  || die "v1 adoption-pilot evidence harness is missing or not executable"
[[ -x "${SCRIPT_DIR}/test-v1-operational-readiness.sh" ]] \
  || die "v1 operational-readiness evidence harness is missing or not executable"
[[ -x "${SCRIPT_DIR}/record-v1-platform-rehearsal.sh" ]] \
  || die "v1 platform-rehearsal recorder is missing or not executable"
[[ -x "${SCRIPT_DIR}/record-v1-external-pilot-feedback.sh" ]] \
  || die "v1 external-pilot feedback recorder is missing or not executable"
grep -Fq 'new-compose-without-processkit' \
  "${SCRIPT_DIR}/record-v1-external-pilot-feedback.sh" \
  || die "external-pilot evidence does not require the processkit-disabled journey"
grep -Fq '.operatorFeedback' \
  "${SCRIPT_DIR}/record-v1-external-pilot-feedback.sh" \
  || die "external-pilot evidence does not require operator feedback"
grep -Fq 'tar -tzf "${archive}"' "${SCRIPT_DIR}/record-v1-platform-rehearsal.sh" \
  || die "platform rehearsal does not inspect archive contents"
grep -Fq 'release phase=host status=passed candidate=' \
  "${SCRIPT_DIR}/record-v1-platform-rehearsal.sh" \
  || die "platform rehearsal does not require exact-candidate host evidence"
grep -Fq 'rollback status=passed candidate=' \
  "${SCRIPT_DIR}/record-v1-platform-rehearsal.sh" \
  || die "platform rehearsal does not require exact-candidate rollback evidence"
declare -f release_v1_stable_evidence_gate |
  grep -Fq 'test-v1-operational-readiness.sh' \
  || die "stable-v1 release does not run the operational evidence producer"
if declare -f release_v1_alpha_evidence_gate | grep -Fq 'config release-readiness'; then
  die "v1 alpha publication must not require stable-v1 readiness"
fi
release_parse_steps all
release_step_requested docs-check \
  || die "the full release process must include docs-check"

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

printf 'primary dirty\n' >> "${host_primary}/tracked.txt"
printf 'primary untracked\n' > "${host_primary}/primary-untracked.txt"
printf 'linked dirty\n' >> "${host_linked}/tracked.txt"
printf 'linked untracked\n' > "${host_linked}/linked-untracked.txt"

saved_project_root="${PROJECT_ROOT}"
PROJECT_ROOT="${host_primary}"
prepare_release_host_checkout 0.28.13
[[ "$(git -C "${host_primary}" branch --show-current)" == "v0.x-release" ]] \
  || die "release-host preparation did not switch the primary checkout"
[[ "$(git -C "${host_primary}" rev-parse HEAD)" == \
   "$(git -C "${host_primary}" rev-parse origin/v0.x-release)" ]] \
  || die "release-host preparation did not synchronize the release branch"
[[ -z "$(git -C "${host_primary}" status --porcelain --untracked-files=all)" ]] \
  || die "release-host preparation left the primary checkout dirty"
[[ -z "$(git -C "${host_linked}" branch --show-current)" ]] \
  || die "release-host preparation did not detach the linked release worktree"
[[ -z "$(git -C "${host_linked}" status --porcelain --untracked-files=all)" ]] \
  || die "release-host preparation left the linked release worktree dirty"
[[ "$(git -C "${host_primary}" stash list --format='%s' | \
    grep -c 'pre-release-host-v0.28.13-')" -eq 2 ]] \
  || die "release-host preparation did not preserve both dirty checkouts"
PROJECT_ROOT="${saved_project_root}"

ok "maintain.sh release evidence probe passed"
