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

if "${SCRIPT_DIR}/release-host-gate.sh" --dry-run --dry-run >/dev/null 2>&1; then
  echo "release-host gate accepted duplicate dry-run flags" >&2
  exit 1
fi

grep -Fq '"${OWNER_HOME}/.local/bin/uv"' "${SCRIPT_DIR}/release-host-gate.sh" || {
  echo "release-host gate does not accept uv's owner-local installer path" >&2
  exit 1
}
grep -Fq '"${OWNER_HOME}/.local/bin/uv"' "${SCRIPT_DIR}/release-host-publish.sh" || {
  echo "release-host publisher does not accept uv's owner-local installer path" >&2
  exit 1
}
if grep -Eq '(command -v|which)[[:space:]]+uv' "${SCRIPT_DIR}/release-host-gate.sh"; then
  echo "release-host gate must not resolve uv through inherited PATH" >&2
  exit 1
fi
grep -Fq '"${UV_BIN}" run --no-project --python 3.14.6 --' "${SCRIPT_DIR}/release-host-gate.sh" || {
  echo "release-host gate does not let uv manage its exact Python" >&2
  exit 1
}
grep -Fq 'python "${SCRIPT_DIR}/release_host_gate.py"' "${SCRIPT_DIR}/release-host-gate.sh" || {
  echo "release-host gate exposes its script to uv metadata handling" >&2
  exit 1
}

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
if grep -Fq '"status", "--porcelain", "--untracked-files=no"' "${SCRIPT_DIR}/release_host_gate.py"; then
  echo "release-host gate must not reject unrelated host worktree changes" >&2
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
assert set(gate.TRUSTED_CONTROL_PATHS) == {
    "scripts/maintain.sh",
    "scripts/release-host-gate.sh",
    "scripts/release-host-publish.sh",
    "scripts/release_host_gate.py",
    "scripts/release_host_publish.py",
}
source = (script_dir / "release_host_gate.py").read_text()
assert '"CARGO_HOME": str(cargo_home)' in source
assert '"cargo", "fetch", "--locked"' in source
assert 'fetch_env = {**fixed_env, "CARGO_NET_OFFLINE": "false"}' in source
assert '"DOCKER_BUILDKIT": "1"' in source
assert '["docker", "buildx", "version"]' in source
assert gate.RUN_ID.fullmatch("v0.31.2-20260809T120000Z-0123456789ab")
assert gate.RUN_ID.fullmatch("v1.0.0-alpha.2-20260809T120000Z-0123456789ab")
assert gate.VERSION_TAG.fullmatch("v1.0.0-alpha.2")
assert gate.dry_run_enabled(None) is False
assert gate.dry_run_enabled("0") is False
assert gate.dry_run_enabled("1") is True
assert any("brew install syft" in value for value in gate.main.__code__.co_consts if isinstance(value, str))
try:
    gate.dry_run_enabled("true")
except SystemExit:
    pass
else:
    raise AssertionError("ambiguous dry-run value was accepted")
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
