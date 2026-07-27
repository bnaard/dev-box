#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCS_ROOT="${PROJECT_ROOT}/docs-site"
DOCS_BASE_URL="${DOCS_BASE_URL:-https://projectious-work.github.io/aibox/v1.x/}"
BUILD_DIR="${DOCS_ROOT}/public"

BUILD_ARGS=("$@")
for ((i = 0; i < ${#BUILD_ARGS[@]}; i++)); do
  case "${BUILD_ARGS[i]}" in
    --destination)
      ((i + 1 < ${#BUILD_ARGS[@]})) || {
        echo "--destination requires a value." >&2
        exit 1
      }
      BUILD_DIR="${BUILD_ARGS[i + 1]}"
      ;;
    --destination=*)
      BUILD_DIR="${BUILD_ARGS[i]#--destination=}"
      ;;
  esac
done

if [[ "${BUILD_DIR}" != /* ]]; then
  BUILD_DIR="${DOCS_ROOT}/${BUILD_DIR}"
fi

command -v hugo >/dev/null 2>&1 || {
  echo "Hugo extended is required: https://gohugo.io/installation/" >&2
  exit 1
}
command -v npm >/dev/null 2>&1 || {
  echo "Node.js and npm are required for Docsy assets." >&2
  exit 1
}

if [[ ! -f "${DOCS_ROOT}/themes/docsy/theme.toml" ]]; then
  git -C "${PROJECT_ROOT}" submodule update --init --recursive docs-site/themes/docsy
fi

if [[ ! -d "${DOCS_ROOT}/node_modules" ]]; then
  npm --prefix "${DOCS_ROOT}" ci
fi

hugo --source "${DOCS_ROOT}" --gc --minify --cleanDestinationDir \
  --baseURL "${DOCS_BASE_URL}" "${BUILD_ARGS[@]}"

: > "${BUILD_DIR}/.nojekyll"
