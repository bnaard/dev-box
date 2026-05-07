---
sidebar_position: 3
title: E2E Test Catalogue
---

# E2E Test Catalogue

This page describes every end-to-end and integration test in `cli/tests/e2e/`
in plain language. Each entry states the precondition and the expected outcome,
followed by a reference to the exact test function for traceability.

## Test tiers

| Tier | Where it runs | What it needs |
|---|---|---|
| **Tier 1** | Local temp directory | The compiled `aibox` binary only |
| **Tier 1 + Mock** | Local temp directory | Binary + mock `docker`/`podman` scripts on PATH |
| **Tier 2** | Remote SSH companion | `aibox-e2e-testrunner` container reachable (feature flag `e2e`) |

Tier 1 tests run automatically with `cargo test`. Tier 2 tests require the
companion container and the `--features e2e` flag. The expensive visual matrix
tests are opt-in and are not included in the default Tier 2 command.

The container-side release command runs Tier 2 as part of Phase 1 with
`cargo test --features e2e --test e2e`, so the SSH companion, generated
runtime probes, and non-ignored asciinema checks are release gates.

The release process also has a host-side generated-runtime smoke:
`./scripts/maintain.sh release-runtime-smoke X.Y.Z`. It is not an SSH
companion test; it runs on the macOS host during `release-host`, creates a
fresh downstream-style project, runs `aibox init` and
`aibox apply --standardize-config`, starts the generated container, probes the
sidecar-backed Zellij status plugin, and writes logs under
`dist/release-smoke/vX.Y.Z/`. The default `AIBOX_RELEASE_SMOKE_TIER=minimal`
skips optional addon probes; use `addons` to include `git-ui`, or `full` to
include preview addons and force `--no-cache`.

### Opt-in visual E2E

Use these commands when the release diff touches generated runtime visuals or
when the periodic full visual sweep is due:

| Command | Covers |
|---|---|
| `./scripts/maintain.sh test-e2e-visual-status` | all generated layouts across all themes, sidecar Zellij status/key rows, and theme RGB signatures |
| `./scripts/maintain.sh test-e2e-visual-tabs` | tab traversal, Yazi surface, Vim, shell, lazygit, and every enabled AI harness |
| `./scripts/maintain.sh test-e2e-visual-yazi` | Yazi preview plugins, optional preview tools, git symbols, and preview modes |
| `./scripts/maintain.sh test-e2e-visual` | all visual tiers |
| `./scripts/maintain.sh test-e2e-doc-captures` | all visual tiers plus `.cast`, `.screen.txt`, Zellij log, and metadata artifacts under `docs-site/static/img/e2e/` |

Set `AIBOX_E2E_VISUAL_ARTIFACT_DIR` to write documentation capture artifacts
elsewhere. The artifacts are intended as source material for current-release
website screenshots and screencasts.

---

## Lifecycle — `lifecycle.rs`

**Companion is reachable**
If the SSH connection to `aibox-e2e-testrunner` is attempted, then the host
must respond with `ok`, confirming the companion container is up and reachable
before any other Tier 2 test runs. The test also asserts that the companion
image has the pinned Zellij and Yazi versions expected by the visual/runtime
tests; stale companion images fail here with a rebuild hint.
`[lifecycle.rs · companion_is_reachable]`

**Init then apply produces valid project**
If `aibox init` is run followed by `aibox apply`, then
`aibox.toml`, `.devcontainer/Dockerfile`, `.devcontainer/docker-compose.yml`,
and `CLAUDE.md` must all exist in the workspace.
`[lifecycle.rs · lifecycle_init_apply]`

**Generated container starts**
If a fresh project is initialized, applied, and the generated Compose service is
started on the companion runtime, then the running container must expose
`/etc/aibox-version`, Zellij, Yazi, and valid `aibox-status --plugin-json`
output.
`[lifecycle.rs · lifecycle_apply_starts_generated_container]`

