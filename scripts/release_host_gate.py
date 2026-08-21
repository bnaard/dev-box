#!/usr/bin/env python3
"""Validate and publish an immutable aibox release candidate on macOS.

The shell entrypoint supplies a deliberately sparse environment and an exact
Python interpreter. This module then verifies the prepared input, runs
candidate-controlled build and test commands without owner credentials,
records checksummed evidence, and—unless dry-run mode was explicitly selected—
invokes the separately constrained publisher only after every mandatory and
impact-selected check passes.

This is intentionally a macOS gate: it builds both Darwin artifacts, natively
smokes the current architecture, and uses Apple's ``sandbox-exec`` boundary.
"""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import queue
import re
import shlex
import shutil
import stat
import subprocess
import sys
import tarfile
import threading
import time
import urllib.request
from typing import NamedTuple

REPOSITORY = "projectious-work/aibox"
EXPECTED_INPUTS = {"checksums.sha256", "provenance.json", "source.tar.gz"}
RUN_ID = re.compile(r"^v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z][0-9A-Za-z.-]*)?-[0-9]{8}T[0-9]{6}Z-[0-9a-f]{12}$")
VERSION_TAG = re.compile(r"^v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z][0-9A-Za-z.-]*)?$")
TRUSTED_CONTROL_PATHS = (
    "scripts/maintain.sh",
    "scripts/release-host-gate.sh",
    "scripts/release-host-publish.sh",
    "scripts/release_host_gate.py",
    "scripts/release_host_publish.py",
    "scripts/release-host-ui.in",
    "scripts/release-host-ui.lock",
)

TEXTUAL_VERSION = "8.2.8"
UI_MODES = {"auto", "textual", "plain"}
TASK_PLAN = (
    "Validate immutable candidate", "Verify isolated prerequisites",
    "Fetch locked Rust dependencies", "Build Darwin release artifacts",
    "Smoke native Darwin binary", "Build foundation image", "Build runtime image",
    "Prepare local runtime image for smoke", "Inspect runtime image",
    "Generate image SBOM", "Scan image vulnerabilities", "Vulnerability policy",
    "Run candidate container lifecycle", "addon-languages", "addon-platforms",
    "addon-tools", "latex-lifecycle", "rootless-podman", "Assemble evidence manifest",
    "Dry-run completion", "Publication",
)
# Keep the selection size explicit: it is both an operator affordance and a
# stable contract for the headless dashboard tests.
LAST_LOG_LINES = 20
TASK_GLYPHS = {
    "pending": "○",
    "running": "⠋",
    "passed": "✓",
    "warned": "!",
    "skipped": "–",
    "failed": "✗",
}
TASK_STATE_LABELS = {
    "pending": "pending",
    "running": "running",
    "passed": "passed",
    "warned": "warning",
    "skipped": "skipped",
    "failed": "failed",
}


class Problem(NamedTuple):
    """Canonical dashboard problem model shared by the Problems view and yanks."""

    task: str
    state: str
    lines: tuple[str, ...]


def classify_log_line(text: str) -> str | None:
    """Classify diagnostic lines without treating normal progress as errors.

    Raw command output is sanitized only for this presentation model. The
    authoritative evidence files retain the original text. Warning lines are
    deliberately distinct: they become Problems only when the task itself
    enters the warning state (for example, an accepted no-fix scan result).
    """
    line = sanitize_display(text).strip()
    if not line:
        return None
    lowered = line.casefold()
    if re.search(r"\b(?:0|no)\s+(?:fatal\s+)?errors?\b", lowered):
        return None
    if re.search(r"\b(?:error|fatal|panic|traceback|exception|failure)\b", lowered):
        return "error"
    if re.search(r"\bwarn(?:ing)?\b", lowered):
        return "warning"
    return None


class PresentationEvent(NamedTuple):
    """Presentation-neutral state emitted only after evidence is flushed."""

    kind: str
    task: str = ""
    text: str = ""
    elapsed: float = 0.0
    state: str = ""


class PlainRenderer:
    """Preserve the line-oriented interface used by automation and captures."""

    def emit(self, event: PresentationEvent) -> None:
        if event.kind == "output":
            print(event.text, end="", flush=True)
        elif event.kind in {"started", "heartbeat", "passed", "warned", "failed", "skipped"}:
            icons = {"started": "…", "heartbeat": "…", "passed": "✓", "warned": "!", "failed": "✗", "skipped": "-"}
            state = event.state or ("running" if event.kind in {"started", "heartbeat"} else event.kind)
            print(f"{icons[event.kind]} {event.task} [{state}; {event.elapsed:.1f}s]", flush=True)


def parse_ui_mode(value: str | None) -> str:
    """Validate the wrapper-selected presentation mode."""
    mode = value or "auto"
    if mode not in UI_MODES:
        fail("AIBOX_RELEASE_HOST_UI must be exactly auto, textual, or plain")
    return mode


def textual_terminal_available(stdin: object = sys.stdin, stdout: object = sys.stdout,
                               term: str | None = None) -> bool:
    """Return whether an interactive dashboard is appropriate."""
    return bool(getattr(stdin, "isatty", lambda: False)()
                and getattr(stdout, "isatty", lambda: False)()
                and (term if term is not None else os.environ.get("TERM", "")) not in {"", "dumb"})


def sanitize_display(text: str) -> str:
    """Strip terminal controls for display while evidence retains raw text."""
    text = re.sub(r"\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)", "", text)
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", text)
    return "".join(character for character in text if character in "\n\t" or ord(character) >= 32)


def locked_ui_dependencies(lock: Path) -> list[str]:
    """Return the exact top-level package pins recorded in the uv lock."""
    return [line.split(" \\", 1)[0] for line in lock.read_text(encoding="utf-8").splitlines()
            if re.fullmatch(r"[a-z0-9-]+==[^ ]+ \\", line)]


class TextualRenderer:
    """Thread-safe bridge from the gate runner to the Textual application."""

    def __init__(self, app: object) -> None:
        self.app = app

    def emit(self, event: PresentationEvent) -> None:
        self.app.call_from_thread(self.app.apply_gate_event, event)


