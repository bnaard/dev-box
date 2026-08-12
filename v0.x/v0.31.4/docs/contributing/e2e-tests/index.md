# E2E Test Catalogue

LLMS index: [llms.txt](/aibox/v0.x/v0.31.4/llms.txt)

---

# E2E Test Catalogue

aibox deliberately has no container-runtime bridge from its development
container. Tests are placed according to the product behavior they prove, not
according to the historical runner that happened to execute them.

## Execution surfaces

| Surface | Where it runs | Contracts |
|---|---|---|
| Local | Development container, unique temporary directories | CLI parsing, init/apply, generated files, preservation, configuration, migrations, mocked runtime behavior, terminal rendering |
| Local visual | Development container, per-test tmux socket | Real panes, layouts, keybindings, Yazi/Vim interaction, asciinema output |
| Release host | Owner-controlled macOS gate | Native Darwin binaries and genuine candidate-image build/start/probe/down behavior |

Run the complete local suite with `cd cli && cargo test`. The compatibility
feature `e2e` no longer grants access to another machine or runtime. Opt-in
visual matrices run with:

```bash
./scripts/maintain.sh test-e2e-visual-status
./scripts/maintain.sh test-e2e-visual-tabs
./scripts/maintain.sh test-e2e-visual-yazi
./scripts/maintain.sh test-e2e-visual
```

The canonical release and local-E2E gates pass `--test-threads=1`. The visual
tests share host PTY and tmux scheduling even though each owns a separate
socket; keeping the default serialized prevents slow Yazi/Vim cases from
timing out behind another process-level visual test.

Every process-level visual test removes inherited `TMUX` and `TMUX_PANE`, owns
a socket beneath its `TempDir`, and cleans up only its own server. Tests must
not use global `tmux kill-server`, broad `/tmp/tmux-*` removal, or global
`pkill`; those operations can terminate the developer's live session.

## Removed companion

The former `aibox-e2e-testrunner`, SSH/SCP runner, nested Podman/systemd/kind
image, passwordless elevation, capabilities, devices, host module mount, and
relaxed security profile were removed for issue #372. No test required a
separate unprivileged clean-room service. The generic Alpine pull/run test was
also removed because it supplied weaker evidence than the actual generated
aibox image lifecycle.

The remaining runtime-only contracts are:

- build the candidate foundation and runtime image from the release source;
- generate a downstream project with the candidate Darwin binary;
- run `aibox apply` and Compose up against that locally built image;
- verify identity, non-root execution, readiness, tmux/Yazi/status tooling,
  and the `--forget-tmux-state` attach behavior;
- bring down only that run's Compose project and fail if cleanup fails;
- generate a CycloneDX SBOM and fail the vulnerability gate on high severity.

These contracts inherently need ordinary host Docker and therefore run only in
the macOS release gate. Missing Docker, Apple targets, Syft, Grype, or any other
mandatory prerequisite is a failure, never a passing skip.

Three expensive surfaces remain mandatory when their attested inputs change:

- grouped builds for affected download-based addons;
- a two-revision LaTeX watcher build with byte-identical preview-sidecar output;
- the infrastructure addon's nested Podman probe, including a true rootless report.

The immutable provenance records the previous version-line tag and commit plus
the exact changed-path list. The gate recomputes that diff before selecting
checks. Base-image, generator, addon-loader, template, or host-gate changes
select every conditional surface; an addon definition selects its group, and
LaTeX or infrastructure changes select their dedicated lifecycle probe. If no
comparison tag exists, all conditional checks run. Every selection and
non-selection is written to evidence. A selected check cannot pass by skipping.

## Host run-directory protocol

The container-side release creates immutable input under:

```text
tmp/host-gates/aibox-release/<run-id>/
└── input/
    ├── provenance.json
    ├── checksums.sha256
    └── source.tar.gz
```

The owner runs the single command printed in `dist/RELEASE-PROMPT.md`:

```bash
./scripts/maintain.sh release-host tmp/host-gates/aibox-release/<run-id>
```

