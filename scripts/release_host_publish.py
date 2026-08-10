#!/usr/bin/env python3
"""Publish only manifest-listed aibox release outputs after validation."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import shlex
import stat
import subprocess
import sys

REPOSITORY = "projectious-work/aibox"
RUN_ID = re.compile(r"^v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z][0-9A-Za-z.-]*)?-[0-9]{8}T[0-9]{6}Z-[0-9a-f]{12}$")
BASE_REQUIRED_EVIDENCE = {
    "evidence/darwin-smoke/complete.json",
    "evidence/container-build/image-inspect.json",
    "evidence/container-e2e/metadata.env",
    "evidence/container-e2e/impact-selection.json",
    "evidence/security/image-sbom.cdx.json",
    "evidence/security/vulnerability-scan.json",
    "evidence/commands.log",
    "evidence/command-results.log",
}
IMPACT_EVIDENCE = {
    "addon-languages": "evidence/container-e2e/addon-languages.json",
    "addon-platforms": "evidence/container-e2e/addon-platforms.json",
    "addon-tools": "evidence/container-e2e/addon-tools.json",
    "latex-lifecycle": "evidence/container-e2e/latex-lifecycle.json",
    "rootless-podman": "evidence/container-e2e/rootless-podman.json",
}


def fail(message: str) -> "None":
    raise SystemExit(f"release-host publisher: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command: list[str], log: Path) -> str:
    rendered = shlex.join(command)
    print(f"+ {rendered}", flush=True)
    with log.open("a", encoding="utf-8") as stream:
        stream.write(rendered + "\n")
    completed = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    with (log.parent / "results.log").open("a", encoding="utf-8") as results:
        results.write(f"$ {rendered}\n{completed.stdout}\nexit={completed.returncode}\n")
    print(completed.stdout, end="")
    completed.check_returncode()
    return completed.stdout


def main() -> None:
    if len(sys.argv) != 2:
        fail("expected exactly one run-directory argument")
    project_root = Path(__file__).resolve().parent.parent
    approved = (project_root / "tmp/host-gates/aibox-release").resolve()
    run_dir = Path(sys.argv[1])
    if not run_dir.is_absolute():
        run_dir = project_root / run_dir
    if run_dir.is_symlink():
        fail("run directory must not be a symlink")
    run_dir = run_dir.resolve(strict=True)
    if run_dir.parent != approved or not RUN_ID.fullmatch(run_dir.name):
        fail("run directory is outside the approved release gate root")

    evidence = run_dir / "evidence"
    manifest_path = evidence / "release-manifest.json"
    checksum_line = (evidence / "release-manifest.sha256").read_text(encoding="utf-8").strip()
    if checksum_line != f"{sha256(manifest_path)}  release-manifest.json":
        fail("release manifest checksum mismatch")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    expected_manifest_keys = {"schema_version", "repository", "version", "tag", "commit", "container_runtime", "images", "artifacts", "evidence"}
    if set(manifest) != expected_manifest_keys or manifest["schema_version"] != 2:
        fail("release manifest schema is not the reviewed v2 shape")
    if manifest["repository"] != REPOSITORY or manifest["tag"] != f"v{manifest['version']}":
        fail("manifest release coordinates are not the fixed aibox repository contract")
    expected_images = [
        f"ghcr.io/{REPOSITORY}:base-debian-foundation-v{manifest['version']}",
        f"ghcr.io/{REPOSITORY}:base-debian-runtime-v{manifest['version']}",
        f"ghcr.io/{REPOSITORY}:base-debian-runtime-latest",
    ]
    if manifest["images"] != expected_images:
        fail("manifest images are outside the fixed aibox package coordinates")
    container_runtime = manifest["container_runtime"]
    if container_runtime not in {"docker", "podman"}:
        fail("manifest container runtime is outside the reviewed contract")

    artifacts: list[Path] = []
    for entry in manifest["artifacts"]:
        path = (run_dir / entry["path"]).resolve(strict=True)
        info = path.lstat()
        if (path.parent != (evidence / "darwin-build").resolve()
                or not stat.S_ISREG(info.st_mode) or info.st_nlink != 1
                or sha256(path) != entry["sha256"]):
            fail("manifest artifact path or checksum is invalid")
        artifacts.append(path)
    if len(artifacts) != 4:
        fail("manifest must list exactly two Darwin archives and two checksums")
    expected_names = {
        f"aibox-v{manifest['version']}-aarch64-apple-darwin.tar.gz",
        f"aibox-v{manifest['version']}-aarch64-apple-darwin.tar.gz.sha256",
        f"aibox-v{manifest['version']}-x86_64-apple-darwin.tar.gz",
        f"aibox-v{manifest['version']}-x86_64-apple-darwin.tar.gz.sha256",
    }
    if {artifact.name for artifact in artifacts} != expected_names:
        fail("manifest Darwin artifact names are outside the fixed release contract")
    required_evidence = set(BASE_REQUIRED_EVIDENCE)
    selection = json.loads((evidence / "container-e2e/impact-selection.json").read_text(encoding="utf-8"))
    if set(selection) != {"comparison_tag", "comparison_commit", "changed_paths", "selected", "skipped"}:
        fail("impact-selection evidence has an unexpected schema")
    selected = set(selection["selected"])
    skipped = set(selection["skipped"])
    if selected & skipped or selected | skipped != set(IMPACT_EVIDENCE):
        fail("impact-selection evidence does not partition every reviewed conditional check")
    required_evidence.update(IMPACT_EVIDENCE[check] for check in selected)
    evidence_entries = {entry["path"]: entry["sha256"] for entry in manifest["evidence"]}
    if set(evidence_entries) != required_evidence:
        fail("manifest does not enumerate the complete required evidence set")
    for relative, expected in evidence_entries.items():
        path = (run_dir / relative).resolve(strict=True)
        if evidence.resolve() not in path.parents or sha256(path) != expected:
            fail(f"required evidence path or checksum is invalid: {relative}")

    publication = evidence / "publication"
    log = publication / "commands.log"
    run(["gh", "release", "view", manifest["tag"], "--repo", REPOSITORY], log)
    run(["gh", "release", "upload", manifest["tag"], *map(str, artifacts),
         "--repo", REPOSITORY, "--clobber"], log)
    for image in expected_images:
        run([container_runtime, "push", image], log)
        if container_runtime == "docker":
            run(["docker", "manifest", "inspect", image], log)
        else:
            run(["podman", "manifest", "inspect", f"docker://{image}"], log)
    assets_output = run(["gh", "release", "view", manifest["tag"], "--repo", REPOSITORY,
                         "--json", "assets"], log)
    remote_names = {asset["name"] for asset in json.loads(assets_output)["assets"]}
    if not expected_names.issubset(remote_names):
        fail(f"remote release is missing assets: {sorted(expected_names - remote_names)}")
    (publication / "complete.json").write_text(
        json.dumps({"tag": manifest["tag"], "images": expected_images, "status": "verified"}, indent=2) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
