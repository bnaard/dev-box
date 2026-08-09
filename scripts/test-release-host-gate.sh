#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

bash -n \
  "${SCRIPT_DIR}/release-host-prepare.sh" \
  "${SCRIPT_DIR}/release-host-gate.sh" \
  "${SCRIPT_DIR}/release-host-publish.sh"
/usr/bin/python3 -m py_compile \
  "${SCRIPT_DIR}/release_host_gate.py" \
  "${SCRIPT_DIR}/release_host_publish.py"

for entrypoint in release-host-gate.sh release-host-publish.sh; do
  if grep -Eq '(^|[[:space:]])(sudo|su|doas|eval|source)([[:space:]]|$)|bash -c|sh -c' \
      "${SCRIPT_DIR}/${entrypoint}"; then
    echo "${entrypoint} contains a prohibited command-construction or elevation surface" >&2
    exit 1
  fi
done

if grep -Fq '["docker", "build"' "${SCRIPT_DIR}/release_host_publish.py" ||
   grep -Fq '["cargo"' "${SCRIPT_DIR}/release_host_publish.py" ||
   grep -Fq 'subprocess.run(["git"' "${SCRIPT_DIR}/release_host_publish.py"; then
  echo "publisher may not build, test, or mutate repository state" >&2
  exit 1
fi

/usr/bin/python3 -I - "${SCRIPT_DIR}" <<'PY'
import importlib.util
from pathlib import Path
import sys

script_dir = Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("gate", script_dir / "release_host_gate.py")
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)

assert gate.EXPECTED_INPUTS == {"checksums.sha256", "provenance.json", "source.tar.gz"}
assert gate.RUN_ID.fullmatch("v0.31.2-20260809T120000Z-0123456789ab")
assert gate.RUN_ID.fullmatch("v1.0.0-alpha.2-20260809T120000Z-0123456789ab")
assert not gate.RUN_ID.fullmatch("../v0.31.2-20260809T120000Z-0123456789ab")
assert not gate.RUN_ID.fullmatch("v0.31.2/latest")
PY

echo "release host gate contract tests passed"
