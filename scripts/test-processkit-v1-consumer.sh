#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROCESSKIT_VERSION="v1.0.0-alpha.3"
PROCESSKIT_RELEASE_BASE="https://github.com/projectious-work/processkit/releases/download/${PROCESSKIT_VERSION}"
PROCESSKIT_ARCHIVE_SHA256="cfeb5d028c961437aa394d15689490eb95a6d69e33a0bda567a0d5e4f5c09184"
PROCESSKIT_INSTALLER_SHA256="1aa51614830dd4b7e844f1a7ab7c1b1c76aaf480b5c5a3ffbfe83b20fdba3a26"
PROCESSKIT_SIGNING_KEY_ID="dbf471226f3124c2171510e8b931ab3e5c27d1943f22d562333b0a1468e8f188"
PROCESSKIT_TARGET="aarch64-unknown-linux-gnu"

run_gate() {
  cargo test \
    --manifest-path "${PROJECT_ROOT}/cli/Cargo.toml" \
    processkit_protocol::tests::real_producer_ \
    -- --nocapture
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
archive_checksum="${archive}.sha256"
installer="processkit-${PROCESSKIT_VERSION}-${PROCESSKIT_TARGET}"
installer_checksum="${installer}.sha256"
envelope="processkit-${PROCESSKIT_VERSION}.release.json"
signature="processkit-${PROCESSKIT_VERSION}.release.sig"
public_key="processkit-${PROCESSKIT_VERSION}.release.pub.pem"

for asset in "${archive}" "${archive_checksum}" "${installer}" "${installer_checksum}" "${envelope}" "${signature}" "${public_key}"; do
  curl --fail --silent --show-error --location \
    --output "${RELEASE_DIR}/${asset}" \
    "${PROCESSKIT_RELEASE_BASE}/${asset}"
done

printf '%s  %s\n' "${PROCESSKIT_ARCHIVE_SHA256}" "${RELEASE_DIR}/${archive}" \
  | sha256sum --check
printf '%s  %s\n' "${PROCESSKIT_INSTALLER_SHA256}" "${RELEASE_DIR}/${installer}" \
  | sha256sum --check
sed "s|  |  ${RELEASE_DIR}/|" "${RELEASE_DIR}/${archive_checksum}" | sha256sum --check
sed "s|  |  ${RELEASE_DIR}/|" "${RELEASE_DIR}/${installer_checksum}" | sha256sum --check
grep --fixed-strings --quiet "\"version\": \"${PROCESSKIT_VERSION}\"" "${RELEASE_DIR}/${envelope}"
grep --fixed-strings --quiet "\"keyId\": \"${PROCESSKIT_SIGNING_KEY_ID}\"" "${RELEASE_DIR}/${envelope}"
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
