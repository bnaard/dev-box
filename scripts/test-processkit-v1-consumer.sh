#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROCESSKIT_VERSION="v1.0.0-alpha.2"
PROCESSKIT_RELEASE_BASE="https://github.com/projectious-work/processkit/releases/download/${PROCESSKIT_VERSION}"
PROCESSKIT_ARCHIVE_SHA256="7fae02905fb07c56261a38aee1c0c07aea7c1c96152ae226ddde8898eeaf7b5c"
PROCESSKIT_INSTALLER_SHA256="7a67abedb63bc151700c4a72a8ec7e71b35d820da052c59ad75c7e3bdf095d1e"
PROCESSKIT_SIGNING_KEY_ID="74d3034333501f61162b8d392392b4533982b5f7d020a838d724fe0d429bf01f"
PROCESSKIT_TARGET="aarch64-unknown-linux-gnu"

run_gate() {
  cargo test \
    --manifest-path "${PROJECT_ROOT}/cli/Cargo.toml" \
    processkit_protocol::tests::real_producer_lifecycle_when_configured \
    -- --exact --nocapture
}

if [[ -n "${1:-}" ]]; then
  PROCESSKIT_ROOT="$1"
  if [[ ! -f "${PROCESSKIT_ROOT}/installer/Cargo.toml" ]]; then
    echo "usage: $0 [/path/to/processkit-checkout]" >&2
    exit 2
  fi
  cargo build --manifest-path "${PROCESSKIT_ROOT}/installer/Cargo.toml" --bin processkit
  AIBOX_PROCESSKIT_V1_TEST_CLI="${PROCESSKIT_ROOT}/installer/target/debug/processkit" \
  AIBOX_PROCESSKIT_V1_TEST_DISTRIBUTION="${PROCESSKIT_ROOT}/src" \
    run_gate
  exit
fi

command -v curl >/dev/null || {
  echo "curl is required for the tagged processkit consumer gate" >&2
  exit 2
}
command -v sha256sum >/dev/null || {
  echo "sha256sum is required for the tagged processkit consumer gate" >&2
  exit 2
}

RELEASE_DIR="$(mktemp -d)"
trap 'rm -rf "${RELEASE_DIR}"' EXIT

archive="processkit-${PROCESSKIT_VERSION}.tar.gz"
installer="processkit-${PROCESSKIT_VERSION}-${PROCESSKIT_TARGET}"
envelope="processkit-${PROCESSKIT_VERSION}.release.json"
signature="processkit-${PROCESSKIT_VERSION}.release.sig"
public_key="processkit-${PROCESSKIT_VERSION}.release.pub.pem"

for asset in "${archive}" "${installer}" "${envelope}" "${signature}" "${public_key}"; do
  curl --fail --silent --show-error --location \
    --output "${RELEASE_DIR}/${asset}" \
    "${PROCESSKIT_RELEASE_BASE}/${asset}"
done

printf '%s  %s\n' "${PROCESSKIT_ARCHIVE_SHA256}" "${RELEASE_DIR}/${archive}" \
  | sha256sum --check
printf '%s  %s\n' "${PROCESSKIT_INSTALLER_SHA256}" "${RELEASE_DIR}/${installer}" \
  | sha256sum --check
chmod 0755 "${RELEASE_DIR}/${installer}"

cat >"${RELEASE_DIR}/trust-store.json" <<EOF
{
  "apiVersion": "processkit.projectious.work/local-trust/v1alpha1",
  "kind": "TrustStore",
  "keys": [
    {
      "keyId": "${PROCESSKIT_SIGNING_KEY_ID}",
      "algorithm": "Ed25519",
      "publicKeyFile": "${public_key}",
      "status": "active"
    }
  ]
}
EOF

AIBOX_PROCESSKIT_V1_TEST_CLI="${RELEASE_DIR}/${installer}" \
AIBOX_PROCESSKIT_V1_TEST_ENVELOPE="${RELEASE_DIR}/${envelope}" \
AIBOX_PROCESSKIT_V1_TEST_SIGNATURE="${RELEASE_DIR}/${signature}" \
AIBOX_PROCESSKIT_V1_TEST_TRUST_STORE="${RELEASE_DIR}/trust-store.json" \
  run_gate