def run_textual_dashboard(run_argument: str, dry_run: bool) -> int:
    """Run the blocking gate in a worker while Textual owns the terminal."""
    from textual.app import App, ComposeResult
    from textual.binding import Binding
    from textual.containers import Horizontal, Vertical
    from textual.widgets import Footer, Header, Label, ListItem, ListView, ProgressBar, Static, TextArea

    class GateDashboard(App[int]):
        CSS = """
        Screen { layout: vertical; }
        #summary { height: 3; border: round $accent; padding: 0 1; }
        #progress-box { width: 100%; height: 4; padding: 0 1; }
        #progress { width: 100%; height: 3; }
        #progress-label { height: 1; color: $text-muted; }
        #body { height: 1fr; }
        #tasks-panel { width: 34%; min-width: 28; }
        #tasks { height: 1fr; border: round $accent; }
        #legend { height: 1; padding: 0 1; color: $text-muted; }
        #problems-title { height: 1; padding: 0 1; color: $warning; }
        #problems { height: 7; border: round $warning; }
        #log-panel { width: 66%; }
        #log { height: 1fr; border: round $accent; }
        #log-status { height: 1; color: $text-muted; }
        ListItem { padding: 0 1; }
        ListItem.-active { background: $boost; }
        """
        BINDINGS = [
            Binding("space", "toggle_follow", "Follow"),
            Binding("w", "toggle_wrap", "Wrap"),
            Binding("end", "follow_tail", "Live tail"),
            Binding("ctrl+a", "select_log", "Select log"),
            Binding("ctrl+c", "copy_log", "Copy"),
            Binding("l", "select_last_lines", f"Last {LAST_LOG_LINES}"),
            Binding("y", "copy_log", "Yank selection"),
            Binding("Y", "copy_task_log", "Yank task log"),
            Binding("e", "yank_errors", "Yank errors"),
            Binding("p", "show_log_path", "Evidence path"),
            Binding("question_mark", "help", "Keys"),
            Binding("q", "quit_when_done", "Quit"),
        ]

        def __init__(self) -> None:
            super().__init__()
            self.started_at = time.monotonic()
            self.finished = False
            self.follow = True
            self.wrap = True
            self.task_states: dict[str, tuple[str, float]] = {}
            self.task_logs: dict[str, list[str]] = {"All output": []}
            self.task_order = ["All output"]
            self.selected_task = "All output"
            self.candidate = "pending"
            self.container_runtime = "detecting"
            self.spinner_index = 0
            self.log_render_pending = False
            self.diagnostics: dict[str, dict[str, list[str]]] = {}
            self.problems: dict[str, Problem] = {}
            self.problem_order: list[str] = []

        def compose(self) -> ComposeResult:
            mode = "dry run" if dry_run else "publication"
            yield Header(show_clock=True)
            yield Static(f"release-host • {mode} • {Path(run_argument).name}", id="summary")
            with Vertical(id="progress-box"):
                yield ProgressBar(total=len(TASK_PLAN), show_eta=False, id="progress")
                yield Static(f"0/{len(TASK_PLAN)} tasks complete (0%)", id="progress-label")
            with Horizontal(id="body"):
                with Vertical(id="tasks-panel"):
                    yield ListView(
                        ListItem(Label("● All output [all output]"), id="task-0"),
                        *(ListItem(Label(f"○ {task} [pending]")) for task in TASK_PLAN), id="tasks",
                    )
                    yield Static(
                        "Legend: ○ pending  ⠋ running  ✓ passed  ! warning  – skipped  ✗ failed",
                        id="legend",
                    )
                    yield Static("Problems", id="problems-title")
                    yield ListView(
                        *(ListItem(Label("", markup=False), id=f"problem-{index}")
                          for index in range(len(TASK_PLAN) + 8)),
                        id="problems",
                    )
                with Vertical(id="log-panel"):
                    yield TextArea("", read_only=True, show_line_numbers=False, soft_wrap=True,
                                   id="log", classes="log-view")
                    yield Static("FOLLOW • WRAP • full evidence: evidence/command-results.log", id="log-status")
            yield Footer()

        def on_mount(self) -> None:
            for task in TASK_PLAN:
                self.task_logs[task] = []
                self.task_order.append(task)
                self.task_states[task] = ("pending", 0.0)
            self.run_worker(self._execute_gate, thread=True, exclusive=True)
            self.set_interval(1.0, self._refresh_elapsed)

        def _execute_gate(self) -> None:
            code = 0
            try:
                run_gate(TextualRenderer(self))
            except SystemExit as error:
                code = error.code if isinstance(error.code, int) else 1
                self.call_from_thread(self.apply_gate_event, PresentationEvent("output", text=f"{error}\n"))
            except BaseException as error:  # retain an actionable failure in the UI
                code = 1
                self.call_from_thread(self.apply_gate_event, PresentationEvent("output", text=f"{type(error).__name__}: {error}\n"))
            self.call_from_thread(self._finish, code)

        def _finish(self, code: int) -> None:
            self.finished = True
            self.exit_code = code
            self.notify("Gate completed" if code == 0 else f"Gate failed (exit {code})",
                        severity="information" if code == 0 else "error", timeout=10)
            if not sys.stdin.isatty():
                self.exit(code)

        def apply_gate_event(self, event: PresentationEvent) -> None:
            visible_task = event.task in TASK_PLAN or event.task.startswith("Cleanup ")
            if event.task and visible_task and event.task not in self.task_logs:
                self.task_logs[event.task] = []
                self.task_order.append(event.task)
                self.query_one("#tasks", ListView).append(ListItem(Label(f"○ {event.task}")))
            if event.kind == "output":
                clean = sanitize_display(event.text)
                self.task_logs["All output"].append(clean)
                if visible_task:
                    self.task_logs.setdefault(event.task, []).append(clean)
                    for line in clean.splitlines():
                        classification = classify_log_line(line)
                        if classification:
                            bucket = self.diagnostics.setdefault(event.task, {"error": [], "warning": []})
                            if line.strip() and line not in bucket[classification]:
                                bucket[classification].append(line)
                if self.selected_task in {"All output", event.task}:
                    self._schedule_log_render()
                return
            if event.kind == "metadata":
                metadata = json.loads(event.text)
                self.candidate = metadata.get("commit", self.candidate)
                self.container_runtime = metadata.get("container_runtime", self.container_runtime)
                self._refresh_elapsed()
                return
            if event.kind == "plan":
                self._refresh_progress()
                return
            if visible_task:
                self.task_states[event.task] = (event.state or event.kind, event.elapsed)
                state = event.state or event.kind
                if state in {"warned", "failed"}:
                    self._record_problem(event.task, state)
                self._refresh_tasks()
                self._refresh_progress()

        def _refresh_tasks(self) -> None:
            spinner = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"[self.spinner_index]
            view = self.query_one("#tasks", ListView)
            for index, task in enumerate(self.task_order[1:], start=1):
                state, elapsed = self.task_states.get(task, ("pending", 0.0))
                icon = spinner if state == "running" else TASK_GLYPHS.get(state, TASK_GLYPHS["pending"])
                state_label = TASK_STATE_LABELS.get(state, state)
                duration = f"{elapsed:.1f}s" if state != "skipped" else "skipped"
                view.children[index].query_one(Label).update(
                    f"{icon} {task} [{state_label}]  {duration}"
                )

        def _refresh_progress(self) -> None:
            """Update fixed-denominator numeric progress, including skips."""
            completed = sum(
                self.task_states.get(task, ("pending", 0.0))[0]
                in {"passed", "warned", "failed", "skipped"}
                for task in TASK_PLAN
            )
            total = len(TASK_PLAN)
            percent = int((completed * 100) / total) if total else 100
            self.query_one("#progress", ProgressBar).update(total=total, progress=completed)
            self.query_one("#progress-label", Static).update(
                f"{completed}/{total} tasks complete ({percent}%)"
            )

        def _record_problem(self, task: str, state: str) -> None:
            """Reduce state and captured diagnostics into one canonical problem."""
            bucket = self.diagnostics.get(task, {"error": [], "warning": []})
            lines = list(bucket["error"])
            if state == "warned":
                lines.extend(line for line in bucket["warning"] if line not in lines)
            if state == "failed" and not lines:
                # Some tools exit non-zero without printing a line containing
                # a classifier word such as "error". Preserve a bounded tail
                # so Problems and its yank remain useful for diagnosis.
                task_tail = [
                    line for chunk in self.task_logs.get(task, [])
                    for line in chunk.splitlines() if line.strip()
                ]
                lines = task_tail[-LAST_LOG_LINES:]
            if not lines:
                lines = [f"{TASK_STATE_LABELS.get(state, state).capitalize()}: {task}"]
            previous = self.problems.get(task)
            if previous and previous.state == "failed":
                state = "failed"
            self.problems[task] = Problem(task, state, tuple(lines))
            if task not in self.problem_order:
                self.problem_order.append(task)
            self._refresh_problems()

        def _refresh_problems(self) -> None:
            view = self.query_one("#problems", ListView)
            for index, child in enumerate(view.children):
                if index >= len(self.problem_order):
                    child.display = index == 0 and not self.problem_order
                    if index == 0 and not self.problem_order:
                        child.query_one(Label).update("No warnings or failures")
                    continue
                child.display = True
                task = self.problem_order[index]
                problem = self.problems[task]
                label = TASK_STATE_LABELS.get(problem.state, problem.state)
                text = problem.lines[0][:100]
                child.query_one(Label).update(
                    f"{TASK_GLYPHS[problem.state]} {task} [{label}] {text}"
                )

        def _schedule_log_render(self) -> None:
            """Coalesce high-volume subprocess lines into one screen refresh."""
            if not self.log_render_pending:
                self.log_render_pending = True
                self.set_timer(0.01, self._flush_log_render)

        def _flush_log_render(self) -> None:
            self.log_render_pending = False
            self._render_log()

        def _render_log(self) -> None:
            log = self.query_one("#log", TextArea)
            log.load_text("".join(self.task_logs.get(self.selected_task, [])))
            if self.follow:
                log.move_cursor(log.document.end)

        def on_list_view_selected(self, event: ListView.Selected) -> None:
            if event.list_view.id == "problems":
                index = event.list_view.index or 0
                if index < len(self.problem_order):
                    self.selected_task = self.problem_order[index]
            else:
                self.selected_task = self.task_order[event.list_view.index or 0]
            self._render_log()

        def _refresh_elapsed(self) -> None:
            self.spinner_index = (self.spinner_index + 1) % 10
            self._refresh_tasks()
            self.query_one("#summary", Static).update(
                f"release-host • {'dry run' if dry_run else 'publication'} • {Path(run_argument).name} • "
                f"{self.candidate[:12]} • {self.container_runtime} • {time.monotonic() - self.started_at:.0f}s"
            )

        def action_toggle_follow(self) -> None:
            self.follow = not self.follow
            self._update_log_status()

        def action_toggle_wrap(self) -> None:
            self.wrap = not self.wrap
            self.query_one("#log", TextArea).soft_wrap = self.wrap
            self._update_log_status()

        def action_follow_tail(self) -> None:
            self.follow = True
            self._render_log()
            self._update_log_status()

        def _update_log_status(self) -> None:
            self.query_one("#log-status", Static).update(
                f"{'FOLLOW' if self.follow else 'PAUSED'} • {'WRAP' if self.wrap else 'NO WRAP'} • full evidence: evidence/command-results.log"
            )

        def action_select_log(self) -> None:
            self.query_one("#log", TextArea).select_all()

        def action_copy_log(self) -> None:
            log = self.query_one("#log", TextArea)
            text = log.selected_text
            if text:
                self.copy_to_clipboard(text)
                self.notify("Selection copied")
            else:
                self.notify("No log selection to copy", severity="warning", timeout=3)

        def action_select_last_lines(self) -> None:
            log = self.query_one("#log", TextArea)
            line_count = log.document.line_count
            if not line_count:
                self.notify("No log lines to select", severity="warning", timeout=3)
                return
            start = max(0, line_count - LAST_LOG_LINES)
            end = line_count - 1
            # Textual's public select_line API handles one row only. Its
            # Selection reactive accepts the same (line, column) tuples and
            # preserves normal mouse/keyboard selection behavior.
            log.selection = ((start, 0), (end, len(log.document[end])))
            self.notify(f"Selected last {min(LAST_LOG_LINES, line_count)} log lines")

        def action_yank_errors(self) -> None:
            if not self.problems:
                self.notify("No warnings or failures to yank", severity="information", timeout=3)
                return
            lines = ["aibox release-host problems"]
            for task in self.problem_order:
                problem = self.problems[task]
                lines.append(f"\n[{TASK_STATE_LABELS.get(problem.state, problem.state).upper()}] {task}")
                lines.extend(f"  {line}" for line in problem.lines)
            self.copy_to_clipboard("\n".join(lines) + "\n")
            self.notify(f"Copied {len(self.problems)} warning/failure problem(s)")

        def action_copy_task_log(self) -> None:
            self.copy_to_clipboard("".join(self.task_logs.get(self.selected_task, [])))
            self.notify(f"Copied {self.selected_task}")

        def action_show_log_path(self) -> None:
            self.notify("Authoritative log: evidence/command-results.log", timeout=8)

        def action_help(self) -> None:
            self.notify(
                f"Tab focus • arrows/page scroll • Space follow • w wrap • Ctrl+A/C select/copy • "
                f"l last {LAST_LOG_LINES} lines • y selection • Y task log • e yank errors • End tail • p evidence path • q quit",
                timeout=12,
            )

        def action_quit_when_done(self) -> None:
            if self.finished:
                self.exit(getattr(self, "exit_code", 0))
            else:
                self.notify("Gate is running; quit is disabled to protect cleanup", severity="warning")

    result = GateDashboard().run()
    return int(result or 0)

