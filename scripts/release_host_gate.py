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
import re
import shlex
import shutil
import stat
import subprocess
import sys
import tarfile
import time
import urllib.request

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
)

ADDON_GROUPS = {
    "addon-languages": ["docs-hugo", "docs-mdbook", "go", "go-quality", "go-release", "node", "rust", "typst"],
    "addon-platforms": ["cloud-aws", "cloud-gcp", "cloudflare", "infrastructure", "kubernetes"],
    "addon-tools": ["ai-claude", "ai-opencode", "preview-archive", "supply-chain"],
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


def sha256(path: Path) -> str:
    """Return the streaming SHA-256 digest of *path* as lowercase hex."""
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


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

    def __init__(self, evidence: Path, env: dict[str, str]) -> None:
        """Bind command logs to *evidence* and freeze the subprocess environment."""
        self.log = evidence / "commands.log"
        self.env = env

    def run(self, command: list[str], *, cwd: Path | None = None, output: Path | None = None) -> None:
        """Run *command*, optionally writing stdout as a binary evidence file."""
        rendered = shlex.join(command)
        print(f"+ {rendered}", flush=True)
        with self.log.open("a", encoding="utf-8") as log:
            log.write(rendered + "\n")
        result_log = self.log.parent / "command-results.log"
        if output is None:
            with result_log.open("a", encoding="utf-8") as results:
                completed = subprocess.run(command, cwd=cwd, env=self.env, text=True,
                                           stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
                results.write(f"$ {rendered}\n{completed.stdout}\nexit={completed.returncode}\n")
                print(completed.stdout, end="")
                completed.check_returncode()
        else:
            with output.open("wb") as stream, result_log.open("ab") as results:
                completed = subprocess.run(command, cwd=cwd, env=self.env, stdout=stream,
                                           stderr=results)
                results.write(f"\n$ {rendered}\nexit={completed.returncode}\n".encode())
                completed.check_returncode()

    def capture(self, command: list[str], *, cwd: Path | None = None) -> str:
        """Run *command*, log it, and return its combined textual output."""
        rendered = shlex.join(command)
        print(f"+ {rendered}", flush=True)
        with self.log.open("a", encoding="utf-8") as log:
            log.write(rendered + "\n")
        completed = subprocess.run(command, cwd=cwd, env=self.env, text=True,
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        with (self.log.parent / "command-results.log").open("a", encoding="utf-8") as results:
            results.write(f"$ {rendered}\n{completed.stdout}\nexit={completed.returncode}\n")
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


def stage_docker_compose_plugin(owner_home: Path, docker_config: Path) -> str | None:
    """Copy only a trusted Compose plugin into the credential-empty config.

    Docker CLI plugins normally live beside ``config.json``. Replacing
    ``DOCKER_CONFIG`` protects registry credentials but would also hide
    OrbStack's or Docker Desktop's Compose plugin. Staging the executable alone
    restores the command without copying any owner configuration or secrets.
    """
    candidates = (
        owner_home / ".docker/cli-plugins/docker-compose",
        Path("/Applications/OrbStack.app/Contents/MacOS/xbin/docker-compose"),
        Path("/Applications/Docker.app/Contents/Resources/cli-plugins/docker-compose"),
    )
    for candidate in candidates:
        if not candidate.is_file() or not os.access(candidate, os.X_OK):
            continue
        info = candidate.stat()
        if info.st_uid not in {0, os.getuid()} or info.st_mode & 0o022:
            continue
        destination = docker_config / "cli-plugins/docker-compose"
        destination.parent.mkdir(mode=0o700)
        shutil.copy2(candidate, destination)
        destination.chmod(0o500)
        return str(candidate)
    return None


def pin_release_version(config_path: Path, version: str) -> None:
    """Pin a generated project to the locally built candidate image version."""
    config = config_path.read_text(encoding="utf-8")
    updated, count = re.subn(r'(?m)^release_version = ".*"', f'release_version = "{version}"', config, count=1)
    if count != 1:
        fail(f"could not pin release version in {config_path}")
    config_path.write_text(updated, encoding="utf-8")


def compose_cleanup(runner: Runner, container_runtime: str, compose_file: Path) -> None:
    """Remove the probe's containers, volumes, orphans, and derived local image.

    Cleanup is deliberately fail-closed: callers invoke this from ``finally``
    blocks, and a cleanup error prevents publication just like a failed probe.
    """
    runner.run([container_runtime, "compose", "-f", str(compose_file), "down", "-v",
                "--remove-orphans", "--rmi", "local"], cwd=compose_file.parent.parent)


def init_project(runner: Runner, profile: Path, env: dict[str, str], candidate_bin: Path,
                 project: Path, name: str, addons: list[str]) -> None:
    """Create an isolated downstream project with the candidate Darwin binary."""
    project.mkdir()
    command = [str(candidate_bin), "--yes", "init", name, "--base", "debian",
               "--processkit-version", "unset", "--no-container"]
    if addons:
        command.extend(["--addon", *addons])
    runner.run(sandboxed(profile, command, env), cwd=project)


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
        runner.run(sandboxed(profile, [str(candidate_bin), "--yes", "apply", "--no-cache"], env), cwd=project)
        marker = evidence / "container-e2e" / f"{group}.json"
        marker.write_text(json.dumps({"status": "passed", "addons": ADDON_GROUPS[group]},
                                     indent=2, sort_keys=True) + "\n", encoding="utf-8")
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
        runner.run([container_runtime, "compose", "-f", str(compose_file), "build", name], cwd=project)
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
        return result
    finally:
        if compose_file.exists():
            compose_cleanup(runner, container_runtime, compose_file)


def run_rootless_podman(runner: Runner, profile: Path, env: dict[str, str], candidate_bin: Path,
                        container_runtime: str, runtime: Path, evidence: Path, version: str) -> Path:
    """Build the infrastructure addon and prove nested Podman is rootless."""
    name = "host-gate-podman"
    project = runtime / name
    init_project(runner, profile, env, candidate_bin, project, name, [])
    pin_release_version(project / "aibox.toml", version)
    with (project / "aibox.toml").open("a", encoding="utf-8") as config:
        config.write("\n[addons.infrastructure.tools]\nopentofu = { enabled = false }\n"
                     "ansible = { enabled = false }\npacker = { enabled = false }\npodman = {}\n")
    compose_file = project / ".devcontainer/docker-compose.yml"
    try:
        runner.run(sandboxed(profile, [str(candidate_bin), "--yes", "apply", "--no-cache"], env), cwd=project)
        runner.run([container_runtime, "compose", "-f", str(compose_file), "up", "-d", name], cwd=project)
        runner.run([container_runtime, "exec", "--user", "aibox", name, "podman", "--version"])
        runner.run([container_runtime, "exec", "--user", "aibox", name, "podman-compose", "--version"])
        rootless = runner.capture([container_runtime, "exec", "--user", "aibox", name, "podman",
                                   "info", "--format", "{{.Host.Security.Rootless}}"])
        if rootless.strip().lower() != "true":
            fail("nested Podman did not report a rootless runtime")
        result = evidence / "container-e2e/rootless-podman.json"
        result.write_text(json.dumps({"status": "passed", "rootless": True}, indent=2) + "\n",
                          encoding="utf-8")
        return result
    finally:
        if compose_file.exists():
            compose_cleanup(runner, container_runtime, compose_file)


def main() -> None:
    """Execute the complete fail-closed validation and publication sequence."""
    # Phase 1: constrain the platform and validate the immutable input envelope.
    if len(sys.argv) != 2:
        fail("expected exactly one run-directory argument")
    dry_run = dry_run_enabled(os.environ.get("AIBOX_RELEASE_HOST_DRY_RUN"))
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
    if head_commit != provenance["commit"]:
        fail("host checkout HEAD must be the tagged candidate commit")
    control_plane_diff = subprocess.run(
        ["/usr/bin/git", "-C", str(project_root), "diff", "--name-only", "HEAD", "--",
         *TRUSTED_CONTROL_PATHS],
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    if control_plane_diff:
        fail("host release control-plane files must match the tagged candidate commit")

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
    home = runtime / "home"
    docker_config = runtime / "docker-config"
    cargo_home = runtime / "cargo-home"
    home.mkdir(mode=0o700)
    docker_config.mkdir(mode=0o700)
    cargo_home.mkdir(mode=0o700)
    original_home = Path.home()
    compose_plugin_source = stage_docker_compose_plugin(original_home, docker_config)
    fixed_env = {
        "PATH": f"{original_home}/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin",
        "HOME": str(home), "TMPDIR": str(runtime / "tmp"),
        "DOCKER_CONFIG": str(docker_config), "GH_CONFIG_DIR": str(runtime / "gh-config"),
        "DOCKER_BUILDKIT": "1", "COMPOSE_DOCKER_CLI_BUILD": "1",
        "CARGO_HOME": str(cargo_home), "RUSTUP_HOME": str(original_home / ".rustup"),
        "CARGO_NET_OFFLINE": "true", "RUST_BACKTRACE": "1",
        "AIBOX_RELEASE_HOST_OFFLINE": "1",
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
    runner = Runner(evidence, fixed_env)
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
    compose = subprocess.run(
        [container_runtime, "compose", "version"], env=fixed_env,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
    )
    if compose.returncode != 0:
        hint = ("enable Docker Compose in OrbStack or Docker Desktop" if container_runtime == "docker"
                else "install a Podman Compose provider, for example: brew install podman-compose")
        fail(f"{container_runtime} is responsive but its Compose command is unavailable; {hint}")
    (evidence / "darwin-build/toolchain.json").write_text(json.dumps({
        "python": sys.version, "python_executable": sys.executable,
        "python_requirement": "3.14.6", "uv": os.environ["AIBOX_HOST_GATE_UV_BIN"],
        "uv_cache_dir": os.environ["UV_CACHE_DIR"],
        "uv_python_install_dir": os.environ["UV_PYTHON_INSTALL_DIR"],
        "cargo_home": fixed_env["CARGO_HOME"], "rustup_home": fixed_env["RUSTUP_HOME"],
        "home": fixed_env["HOME"], "docker_config": fixed_env["DOCKER_CONFIG"],
        "docker_buildkit": fixed_env["DOCKER_BUILDKIT"],
        "container_runtime": container_runtime,
        "compose_plugin_source": compose_plugin_source,
        "gh_config_dir": fixed_env["GH_CONFIG_DIR"],
    }, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    # Phase 5: fetch the exact lockfile graph into the credential-free gate
    # cache, then build offline. This avoids depending on the owner's cache
    # without granting candidate build scripts network access.
    fetch_env = {**fixed_env, "CARGO_NET_OFFLINE": "false"}
    runner.run(sandboxed(profile, [
        "cargo", "fetch", "--locked", "--manifest-path", str(source_root / "cli/Cargo.toml"),
        "--target", "aarch64-apple-darwin", "--target", "x86_64-apple-darwin",
    ], fetch_env), cwd=source_root)

    # Build both Darwin archives and natively smoke the host target.
    build_script = source_root / "scripts/build-macos.sh"
    runner.run(sandboxed(profile, [str(build_script), version], fixed_env), cwd=source_root)
    artifacts = sorted((source_root / "dist").glob(f"aibox-v{version}-*-apple-darwin.tar.gz*"))
    if len(artifacts) != 4:
        fail("Darwin build did not produce both archives and checksums")
    for artifact in artifacts:
        shutil.copy2(artifact, evidence / "darwin-build" / artifact.name)

    machine = subprocess.run(["/usr/bin/uname", "-m"], check=True, capture_output=True, text=True).stdout.strip()
    target = "aarch64-apple-darwin" if machine == "arm64" else "x86_64-apple-darwin"
    candidate_bin = source_root / f"cli/target/{target}/release/aibox"
    runner.run(sandboxed(profile, [str(candidate_bin), "--version"], fixed_env), cwd=source_root)
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
                str(source_root / "images/base-debian")])
    runner.run([container_runtime, "build", "--target", "runtime", "--tag", runtime_image,
                "--tag", latest_image, "--file", str(source_root / "images/base-debian/Dockerfile"),
                *build_args,
                str(source_root / "images/base-debian")])
    runner.run([container_runtime, "image", "inspect", runtime_image], output=evidence / "container-build/image-inspect.json")
    scanner_image = runtime_image if container_runtime == "docker" else f"podman:{runtime_image}"
    runner.run(["syft", scanner_image, "-o", "cyclonedx-json"], output=evidence / "security/image-sbom.cdx.json")
    runner.run(["grype", scanner_image, "--fail-on", "high", "-o", "json"], output=evidence / "security/vulnerability-scan.json")

    # The canonical runtime smoke is mandatory for every candidate, independent
    # of the changed-path-selected expensive checks below.
    smoke_env = dict(fixed_env)
    smoke_env.update({
        "AIBOX_RELEASE_SMOKE_BIN": str(candidate_bin),
        "AIBOX_RELEASE_SMOKE_DIR": str(evidence / "container-e2e"),
        "AIBOX_RELEASE_SMOKE_PROJECT_DIR": str(runtime / "smoke-project"),
        "AIBOX_RELEASE_SMOKE_CONTAINER": f"aibox-host-gate-{run_dir.name.lower()}",
        "AIBOX_RELEASE_SMOKE_TIER": "full",
    })
    runner.run(sandboxed(profile, [str(source_root / "scripts/release-runtime-smoke.sh"), version], smoke_env), cwd=source_root)

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
    conditional_evidence: list[Path] = []
    for group in ADDON_GROUPS:
        if group in selected_checks:
            conditional_evidence.append(run_addon_group(
                runner, profile, fixed_env, candidate_bin, container_runtime,
                runtime, evidence, version, group))
    if "latex-lifecycle" in selected_checks:
        preview_port = 18000 + int(provenance["commit"][:8], 16) % 1000
        conditional_evidence.append(run_latex_lifecycle(
            runner, profile, fixed_env, candidate_bin, container_runtime,
            runtime, evidence, version, preview_port))
    if "rootless-podman" in selected_checks:
        conditional_evidence.append(run_rootless_podman(
            runner, profile, fixed_env, candidate_bin, container_runtime, runtime, evidence, version))

    # Phase 8: assemble a fixed, checksummed manifest. When publication is
    # enabled it receives no arbitrary artifact list; dry-run stops after the
    # same manifest is complete and prints the separate promotion command.
    publisher = script_dir / "release-host-publish.sh"
    publisher_command = [str(publisher), str(run_dir)]
    if not dry_run:
        rendered_publisher = shlex.join(publisher_command)
        print(f"+ {rendered_publisher}", flush=True)
        with runner.log.open("a", encoding="utf-8") as log:
            log.write(rendered_publisher + "\n")

    required_paths = [
        evidence / "darwin-smoke/complete.json",
        evidence / "container-build/image-inspect.json",
        evidence / "container-e2e/metadata.env",
        coverage_path,
        evidence / "security/image-sbom.cdx.json",
        evidence / "security/vulnerability-scan.json",
        evidence / "commands.log",
        evidence / "command-results.log",
    ]
    for required in required_paths:
        if not required.exists() or (required.is_file() and required.stat().st_size == 0):
            fail(f"required evidence is missing or empty: {required.relative_to(run_dir)}")

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
        print("release-host validation complete; publication was not invoked", flush=True)
        print(f"To publish this verified run: {shlex.join(publisher_command)}", flush=True)
        return

    subprocess.run(publisher_command, check=True)


if __name__ == "__main__":
    main()