**CLAUDE.md user content is preserved on apply**
If a user edits `CLAUDE.md` after `aibox init` and then runs `aibox apply`,
then the edited content must still be present — aibox must not overwrite
user-modified files.
`[lifecycle.rs · claudemd_preserved_on_sync]`

**Generated files are overwritten on apply**
If a generated file (e.g. `.devcontainer/Dockerfile`) is manually tampered
with and `aibox apply` is run, then the file must contain regenerated content
and the tampered content must be gone.
`[lifecycle.rs · generated_files_overwritten_on_sync]`

**Status reports missing when no container exists**
If `aibox get runtime` is run in a project with no running container, then the
output must contain `missing` or equivalent wording.
`[lifecycle.rs · status_without_container_shows_missing]`

**Managed package writes the slim project skeleton**
If `aibox init --context managed` is run, then the slim project skeleton must
exist: `aibox.toml`, an empty `context/` directory, `AGENTS.md`, and the thin
provider pointer files for enabled harnesses. The single-file context tracks
(`BACKLOG.md`, `DECISIONS.md`, `STANDUPS.md`) are **not** scaffolded by `init`
— the corresponding processkit skills create entities in place on first use.
`[lifecycle.rs · init_with_managed_preset_creates_context_files]`

**Software package selection is recorded in aibox.toml**
If `aibox init --context software` is run, then the same slim project skeleton
must exist. Concrete processkit content lands under `context/skills/` only
after `aibox apply` with a real `[processkit].version` pinned.
`[lifecycle.rs · init_with_software_preset_creates_code_files]`

---

## Addon management — `addon.rs`

**Addon add writes to aibox.toml**
If `aibox set addon python` is run in an initialized project, then
`aibox.toml` must contain an `[addons.python]` section afterwards.
`[addon.rs · set_addon_modifies_toml]`

**Addon remove cleans aibox.toml**
If a project is initialized with the `python` addon and then `aibox delete
addon python` is run, then the `[addons.python]` section must no
longer appear in `aibox.toml`.
`[addon.rs · delete_addon_cleans_toml]`

**Addon content appears in generated Dockerfile after apply**
If a project is initialized with the `python` addon and `aibox apply` is run,
then `.devcontainer/Dockerfile` must contain Python-related content (install
commands or references to `uv`).
`[addon.rs · addon_rebuild_includes_tools_in_dockerfile]`

**Addon list shows available addons**
If `aibox get addon` is run in an initialized project, then the output must
list known addons such as `python`.
`[addon.rs · addon_list_shows_available]`

---

## Reset and backup — `reset.rs`

**Reset with backup removes files and creates backup directory**
If `aibox reset project --yes` is run in an initialized project, then `aibox.toml`
must be deleted and `.aibox/backup/` must be created containing the backed-up
files.
`[reset.rs · reset_creates_backup]`

**Reset with --no-backup removes all files without creating a backup**
If `aibox reset project --no-backup --yes` is run, then `aibox.toml` and
`.devcontainer/` must be deleted and `.aibox/backup/` must not be created.
`[reset.rs · reset_no_backup_deletes_all]`

---

## Doctor diagnostics — `doctor.rs`

**Doctor without a config reports an error**
If `aibox doctor` is run in a directory that has no `aibox.toml`, then the
output must mention the missing config or config error and the command must
still exit 0 (doctor is always non-fatal).
`[doctor.rs · doctor_reports_missing_files]`

**Doctor after init reports healthy checks**
If `aibox doctor` is run immediately after a successful `aibox init`, then
the output must contain at least one passing check indicator (`ok`, `✓`, or
similar).
`[doctor.rs · doctor_after_init_reports_healthy]`

---

## Version upgrade flows — `version_upgrade.rs`

**Generated Dockerfile contains version label**
If `aibox init` is run, then the generated `.devcontainer/Dockerfile` must
contain a `LABEL aibox.version` line so the built image carries a
machine-readable version stamp.
`[version_upgrade.rs · dockerfile_contains_aibox_version_label]`

