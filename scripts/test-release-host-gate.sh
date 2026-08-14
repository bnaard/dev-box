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
if "${SCRIPT_DIR}/release-host-gate.sh" --reuse-cache --reuse-cache >/dev/null 2>&1; then
  echo "release-host gate accepted duplicate cache-reuse flags" >&2
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
if ! grep -Fq '"${UV_BIN}" run --no-project --python 3.14.6 \' "${SCRIPT_DIR}/release-host-gate.sh" ||
   ! grep -Fq -- '--with-requirements "${SCRIPT_DIR}/release-host-ui.lock"' "${SCRIPT_DIR}/release-host-gate.sh"; then
  echo "release-host gate does not let uv manage its exact Python" >&2
  exit 1
fi
grep -Fq 'python "${SCRIPT_DIR}/release_host_gate.py"' "${SCRIPT_DIR}/release-host-gate.sh" || {
  echo "release-host gate exposes its script to uv metadata handling" >&2
  exit 1
}
grep -Fq 'textual==8.2.8' "${SCRIPT_DIR}/release-host-ui.lock" || {
  echo "release-host Textual dependency is not exactly pinned" >&2
  exit 1
}
grep -Fq -- '--hash=sha256:' "${SCRIPT_DIR}/release-host-ui.lock" || {
  echo "release-host UI dependency graph is not hash locked" >&2
  exit 1
}
if grep -Fqi textual "${SCRIPT_DIR}/release_host_publish.py"; then
  echo "release-host publisher must remain Textual-free" >&2
  exit 1
fi

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
    "scripts/release-host-ui.in",
    "scripts/release-host-ui.lock",
}
source = (script_dir / "release_host_gate.py").read_text()
assert '"CARGO_HOME": str(cargo_home)' in source
assert '"CARGO_TARGET_DIR": str(cargo_target_dir)' in source
assert '"cargo_cache_scope": provenance["commit"]' in source
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
assert gate.cache_reuse_enabled(None) is False
assert gate.cache_reuse_enabled("0") is False
assert gate.cache_reuse_enabled("1") is True
with tempfile.TemporaryDirectory() as temporary:
    root = Path(temporary)
    old_evidence = root / "old" / "container-e2e"
    new_evidence = root / "new"
    old_evidence.mkdir(parents=True)
    (new_evidence / "container-e2e").mkdir(parents=True)
    marker = old_evidence / "addon-tools.json"
    marker.write_text(__import__("json").dumps({
        "status": "passed", "addons": gate.ADDON_GROUPS["addon-tools"],
        "browser_fixture": {
            "title": "Fixture", "violations": 0, "violation_details": [],
        },
    }) + "\n")
    gate.seal_checkpoint(marker)
    reused = gate.reuse_checkpoint(root / "old", new_evidence, "addon-tools")
    assert reused and reused.read_text() == marker.read_text()
    marker.write_text('{"status":"failed"}\n')
    assert gate.reuse_checkpoint(root / "old", new_evidence, "addon-tools") is None
assert "browser-testing" in gate.ADDON_GROUPS["addon-tools"]
assert "@axe-core/playwright" in source
assert 'chromium.launch({ headless: true, channel: "chromium" })' in source
assert 'const context = await browser.newContext()' in source
assert 'const page = await context.newPage()' in source
assert 'await context.close(); await browser.close()' in source
assert 'browser.newPage()' not in source
assert '<main><h1>Fixture</h1>' in source
assert 'violation_details = results.violations.map' in source
assert 'failureSummary: node.failureSummary' in source
assert '"browser_fixture"' in source
assert gate.parse_ui_mode(None) == "auto"
assert gate.parse_ui_mode("textual") == "textual"
assert gate.sanitize_display("[bold]literal[/bold]\x1b[31m red\x1b[0m\x1b]0;title\x07") == "[bold]literal[/bold] red"
assert gate.LAST_LOG_LINES == 20
assert gate.classify_log_line("error: candidate failed [literal]") == "error"
assert gate.classify_log_line("warning: no fix listed") == "warning"
assert gate.classify_log_line("0 errors found") is None
assert gate.classify_log_line("Build runtime image [running; 12.0s]") is None
assert 'Binding("e", "yank_errors"' in source
assert 'Binding("l", "select_last_lines"' in source
assert 'Binding("y", "copy_log", "Yank selection")' in source
assert 'Binding("Y", "copy_task_log", "Yank task log")' in source
assert 'No log selection to copy' in source
assert 'id="problems"' in source
assert 'total=len(TASK_PLAN)' in source
assert '#progress-box { width: 100%;' in source
assert '#progress { width: 100%;' in source
assert '#log-panel { width: 66%; }' in source
assert '#log-panel { width: 66%; border:' not in source
assert '#legend { height: 1; padding: 0 1; color: $text-muted; }' in source
class FakeTTY:
    def __init__(self, tty): self.tty = tty
    def isatty(self): return self.tty
