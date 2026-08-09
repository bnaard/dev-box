#!/usr/bin/env bash
# Prepare immutable inputs for the owner-reviewed macOS release host gate.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
VERSION="${1:-}"

if [[ ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
  echo "Usage: ./scripts/release-host-prepare.sh <version>" >&2
  exit 2
fi

COMMIT="$(git -C "${PROJECT_ROOT}" rev-parse "v${VERSION}^{commit}")"
HEAD_COMMIT="$(git -C "${PROJECT_ROOT}" rev-parse HEAD)"
[[ "${COMMIT}" == "${HEAD_COMMIT}" ]] || {
  echo "Release tag v${VERSION} must point at HEAD before preparing host inputs." >&2
  exit 1
}

VERSION_LINE="${VERSION%%.*}"
COMPARISON_TAG="$(git -C "${PROJECT_ROOT}" describe --tags --abbrev=0 \
  --match "v${VERSION_LINE}.*" "${COMMIT}^" 2>/dev/null || true)"
if [[ -n "${COMPARISON_TAG}" ]]; then
  COMPARISON_COMMIT="$(git -C "${PROJECT_ROOT}" rev-parse "${COMPARISON_TAG}^{commit}")"
  CHANGED_PATHS_JSON="$(
    git -C "${PROJECT_ROOT}" diff --name-only "${COMPARISON_COMMIT}" "${COMMIT}" |
      jq -R -s 'split("\n") | map(select(length > 0))'
  )"
else
  COMPARISON_COMMIT=""
  CHANGED_PATHS_JSON='["*"]'
fi

RUN_ID="v${VERSION}-$(date -u +%Y%m%dT%H%M%SZ)-${COMMIT:0:12}"
RUN_DIR="${PROJECT_ROOT}/tmp/host-gates/aibox-release/${RUN_ID}"
INPUT_DIR="${RUN_DIR}/input"
mkdir -p "${INPUT_DIR}"
chmod 0700 "${RUN_DIR}"

git -C "${PROJECT_ROOT}" archive --format=tar.gz --prefix=source/ \
  --output="${INPUT_DIR}/source.tar.gz" "${COMMIT}"
jq -n \
  --arg version "${VERSION}" \
  --arg tag "v${VERSION}" \
  --arg commit "${COMMIT}" \
  --arg comparison_tag "${COMPARISON_TAG}" \
  --arg comparison_commit "${COMPARISON_COMMIT}" \
  --argjson changed_paths "${CHANGED_PATHS_JSON}" \
  --arg repository "projectious-work/aibox" \
  '{schema_version:2,version:$version,tag:$tag,commit:$commit,
    comparison_tag:$comparison_tag,comparison_commit:$comparison_commit,
    changed_paths:$changed_paths,repository:$repository,source_archive:"source.tar.gz"}' \
  > "${INPUT_DIR}/provenance.json"
(
  cd "${INPUT_DIR}"
  shasum -a 256 provenance.json source.tar.gz > checksums.sha256
)
chmod 0444 "${INPUT_DIR}"/*
chmod 0555 "${INPUT_DIR}"
printf '%s\n' "${RUN_DIR}"