**Generated Dockerfile writes version to /etc/aibox-version**
If `aibox init` is run, then the generated `.devcontainer/Dockerfile` must
contain a `RUN` statement that writes to `/etc/aibox-version` inside the
image, making the build version queryable from within a running container.
`[version_upgrade.rs · dockerfile_contains_etc_aibox_version_write]`

**Up fails when container image version mismatches config**
If an existing container was built from image `v0.0.1` (mock label) and
`aibox.toml` pins the current version, then `aibox up` must exit non-zero
and output a message containing `mismatch` and a suggestion to run
`aibox apply`.
`[version_upgrade.rs · start_fails_on_image_version_mismatch]`

**Up succeeds when container image version matches config**
If an existing container reports the same image version as the one pinned in
`aibox.toml`, then `aibox up` must not produce a version mismatch error.
`[version_upgrade.rs · start_does_not_error_when_versions_match]`

**Update -y exits zero without hanging**
If `aibox self update -y` is run (the global `--yes` flag), then the command must
exit 0 regardless of registry availability — confirming the flag is correctly
wired to `cmd_update` and does not block on an interactive prompt.
`[version_upgrade.rs · update_yes_flag_exits_zero]`

**Update --dry-run does not mention .aibox-version**
If `aibox self update --dry-run` is run, then the output must not contain the
phrase `Would update .aibox-version` — that write was removed in BACK-060
because the image version is now tracked exclusively in `aibox.toml`.
`[version_upgrade.rs · update_dry_run_does_not_mention_aibox_version_file]`

**Doctor warns when running container has a stale image label**
If the running container reports `aibox.version=0.0.1` (mock label) but
`aibox.toml` pins the current version, then `aibox doctor` must emit a
warning containing `mismatch` while still exiting 0.
`[version_upgrade.rs · doctor_warns_on_container_version_mismatch]`

**Doctor warns when .aibox-version is outdated**
If `.aibox-version` is overwritten with `0.0.1` (an old CLI version) and
`aibox doctor` is run, then the output must contain `CLI version mismatch`
and suggest running `aibox apply` to update generated files.
`[version_upgrade.rs · doctor_warns_on_cli_version_file_mismatch]`

---

## Migration — `migration.rs`

**Apply absorbs legacy .aibox-version into aibox.lock**
If `.aibox-version` is overwritten with `0.1.0` (an old version) and
`aibox apply` is run, then `.aibox-version` must be removed and `aibox.lock`
must contain the current `[aibox].cli_version` sync state.
`[migration.rs · apply_absorbs_legacy_version_file_into_lock]`

---

## Update command — `update.rs`

**Update exits zero when registry returns an error**
If `aibox self update` is run in a project where the GHCR registry is unreachable
or returns a non-2xx response, then the command must still exit 0 — the error
must be treated as a warning, not a hard failure.
`[update.rs · update_runs_without_crashing_in_derived_project]`

**Update --check exits zero**
If `aibox self update --check` is run in an initialized project, then the command
must exit 0 and print output containing either `Current CLI version:` or
`Checking for updates`, regardless of whether the registry is reachable.
`[update.rs · update_check_exits_cleanly]`

---

## Appearance — `appearance.rs`