ADDON_GROUPS = {
    "addon-languages": ["docs-hugo", "docs-mdbook", "go", "go-quality", "go-release", "node", "rust", "typst"],
    "addon-platforms": ["cloud-aws", "cloud-gcp", "cloudflare", "infrastructure", "kubernetes"],
    "addon-tools": ["ai-claude", "ai-opencode", "browser-testing", "preview-archive", "supply-chain"],
}
ADDON_GROUP_TRIGGERS = {
    **ADDON_GROUPS,
    # go-release consumes the shared release addon transitively.
    "addon-languages": [*ADDON_GROUPS["addon-languages"], "release"],
}
ALL_IMPACT_CHECKS = {*ADDON_GROUPS, "latex-lifecycle", "rootless-podman"}
BROAD_IMPACT_PREFIXES = (
    "images/base-debian/", "cli/src/addon_loader.rs", "cli/src/addons.rs",
    "cli/src/cli.rs", "cli/src/config.rs", "cli/src/container.rs", "cli/src/generate.rs",
    "cli/src/runtime.rs", "cli/src/templates/", "scripts/release-host-",
    "scripts/release_host_",
)


def select_impact_checks(changed_paths: list[str]) -> dict[str, str]:
    """Select expensive host checks from an already verified release diff.

    The returned mapping is both the execution plan and human-readable audit
    evidence: each key is a reviewed conditional check and each value explains
    which path or fail-safe rule selected it. Broad runtime machinery selects
    every conditional check. A missing comparison tag is represented by
    ``["*"]`` and also selects everything rather than producing a passing skip.
    """
    if changed_paths == ["*"]:
        return {check: "no comparison tag; fail-safe full selection" for check in sorted(ALL_IMPACT_CHECKS)}
    broad = next((path for path in changed_paths if path.startswith(BROAD_IMPACT_PREFIXES)), None)
    if broad:
        return {check: f"broad runtime impact: {broad}" for check in sorted(ALL_IMPACT_CHECKS)}

    selected: dict[str, str] = {}
    for group, addons in ADDON_GROUP_TRIGGERS.items():
        for path in changed_paths:
            if any(path.endswith(f"/{addon}.yaml") for addon in addons):
                selected[group] = path
                break
    for path in changed_paths:
        if path.endswith("/latex.yaml") or path.startswith("cli/src/latex.rs") or "aibox-latex-" in path:
            selected["latex-lifecycle"] = path
        if path.endswith("/infrastructure.yaml") or "podman" in path.lower():
            selected["rootless-podman"] = path
    return selected


def grype_policy_summary(report: Path) -> dict[str, object]:
    """Group High/Critical Grype matches and classify fix availability."""
    payload = json.loads(report.read_text(encoding="utf-8"))
    advisories: dict[str, dict[str, object]] = {}
    package_matches = 0
    actionable_matches = 0
    for match in payload.get("matches", []):
        vulnerability = match.get("vulnerability", {})
        severity = str(vulnerability.get("severity", "unknown"))
        if severity.lower() not in {"high", "critical"}:
            continue
        package_matches += 1
        artifact = match.get("artifact", {})
        fix_versions = sorted(set(vulnerability.get("fix", {}).get("versions", [])))
        actionable_matches += bool(fix_versions)
        identifier = str(vulnerability.get("id", "unknown"))
        advisory = advisories.setdefault(identifier, {
            "id": identifier, "severity": severity, "actionable": False, "packages": [],
        })
        if severity.lower() == "critical":
            advisory["severity"] = "Critical"
        advisory["actionable"] = bool(advisory["actionable"] or fix_versions)
        advisory["packages"].append({
            "name": str(artifact.get("name", "unknown")),
            "version": str(artifact.get("version", "unknown")),
            "fix_versions": fix_versions,
        })
    grouped = sorted(advisories.values(), key=lambda item: (not item["actionable"], item["id"]))
    for advisory in grouped:
        advisory["packages"] = sorted(
            advisory["packages"], key=lambda package: (package["name"], package["version"])
        )
    actionable_advisories = sum(bool(advisory["actionable"]) for advisory in grouped)
    return {
        "schema_version": 1,
        "policy": "block-high-critical-with-listed-fix",
        "high_critical_package_matches": package_matches,
        "unique_advisories": len(grouped),
        "actionable_package_matches": actionable_matches,
        "no_fix_package_matches": package_matches - actionable_matches,
        "actionable_advisories": actionable_advisories,
        "no_fix_advisories": len(grouped) - actionable_advisories,
        "advisories": grouped,
    }


def print_grype_policy_summary(summary: dict[str, object], renderer: object | None = None) -> None:
    """Render a bounded advisory-level view of the vulnerability policy."""
    lines = [
        "Grype policy: "
        f"{summary['unique_advisories']} unique High/Critical advisories across "
        f"{summary['high_critical_package_matches']} package matches; "
        f"{summary['actionable_advisories']} actionable, "
        f"{summary['no_fix_advisories']} no-fix warnings."
    ]
    for advisory in summary["advisories"][:20]:
        packages = advisory["packages"]
        package_names = ", ".join(sorted({package["name"] for package in packages})[:4])
        if len({package["name"] for package in packages}) > 4:
            package_names += ", …"
        fixes = sorted({version for package in packages for version in package["fix_versions"]})
        disposition = f"BLOCK; fixed in {', '.join(fixes)}" if fixes else "WARN; no fix listed"
        lines.append(f"  - {advisory['severity']}: {advisory['id']} ({package_names}) — {disposition}")
    if len(summary["advisories"]) > 20:
        lines.append(f"  - … and {len(summary['advisories']) - 20} more advisories")
    (renderer or PlainRenderer()).emit(PresentationEvent("output", task="Vulnerability policy",
                                                          text="\n".join(lines) + "\n"))


def fail(message: str) -> "None":
    """Terminate the gate with a consistently prefixed operator error."""
    raise SystemExit(f"release-host gate: {message}")


def dry_run_enabled(value: str | None) -> bool:
    """Parse the wrapper-controlled publication mode without truthy ambiguity."""
    if value in (None, "0"):
        return False
    if value == "1":
        return True
    fail("AIBOX_RELEASE_HOST_DRY_RUN must be exactly 0 or 1")


def cache_reuse_enabled(value: str | None) -> bool:
    """Parse the reviewed cache policy without accepting ambiguous values."""
    if value in (None, "0"):
        return False
    if value == "1":
        return True
    fail("AIBOX_RELEASE_HOST_REUSE_CACHE must be exactly 0 or 1")


def sha256(path: Path) -> str:
    """Return the streaming SHA-256 digest of *path* as lowercase hex."""
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def seal_checkpoint(path: Path) -> None:
    """Bind a completed conditional check to its exact evidence bytes."""
    path.with_suffix(path.suffix + ".sha256").write_text(
        f"{sha256(path)}  {path.name}\n", encoding="utf-8"
    )


def reuse_checkpoint(retry_evidence: Path, evidence: Path, check: str) -> Path | None:
    """Copy a valid completed conditional check from a candidate-identical run."""
    marker = retry_evidence / "container-e2e" / f"{check}.json"
    checksum = marker.with_suffix(marker.suffix + ".sha256")
    if (not marker.is_file() or marker.is_symlink() or
            not checksum.is_file() or checksum.is_symlink()):
        return None
    fields = checksum.read_text(encoding="utf-8").strip().split(maxsplit=1)
    if len(fields) != 2 or fields[0] != sha256(marker) or fields[1].lstrip("*") != marker.name:
        return None
    payload = json.loads(marker.read_text(encoding="utf-8"))
    if payload.get("status") != "passed":
        return None
    if check in ADDON_GROUPS:
        expected = {"status": "passed", "addons": ADDON_GROUPS[check]}
        if check == "addon-tools":
            expected["browser_fixture"] = {
                "title": "Fixture", "violations": 0, "violation_details": [],
            }
        if payload != expected:
            return None
    elif check == "latex-lifecycle":
        if payload.get("revisions") != 2 or not isinstance(payload.get("preview_port"), int):
            return None
    elif check == "rootless-podman":
        if payload.get("rootless_readiness") is not True:
            return None
    else:
        return None
    destination = evidence / "container-e2e" / marker.name
    shutil.copy2(marker, destination)
    shutil.copy2(checksum, destination.with_suffix(destination.suffix + ".sha256"))
    return destination


def validate_regular(path: Path) -> None:
    """Require an immutable, regular, single-link input file."""
    info = path.lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
        fail(f"input must be a regular single-link file: {path.name}")
    if info.st_mode & (stat.S_IWUSR | stat.S_IWGRP | stat.S_IWOTH):
        fail(f"immutable input is writable: {path.name}")


def validate_directory(path: Path) -> None:
    """Require a real directory that is not writable by group or others."""
    info = path.lstat()
    if not stat.S_ISDIR(info.st_mode) or path.is_symlink():
        fail(f"expected a real directory: {path}")
    if info.st_mode & (stat.S_IWGRP | stat.S_IWOTH):
        fail(f"directory has unsafe write permissions: {path}")


def resolve_run_dir(argument: str, project_root: Path) -> Path:
    """Resolve one run directory directly beneath the fixed release-gate root.

    Keeping the accepted shape this narrow prevents traversal, symlink, and
    arbitrary-path arguments from expanding the publisher's authority.
    """
    approved = (project_root / "tmp/host-gates/aibox-release").resolve()
    supplied = Path(argument)
    if not supplied.is_absolute():
        supplied = project_root / supplied
    if supplied.is_symlink():
        fail("run directory must not be a symlink")
    resolved = supplied.resolve(strict=True)
    if resolved.parent != approved or not RUN_ID.fullmatch(resolved.name):
        fail(f"run directory must be one direct child of {approved}")
    return resolved


