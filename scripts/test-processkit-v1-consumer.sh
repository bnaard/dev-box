#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROCESSKIT_ROOT="${1:-}"

if [[ -z "${PROCESSKIT_ROOT}" || ! -f "${PROCESSKIT_ROOT}/installer/Cargo.toml" ]]; then
  echo "usage: $0 /path/to/processkit-checkout" >&2
  exit 2
fi

cargo build --manifest-path "${PROCESSKIT_ROOT}/installer/Cargo.toml" --bin processkit

AIBOX_PROCESSKIT_V1_TEST_CLI="${PROCESSKIT_ROOT}/installer/target/debug/processkit" \
AIBOX_PROCESSKIT_V1_TEST_DISTRIBUTION="${PROCESSKIT_ROOT}/src" \
  cargo test \
    --manifest-path "${PROJECT_ROOT}/cli/Cargo.toml" \
    processkit_protocol::tests::real_producer_lifecycle_when_configured \
    -- --exact --nocapture