**All themes render without error and without leftover placeholders**
If `aibox init` is run for each of the seven supported themes
(`gruvbox-dark`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`,
`tokyo-night`, `nord`, `projectious`), then the seeded config files must
contain no unreplaced template placeholders such as `AIBOX_THEME` or
`AIBOX_VIM_COLORSCHEME`.
`[appearance.rs · all_themes_render_without_error]`

**Gruvbox theme sets the correct vim colorscheme and zellij theme**
If `aibox init --theme gruvbox-dark` is run, then `vimrc` must contain
`gruvbox` or `retrobox` as the colorscheme and `config.kdl` must reference
`gruvbox-dark`.
`[appearance.rs · theme_gruvbox_renders_correctly]`

**Catppuccin-mocha theme is reflected in zellij config**
If `aibox init --theme catppuccin-mocha` is run, then `config.kdl` must
reference `catppuccin-mocha`.
`[appearance.rs · theme_catppuccin_mocha_renders]`

**Changing the theme updates all themed tool configs**
If a project is initialized with `gruvbox-dark` and the theme is changed to
`dracula` via `aibox apply`, then `config.kdl` must contain `dracula` and no
longer `gruvbox-dark`, and `vimrc`, `yazi/theme.toml`, and `starship.toml`
must be updated. Lazygit config is optional and is checked only when the
`git-ui` addon enables it.
`[appearance.rs · theme_change_auto_applies_untouched_runtime_files]`

**Each theme produces matching configs across all tools**
If `aibox init` is run for each of five themes with known vim colorscheme
names, then `vimrc` must contain the exact `colorscheme <name>` line,
`config.kdl` must reference the theme name, yazi and starship configs must
be non-empty, and lazygit config must be non-empty when present.
`[appearance.rs · theme_alignment_all_tools_match_selected_theme]`

**Yazi keymap includes the open-in-editor binding**
If `aibox init` is run, then `yazi/keymap.toml` must contain an `"e"` key
binding that invokes `open-in-editor`.
`[appearance.rs · yazi_keymap_includes_edit_in_pane_binding]`

**All prompt presets produce a non-empty starship config**
If `aibox init` is run for each of the six prompt presets (`default`,
`plain`, `minimal`, `nerd-font`, `pastel`, `bracketed`), then
`starship.toml` must exist and be non-empty.
`[appearance.rs · all_prompts_render_without_error]`

**Default prompt includes directory and git_branch modules**
If `aibox init --prompt default` is run, then `starship.toml` must contain
both `directory` and `git_branch` module sections.
`[appearance.rs · prompt_default_generates_starship]`

**Plain prompt uses ASCII-only symbols**
If `aibox init --prompt plain` is run, then `starship.toml` must not contain
Nerd Font glyph characters (e.g. no `\ue0b0` powerline arrow).
`[appearance.rs · prompt_plain_no_nerd_font]`

---

## Config coverage — `config_coverage.rs`

**Container name appears in docker-compose.yml**
If `aibox.toml` specifies a container name and `aibox apply` is run, then
`docker-compose.yml` must contain that name.
`[config_coverage.rs · container_name_in_compose]`

**Container hostname appears in docker-compose.yml**
If `aibox.toml` specifies a hostname and `aibox apply` is run, then
`docker-compose.yml` must contain that hostname.
`[config_coverage.rs · container_hostname_in_compose]`

**Port mappings appear in docker-compose.yml**
If `aibox.toml` defines ports (e.g. `"8080:80"`) and `aibox apply` is run,
then `docker-compose.yml` must contain those port entries.
`[config_coverage.rs · container_ports_in_compose]`

**Extra packages appear in the generated Dockerfile**
If `aibox.toml` lists extra packages and `aibox apply` is run, then
`.devcontainer/Dockerfile` must contain those package names in an apt install
block.
`[config_coverage.rs · container_extra_packages_in_dockerfile]`

**Environment variables appear in docker-compose.yml**
If `aibox.toml` defines environment variables and `aibox apply` is run, then
`docker-compose.yml` must contain those key-value pairs.
`[config_coverage.rs · container_environment_in_compose]`

**Extra volumes appear in docker-compose.yml**
If `aibox.toml` defines extra volume mounts and `aibox apply` is run, then
`docker-compose.yml` must contain those source and target paths.
`[config_coverage.rs · container_extra_volumes_in_compose]`

**Claude AI provider adds volume mount**
If `aibox.toml` lists `claude` as an AI provider and `aibox apply` is run,
then `docker-compose.yml` must contain a volume mount for the `.claude`
config directory.
`[config_coverage.rs · ai_claude_provider_volume_mount]`

**Aider AI provider adds volume mount**
If `aibox.toml` lists `aider` as an AI provider and `aibox apply` is run,
then `docker-compose.yml` must contain a volume mount for the `.aider`
config directory.
`[config_coverage.rs · ai_aider_provider_volume_mount]`

**Multiple AI providers each add their own volume mounts**
If `aibox.toml` lists both `claude` and `gemini` as providers and `aibox
apply` is run, then `docker-compose.yml` must contain volume mounts for both
`.claude` and `.gemini`.
`[config_coverage.rs · ai_multiple_providers_volume_mounts]`

**Audio enabled adds PulseAudio mounts and socket**
If `aibox.toml` enables audio and `aibox apply` is run, then
`docker-compose.yml` must contain audio-related volume mounts or socket
references.
`[config_coverage.rs · audio_enabled_adds_mounts]`

**Audio disabled produces no audio mounts**
If `aibox.toml` has audio disabled (the default) and `aibox apply` is run,
then `docker-compose.yml` must not contain audio-related content.
`[config_coverage.rs · audio_disabled_no_mounts]`

**Python addon adds install commands to Dockerfile**
If `aibox.toml` includes the `python` addon and `aibox apply` is run, then
`.devcontainer/Dockerfile` must contain Python install instructions.
`[config_coverage.rs · addon_python_in_dockerfile]`

**Rust addon adds rustup install to Dockerfile**
If `aibox.toml` includes the `rust` addon and `aibox apply` is run, then
`.devcontainer/Dockerfile` must contain `rustup` installation instructions.
`[config_coverage.rs · addon_rust_in_dockerfile]`

**Multiple addons each contribute to the Dockerfile**
If `aibox.toml` includes both the `python` and `rust` addons and `aibox apply`
is run, then `.devcontainer/Dockerfile` must contain install content for
both.
`[config_coverage.rs · addon_multiple_in_dockerfile]`

**Minimal package creates slim project skeleton**
If `aibox init --context minimal` is run, then `aibox.toml`, `aibox.lock`,
an empty `context/` directory, and a thin `CLAUDE.md` pointer must exist.
The single-file context tracks (`BACKLOG.md`, `DECISIONS.md`, `STANDUPS.md`)
are **not** created at init time — the corresponding processkit skills create
them in place on first use.

**Managed package is the recommended default**
If `aibox init --context managed` is run, then the slim project skeleton must
exist. With a real `[processkit].version` pinned, `aibox apply` then installs
the full processkit skill catalogue under `context/skills/` and the immutable
upstream snapshot under `context/templates/processkit/<version>/`.

**Product / research / software packages**
The five processkit packages (`minimal`, `managed`, `software`, `research`,
`product`) are declarative metadata in `[processkit.context].packages`. In v0.16.0 they
do not change which files land on disk — every project gets every processkit
skill — but they tell agents which subset to prefer.

---

## File preview — `preview.rs`

**svg.yazi plugin is seeded into .aibox-home after init**
If `aibox init` is run, then
`.aibox-home/.config/yazi/plugins/svg.yazi/init.lua` must exist.
`[preview.rs · svg_yazi_plugin_seeded]`

**eps.yazi plugin is seeded into .aibox-home after init**
If `aibox init` is run, then
`.aibox-home/.config/yazi/plugins/eps.yazi/init.lua` must exist.
`[preview.rs · eps_yazi_plugin_seeded]`

**svg.yazi plugin invokes resvg for conversion**
If `svg.yazi/init.lua` is read after init, then its content must reference
`resvg` as the SVG-to-PNG conversion tool.
`[preview.rs · svg_yazi_plugin_uses_resvg]`

**eps.yazi plugin invokes ghostscript for conversion**
If `eps.yazi/init.lua` is read after init, then its content must reference
`gs` (ghostscript) as the EPS-to-PNG conversion tool.
`[preview.rs · eps_yazi_plugin_uses_ghostscript]`

**yazi.toml has a [plugin] section with prepend_previewers**
If `aibox init` is run, then `yazi.toml` must contain a `[plugin]` section
that defines `prepend_previewers`.
`[preview.rs · yazi_toml_has_plugin_section]`

**yazi.toml routes *.svg to the svg previewer**
If `aibox init` is run, then `yazi.toml` must contain a
`prepend_previewers` entry matching `*.svg` with `run = "svg"`.
`[preview.rs · yazi_toml_svg_previewer_entry]`

**yazi.toml routes *.eps to the eps previewer**
If `aibox init` is run, then `yazi.toml` must contain a
`prepend_previewers` entry matching `*.eps` with `run = "eps"`.
`[preview.rs · yazi_toml_eps_previewer_entry]`

**SVG and EPS entries appear before built-in image entries**
If `aibox init` is run, then the `*.svg` and `*.eps` entries in
`prepend_previewers` must appear at a lower byte offset than the `*.jpg`
entry, ensuring first-match semantics dispatch SVG/EPS to the custom
plugins rather than the built-in image previewer.
`[preview.rs · yazi_toml_svg_and_eps_precede_builtin_previewers]`

**sample.svg fixture is valid XML**
If `tests/e2e/fixtures/sample.svg` is read, then its content must start
with `<svg` or `<?xml`, confirming the fixture file is intact.
`[preview.rs · fixture_sample_svg_is_valid_xml]`

**sample.eps fixture has a valid EPS header**
If `tests/e2e/fixtures/sample.eps` is read, then its content must start
with `%!PS-Adobe` or contain `%%BoundingBox`, confirming the fixture file
is intact.
`[preview.rs · fixture_sample_eps_has_eps_header]`

---

## Generated runtime — `runtime_generated.rs`

**Generated runtime tools are usable**
If a fresh project is initialized with git-ui and shell status enabled, then
`aibox apply --no-container --standardize-config` must generate Yazi config
that parses with the pinned Yazi binary, lazygit state directories that permit
startup, and an `aibox-status --plugin-json` payload with required fields.
`[runtime_generated.rs · generated_runtime_yazi_lazygit_and_status_are_usable]`

**Generated Zellij status plugin renders**
If the generated dev layout is launched under asciinema with sidecar status
enabled, then the cast must show key/status row text and Zellij logs must not
contain plugin load errors or panics.
`[runtime_generated.rs · generated_runtime_zellij_status_plugin_renders_key_and_status_rows]`

---

## Visual matrix — `visual_matrix.rs`

These tests are ignored by default and run only through the explicit visual
E2E commands above.

**Generated layouts render across all themes**
If each generated layout is launched for each supported theme, then the
recording must include the theme RGB signature and sidecar Zellij status/key row
text, and Zellij logs must not contain plugin load errors, panics, or
`Unknown component: z`.
`[visual_matrix.rs · visual_generated_layouts_render_across_all_themes]`

**Generated tools and harness tabs render when enabled**
If all harnesses and visual runtime addons are enabled, then tab traversal must
show the expected Yazi surface, Vim, shell, lazygit, and every harness marker.
`[visual_matrix.rs · visual_generated_tools_and_harness_tabs_render_when_enabled]`

**Yazi previews, git symbols, and optional plugins render**
If the Yazi preview addons are enabled, then generated Yazi config must parse,
preview plugins must be installed, git symbols must be configured, and directory,
Markdown, CSV, TSV, and SQLite previews must render their markers.
`[visual_matrix.rs · visual_yazi_previews_git_symbols_and_optional_plugins_render]`

---

## Smoke tests — `smoke.rs`

These tests validate that the Tier 2 companion container's container runtime
is functional end-to-end (Tier 2 only).

**Container runtime is available on the companion**
If the companion container is queried for its selected runtime, then the
command must succeed and the output must contain either `docker` or `podman`.
`[smoke.rs · runtime_available_on_companion]`

**Container runtime can pull an image**
If the selected runtime pulls `docker.io/library/alpine:latest` on the
companion, then the pull must succeed, confirming registry access works.
`[smoke.rs · runtime_can_pull_image]`

**Container runtime can run a container**
If the selected runtime runs `alpine echo hello-e2e` on the companion, then
the command must succeed and the output must contain `hello-e2e`, confirming
full container execution.
`[smoke.rs · runtime_can_run_container]`
