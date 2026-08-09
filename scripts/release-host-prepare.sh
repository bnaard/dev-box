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
  --arg repository "projectious-work/aibox" \
  '{schema_version:1,version:$version,tag:$tag,commit:$commit,repository:$repository,source_archive:"source.tar.gz"}' \
  > "${INPUT_DIR}/provenance.json"
(
  cd "${INPUT_DIR}"
  shasum -a 256 provenance.json source.tar.gz > checksums.sha256
)
chmod 0444 "${INPUT_DIR}"/*
chmod 0555 "${INPUT_DIR}"
printf '%s\n' "${RUN_DIR}"
