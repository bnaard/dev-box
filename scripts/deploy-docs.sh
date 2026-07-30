#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SITE_BASE_URL="${SITE_BASE_URL:-https://projectious-work.github.io/aibox/}"
PAGES_BRANCH="${PAGES_BRANCH:-gh-pages}"
LINE=""
VERSION=""
DRY_RUN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --line) LINE="${2:?--line requires a value}"; shift 2 ;;
    --version) VERSION="${2:?--version requires a value}"; shift 2 ;;
    --dry-run) DRY_RUN=1; shift ;;
    *) echo "Unknown argument: $1" >&2; exit 2 ;;
  esac
done

[[ "${LINE}" =~ ^v[01]\.x$ ]] || {
  echo "--line must be v0.x or v1.x" >&2
  exit 2
}
[[ -z "${VERSION}" || "${VERSION}" =~ ^v[01]\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]] || {
  echo "--version must be a v-prefixed v0 or v1 semver" >&2
  exit 2
}

if [[ "${LINE}" == "v0.x" ]]; then
  CURRENT_URL="${SITE_BASE_URL}"
  CURRENT_PATH=""
else
  CURRENT_URL="${SITE_BASE_URL}${LINE}/"
  CURRENT_PATH="${LINE}"
fi

BUILD_DIR="$(mktemp -d)"
PAGES_DIR="$(mktemp -d)"
cleanup() {
  git -C "${ROOT_DIR}" worktree remove --force "${PAGES_DIR}" >/dev/null 2>&1 || true
  rm -rf -- "${BUILD_DIR}" "${PAGES_DIR}"
}
trap cleanup EXIT

DOCS_BASE_URL="${CURRENT_URL}" "${ROOT_DIR}/scripts/build-docs.sh" \
  --destination "${BUILD_DIR}/current"
if [[ -n "${VERSION}" ]]; then
  DOCS_BASE_URL="${SITE_BASE_URL}${LINE}/${VERSION}/" \
    "${ROOT_DIR}/scripts/build-docs.sh" --destination "${BUILD_DIR}/archive"
fi

if git -C "${ROOT_DIR}" show-ref --verify --quiet "refs/heads/${PAGES_BRANCH}"; then
  git -C "${ROOT_DIR}" worktree add "${PAGES_DIR}" "${PAGES_BRANCH}"
else
  git -C "${ROOT_DIR}" fetch origin "${PAGES_BRANCH}:${PAGES_BRANCH}"
  git -C "${ROOT_DIR}" worktree add "${PAGES_DIR}" "${PAGES_BRANCH}"
fi

TARGET="${PAGES_DIR}/${CURRENT_PATH}"
mkdir -p "${TARGET}"
while IFS= read -r -d '' entry; do
  name="$(basename "${entry}")"
  [[ "${name}" == ".git" ]] && continue
  [[ -z "${CURRENT_PATH}" && "${name}" =~ ^v[01]\.x$ ]] && continue
  [[ -n "${CURRENT_PATH}" && "${name}" =~ ^v[01]\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]] && continue
  rm -rf -- "${entry}"
done < <(find "${TARGET}" -mindepth 1 -maxdepth 1 -print0)
cp -R "${BUILD_DIR}/current/." "${TARGET}/"

if [[ -n "${VERSION}" ]]; then
  ARCHIVE="${PAGES_DIR}/${LINE}/${VERSION}"
  mkdir -p "${ARCHIVE}"
  find "${ARCHIVE}" -mindepth 1 -maxdepth 1 -exec rm -rf -- {} +
  cp -R "${BUILD_DIR}/archive/." "${ARCHIVE}/"
fi

mkdir -p "${PAGES_DIR}/v0.x" "${PAGES_DIR}/v1.x"

release_array() {
  local line="$1"
  find "${PAGES_DIR}/${line}" -mindepth 1 -maxdepth 1 -type d \
    -printf '%f\n' 2>/dev/null |
    sed -n -E '/^v[01]\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$/p' |
    sort -Vr |
    jq -R -s --arg base "${SITE_BASE_URL}${line}/" \
      'split("\n") | map(select(length > 0) | {version: ., url: ($base + . + "/")})'
}

v0_releases="$(release_array v0.x)"
v1_releases="$(release_array v1.x)"
v0_current="$(jq -r '(.lines[]? | select(.line == "v0.x") | .current.version) // empty' \
  "${PAGES_DIR}/releases.json" 2>/dev/null || true)"
v1_current="$(jq -r '(.lines[]? | select(.line == "v1.x") | .current.version) // empty' \
  "${PAGES_DIR}/releases.json" 2>/dev/null || true)"
[[ "${LINE}" == "v0.x" && -n "${VERSION}" ]] && v0_current="${VERSION}"
[[ "${LINE}" == "v1.x" && -n "${VERSION}" ]] && v1_current="${VERSION}"
[[ -n "${v0_current}" ]] || v0_current="v0.x"
[[ -n "${v1_current}" ]] || v1_current="preview"

jq -n \
  --arg siteBase "${SITE_BASE_URL}" \
  --arg v0Current "${v0_current}" \
  --arg v1Current "${v1_current}" \
  --argjson v0Releases "${v0_releases}" \
  --argjson v1Releases "${v1_releases}" \
  '{
    schemaVersion: 1,
    siteBase: $siteBase,
    lines: [
      {line: "v0.x", current: {version: $v0Current, url: $siteBase}, releases: $v0Releases},
      {line: "v1.x", current: {version: $v1Current, url: ($siteBase + "v1.x/")}, releases: $v1Releases}
    ]
  }' > "${PAGES_DIR}/releases.json"
: > "${PAGES_DIR}/.nojekyll"

git -C "${PAGES_DIR}" add -A
if git -C "${PAGES_DIR}" diff --cached --quiet; then
  echo "No documentation changes to deploy."
  exit 0
fi
if [[ "${DRY_RUN}" -eq 1 ]]; then
  git -C "${PAGES_DIR}" diff --cached --stat
  exit 0
fi

git -C "${PAGES_DIR}" commit -m "docs: deploy ${LINE}${VERSION:+ ${VERSION}}"
git -C "${PAGES_DIR}" push origin "${PAGES_BRANCH}"