class Runner:
    """Run exact argv commands with a fixed environment and append-only logs.

    Commands never pass through a shell. Both the rendered argv and combined
    output/exit status are retained so a host rehearsal can be audited without
    trusting terminal scrollback.
    """

    def __init__(self, evidence: Path, env: dict[str, str], *, heartbeat_interval: float = 10.0,
                 renderer: object | None = None) -> None:
        """Bind command logs to *evidence* and freeze the subprocess environment."""
        self.log = evidence / "commands.log"
        self.steps = evidence / "steps.log"
        self.env = env
        self.heartbeat_interval = heartbeat_interval
        self.renderer = renderer or PlainRenderer()
        self.output_task: str | None = None

    def _emit(self, event: PresentationEvent) -> None:
        """Deliver one event after its authoritative evidence write."""
        self.renderer.emit(event)

    def run(self, command: list[str], *, cwd: Path | None = None, output: Path | None = None,
            label: str | None = None) -> None:
        """Run *command* with live output and periodic quiet-command heartbeats."""
        rendered = shlex.join(command)
        step = label or " ".join(command[:2])
        started = time.monotonic()
        self._step("running", step, 0.0)
        with self.log.open("a", encoding="utf-8") as log:
            log.write(rendered + "\n")
            log.flush()
        self._emit(PresentationEvent("output", task=self.output_task or step, text=f"+ {rendered}\n"))
        result_log = self.log.parent / "command-results.log"
        stdout_target = subprocess.PIPE if output is None else output.open("wb")
        try:
            process = subprocess.Popen(
                command, cwd=cwd, env=self.env, text=True, bufsize=1,
                stdin=subprocess.DEVNULL,
                stdout=stdout_target, stderr=subprocess.STDOUT if output is None else subprocess.PIPE,
            )
            stream = process.stdout if output is None else process.stderr
            assert stream is not None
            messages: queue.Queue[str | None] = queue.Queue()

            def pump() -> None:
                """Read command output without blocking heartbeat rendering."""
                for line in iter(stream.readline, ""):
                    messages.put(line)
                messages.put(None)

            reader = threading.Thread(target=pump, daemon=True)
            reader.start()
            last_activity = time.monotonic()
            with result_log.open("a", encoding="utf-8") as results:
                results.write(f"$ {rendered}\n")
                while True:
                    try:
                        message = messages.get(timeout=min(1.0, self.heartbeat_interval))
                    except queue.Empty:
                        now = time.monotonic()
                        if now - last_activity >= self.heartbeat_interval:
                            self._step("running", step, now - started)
                            last_activity = now
                        continue
                    if message is None:
                        break
                    results.write(message)
                    results.flush()
                    self._emit(PresentationEvent("output", task=self.output_task or step, text=message))
                    last_activity = time.monotonic()
                returncode = process.wait()
                elapsed = time.monotonic() - started
                results.write(f"exit={returncode}\n")
            if returncode != 0:
                self._step("failed", step, elapsed)
                raise subprocess.CalledProcessError(returncode, command)
            self._step("passed", step, elapsed)
        finally:
            if output is not None:
                stdout_target.close()

    def _step(self, state: str, label: str, elapsed: float) -> None:
        """Print and retain one high-level progress transition."""
        icons = {"running": "…", "passed": "✓", "failed": "✗", "warned": "!", "skipped": "-"}
        line = f"{icons[state]} {label} [{state}; {elapsed:.1f}s]"
        with self.steps.open("a", encoding="utf-8") as stream:
            stream.write(line + "\n")
            stream.flush()
        kind = "heartbeat" if state == "running" and elapsed else "started" if state == "running" else state
        self._emit(PresentationEvent(kind, task=label, elapsed=elapsed, state=state))

    def capture(self, command: list[str], *, cwd: Path | None = None) -> str:
        """Run *command*, log it, and return its combined textual output."""
        rendered = shlex.join(command)
        with self.log.open("a", encoding="utf-8") as log:
            log.write(rendered + "\n")
            log.flush()
        task = self.output_task or " ".join(command[:2])
        self._emit(PresentationEvent("output", task=task, text=f"+ {rendered}\n"))
        completed = subprocess.run(command, cwd=cwd, env=self.env, text=True,
                                   stdin=subprocess.DEVNULL,
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        with (self.log.parent / "command-results.log").open("a", encoding="utf-8") as results:
            results.write(f"$ {rendered}\n{completed.stdout}\nexit={completed.returncode}\n")
        if completed.stdout:
            self._emit(PresentationEvent("output", task=task,
                                         text=completed.stdout + ("" if completed.stdout.endswith("\n") else "\n")))
        completed.check_returncode()
        return completed.stdout


def sandboxed(profile: Path, command: list[str], env: dict[str, str]) -> list[str]:
    """Wrap candidate-controlled argv in the macOS credential-denial sandbox."""
    assignments = [f"{key}={value}" for key, value in sorted(env.items())]
    return ["/usr/bin/sandbox-exec", "-f", str(profile), "/usr/bin/env", "-i", *assignments, *command]


def select_container_runtime(env: dict[str, str]) -> str:
    """Return the first responsive Docker-compatible host runtime.

    Docker is preferred because both Docker Desktop and OrbStack expose that
    CLI contract. Podman is the fallback. Selection requires a responsive
    daemon or machine, not merely an executable on ``PATH``.
    """
    for candidate in ("docker", "podman"):
        if shutil.which(candidate, path=env["PATH"]) is None:
            continue
        probe = subprocess.run([candidate, "info"], env=env, stdout=subprocess.DEVNULL,
                               stderr=subprocess.DEVNULL)
        if probe.returncode == 0:
            return candidate
    fail("no responsive container runtime found; start OrbStack or Docker Desktop, or install and start Podman")


def stage_docker_cli_plugins(owner_home: Path, docker_config: Path) -> dict[str, str]:
    """Copy trusted Compose and Buildx plugins into the empty config.

    Docker CLI plugins normally live beside ``config.json``. Replacing
    ``DOCKER_CONFIG`` protects registry credentials but would also hide
    OrbStack's or Docker Desktop's plugins. Staging the executables alone
    restores Compose and BuildKit without copying owner configuration or secrets.
    """
    staged: dict[str, str] = {}
    destination_dir = docker_config / "cli-plugins"
    destination_dir.mkdir(mode=0o700)
    for plugin in ("docker-compose", "docker-buildx"):
        candidates = (
            owner_home / ".docker/cli-plugins" / plugin,
            Path("/Applications/OrbStack.app/Contents/MacOS/xbin") / plugin,
            Path("/Applications/Docker.app/Contents/Resources/cli-plugins") / plugin,
            Path("/opt/homebrew/lib/docker/cli-plugins") / plugin,
            Path("/usr/local/lib/docker/cli-plugins") / plugin,
        )
        for candidate in candidates:
            if not candidate.is_file() or not os.access(candidate, os.X_OK):
                continue
            info = candidate.stat()
            if info.st_uid not in {0, os.getuid()} or info.st_mode & 0o022:
                continue
            destination = destination_dir / plugin
            shutil.copy2(candidate, destination)
            destination.chmod(0o500)
            staged[plugin] = str(candidate)
            break
    return staged


def pin_release_version(config_path: Path, version: str) -> None:
    """Pin a generated project to the locally built candidate image version."""
    config = config_path.read_text(encoding="utf-8")
    updated, count = re.subn(
        r'(?m)^release_version = "[^"]*"',
        f'release_version = "{version}"',
        config,
        count=1,
    )
    if count != 1:
        fail(f"could not pin release version in {config_path}")
    config_path.write_text(updated, encoding="utf-8")


def compose_cleanup(runner: Runner, container_runtime: str, compose_file: Path) -> None:
    """Remove the probe's containers, volumes, orphans, and derived local image.

    Cleanup is deliberately fail-closed: callers invoke this from ``finally``
    blocks, and a cleanup error prevents publication just like a failed probe.
    """
    runner.run([container_runtime, "compose", "-f", str(compose_file), "down", "-v",
                "--remove-orphans", "--rmi", "local"], cwd=compose_file.parent.parent,
               label=f"Cleanup {compose_file.parent.parent.name}")


def init_project(runner: Runner, profile: Path, env: dict[str, str], candidate_bin: Path,
                 project: Path, name: str, addons: list[str]) -> None:
    """Create an isolated downstream project with the candidate Darwin binary."""
    project.mkdir()
    command = [str(candidate_bin), "--yes", "init", name, "--base", "debian",
               "--profile", "human-dev", "--theme", "tokyo-night", "--prompt", "arrow",
               "--tmux-status", "extended", "--processkit-version", "unset", "--no-container"]
    if addons:
        command.extend(["--addon", *addons])
    runner.run(sandboxed(profile, command, env), cwd=project)


def prepare_project_build(runner: Runner, profile: Path, env: dict[str, str], candidate_bin: Path,
                          container_runtime: str, project: Path, version: str) -> None:
    """Build a generated project against the unpublished local candidate.

    Docker-compatible Buildx installations can keep attested images in a
    store that their downstream builder cannot resolve by the future GHCR
    name. For those hosts, generate without a runtime, make one exact FROM
    substitution to the gate-built single-manifest alias, and use the daemon
    builder. Podman continues through the normal candidate ``apply`` path.
    """
    local_ref = env.get("AIBOX_RELEASE_SMOKE_LOCAL_CANDIDATE_REF", "")
    reuse_cache = env.get("AIBOX_RELEASE_HOST_REUSE_CACHE") == "1"
    if not local_ref:
        apply_command = [str(candidate_bin), "--yes", "apply"]
        if not reuse_cache:
            apply_command.append("--no-cache")
        runner.run(sandboxed(profile, apply_command, env),
                   cwd=project)
        return

    runner.run(sandboxed(profile, [str(candidate_bin), "--yes", "apply", "--no-container"], env),
               cwd=project)
    dockerfile = project / ".devcontainer/Dockerfile"
    expected = f"FROM ghcr.io/{REPOSITORY}:base-debian-runtime-v{version} AS aibox"
    replacement = f"FROM {local_ref} AS aibox"
    content = dockerfile.read_text(encoding="utf-8")
    if content.count(expected) != 1:
        fail(f"generated Dockerfile in {project.name} has no unique candidate FROM line")
    dockerfile.write_text(content.replace(expected, replacement, 1), encoding="utf-8")
    compose_file = project / ".devcontainer/docker-compose.yml"
    build_command = ["/usr/bin/env", "DOCKER_BUILDKIT=0", "COMPOSE_DOCKER_CLI_BUILD=0",
                     container_runtime, "compose", "-f", str(compose_file), "build"]
    if not reuse_cache:
        build_command.append("--no-cache")
    runner.run(build_command,
               cwd=project)


def run_addon_group(runner: Runner, profile: Path, env: dict[str, str], candidate_bin: Path,
                    container_runtime: str, runtime: Path, evidence: Path,
                    version: str, group: str) -> Path:
    """Build one reviewed group of download-based addons and record success."""
    name = f"host-gate-{group}"
    project = runtime / name
    init_project(runner, profile, env, candidate_bin, project, name, ADDON_GROUPS[group])
    pin_release_version(project / "aibox.toml", version)
    compose_file = project / ".devcontainer/docker-compose.yml"
    try:
        prepare_project_build(runner, profile, env, candidate_bin, container_runtime,
                              project, version)
        browser_fixture = None
        if group == "addon-tools":
            runner.run([container_runtime, "compose", "-f", str(compose_file), "up", "-d", name],
                       cwd=project)
            fixture_script = (
                'const { chromium } = require("@playwright/test"); '
                'const AxeBuilder = require("@axe-core/playwright").default; '
                '(async () => { const browser = await chromium.launch({ headless: true, channel: "chromium" }); '
                'const context = await browser.newContext(); '
                'const page = await context.newPage(); '
                'await page.setContent("<!doctype html><html lang=\\"en\\"><head><meta charset=\\"utf-8\\">'
                '<title>Fixture</title></head><body><main><h1>Fixture</h1>'
                '<button type=\\"button\\">Ready</button></main></body></html>"); '
                'const results = await new AxeBuilder({ page }).analyze(); '
                'const violation_details = results.violations.map(violation => ({ '
                'id: violation.id, impact: violation.impact, help: violation.help, '
                'helpUrl: violation.helpUrl, nodes: violation.nodes.map(node => ({ '
                'target: node.target, html: node.html, failureSummary: node.failureSummary })) })); '
                'console.log(JSON.stringify({ title: await page.title(), '
                'violations: violation_details.length, violation_details })); '
                'await context.close(); await browser.close(); '
                '})().catch(error => { console.error(error); process.exit(1); });'
            )
            output = runner.capture([container_runtime, "exec", "--user", "aibox", name,
                                     "node", "-e", fixture_script])
            browser_fixture = json.loads(output.strip().splitlines()[-1])
            if browser_fixture != {
                    "title": "Fixture", "violations": 0, "violation_details": []}:
                fail(f"browser-testing fixture returned unexpected evidence: {browser_fixture}")
        marker = evidence / "container-e2e" / f"{group}.json"
        payload = {"status": "passed", "addons": ADDON_GROUPS[group]}
        if browser_fixture is not None:
            payload["browser_fixture"] = browser_fixture
        marker.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        seal_checkpoint(marker)
        return marker
    finally:
        if compose_file.exists():
            compose_cleanup(runner, container_runtime, compose_file)


def wait_until(description: str, probe, *, attempts: int = 90, delay: float = 2.0) -> object:
    """Poll a transient readiness probe and fail after a bounded timeout."""
    last_error: Exception | None = None
    for _ in range(attempts):
        try:
            value = probe()
            if value:
                return value
        except Exception as error:  # Expected while containers and watchers start.
            last_error = error
        time.sleep(delay)
    fail(f"timed out waiting for {description}: {last_error or 'probe remained false'}")


def latex_document(marker: str) -> str:
    """Return the minimal LaTeX fixture used to detect live rebuilds."""
    return f"\\documentclass{{article}}\n\\begin{{document}}\n{marker}\n\\end{{document}}\n"


def run_latex_lifecycle(runner: Runner, profile: Path, env: dict[str, str], candidate_bin: Path,
                        container_runtime: str, runtime: Path, evidence: Path,
                        version: str, port: int) -> Path:
    """Prove LaTeX watch, rebuild, and preview behavior across two revisions.

    The check builds the candidate-derived container, starts its preview
    sidecar, launches the watcher without a shell wrapper, observes two PDF
    revisions, and requires the served PDF to match the generated file byte for
    byte. Cleanup runs regardless of the probe outcome.
    """
    name = "host-gate-latex"
    project = runtime / name
    init_project(runner, profile, env, candidate_bin, project, name, ["latex"])
    pin_release_version(project / "aibox.toml", version)
    config_path = project / "aibox.toml"
    config = config_path.read_text(encoding="utf-8")
    marker = "[latex.preview]"
    if config.count(marker) != 1:
        fail("generated LaTeX configuration has no unique preview section")
    document = ('[[latex.documents]]\nname = "overview"\nsource = "docs/overview.tex"\n'
                'output_dir = ".latex-cache/overview"\n\n')
    before, after = config.split(marker)
    after, enabled_count = re.subn(r'(?m)^enabled = false', "enabled = true", after, count=1)
    after, port_count = re.subn(r'(?m)^port = [0-9]+.*$', f"port = {port}", after, count=1)
    if enabled_count != 1 or port_count != 1:
        fail("generated LaTeX preview settings could not be enabled")
    config_path.write_text(before + document + marker + after, encoding="utf-8")
    (project / "docs").mkdir(exist_ok=True)
    source = project / "docs/overview.tex"
    source.write_text(latex_document("WATCHREVISIONONE"), encoding="utf-8")
    compose_file = project / ".devcontainer/docker-compose.yml"
    try:
        runner.run(sandboxed(profile, [str(candidate_bin), "--yes", "apply", "--no-container"], env), cwd=project)
        local_ref = env.get("AIBOX_RELEASE_SMOKE_LOCAL_CANDIDATE_REF", "")
        if local_ref:
            dockerfile = project / ".devcontainer/Dockerfile"
            expected = f"FROM ghcr.io/{REPOSITORY}:base-debian-runtime-v{version} AS aibox"
            content = dockerfile.read_text(encoding="utf-8")
            if content.count(expected) != 1:
                fail("generated LaTeX Dockerfile has no unique candidate FROM line")
            dockerfile.write_text(content.replace(expected, f"FROM {local_ref} AS aibox", 1),
                                  encoding="utf-8")
            build_command = ["/usr/bin/env", "DOCKER_BUILDKIT=0", "COMPOSE_DOCKER_CLI_BUILD=0"]
        else:
            build_command = []
        runner.run([*build_command, container_runtime, "compose", "-f", str(compose_file),
                    "build", name], cwd=project)
        runner.run([container_runtime, "compose", "-f", str(compose_file), "up", "-d", name,
                    f"{name}-latex-preview"], cwd=project)
        wait_until("latexmk", lambda: runner.capture(
            [container_runtime, "exec", "--user", "aibox", name, "which", "latexmk"]).strip())
        wait_until("LaTeX watcher", lambda: runner.capture(
            [container_runtime, "exec", "--user", "aibox", name, "which", "aibox-latex-watch"]).strip())
        wait_until("LaTeX preview health", lambda: urllib.request.urlopen(
            f"http://127.0.0.1:{port}/health", timeout=2).read())
        runner.run([container_runtime, "exec", "-d", "--user", "aibox", name,
                    "aibox-latex-watch", "overview"])

        def pdf_contains(expected: str) -> bool:
            """Return whether the current generated PDF contains *expected*."""
            output = runner.capture([container_runtime, "exec", name, "pdftotext",
                                     "/workspace/.latex-cache/overview/overview.pdf", "-"])
            return expected in output

        wait_until("first watched PDF revision", lambda: pdf_contains("WATCHREVISIONONE"))
        source.write_text(latex_document("WATCHREVISIONTWO"), encoding="utf-8")
        wait_until("second watched PDF revision", lambda: pdf_contains("WATCHREVISIONTWO"))
        served = urllib.request.urlopen(
            f"http://127.0.0.1:{port}/documents/overview/document.pdf", timeout=5).read()
        generated = (project / ".latex-cache/overview/overview.pdf").read_bytes()
        if not served.startswith(b"%PDF-") or served != generated:
            fail("LaTeX preview did not serve the watched PDF byte-for-byte")
        runner.run([container_runtime, "exec", name, "pgrep", "-f", "latexmk.*overview.tex"])
        result = evidence / "container-e2e/latex-lifecycle.json"
        result.write_text(json.dumps({"status": "passed", "revisions": 2, "preview_port": port},
                                     indent=2, sort_keys=True) + "\n", encoding="utf-8")
        seal_checkpoint(result)
        return result
    finally:
        if compose_file.exists():
            compose_cleanup(runner, container_runtime, compose_file)


def run_rootless_podman(runner: Runner, profile: Path, env: dict[str, str], candidate_bin: Path,
                        container_runtime: str, runtime: Path, evidence: Path, version: str) -> Path:
    """Build the infrastructure addon and prove rootless Podman readiness.

    Executing Podman inside a Docker-compatible development container requires
    the outer runtime to permit nested user namespaces. The restricted release
    gate deliberately does not widen that boundary. Instead this verifies each
    prerequisite controlled by the candidate image as the unprivileged user.
    """
    name = "host-gate-podman"
    project = runtime / name
    init_project(runner, profile, env, candidate_bin, project, name, [])
    pin_release_version(project / "aibox.toml", version)
    with (project / "aibox.toml").open("a", encoding="utf-8") as config:
        config.write("\n[addons.infrastructure.tools]\nopentofu = { enabled = false }\n"
                     "ansible = { enabled = false }\npacker = { enabled = false }\npodman = {}\n")
    compose_file = project / ".devcontainer/docker-compose.yml"
    try:
        prepare_project_build(runner, profile, env, candidate_bin, container_runtime,
                              project, version)
        runner.run([container_runtime, "compose", "-f", str(compose_file), "up", "-d", name], cwd=project)
        runner.run([container_runtime, "exec", "--user", "aibox", name, "podman", "--version"])
        runner.run([container_runtime, "exec", "--user", "aibox", name, "podman-compose", "--version"])
        for helper in ("/usr/bin/newuidmap", "/usr/bin/newgidmap", "/usr/bin/fuse-overlayfs",
                       "/usr/bin/slirp4netns"):
            runner.run([container_runtime, "exec", "--user", "aibox", name,
                        "/usr/bin/test", "-x", helper])
        for namespace_helper in ("/usr/bin/newuidmap", "/usr/bin/newgidmap"):
            runner.run([container_runtime, "exec", "--user", "aibox", name,
                        "/usr/bin/test", "-u", namespace_helper])
        subuid = runner.capture([container_runtime, "exec", "--user", "aibox", name,
                                 "/usr/bin/grep", "-Fx", "aibox:100000:65536", "/etc/subuid"])
        subgid = runner.capture([container_runtime, "exec", "--user", "aibox", name,
                                 "/usr/bin/grep", "-Fx", "aibox:100000:65536", "/etc/subgid"])
        config = runner.capture([container_runtime, "exec", "--user", "aibox", name,
                                 "/usr/bin/grep", "-Fx", 'cgroup_manager = "cgroupfs"',
                                 "/etc/containers/containers.conf"])
        result = evidence / "container-e2e/rootless-podman.json"
        result.write_text(json.dumps({
            "status": "passed",
            "rootless_readiness": True,
            "nested_runtime_executed": False,
            "execution_boundary": "outer runtime user-namespace privileges were not widened",
            "subuid": subuid.strip(), "subgid": subgid.strip(), "containers_conf": config.strip(),
        }, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        seal_checkpoint(result)
        return result
    finally:
        if compose_file.exists():
            compose_cleanup(runner, container_runtime, compose_file)


def run_gate(renderer: object | None = None) -> None:
    """Execute the release gate independently of its terminal presentation."""
    renderer = renderer or PlainRenderer()
    validation_started = time.monotonic()
    """Execute the complete fail-closed validation and publication sequence."""
    # Phase 1: constrain the platform and validate the immutable input envelope.
    if len(sys.argv) != 2:
        fail("expected exactly one run-directory argument")
    dry_run = dry_run_enabled(os.environ.get("AIBOX_RELEASE_HOST_DRY_RUN"))
    reuse_cache = cache_reuse_enabled(os.environ.get("AIBOX_RELEASE_HOST_REUSE_CACHE"))
    if subprocess.run(["/usr/bin/uname", "-s"], check=True, capture_output=True, text=True).stdout.strip() != "Darwin":
        fail("this gate must run on macOS")

    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    run_dir = resolve_run_dir(sys.argv[1], project_root)
    validate_directory(run_dir)
    input_dir = run_dir / "input"
    validate_directory(input_dir)
    found = {item.name for item in input_dir.iterdir()}
    if found != EXPECTED_INPUTS:
        fail(f"unexpected input set: {sorted(found)}")
    for item in input_dir.iterdir():
        validate_regular(item)

    # The checksum allowlist prevents an input producer from adding an
    # unreviewed payload that later code might accidentally consume.
    checksums: dict[str, str] = {}
    for line in (input_dir / "checksums.sha256").read_text(encoding="utf-8").splitlines():
        digest, name = line.split(maxsplit=1)
        name = name.lstrip("*")
        if name in checksums or name not in {"provenance.json", "source.tar.gz"} or not re.fullmatch(r"[0-9a-f]{64}", digest):
            fail("checksums.sha256 contains an invalid entry")
        checksums[name] = digest
    if set(checksums) != {"provenance.json", "source.tar.gz"}:
        fail("checksums.sha256 must enumerate every immutable payload")
    for name, expected in checksums.items():
        if sha256(input_dir / name) != expected:
            fail(f"checksum mismatch: {name}")

    retry_evidence: Path | None = None
    retry_argument = os.environ.get("AIBOX_RELEASE_HOST_RETRY_FROM", "")
    if retry_argument:
        retry_run = resolve_run_dir(retry_argument, project_root)
        if retry_run == run_dir:
            fail("retry source must be a different prepared run")
        retry_input = retry_run / "input"
        retry_evidence_candidate = retry_run / "evidence"
        if not retry_input.is_dir() or not retry_evidence_candidate.is_dir():
            fail("retry source has no input/evidence trees")
        for name in ("checksums.sha256", "provenance.json", "source.tar.gz"):
            retry_item = retry_input / name
            if (not retry_item.is_file() or retry_item.is_symlink() or
                    sha256(retry_item) != sha256(input_dir / name)):
                fail(f"retry source candidate differs: {name}")
        retry_evidence = retry_evidence_candidate

    # Phase 2: bind the archive and changed-path selection to real Git objects.
    provenance = json.loads((input_dir / "provenance.json").read_text(encoding="utf-8"))
    expected_keys = {"schema_version", "version", "tag", "commit", "comparison_tag",
                     "comparison_commit", "changed_paths", "repository", "source_archive"}
    if set(provenance) != expected_keys or provenance["schema_version"] != 2:
        fail("provenance schema is not the reviewed v2 shape")
    version = provenance["version"]
    if provenance["tag"] != f"v{version}" or provenance["repository"] != REPOSITORY:
        fail("provenance release identity is invalid")
    if not run_dir.name.startswith(f"v{version}-"):
        fail("run ID does not match the provenance version")
    tag_commit = subprocess.run(
        ["/usr/bin/git", "-C", str(project_root), "rev-parse", f"{provenance['tag']}^{{commit}}"],
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    if tag_commit != provenance["commit"]:
        fail("release tag does not resolve to the checksummed provenance commit")
    changed_paths = provenance["changed_paths"]
    if (not isinstance(changed_paths, list) or not changed_paths or
            any(not isinstance(path, str) or not path or path.startswith("/") or
                "\x00" in path or "\n" in path or ".." in Path(path).parts
                for path in changed_paths)):
        fail("provenance changed_paths must be a non-empty string array")
    comparison_tag = provenance["comparison_tag"]
    comparison_commit = provenance["comparison_commit"]
    if comparison_tag or comparison_commit:
        if not comparison_tag or not comparison_commit:
            fail("comparison tag and commit must both be set")
        if (not VERSION_TAG.fullmatch(comparison_tag) or
                comparison_tag[1:].split(".", 1)[0] != version.split(".", 1)[0] or
                not re.fullmatch(r"[0-9a-f]{40,64}", comparison_commit)):
            fail("comparison release identity is invalid")
        resolved_comparison = subprocess.run(
            ["/usr/bin/git", "-C", str(project_root), "rev-parse", f"{comparison_tag}^{{commit}}"],
            check=True, capture_output=True, text=True,
        ).stdout.strip()
        if resolved_comparison != comparison_commit:
            fail("comparison tag does not resolve to the attested comparison commit")
        actual_changed_paths = subprocess.run(
            ["/usr/bin/git", "-C", str(project_root), "diff", "--name-only",
             comparison_commit, provenance["commit"]], check=True, capture_output=True, text=True,
        ).stdout.splitlines()
        if changed_paths != actual_changed_paths:
            fail("attested changed paths do not match the comparison diff")
    elif changed_paths != ["*"]:
        fail("missing comparison provenance must select every impact check")
    head_commit = subprocess.run(
        ["/usr/bin/git", "-C", str(project_root), "rev-parse", "HEAD"],
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    tag_reaches_head = subprocess.run(
        ["/usr/bin/git", "-C", str(project_root), "merge-base", "--is-ancestor",
         provenance["commit"], head_commit],
        check=False, capture_output=True, text=True,
    )
    if tag_reaches_head.returncode != 0:
        fail("host checkout HEAD must descend from the tagged candidate commit")
    control_plane_diff = subprocess.run(
        ["/usr/bin/git", "-C", str(project_root), "diff", "--name-only", "HEAD", "--",
         *TRUSTED_CONTROL_PATHS],
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    if control_plane_diff:
        fail("host release control-plane files must be clean in the current protected-branch checkout")

    # Phase 3: create fresh mutable runtime/evidence trees. Refusing preexisting
    # trees prevents a partial or previously published run from being resumed.
    runtime = run_dir / "runtime"
    evidence = run_dir / "evidence"
    if runtime.exists() or evidence.exists():
        fail("runtime/ and evidence/ must not exist before a gate run")
    runtime.mkdir(mode=0o700)
    evidence.mkdir(mode=0o700)
    for name in ("darwin-build", "darwin-smoke", "container-build", "container-e2e", "security", "publication"):
        (evidence / name).mkdir()

    # Reject links, devices, and traversal before extracting candidate source.
    source_root = runtime / "source"
    with tarfile.open(input_dir / "source.tar.gz", "r:gz") as archive:
        for member in archive.getmembers():
            target = (runtime / member.name).resolve()
            if runtime not in target.parents or member.issym() or member.islnk() or member.isdev():
                fail(f"unsafe source archive member: {member.name}")
        archive.extractall(runtime, filter="data")

    # Phase 4: isolate mutable tool state and deny candidate access to owner
    # credentials, Keychain services, Git metadata, and immutable inputs.
    original_home = Path.home()
    home = runtime / "home"
    docker_config = runtime / "docker-config"
    cache_root = original_home / "Library/Caches/aibox-host-gates"
    cargo_home = cache_root / "cargo-home"
    cargo_target_dir = cache_root / "cargo-target" / provenance["commit"]
    home.mkdir(mode=0o700)
    docker_config.mkdir(mode=0o700)
    cache_root.mkdir(mode=0o700, parents=True, exist_ok=True)
    cargo_home.mkdir(mode=0o700, exist_ok=True)
    cargo_target_dir.mkdir(mode=0o700, parents=True, exist_ok=True)
    docker_cli_plugin_sources = stage_docker_cli_plugins(original_home, docker_config)
    fixed_env = {
        "PATH": f"{original_home}/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin",
        "HOME": str(home), "TMPDIR": str(runtime / "tmp"),
        "DOCKER_CONFIG": str(docker_config), "GH_CONFIG_DIR": str(runtime / "gh-config"),
        "DOCKER_BUILDKIT": "1", "COMPOSE_DOCKER_CLI_BUILD": "1",
        "CARGO_HOME": str(cargo_home), "CARGO_TARGET_DIR": str(cargo_target_dir),
        "RUSTUP_HOME": str(original_home / ".rustup"),
        "CARGO_NET_OFFLINE": "true", "RUST_BACKTRACE": "1",
        "AIBOX_RELEASE_HOST_OFFLINE": "1",
        "AIBOX_RELEASE_HOST_REUSE_CACHE": "1" if reuse_cache else "0",
        "AIBOX_ADDONS_DIR": str(source_root / "addons"),
    }
    (runtime / "tmp").mkdir()
    profile = runtime / "credential-free.sb"
    profile.write_text(
        '(version 1)\n(allow default)\n'
        f'(deny file-read-data (subpath "{original_home}/.config/gh") '
        f'(subpath "{original_home}/.docker") (subpath "{original_home}/.ssh") '
        f'(subpath "{original_home}/Library/Keychains"))\n'
        f'(deny file-write* (subpath "{script_dir}") (subpath "{project_root}/.git") '
        f'(subpath "{input_dir}"))\n'
        '(deny mach-lookup (global-name "com.apple.securityd") '
        '(global-name "com.apple.securityd.xpc"))\n', encoding="utf-8"
    )
    runner = Runner(evidence, fixed_env, renderer=renderer)
    runner._step("passed", "Validate immutable candidate", time.monotonic() - validation_started)
    renderer.emit(PresentationEvent("metadata", text=json.dumps({"commit": provenance["commit"]})))
    prerequisites_started = time.monotonic()
    runner._step("running", "Verify isolated prerequisites", 0.0)
    prerequisite_hints = {
        "cargo": "install Rust with rustup from https://rustup.rs",
        "rustc": "install Rust with rustup from https://rustup.rs",
        "syft": "install Syft with: brew install syft",
        "grype": "install Grype with: brew install grype",
        "sandbox-exec": "sandbox-exec must be available at /usr/bin/sandbox-exec on macOS",
    }
    for tool, hint in prerequisite_hints.items():
        if shutil.which(tool, path=fixed_env["PATH"]) is None:
            fail(f"required host prerequisite is missing: {tool}; {hint}")
    container_runtime = select_container_runtime(fixed_env)
    renderer.emit(PresentationEvent("metadata", text=json.dumps({"container_runtime": container_runtime})))
    compose = subprocess.run(
        [container_runtime, "compose", "version"], env=fixed_env,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
    )
    if compose.returncode != 0:
        hint = ("enable Docker Compose in OrbStack or Docker Desktop" if container_runtime == "docker"
                else "install a Podman Compose provider, for example: brew install podman-compose")
        fail(f"{container_runtime} is responsive but its Compose command is unavailable; {hint}")
    if container_runtime == "docker":
        buildx = subprocess.run(
            ["docker", "buildx", "version"], env=fixed_env,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
        )
        if buildx.returncode != 0:
            fail("the Docker-compatible runtime cannot load its Buildx plugin from the isolated config; ensure OrbStack or Docker Desktop provides docker-buildx")
    runner._step("passed", "Verify isolated prerequisites", time.monotonic() - prerequisites_started)
    (evidence / "darwin-build/toolchain.json").write_text(json.dumps({
        "python": sys.version, "python_executable": sys.executable,
        "python_requirement": "3.14.6", "uv": os.environ["AIBOX_HOST_GATE_UV_BIN"],
        "textual": TEXTUAL_VERSION,
        "ui_lock": str(script_dir / "release-host-ui.lock"),
        "ui_lock_sha256": sha256(script_dir / "release-host-ui.lock"),
        "ui_dependencies": locked_ui_dependencies(script_dir / "release-host-ui.lock"),
        "uv_cache_dir": os.environ["UV_CACHE_DIR"],
        "uv_python_install_dir": os.environ["UV_PYTHON_INSTALL_DIR"],
        "cargo_home": fixed_env["CARGO_HOME"],
        "cargo_target_dir": fixed_env["CARGO_TARGET_DIR"],
        "cargo_cache_scope": provenance["commit"],
        "rustup_home": fixed_env["RUSTUP_HOME"],
        "home": fixed_env["HOME"], "docker_config": fixed_env["DOCKER_CONFIG"],
        "docker_buildkit": fixed_env["DOCKER_BUILDKIT"],
        "container_runtime": container_runtime,
        "container_cache_policy": "reuse content-addressed layers" if reuse_cache else "force downstream rebuilds",
        "retry_evidence": str(retry_evidence) if retry_evidence else None,
        "docker_cli_plugin_sources": docker_cli_plugin_sources,
        "gh_config_dir": fixed_env["GH_CONFIG_DIR"],
    }, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    # Phase 5: fetch the exact lockfile graph into the credential-free gate
    # cache, then build offline. This avoids depending on the owner's cache
    # without granting candidate build scripts network access.
    fetch_env = {**fixed_env, "CARGO_NET_OFFLINE": "false"}
    runner.run(sandboxed(profile, [
        "cargo", "fetch", "--locked", "--manifest-path", str(source_root / "cli/Cargo.toml"),
        "--target", "aarch64-apple-darwin", "--target", "x86_64-apple-darwin",
    ], fetch_env), cwd=source_root, label="Fetch locked Rust dependencies")

    # Build both Darwin archives and natively smoke the host target.
    build_script = source_root / "scripts/build-macos.sh"
    runner.run(sandboxed(profile, [str(build_script), version], fixed_env), cwd=source_root,
               label="Build Darwin release artifacts")
    artifacts = sorted((source_root / "dist").glob(f"aibox-v{version}-*-apple-darwin.tar.gz*"))
    if len(artifacts) != 4:
        fail("Darwin build did not produce both archives and checksums")
    for artifact in artifacts:
        shutil.copy2(artifact, evidence / "darwin-build" / artifact.name)

    machine = subprocess.run(["/usr/bin/uname", "-m"], check=True, capture_output=True, text=True).stdout.strip()
    target = "aarch64-apple-darwin" if machine == "arm64" else "x86_64-apple-darwin"
    candidate_bin = cargo_target_dir / f"{target}/release/aibox"
    runner.run(sandboxed(profile, [str(candidate_bin), "--version"], fixed_env), cwd=source_root,
               label="Smoke native Darwin binary")
    (evidence / "darwin-smoke/complete.json").write_text(
        json.dumps({"target": target, "binary": str(candidate_bin), "status": "passed"}, indent=2) + "\n",
        encoding="utf-8",
    )

    # Phase 6: build the candidate images, then produce supply-chain evidence.
    foundation_image = f"ghcr.io/{REPOSITORY}:base-debian-foundation-v{version}"
    runtime_image = f"ghcr.io/{REPOSITORY}:base-debian-runtime-v{version}"
    latest_image = f"ghcr.io/{REPOSITORY}:base-debian-runtime-latest"
    build_args = ["--build-arg", f"AIBOX_IMAGE_SOURCE_SHA={provenance['commit']}",
                  "--build-arg", f"AIBOX_FOUNDATION_SOURCE_SHA={provenance['commit']}",
                  "--build-arg", f"AIBOX_RUNTIME_SOURCE_SHA={provenance['commit']}",
                  "--build-arg", f"AIBOX_IMAGE_BUILD_VERSION={version}"]
    runner.run([container_runtime, "build", "--target", "foundation", "--tag", foundation_image,
                *build_args,
                "--file", str(source_root / "images/base-debian/Dockerfile"),
                str(source_root / "images/base-debian")], label="Build foundation image")
    runner.run([container_runtime, "build", "--target", "runtime", "--tag", runtime_image,
                "--tag", latest_image, "--file", str(source_root / "images/base-debian/Dockerfile"),
                *build_args,
                str(source_root / "images/base-debian")], label="Build runtime image")
    smoke_runtime_image = ""
    if container_runtime == "docker":
        # OrbStack can inspect an attested BuildKit manifest but its classic
        # daemon-local builder cannot consume that manifest as a base image.
        # Produce a cached, single-manifest alias solely for the unpublished
        # downstream lifecycle; the publication candidate above is untouched.
        smoke_runtime_image = f"aibox-release-smoke-base:{provenance['commit'][:12]}"
        runner.run([
            container_runtime, "build", "--provenance=false", "--target", "runtime",
            "--tag", smoke_runtime_image, "--file", str(source_root / "images/base-debian/Dockerfile"),
            *build_args, str(source_root / "images/base-debian"),
        ], label="Prepare local runtime image for smoke")
        fixed_env["AIBOX_RELEASE_SMOKE_LOCAL_CANDIDATE_REF"] = smoke_runtime_image
    runner.run([container_runtime, "image", "inspect", runtime_image],
               output=evidence / "container-build/image-inspect.json", label="Inspect runtime image")
    scanner_image = runtime_image if container_runtime == "docker" else f"podman:{runtime_image}"
    runner.run(["syft", scanner_image, "-o", "cyclonedx-json"],
               output=evidence / "security/image-sbom.cdx.json", label="Generate image SBOM")
    vulnerability_report = evidence / "security/vulnerability-scan.json"
    try:
        runner.run(["grype", scanner_image, "-o", "json"],
                   output=vulnerability_report, label="Scan image vulnerabilities")
    except subprocess.CalledProcessError as error:
        fail(f"Grype scan failed with exit code {error.returncode}; see {runner.log.parent / 'command-results.log'}")
    vulnerability_summary = grype_policy_summary(vulnerability_report)
    vulnerability_policy = evidence / "security/vulnerability-policy.json"
    vulnerability_policy.write_text(
        json.dumps(vulnerability_summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print_grype_policy_summary(vulnerability_summary, renderer)
    if vulnerability_summary["actionable_advisories"]:
        runner._step("failed", "Vulnerability policy", 0.0)
        fail(
            f"Grype found {vulnerability_summary['actionable_advisories']} fixable High/Critical advisories; "
            f"policy report: {vulnerability_policy}"
        )
    runner._step("warned" if vulnerability_summary["no_fix_advisories"] else "passed",
                 "Vulnerability policy", 0.0)

    # The canonical runtime smoke is mandatory for every candidate, independent
    # of the changed-path-selected expensive checks below.
    smoke_env = dict(fixed_env)
    smoke_env.update({
        "AIBOX_RELEASE_SMOKE_BIN": str(candidate_bin),
        "AIBOX_RELEASE_SMOKE_DIR": str(evidence / "container-e2e"),
        "AIBOX_RELEASE_SMOKE_PROJECT_DIR": str(runtime / "smoke-project"),
        "AIBOX_RELEASE_SMOKE_CONTAINER": f"aibox-host-gate-{run_dir.name.lower()}",
        "AIBOX_RELEASE_SMOKE_TIER": "full",
        # The versioned image intentionally exists only in the local runtime
        # until publication. Keep the generated GHCR reference unchanged but
        # make Docker/OrbStack resolve it from their daemon-local image store.
        "AIBOX_RELEASE_SMOKE_LOCAL_CANDIDATE_IMAGE": "1",
    })
    if smoke_runtime_image:
        smoke_env["AIBOX_RELEASE_SMOKE_LOCAL_CANDIDATE_REF"] = smoke_runtime_image
    runner.run(sandboxed(profile, [str(source_root / "scripts/release-runtime-smoke.sh"), version], smoke_env),
               cwd=source_root, label="Run candidate container lifecycle")

    # Phase 7: execute only affected expensive surfaces, while recording a
    # complete selected/skipped partition for publisher verification.
    selected_checks = select_impact_checks(changed_paths)
    coverage_path = evidence / "container-e2e/impact-selection.json"
    coverage = {
        "comparison_tag": comparison_tag or None,
        "comparison_commit": comparison_commit or None,
        "changed_paths": changed_paths,
        "selected": selected_checks,
        "skipped": {
            check: "no attested changed path affects this surface"
            for check in sorted(ALL_IMPACT_CHECKS - set(selected_checks))
        },
    }
    coverage_path.write_text(json.dumps(coverage, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    for check, reason in coverage["skipped"].items():
        runner._step("skipped", check, 0.0)
    if not smoke_runtime_image:
        runner._step("skipped", "Prepare local runtime image for smoke", 0.0)
    renderer.emit(PresentationEvent("plan"))
    conditional_evidence: list[Path] = []
    for group in ADDON_GROUPS:
        if group in selected_checks:
            reused = reuse_checkpoint(retry_evidence, evidence, group) if retry_evidence else None
            if reused:
                conditional_evidence.append(reused)
                runner._step("passed", f"{group} (reused candidate-bound checkpoint)", 0.0)
                continue
            started = time.monotonic()
            runner._step("running", group, 0.0)
            runner.output_task = group
            try:
                conditional_evidence.append(run_addon_group(
                    runner, profile, fixed_env, candidate_bin, container_runtime,
                    runtime, evidence, version, group))
            except BaseException:
                runner._step("failed", group, time.monotonic() - started)
                raise
            finally:
                runner.output_task = None
            runner._step("passed", group, time.monotonic() - started)
    if "latex-lifecycle" in selected_checks:
        reused = reuse_checkpoint(retry_evidence, evidence, "latex-lifecycle") if retry_evidence else None
        if reused:
            conditional_evidence.append(reused)
            runner._step("passed", "latex-lifecycle (reused candidate-bound checkpoint)", 0.0)
        else:
            preview_port = 18000 + int(provenance["commit"][:8], 16) % 1000
            started = time.monotonic()
            runner._step("running", "latex-lifecycle", 0.0)
            runner.output_task = "latex-lifecycle"
            try:
                conditional_evidence.append(run_latex_lifecycle(
                    runner, profile, fixed_env, candidate_bin, container_runtime,
                    runtime, evidence, version, preview_port))
            except BaseException:
                runner._step("failed", "latex-lifecycle", time.monotonic() - started)
                raise
            finally:
                runner.output_task = None
            runner._step("passed", "latex-lifecycle", time.monotonic() - started)
    if "rootless-podman" in selected_checks:
        reused = reuse_checkpoint(retry_evidence, evidence, "rootless-podman") if retry_evidence else None
        if reused:
            conditional_evidence.append(reused)
            runner._step("passed", "rootless-podman (reused candidate-bound checkpoint)", 0.0)
        else:
            started = time.monotonic()
            runner._step("running", "rootless-podman", 0.0)
            runner.output_task = "rootless-podman"
            try:
                conditional_evidence.append(run_rootless_podman(
                    runner, profile, fixed_env, candidate_bin, container_runtime,
                    runtime, evidence, version))
            except BaseException:
                runner._step("failed", "rootless-podman", time.monotonic() - started)
                raise
            finally:
                runner.output_task = None
            runner._step("passed", "rootless-podman", time.monotonic() - started)

    # Phase 8: assemble a fixed, checksummed manifest. When publication is
    # enabled it receives no arbitrary artifact list; dry-run stops after the
    # same manifest is complete and prints the separate promotion command.
    publisher = script_dir / "release-host-publish.sh"
    publisher_command = [str(publisher), str(run_dir)]
    if not dry_run:
        rendered_publisher = shlex.join(publisher_command)
        with runner.log.open("a", encoding="utf-8") as log:
            log.write(rendered_publisher + "\n")
            log.flush()
        runner._emit(PresentationEvent("output", task="Publication", text=f"+ {rendered_publisher}\n"))

    required_paths = [
        evidence / "darwin-smoke/complete.json",
        evidence / "container-build/image-inspect.json",
        evidence / "container-e2e/metadata.env",
        coverage_path,
        evidence / "security/image-sbom.cdx.json",
        evidence / "security/vulnerability-scan.json",
        evidence / "security/vulnerability-policy.json",
        evidence / "commands.log",
        evidence / "command-results.log",
        evidence / "steps.log",
    ]
    if smoke_runtime_image:
        required_paths.append(evidence / "container-e2e/local-candidate-substitution.env")
    for required in required_paths:
        if not required.exists() or (required.is_file() and required.stat().st_size == 0):
            fail(f"required evidence is missing or empty: {required.relative_to(run_dir)}")

    manifest_started = time.monotonic()
    runner._step("running", "Assemble evidence manifest", 0.0)
    # Finalize every step-log state before hashing it. Publication writes only
    # to evidence/publication/, which is deliberately outside this manifest.
    runner._step("passed", "Assemble evidence manifest", time.monotonic() - manifest_started)
    if dry_run:
        runner._step("passed", "Dry-run completion", 0.0)
        runner._step("skipped", "Publication", 0.0)
    else:
        runner._step("running", "Publication", 0.0)
        runner._step("skipped", "Dry-run completion", 0.0)
    manifest = {
        "schema_version": 2, "repository": REPOSITORY, "version": version,
        "tag": provenance["tag"], "commit": provenance["commit"],
        "container_runtime": container_runtime,
        "images": [foundation_image, runtime_image, latest_image],
        "artifacts": [
            {"path": str(path.relative_to(run_dir)), "sha256": sha256(path)}
            for path in sorted((evidence / "darwin-build").glob(f"aibox-v{version}-*-apple-darwin.tar.gz*"))
        ],
        "evidence": [
            {"path": str(path.relative_to(run_dir)), "sha256": sha256(path)}
            for path in required_paths + conditional_evidence
        ],
    }
    manifest_path = evidence / "release-manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    (evidence / "release-manifest.sha256").write_text(f"{sha256(manifest_path)}  release-manifest.json\n", encoding="utf-8")

    if dry_run:
        runner._emit(PresentationEvent(
            "output", task="Dry-run completion",
            text="release-host validation complete; publication was not invoked\n"
                 f"To publish this verified run: {shlex.join(publisher_command)}\n",
        ))
        return

    publisher_process = subprocess.Popen(
        publisher_command, text=True, stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
    )
    assert publisher_process.stdout is not None
    for line in publisher_process.stdout:
        renderer.emit(PresentationEvent("output", task="Publication", text=line))
    publisher_returncode = publisher_process.wait()
    if publisher_returncode:
        renderer.emit(PresentationEvent("failed", task="Publication", state="failed"))
        raise subprocess.CalledProcessError(publisher_returncode, publisher_command)
    renderer.emit(PresentationEvent("passed", task="Publication", state="passed"))


def main() -> None:
    """Select Textual or plain rendering before candidate execution starts."""
    mode = parse_ui_mode(os.environ.get("AIBOX_RELEASE_HOST_UI"))
    dry_run = dry_run_enabled(os.environ.get("AIBOX_RELEASE_HOST_DRY_RUN"))
    use_textual = mode == "textual" or (mode == "auto" and textual_terminal_available())
    if use_textual:
        try:
            raise SystemExit(run_textual_dashboard(sys.argv[1], dry_run))
        except ImportError as error:
            if mode == "textual":
                fail(f"Textual UI could not start: {error}")
            print(f"release-host gate: Textual UI unavailable; continuing in plain mode: {error}", file=sys.stderr)
    run_gate(PlainRenderer())


if __name__ == "__main__":
    main()