assert gate.textual_terminal_available(FakeTTY(True), FakeTTY(True), "xterm-256color")
assert not gate.textual_terminal_available(FakeTTY(False), FakeTTY(True), "xterm-256color")
assert not gate.textual_terminal_available(FakeTTY(True), FakeTTY(True), "dumb")
assert any("brew install syft" in value for value in gate.run_gate.__code__.co_consts if isinstance(value, str))
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
    observed = []
    class RecordingRenderer:
        def emit(self, event):
            # Every transition/output event must follow its evidence write.
            assert (evidence / ("steps.log" if event.kind != "output" else "commands.log")).exists()
            observed.append(event)
    runner = gate.Runner(evidence, os.environ.copy(), heartbeat_interval=0.02,
                         renderer=RecordingRenderer())
    with contextlib.redirect_stdout(io.StringIO()):
        runner.run([sys.executable, "-c", "import time; time.sleep(0.07)"], label="quiet test")
    progress = (evidence / "steps.log").read_text()
    assert progress.count("quiet test [running") >= 2
    assert "quiet test [passed" in progress
    assert observed[-1].kind == "passed"
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
assert publisher.LOCAL_CANDIDATE_EVIDENCE == "evidence/container-e2e/local-candidate-substitution.env"
assert "evidence/container-e2e/impact-selection.json" in publisher.BASE_REQUIRED_EVIDENCE
with tempfile.TemporaryDirectory() as temporary:
    steps = Path(temporary) / "steps.log"
    attested = "… Assemble evidence manifest [running; 0.0s]\n".encode()
    expected = __import__("hashlib").sha256(attested).hexdigest()
    steps.write_bytes(attested + "✓ Assemble evidence manifest [passed; 0.1s]\n".encode())
    assert publisher.evidence_checksum_matches(steps, "evidence/steps.log", expected)
    steps.write_bytes(attested + "✓ Publication [passed; 0.1s]\n".encode())
    assert not publisher.evidence_checksum_matches(steps, "evidence/steps.log", expected)
PY

echo "release host gate contract tests passed"

uv run --with-requirements "${SCRIPT_DIR}/release-host-ui.lock" python - "${SCRIPT_DIR}" <<'PY'
import asyncio
import importlib.util
from pathlib import Path
import sys
from textual.app import App

script_dir = Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("gate_ui_test", script_dir / "release_host_gate.py")
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)
original_run = App.run
size = [(80, 24)]

def headless_run(self, *args, **kwargs):
    async def verify():
        async with self.run_test(size=size[0]) as pilot:
            await pilot.pause()
            copied = []
            notices = []
            self.copy_to_clipboard = copied.append
            self.notify = lambda message, **kwargs: notices.append(str(message))
            lines = "".join(f"successful line {index}\n" for index in range(25))
            self.apply_gate_event(gate.PresentationEvent(
                "output", task="Build runtime image",
                text=lines + "[bold]literal[/bold] \x1b[31mred\x1b[0m\nerror: candidate failed [literal]\n"
            ))
            self.apply_gate_event(gate.PresentationEvent("failed", task="Build runtime image", state="failed", elapsed=1.2))
            self.apply_gate_event(gate.PresentationEvent("output", task="Vulnerability policy", text="warning: no fix listed\n"))
            self.apply_gate_event(gate.PresentationEvent("warned", task="Vulnerability policy", state="warned", elapsed=2.0))
            self.apply_gate_event(gate.PresentationEvent("output", task="addon-tools", text="tool output without classifier\nexit status 7\n"))
            self.apply_gate_event(gate.PresentationEvent("failed", task="addon-tools", state="failed", elapsed=3.0))
            self.apply_gate_event(gate.PresentationEvent("plan"))
            # Force the UI's intentionally batched log refresh in the pilot;
            # the synthetic gate worker exits faster than the 10 ms timer.
            self._flush_log_render()
            await pilot.pause(0.05)
            screen_width = self.screen.size.width
            progress_region = self.query_one("#progress-box").region
            tasks_region = self.query_one("#tasks-panel").region
            log_panel = self.query_one("#log-panel")
            log = self.query_one("#log")
            legend = self.query_one("#legend")
            assert progress_region.width == screen_width, (progress_region, self.screen.size)
            assert progress_region.width > tasks_region.width
            assert log_panel.styles.border_top[0] == ""
            assert log.styles.border_top[0] == "round"
            assert legend.styles.border_top[0] == ""
            assert legend.region.height == 1
            assert "[bold]literal[/bold] red" in self.query_one("#log").text, repr(self.query_one("#log").text)
            assert self.query_one("#progress").total == len(gate.TASK_PLAN), self.query_one("#progress").total
            assert "3/" in str(self.query_one("#progress-label").render()), self.query_one("#progress-label").render()
            assert set(self.problems) == {"Build runtime image", "Vulnerability policy", "addon-tools"}
            assert self.problems["addon-tools"].lines[-1] == "exit status 7"
            assert sum(child.display for child in self.query_one("#problems").children) == 3
            self.selected_task = "All output"
            self._render_log()
            self.action_select_last_lines()
            selected = self.query_one("#log").selected_text
            assert selected and "successful line 11" in selected and "successful line 10" not in selected
            # `y` and Ctrl+C share action_copy_log: both must behave like a
            # visual-mode yank and copy only the active marked range.
            self.action_copy_log()
            assert copied[-1] == selected
            assert copied[-1] != self.query_one("#log").text
            self.action_copy_task_log()
            full_task_log = "".join(self.task_logs[self.selected_task])
            assert copied[-1] == full_task_log
            self.query_one("#log").selection = ((0, 0), (0, 0))
            self.action_copy_log()
            assert copied[-1] == full_task_log
            assert any("No log selection" in notice for notice in notices)
            self.action_yank_errors()
            error_bundle = copied[-1]
            assert "[FAILED] Build runtime image" in error_bundle
            assert "error: candidate failed [literal]" in error_bundle
            assert "[WARNING] Vulnerability policy" in error_bundle
            assert "warning: no fix listed" in error_bundle
            assert "[FAILED] addon-tools" in error_bundle
            assert "exit status 7" in error_bundle
            assert "successful line 1" not in error_bundle
    asyncio.run(verify())
    return 0

App.run = headless_run
try:
    for terminal_size in ((80, 24), (140, 40)):
        size[0] = terminal_size
        assert gate.run_textual_dashboard("sample-run", True) == 0
finally:
    App.run = original_run
print("release host Textual headless tests passed")
PY
