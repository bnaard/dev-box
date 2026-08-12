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

grep -Fq 'AIBOX_RELEASE_SMOKE_LOCAL_CANDIDATE_IMAGE' "${SCRIPT_DIR}/release_host_gate.py" || {
  echo "release host gate must select the unpublished local candidate image for runtime smoke" >&2
  exit 1
}
grep -Fq 'DOCKER_BUILDKIT=0 COMPOSE_DOCKER_CLI_BUILD=0 "$@"' "${SCRIPT_DIR}/release-runtime-smoke.sh" || {
  echo "release runtime smoke must support Docker-compatible daemon-local candidate images" >&2
  exit 1
}
if grep -Fq 'env DOCKER_BUILDKIT=0 COMPOSE_DOCKER_CLI_BUILD=0' "${SCRIPT_DIR}/release-runtime-smoke.sh"; then
  echo "release runtime smoke must not dispatch shell functions through env" >&2
  exit 1
fi
grep -A3 '^\[processkit\]$' "${SCRIPT_DIR}/release-runtime-smoke.sh" | grep -Fq 'version = "unset"' || {
  echo "release runtime smoke must not install unrelated processkit content" >&2
  exit 1
}
grep -Fq 'local-candidate-substitution.env' "${SCRIPT_DIR}/release-runtime-smoke.sh" || {
  echo "release runtime smoke must retain evidence of its bounded local FROM substitution" >&2
  exit 1
}
grep -Fq '"--provenance=false"' "${SCRIPT_DIR}/release_host_gate.py" || {
  echo "release host gate must prepare a single-manifest local smoke image" >&2
  exit 1
}
grep -Fq 'stdin=subprocess.DEVNULL' "${SCRIPT_DIR}/release_host_gate.py" || {
  echo "release host gate subprocesses must not inherit an interactive terminal" >&2
  exit 1
}
grep -Fq 'def prepare_project_build' "${SCRIPT_DIR}/release_host_gate.py" || {
  echo "release host gate conditional projects must share local-candidate build handling" >&2
  exit 1
}
grep -Fq '"rootless_readiness": True' "${SCRIPT_DIR}/release_host_gate.py" || {
  echo "rootless Podman probe must retain explicit readiness evidence" >&2
  exit 1
}
grep -Fq '"/etc/containers/containers.conf"' "${SCRIPT_DIR}/release_host_gate.py" || {
  echo "rootless Podman readiness must verify configuration outside mounted user config" >&2
  exit 1
}
if grep -Fq 'Security.Rootless' "${SCRIPT_DIR}/release_host_gate.py"; then
  echo "restricted host gate must not require nested user-namespace execution" >&2
  exit 1
fi

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
import contextlib
import io
import os
from pathlib import Path
import sys
import tempfile

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
assert 'for candidate in ("docker", "podman")' in source
assert '[container_runtime, "build"' in source
assert '[container_runtime, "compose"' in source
assert '[container_runtime, "compose", "version"]' in source
assert '["docker", "buildx", "version"]' in source
assert gate.RUN_ID.fullmatch("v0.31.2-20260809T120000Z-0123456789ab")
assert gate.RUN_ID.fullmatch("v1.0.0-alpha.2-20260809T120000Z-0123456789ab")
assert gate.VERSION_TAG.fullmatch("v1.0.0-alpha.2")
assert gate.dry_run_enabled(None) is False
assert gate.dry_run_enabled("0") is False
assert gate.dry_run_enabled("1") is True
assert any("brew install syft" in value for value in gate.main.__code__.co_consts if isinstance(value, str))
with tempfile.TemporaryDirectory() as temporary:
    config = Path(temporary) / "aibox.toml"
    config.write_text(
        '[container.image]\nrelease_version = "latest" # Set to "latest" to resolve newest published image on apply.\n'
    )
    gate.pin_release_version(config, "0.31.2")
    assert config.read_text() == (
        '[container.image]\nrelease_version = "0.31.2" # Set to "latest" to resolve newest published image on apply.\n'
    )
with tempfile.TemporaryDirectory() as temporary:
    root = Path(temporary)
    plugin_dir = root / "owner/.docker/cli-plugins"
    plugin_dir.mkdir(parents=True)
    for name in ("docker-compose", "docker-buildx"):
        plugin = plugin_dir / name
        plugin.write_text(name)
        plugin.chmod(0o700)
    config = root / "config"
    config.mkdir()
    staged_sources = gate.stage_docker_cli_plugins(root / "owner", config)
    assert set(staged_sources) == {"docker-compose", "docker-buildx"}
    for name in staged_sources:
        staged = config / "cli-plugins" / name
        assert staged.read_text() == name
        assert staged.stat().st_mode & 0o777 == 0o500
with tempfile.TemporaryDirectory() as temporary:
    evidence = Path(temporary)
    runner = gate.Runner(evidence, os.environ.copy(), heartbeat_interval=0.02)
    with contextlib.redirect_stdout(io.StringIO()):
        runner.run([sys.executable, "-c", "import time; time.sleep(0.07)"], label="quiet test")
    progress = (evidence / "steps.log").read_text()
    assert progress.count("quiet test [running") >= 2
    assert "quiet test [passed" in progress
with tempfile.TemporaryDirectory() as temporary:
    report = Path(temporary) / "grype.json"
    report.write_text('{"matches":['
                      '{"vulnerability":{"id":"CVE-1","severity":"High","fix":{"versions":["2.0"]}},'
                      '"artifact":{"name":"sample","version":"1.0"}},'
                      '{"vulnerability":{"id":"CVE-1","severity":"High","fix":{"versions":[]}},'
                      '"artifact":{"name":"sample-lib","version":"1.0"}},'
                      '{"vulnerability":{"id":"CVE-3","severity":"Critical","fix":{"versions":[]}},'
                      '"artifact":{"name":"unfixed","version":"1.0"}},'
                      '{"vulnerability":{"id":"CVE-2","severity":"Medium"},'
                      '"artifact":{"name":"ignored","version":"1.0"}}]}')
    summary = gate.grype_policy_summary(report)
    assert summary["high_critical_package_matches"] == 3
    assert summary["unique_advisories"] == 2
    assert summary["actionable_package_matches"] == 1
    assert summary["actionable_advisories"] == 1
    assert summary["no_fix_advisories"] == 1
    assert [advisory["id"] for advisory in summary["advisories"]] == ["CVE-1", "CVE-3"]
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