Append `--dry-run` to execute the same builds, runtime probes, cleanup,
security scans, and manifest creation without invoking publication:

```bash
./scripts/maintain.sh release-host tmp/host-gates/aibox-release/<run-id> --dry-run
```

Add `--ui=textual` to require the interactive dashboard or `--ui=plain` to
retain line-oriented output. `--ui=auto` is the default and selects Textual
only for a suitable TTY. The dashboard shows completed high-level tasks in a
progress bar, keeps passed/failed/skipped task rows visible, and lets the
operator filter a bordered log by task. Space toggles follow mode, `w` toggles
soft wrapping, Ctrl+A selects, Ctrl+C copies only the marked selection, `l`
selects the last 20 log lines, `y` copies the current task log, and `e` copies
the canonical warning/failure Problems bundle. The Problems panel can be
selected to filter the log. End returns to the live tail, and `p` reveals the
evidence path.

The dashboard is presentation, not evidence. Raw command output remains in
`evidence/command-results.log` and high-level transitions remain in
`evidence/steps.log`; use those files for complete copied or attached logs.

The completed run directory can later be published with the exact
`release-host-publish.sh` command printed by the validator. Do not rerun the
gate without `--dry-run`; populated `runtime/` and `evidence/` directories are
intentionally non-resumable.

The entry point accepts that one path plus optional fixed `--dry-run`,
`--reuse-cache`, and `--ui=auto|textual|plain` flags. It canonicalizes the path,
requires one direct child of the approved root, rejects symlinks, special
files, hardlinks, unexpected files, unsafe permissions, bad checksums, and
tag/commit mismatches, then creates `runtime/` and `evidence/` itself.

The entry point uses the owner-installed uv binary from a reviewed fixed path
and requires exact Python `3.14.6` with `--no-project`. It provisions the
hash-locked Textual `8.2.8` environment from `scripts/release-host-ui.lock`;
candidate PEP 723 and project metadata remain disabled. Its cache and
managed-Python roots are fixed beneath
`~/Library/Caches/aibox-host-gates/uv` and
`~/Library/Application Support/aibox-host-gates/python`; candidate
`pyproject.toml`, inline metadata, uv configuration, and inherited `UV_*`
variables cannot affect execution. The resolved uv, Python, Textual, lockfile,
and tool paths are recorded in evidence.

`--reuse-cache` is intended for repeated rehearsals. It permits
content-addressed container layer reuse while retaining the complete command,
runtime, scan, cleanup, and evidence surface. Without it, downstream images
are rebuilt without cache.

Candidate compilation, build scripts, CLI execution, and runtime smoke run
with a fixed sanitized environment and a macOS sandbox that denies access to
GitHub configuration, Docker configuration, SSH material, and Keychain
security services. Docker receives an empty per-run configuration and no
secrets or host mounts. Tool runtime state is fixed under `runtime/`; no
candidate project configuration controls Python or package resolution.

Only after every required validation succeeds does the gate invoke the small
publisher. The publisher re-verifies the release manifest and hashes, uploads
exactly two Darwin archives plus their two checksum files, and pushes only the
fixed foundation-version, runtime-version, and runtime-latest image tags. It
does not build, test, execute candidate code, modify Git, or accept additional
arguments.

Evidence contains exact commands and results, toolchain/runtime metadata,
changed-path selection reasons, Darwin build and smoke records, image
inspection, generated-runtime and selected conditional-check logs, SBOM,
vulnerability scan, release manifest, and remote publication checks.

## Failure and rerun

The gate is fail-closed and does not reuse a partially populated run directory.
On failure, preserve its evidence for diagnosis. After fixing the release
candidate and creating a new tag/commit, run the container-side release again
to prepare a new run ID. Do not edit an old `input/`, delete selected evidence,
or resume publication from an unverified partial run.

The owner must review the entry point before its first real invocation. Any
later change to the gate or publisher requires a fresh owner review before use.
