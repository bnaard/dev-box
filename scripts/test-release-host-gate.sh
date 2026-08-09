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
publisher_spec = importlib.util.spec_from_file_location("publisher", script_dir / "release_host_publish.py")
publisher = importlib.util.module_from_spec(publisher_spec)
publisher_spec.loader.exec_module(publisher)

assert gate.EXPECTED_INPUTS == {"checksums.sha256", "provenance.json", "source.tar.gz"}
assert gate.RUN_ID.fullmatch("v0.31.2-20260809T120000Z-0123456789ab")
assert gate.RUN_ID.fullmatch("v1.0.0-alpha.2-20260809T120000Z-0123456789ab")
assert gate.VERSION_TAG.fullmatch("v1.0.0-alpha.2")
assert not gate.RUN_ID.fullmatch("../v0.31.2-20260809T120000Z-0123456789ab")
assert not gate.RUN_ID.fullmatch("v0.31.2/latest")

assert gate.select_impact_checks(["docs-site/content/docs/index.md"]) == {}
latex = gate.select_impact_checks(["addons/languages/latex.yaml"])
assert latex == {"latex-lifecycle": "addons/languages/latex.yaml"}
infrastructure = gate.select_impact_checks(["addons/tools/infrastructure.yaml"])
assert infrastructure == {
    "addon-platforms": "addons/tools/infrastructure.yaml",
    "rootless-podman": "addons/tools/infrastructure.yaml",
}
assert gate.select_impact_checks(["addons/tools/release.yaml"]) == {
    "addon-languages": "addons/tools/release.yaml"
}
all_checks = gate.select_impact_checks(["images/base-debian/Dockerfile"])
assert set(all_checks) == gate.ALL_IMPACT_CHECKS
assert set(gate.select_impact_checks(["*"])) == gate.ALL_IMPACT_CHECKS
assert set(publisher.IMPACT_EVIDENCE) == gate.ALL_IMPACT_CHECKS
assert "evidence/container-e2e/impact-selection.json" in publisher.BASE_REQUIRED_EVIDENCE
PY

echo "release host gate contract tests passed"
