#!/usr/bin/env python3
"""Fail-closed macOS validation stage for an immutable aibox release input."""

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

REPOSITORY = "projectious-work/aibox"
EXPECTED_INPUTS = {"checksums.sha256", "provenance.json", "source.tar.gz"}
RUN_ID = re.compile(r"^v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z][0-9A-Za-z.-]*)?-[0-9]{8}T[0-9]{6}Z-[0-9a-f]{12}$")


def fail(message: str) -> "None":
    raise SystemExit(f"release-host gate: {message}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate_regular(path: Path) -> None:
    info = path.lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
        fail(f"input must be a regular single-link file: {path.name}")
    if info.st_mode & (stat.S_IWUSR | stat.S_IWGRP | stat.S_IWOTH):
        fail(f"immutable input is writable: {path.name}")


def validate_directory(path: Path) -> None:
    info = path.lstat()
    if not stat.S_ISDIR(info.st_mode) or path.is_symlink():
        fail(f"expected a real directory: {path}")
    if info.st_mode & (stat.S_IWGRP | stat.S_IWOTH):
        fail(f"directory has unsafe write permissions: {path}")


def resolve_run_dir(argument: str, project_root: Path) -> Path:
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
    def __init__(self, evidence: Path, env: dict[str, str]) -> None:
        self.log = evidence / "commands.log"
        self.env = env

    def run(self, command: list[str], *, cwd: Path | None = None, output: Path | None = None) -> None:
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


def sandboxed(profile: Path, command: list[str], env: dict[str, str]) -> list[str]:
    assignments = [f"{key}={value}" for key, value in sorted(env.items())]
    return ["/usr/bin/sandbox-exec", "-f", str(profile), "/usr/bin/env", "-i", *assignments, *command]


def main() -> None:
    if len(sys.argv) != 2:
        fail("expected exactly one run-directory argument")
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

    provenance = json.loads((input_dir / "provenance.json").read_text(encoding="utf-8"))
    expected_keys = {"schema_version", "version", "tag", "commit", "repository", "source_archive"}
    if set(provenance) != expected_keys or provenance["schema_version"] != 1:
        fail("provenance schema is not the reviewed v1 shape")
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
    head_commit = subprocess.run(
        ["/usr/bin/git", "-C", str(project_root), "rev-parse", "HEAD"],
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    tracked_status = subprocess.run(
        ["/usr/bin/git", "-C", str(project_root), "status", "--porcelain", "--untracked-files=no"],
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    if head_commit != provenance["commit"] or tracked_status:
        fail("host checkout must be the clean tagged candidate commit")

    runtime = run_dir / "runtime"
    evidence = run_dir / "evidence"
    if runtime.exists() or evidence.exists():
        fail("runtime/ and evidence/ must not exist before a gate run")
    runtime.mkdir(mode=0o700)
    evidence.mkdir(mode=0o700)
    for name in ("darwin-build", "darwin-smoke", "container-build", "container-e2e", "security", "publication"):
        (evidence / name).mkdir()

    source_root = runtime / "source"
    with tarfile.open(input_dir / "source.tar.gz", "r:gz") as archive:
        for member in archive.getmembers():
            target = (runtime / member.name).resolve()
            if runtime not in target.parents or member.issym() or member.islnk() or member.isdev():
                fail(f"unsafe source archive member: {member.name}")
        archive.extractall(runtime)

    home = runtime / "home"
    docker_config = runtime / "docker-config"
    home.mkdir(mode=0o700)
    docker_config.mkdir(mode=0o700)
    original_home = Path.home()
    fixed_env = {
        "PATH": f"{original_home}/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin",
        "HOME": str(home), "TMPDIR": str(runtime / "tmp"),
        "DOCKER_CONFIG": str(docker_config), "GH_CONFIG_DIR": str(runtime / "gh-config"),
        "CARGO_HOME": str(original_home / ".cargo"), "RUSTUP_HOME": str(original_home / ".rustup"),
        "CARGO_NET_OFFLINE": "true", "RUST_BACKTRACE": "1",
        "AIBOX_RELEASE_HOST_OFFLINE": "1",
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
    for tool in ("cargo", "rustc", "docker", "syft", "grype", "sandbox-exec"):
        if shutil.which(tool, path=fixed_env["PATH"]) is None:
            fail(f"required host prerequisite is missing: {tool}")
    (evidence / "darwin-build/toolchain.json").write_text(json.dumps({
        "python": sys.version, "python_executable": sys.executable,
        "python_requirement": "3.12.11", "uv": os.environ["AIBOX_HOST_GATE_UV_BIN"],
        "uv_cache_dir": os.environ["UV_CACHE_DIR"],
        "uv_python_install_dir": os.environ["UV_PYTHON_INSTALL_DIR"],
        "cargo_home": fixed_env["CARGO_HOME"], "rustup_home": fixed_env["RUSTUP_HOME"],
        "home": fixed_env["HOME"], "docker_config": fixed_env["DOCKER_CONFIG"],
        "gh_config_dir": fixed_env["GH_CONFIG_DIR"],
    }, indent=2, sort_keys=True) + "\n", encoding="utf-8")

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

    foundation_image = f"ghcr.io/{REPOSITORY}:base-debian-foundation-v{version}"
    runtime_image = f"ghcr.io/{REPOSITORY}:base-debian-runtime-v{version}"
    latest_image = f"ghcr.io/{REPOSITORY}:base-debian-runtime-latest"
    build_args = ["--build-arg", f"AIBOX_IMAGE_SOURCE_SHA={provenance['commit']}",
                  "--build-arg", f"AIBOX_FOUNDATION_SOURCE_SHA={provenance['commit']}",
                  "--build-arg", f"AIBOX_RUNTIME_SOURCE_SHA={provenance['commit']}",
                  "--build-arg", f"AIBOX_IMAGE_BUILD_VERSION={version}"]
    runner.run(["docker", "build", "--target", "foundation", "--tag", foundation_image,
                *build_args,
                "--file", str(source_root / "images/base-debian/Dockerfile"),
                str(source_root / "images/base-debian")])
    runner.run(["docker", "build", "--target", "runtime", "--tag", runtime_image,
                "--tag", latest_image, "--file", str(source_root / "images/base-debian/Dockerfile"),
                *build_args,
                str(source_root / "images/base-debian")])
    runner.run(["docker", "image", "inspect", runtime_image], output=evidence / "container-build/image-inspect.json")
    runner.run(["syft", runtime_image, "-o", "cyclonedx-json"], output=evidence / "security/image-sbom.cdx.json")
    runner.run(["grype", runtime_image, "--fail-on", "high", "-o", "json"], output=evidence / "security/vulnerability-scan.json")

    smoke_env = dict(fixed_env)
    smoke_env.update({
        "AIBOX_RELEASE_SMOKE_BIN": str(candidate_bin),
        "AIBOX_RELEASE_SMOKE_DIR": str(evidence / "container-e2e"),
        "AIBOX_RELEASE_SMOKE_PROJECT_DIR": str(runtime / "smoke-project"),
        "AIBOX_RELEASE_SMOKE_CONTAINER": f"aibox-host-gate-{run_dir.name.lower()}",
        "AIBOX_RELEASE_SMOKE_TIER": "full",
    })
    runner.run(sandboxed(profile, [str(source_root / "scripts/release-runtime-smoke.sh"), version], smoke_env), cwd=source_root)

    publisher = script_dir / "release-host-publish.sh"
    publisher_command = [str(publisher), str(run_dir)]
    rendered_publisher = shlex.join(publisher_command)
    print(f"+ {rendered_publisher}", flush=True)
    with runner.log.open("a", encoding="utf-8") as log:
        log.write(rendered_publisher + "\n")

    required_paths = [
        evidence / "darwin-smoke/complete.json",
        evidence / "container-build/image-inspect.json",
        evidence / "container-e2e/metadata.env",
        evidence / "security/image-sbom.cdx.json",
        evidence / "security/vulnerability-scan.json",
        evidence / "commands.log",
        evidence / "command-results.log",
    ]
    for required in required_paths:
        if not required.exists() or (required.is_file() and required.stat().st_size == 0):
            fail(f"required evidence is missing or empty: {required.relative_to(run_dir)}")

    manifest = {
        "schema_version": 1, "repository": REPOSITORY, "version": version,
        "tag": provenance["tag"], "commit": provenance["commit"],
        "images": [foundation_image, runtime_image, latest_image],
        "artifacts": [
            {"path": str(path.relative_to(run_dir)), "sha256": sha256(path)}
            for path in sorted((evidence / "darwin-build").glob(f"aibox-v{version}-*-apple-darwin.tar.gz*"))
        ],
        "evidence": [
            {"path": str(path.relative_to(run_dir)), "sha256": sha256(path)}
            for path in required_paths
        ],
    }
    manifest_path = evidence / "release-manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    (evidence / "release-manifest.sha256").write_text(f"{sha256(manifest_path)}  release-manifest.json\n", encoding="utf-8")

    subprocess.run(publisher_command, check=True)


if __name__ == "__main__":
    main()
