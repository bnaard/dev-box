use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

use crate::config::{
    AiHarness, AiProvider, AiboxConfig, AiboxProfile, BaseImage, McpGatewayMode, StarshipPreset,
    Theme, ThemeMode, ZellijStatusMode,
};
use crate::context;
use crate::generate;
use crate::output;
use crate::runtime::{ContainerState, Runtime};
use crate::seed;

/// Parameters for the init command, grouping all optional CLI arguments.
pub struct InitParams {
    pub name: Option<String>,
    pub base: Option<BaseImage>,
    pub profile: Option<AiboxProfile>,
    pub process: Option<Vec<String>>,
    pub ai: Option<Vec<AiProvider>>,
    pub user: Option<String>,
    pub theme: Option<Theme>,
    pub prompt: Option<StarshipPreset>,
    pub zellij_status: Option<ZellijStatusMode>,
    pub addons: Option<Vec<String>>,
    /// Repeated `addon:tool=version` overrides for individual tools
    /// inside selected addons. See [`parse_addon_tool_override`] for
    /// the syntax. Each override skips the interactive version picker
    /// for that tool and pins the version into `aibox.toml`.
    pub addon_tool: Vec<String>,
    /// Override the processkit source URL. `None` → use the default
    /// upstream from `ProcessKitSection::default()`.
    pub processkit_source: Option<String>,
    /// Pin a specific processkit tag. `None` → list versions at the
    /// configured source and pick interactively (or pick latest in
    /// non-interactive mode; fall back to "unset" if listing fails).
    pub processkit_version: Option<String>,
    /// Track a moving processkit branch. Wins over `processkit_version`
    /// at fetch time per the existing fetcher contract.
    pub processkit_branch: Option<String>,
    /// Skip any container-runtime interaction during init.
    ///
    /// `cmd_init` is currently container-free in practice, so this field
    /// is unused at the moment. It is plumbed through for symmetry with
    /// `cmd_sync` and to future-proof against later additions to init
    /// that touch the runtime. Mirrors `Commands::Init { no_container }`
    /// and the `AIBOX_NO_CONTAINER` env var.
    #[allow(dead_code)]
    pub no_container: bool,
}

pub struct ResolvedInitValues {
    pub project_name: String,
    pub base_image: BaseImage,
    pub profile: AiboxProfile,
    pub process_packages: Vec<String>,
    pub addon_names: Vec<String>,
}

/// Build processkit-package selection items for the interactive prompt.
///
/// processkit ships five tiers under `src/packages/`. They are listed
/// here as static presets so `aibox init` can offer them without yet
/// having a fetched processkit cache to read from.
#[allow(dead_code)]
fn process_selection_items() -> (Vec<String>, Vec<String>) {
    const PRESETS: &[(&str, &str)] = &[
        ("minimal", "solo developers and small side projects"),
        ("managed", "small teams with a shared backlog (recommended)"),
        (
            "software",
            "software engineering teams building production systems",
        ),
        ("research", "research, data science, and ML projects"),
        (
            "product",
            "full product development (engineering + design + ops)",
        ),
    ];
    let labels = PRESETS
        .iter()
        .map(|(n, d)| format!("{} — {}", n, d))
        .collect();
    let values = PRESETS.iter().map(|(n, _)| n.to_string()).collect();
    (labels, values)
}

/// Pure decision: should `cmd_sync` (re-)install processkit content?
///
/// Returns true when the configured version is real (not the `unset`
/// sentinel) AND either there is no lock yet, or the lock disagrees
/// with the config on `(source, version)`. Used by the auto-install
/// path that lets users pin a version after `aibox init` and have
/// `aibox apply` materialize the content (closes the v0.16.0 bug
/// reported in BACK-110).
///
/// As of WS-1 (v0.19.x), `cmd_sync` no longer calls this directly — it
/// goes through [`crate::integrity::decide_sync`], which combines this
/// version-comparison logic with the live install-integrity check.
/// The function is retained for the existing version-comparison unit
/// tests; gated behind `#[cfg(test)]` so it doesn't trip dead-code
/// warnings in release builds.
#[cfg(test)]
fn sync_should_install_processkit(
    config_version: &str,
    config_source: &str,
    lock_pair: Option<(&str, &str)>,
) -> bool {
    if config_version == crate::config::PROCESSKIT_VERSION_UNSET
        || config_version == crate::config::PROCESSKIT_VERSION_LATEST
    {
        return false;
    }
    match lock_pair {
        None => true,
        Some((src, ver)) => src != config_source || ver != config_version,
    }
}

/// Run `install_content_source` for `cmd_sync` and report the result.
///
/// Extracted as a private helper so the `Install` and `Reinstall` arms
/// of `decide_sync` dispatch through identical code (DRY: WS-1 spec).
/// Same warn-and-continue policy as before — a fetch failure is
/// announced but does not abort the rest of sync.
fn run_install(cwd: &std::path::Path, config: &AiboxConfig) {
    match crate::content_init::install_content_source(cwd, config) {
        Ok(report) if report.skipped_due_to_unset => {
            // Defensive — decide_sync already gates on version != unset.
        }
        Ok(report) => {
            output::ok(&format!(
                "Installed {} files from processkit {}@{} ({} groups, {} skipped)",
                report.files_installed,
                report.fetched_from,
                report.fetched_version,
                report.groups_touched,
                report.files_skipped,
            ));
        }
        Err(e) => {
            output::warn(&format!(
                "Processkit install failed: {}. Sync will continue without \
                 fresh content; fix the [processkit] section and re-run \
                 `aibox apply` to retry.",
                e
            ));
        }
    }
}

/// Build the `[processkit]` section from CLI overrides + interactive
/// version picker.
///
/// Strategy:
/// 1. Source: `--processkit-source` if given, else the upstream default
///    from `ProcessKitSection::default()`.
/// 2. Branch: `--processkit-branch` if given, else `None`. A branch
///    override wins over the version at fetch time, but the version
///    field is still recorded so the project can drop the branch later
///    and have a sensible pin to fall back to.
/// 3. Version:
///    - `--processkit-version` if given → use as-is
///    - else: list available versions at the source.
///      - Interactive: show a `dialoguer::Select` with the latest as the
///        default. Includes an "unset (skip processkit install)" entry
///        as the escape hatch when the user explicitly wants no install.
///      - Non-interactive: pick the first (latest) entry. If listing
///        fails or returns nothing, fall back to the `unset` sentinel
///        and warn — the user can edit aibox.toml + re-run sync.
fn resolve_processkit_section(
    source_override: Option<&str>,
    version_override: Option<&str>,
    branch_override: Option<&str>,
    interactive: bool,
) -> Result<crate::config::ProcessKitSection> {
    use crate::config::{PROCESSKIT_VERSION_UNSET, ProcessKitSection};

    let mut section = ProcessKitSection::default();
    if let Some(s) = source_override {
        section.source = s.to_string();
    }
    if let Some(b) = branch_override {
        section.branch = Some(b.to_string());
    }

    if let Some(v) = version_override {
        section.version = v.to_string();
        return Ok(section);
    }

    // No version override — list available versions from the configured source.
    output::info(&format!(
        "Querying available processkit versions at {}...",
        section.source
    ));
    let versions = match crate::content_source::list_versions(&section.source) {
        Ok(v) if !v.is_empty() => v,
        Ok(_) => {
            if interactive {
                output::warn(&format!(
                    "No semver-tagged versions found at {}. Leaving processkit.version = \"{}\"; \
                     edit aibox.toml later and re-run `aibox apply` to install content.",
                    section.source, PROCESSKIT_VERSION_UNSET
                ));
                return Ok(section);
            }
            section.version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION.to_string();
            output::warn(&format!(
                "No semver-tagged versions found at {}. Falling back to compiled-in \
                 processkit.version = \"{}\".",
                section.source, section.version
            ));
            return Ok(section);
        }
        Err(e) => {
            if interactive {
                output::warn(&format!(
                    "Could not list processkit versions at {}: {}. Leaving processkit.version = \"{}\"; \
                     edit aibox.toml later and re-run `aibox apply` to install content.",
                    section.source, e, PROCESSKIT_VERSION_UNSET
                ));
                return Ok(section);
            }
            section.version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION.to_string();
            output::warn(&format!(
                "Could not list processkit versions at {}: {}. Falling back to compiled-in \
                 processkit.version = \"{}\".",
                section.source, e, section.version
            ));
            return Ok(section);
        }
    };

    if interactive {
        // Build the menu with the latest at the top + an explicit
        // "skip" escape hatch at the bottom.
        let mut items: Vec<String> = versions
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if i == 0 {
                    format!("{} (latest)", v)
                } else {
                    v.clone()
                }
            })
            .collect();
        items.push(format!(
            "{} — skip processkit install (configure later)",
            PROCESSKIT_VERSION_UNSET
        ));
        let idx = dialoguer::Select::new()
            .with_prompt("processkit version")
            .items(&items)
            .default(0)
            .interact()?;
        if idx == versions.len() {
            section.version = PROCESSKIT_VERSION_UNSET.to_string();
        } else {
            section.version = versions[idx].clone();
        }
    } else {
        // Non-interactive: pick the latest.
        section.version = versions[0].clone();
        output::ok(&format!(
            "Pinned processkit.version = \"{}\" (latest at {})",
            section.version, section.source
        ));
    }

    Ok(section)
}

// ---------------------------------------------------------------------------
// Addon resolution: requires expansion, default tools, version overrides
// ---------------------------------------------------------------------------

/// Parse a single `--addon-tool addon:tool=version` CLI flag value into
/// its three components. Pure function so it's unit-testable.
///
/// Examples:
/// - `python:python=3.14` → `("python", "python", "3.14")`
/// - `node:pnpm=10` → `("node", "pnpm", "10")`
fn parse_addon_tool_override(s: &str) -> Result<(String, String, String)> {
    let (addon_tool, version) = s.split_once('=').ok_or_else(|| {
        anyhow::anyhow!(
            "--addon-tool '{}' is missing '=<version>'. Expected format: addon:tool=version",
            s
        )
    })?;
    let (addon, tool) = addon_tool.split_once(':').ok_or_else(|| {
        anyhow::anyhow!(
            "--addon-tool '{}' is missing the addon prefix. Expected format: addon:tool=version",
            s
        )
    })?;
    if addon.is_empty() || tool.is_empty() || version.is_empty() {
        anyhow::bail!(
            "--addon-tool '{}' has an empty component. Expected format: addon:tool=version",
            s
        );
    }
    Ok((addon.to_string(), tool.to_string(), version.to_string()))
}

/// Map of `addon -> tool -> version` overrides built from the
/// repeated `--addon-tool` flag values. Used by both the interactive
/// resolver (to skip prompts when a version is already pinned) and
/// the populator (to override the default version).
type ToolOverrides = std::collections::HashMap<String, std::collections::HashMap<String, String>>;

fn build_tool_overrides(values: &[String]) -> Result<ToolOverrides> {
    let mut out: ToolOverrides = std::collections::HashMap::new();
    for v in values {
        let (addon, tool, version) = parse_addon_tool_override(v)?;
        out.entry(addon).or_default().insert(tool, version);
    }
    Ok(out)
}

/// Transitively expand the user's selected addon list to include every
/// addon required (directly or indirectly) by the selection.
///
/// Picking `docs-docusaurus` (which `requires: [node]`) without picking
/// `node` used to error out at sync time with "Addon 'docs-docusaurus'
/// requires 'node'". Now both `aibox init` and `aibox set addon` call
/// this helper so the resulting `aibox.toml` already has the
/// dependencies and `aibox apply` never sees a broken graph.
///
/// Pure function — no I/O. The caller is responsible for surfacing
/// `expanded - initial` to the user via `output::info` if desired.
pub(crate) fn expand_addon_requires(initial: &[String]) -> Vec<String> {
    use std::collections::{HashSet, VecDeque};
    let mut result: Vec<String> = initial.to_vec();
    let mut seen: HashSet<String> = result.iter().cloned().collect();
    let mut queue: VecDeque<String> = result.iter().cloned().collect();
    while let Some(name) = queue.pop_front() {
        if let Some(addon) = crate::addon_loader::get_addon(&name) {
            for req in &addon.requires {
                if seen.insert(req.clone()) {
                    result.push(req.clone());
                    queue.push_back(req.clone());
                }
            }
        }
    }
    result
}

fn complete_missing_required_addons(config: &mut AiboxConfig) -> Vec<(String, String)> {
    use std::collections::{HashSet, VecDeque};

    let mut seen: HashSet<String> = config.addons.addons.keys().cloned().collect();
    let mut queue: VecDeque<String> = seen.iter().cloned().collect();
    let mut added = Vec::new();

    while let Some(addon_name) = queue.pop_front() {
        let Some(addon) = crate::addon_loader::get_addon(&addon_name) else {
            continue;
        };
        for required in &addon.requires {
            if seen.insert(required.clone()) {
                config.addons.addons.insert(
                    required.clone(),
                    crate::config::AddonToolsSection::default(),
                );
                added.push((addon_name.clone(), required.clone()));
                queue.push_back(required.clone());
            }
        }
    }

    added
}

/// Build the `[addons.<name>.tools]` section for a single addon at
/// init time. Populates every `default_enabled` tool at the addon's
/// `default_version`, with three layered override sources (later wins):
///
/// 1. Addon's `default_version`
/// 2. Interactive picker — only when `interactive == true` AND the
///    tool has more than one entry in `supported_versions` AND no
///    explicit override is set
/// 3. `--addon-tool addon:tool=version` CLI flag (highest priority,
///    suppresses the interactive picker for that tool)
///
/// Tools that are NOT `default_enabled` are skipped entirely. Users
/// who want them can edit `aibox.toml` directly afterwards (the
/// `aibox describe addon <name>` command lists them).
fn populate_addon_tools(
    addon_name: &str,
    overrides_for_addon: Option<&std::collections::HashMap<String, String>>,
    interactive: bool,
) -> Result<crate::config::AddonToolsSection> {
    use crate::config::{AddonToolsSection, ToolEntry};
    use std::collections::HashMap;

    let mut tools: HashMap<String, ToolEntry> = HashMap::new();

    let Some(loaded) = crate::addon_loader::get_addon(addon_name) else {
        // Unknown addon — caller will surface this elsewhere; we just
        // return an empty section so the rest of init can proceed.
        return Ok(AddonToolsSection { tools });
    };

    for tool in &loaded.tools {
        if !tool.default_enabled {
            continue;
        }

        // Highest priority: explicit CLI override.
        let override_version = overrides_for_addon.and_then(|m| m.get(&tool.name)).cloned();

        // Second priority: interactive picker (only when there's a
        // real choice and the user hasn't pinned via the CLI).
        let picked_version =
            if override_version.is_none() && interactive && tool.supported_versions.len() > 1 {
                // Build version list: "latest" first, then supported versions
                // with the default marked.
                let default_idx = tool
                    .supported_versions
                    .iter()
                    .position(|v| v == &tool.default_version)
                    .unwrap_or(0);
                let mut items: Vec<String> = vec!["latest (always track newest)".to_string()];
                items.extend(tool.supported_versions.iter().enumerate().map(|(i, v)| {
                    if i == default_idx {
                        format!("{} (default)", v)
                    } else {
                        v.clone()
                    }
                }));
                // Default selection: the pinned default version (offset by 1
                // because "latest" is prepended).
                let idx = dialoguer::Select::new()
                    .with_prompt(format!("{}.{} version", addon_name, tool.name))
                    .items(&items)
                    .default(default_idx + 1)
                    .interact()?;
                if idx == 0 {
                    Some("latest".to_string())
                } else {
                    Some(tool.supported_versions[idx - 1].clone())
                }
            } else {
                None
            };

        // Default version as the floor.
        let version = override_version
            .or(picked_version)
            .unwrap_or_else(|| tool.default_version.clone());

        // Empty string means "no separate version" (e.g. rustfmt is part
        // of the rustup toolchain and has no independent version pin).
        // Represent this as None so the TOML serialises as `tool = {}`.
        let version_opt = if version.is_empty() {
            None
        } else {
            Some(version)
        };
        tools.insert(
            tool.name.clone(),
            ToolEntry {
                version: version_opt,
                enabled: None,
            },
        );
    }

    Ok(AddonToolsSection { tools })
}

/// Determine the default project name from the current directory.
fn default_project_name() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-project".to_string())
}

/// Build the list of addon names available for interactive selection,
/// excluding AI harness addons (those are handled via `[ai]`).
fn selectable_addon_names() -> Vec<String> {
    crate::addon_loader::all_addons()
        .iter()
        .filter(|a| !a.name.starts_with("ai-"))
        .map(|a| a.name.clone())
        .collect()
}

/// Resolve init values, prompting interactively when `interactive` is true and
/// the corresponding argument is `None`.
pub fn resolve_init_values(
    name: Option<String>,
    base: Option<BaseImage>,
    profile: Option<AiboxProfile>,
    process: Option<Vec<String>>,
    addons: Option<Vec<String>>,
    interactive: bool,
) -> Result<ResolvedInitValues> {
    // --- project name ---
    let project_name = match name {
        Some(n) => n,
        None if interactive => {
            let default = default_project_name();
            dialoguer::Input::<String>::new()
                .with_prompt("Project name")
                .default(default)
                .interact_text()?
        }
        None => default_project_name(),
    };

    // --- base image (only debian for now, skip prompt) ---
    let base_image = base.unwrap_or(BaseImage::Debian);

    // --- usage profile ---
    let profile = match profile {
        Some(profile) => profile,
        None if interactive => {
            let labels = [
                "human-dev — interactive development with local provider CLIs",
                "headless-runner — automation-safe runner profile",
            ];
            let profiles = [AiboxProfile::HumanDev, AiboxProfile::HeadlessRunner];
            let idx = dialoguer::Select::new()
                .with_prompt("Usage profile")
                .items(&labels)
                .default(0)
                .interact()?;
            profiles[idx]
        }
        None => AiboxProfile::HumanDev,
    };

    // --- processkit skill set ---
    // Package tiers are deprecated; new projects use the full product set.
    let process_packages = process.unwrap_or_else(|| vec!["product".to_string()]);

    // --- addons ---
    let addon_names = match addons {
        Some(a) => a,
        None if interactive => {
            let available = selectable_addon_names();
            if available.is_empty() {
                vec![]
            } else {
                let selections = dialoguer::MultiSelect::new()
                    .with_prompt("Addons (space to select, enter to confirm)")
                    .items(&available)
                    .interact()?;
                selections
                    .into_iter()
                    .map(|i| available[i].clone())
                    .collect()
            }
        }
        None => vec![],
    };

    Ok(ResolvedInitValues {
        project_name,
        base_image,
        profile,
        process_packages,
        addon_names,
    })
}

/// Build command: load config, generate files, run compose build.
/// Start command: seed, generate, ensure running, attach.
pub fn cmd_start(config_path: &Option<String>, layout: &str) -> Result<()> {
    let mut config = AiboxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;

    let added_required_addons = complete_missing_required_addons(&mut config);
    if !added_required_addons.is_empty() {
        for (addon, required) in &added_required_addons {
            output::warn(&format!(
                "Addon '{}' requires '{}'; using '{}' for this start. \
                 Add [addons.{}.tools] to aibox.toml to make the migration explicit.",
                addon, required, required, required
            ));
        }
    }

    let name = &config.container.name;

    seed::ensure_runtime_dirs(&config)?;
    generate::generate_all(&config)?;

    let state = runtime.container_status(name)?;
    if state != ContainerState::Missing {
        let expected_project = crate::generate::sanitize_compose_project_name(name);
        if let Ok(Some(actual_project)) = runtime.get_container_compose_project(name)
            && actual_project != expected_project
        {
            bail!(
                "Runtime container '{}' belongs to compose project '{}' but current aibox config expects project '{}'.\n\n\
                 This usually means an old container survived a generated compose project-name change. Recreate it:\n\
                 \n    aibox delete runtime && aibox up",
                name,
                actual_project,
                expected_project
            );
        }
    }

    // Version mismatch check: if container exists, ensure its image version matches config.
    // No label = pre-BACK-060 image; allow start without check (backward compat).
    //
    // Two failure modes give the same symptom (the existing container's
    // image label != config.aibox.version) but have different fixes:
    //
    //   A) the image was already rebuilt at the new version by an earlier
    //      `aibox apply`, but the container still references the old image
    //      → fix: `aibox delete runtime && aibox up` to recreate the container
    //   B) the image itself is still at the old version
    //      → fix: `aibox apply` to rebuild the image, then start
    //
    // We can't cheaply distinguish them from inside cmd_start without
    // poking the local image store, so we name both fixes in the error.
    //
    // Skip when aibox.toml pins "latest" — "latest" means "any version is
    // acceptable". Comparing a concrete label (e.g. "0.17.12") against the
    // literal string "latest" would always fire even though the container is
    // correct.
    if state != ContainerState::Missing
        && config.aibox.version != "latest"
        && let Ok(Some(container_version)) = runtime.get_container_image_version(name)
        && container_version != config.aibox.version
    {
        bail!(
            "Version mismatch: the existing container was built from image v{} \
             but aibox.toml pins v{}.\n\n\
             Likely cause: an old container survived an aibox upgrade. Recreate it:\n\
             \n    aibox delete runtime && aibox up\n\n\
             If you have not yet rebuilt the image at the new version, run \
             `aibox apply` first to rebuild it, then the recreate command above.",
            container_version,
            config.aibox.version
        );
    }

    match state {
        ContainerState::Running => {
            output::info("Container already running");
        }
        state @ (ContainerState::Stopped | ContainerState::Missing) => {
            let action = if state == ContainerState::Stopped {
                "Starting stopped"
            } else {
                "Creating and starting"
            };
            output::info(&format!("{} container...", action));
            runtime.compose_up(crate::config::COMPOSE_FILE, name)?;
            runtime.wait_for_running(name, 7500)?;
            output::ok("Container started");
        }
    }

    output::info(&format!("Attaching via zellij (layout: {})...", layout));

    // Kill any existing zellij session with this name to ensure a fresh start
    // with properly initialized panes. This prevents the "waiting to load" issue
    // that occurs when reattaching to a session with dead panes (e.g., after
    // exiting and running `aibox up` again). Best-effort; don't fail if the
    // session doesn't exist. Run via docker exec outside of Runtime.exec_interactive
    // to avoid making this step interactive.
    #[cfg(not(test))]
    {
        let docker_cmd = format!(
            "docker exec {} su -c 'zellij kill-session {} 2>/dev/null || true' {}",
            name, name, &config.container.user
        );
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(&docker_cmd)
            .output();
    }

    // Use a named session matching the container name. With the session killed
    // above, `--create` will always create a fresh session with the given layout.
    // `--layout` is a global flag that must come before the subcommand.
    runtime.exec_interactive(
        name,
        &config.container.user,
        &["zellij", "--layout", layout, "attach", "--create", name],
    )?;

    Ok(())
}

pub fn cmd_stop(config_path: &Option<String>) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    let state = runtime.container_status(name)?;
    match state {
        ContainerState::Running => {
            output::info("Stopping container...");
            runtime.compose_stop_all(crate::config::COMPOSE_FILE)?;
            if runtime.container_status(name)? == ContainerState::Running {
                output::info(
                    "Stopping stale same-name container outside current compose project...",
                );
                runtime.stop_container_by_name(name)?;
            }
            output::ok("Container stopped");
        }
        ContainerState::Stopped => {
            output::info("Container is already stopped");
        }
        ContainerState::Missing => {
            output::warn("No container found");
        }
    }

    Ok(())
}

pub fn cmd_remove(config_path: &Option<String>) -> Result<()> {
    use std::collections::BTreeSet;

    let config = AiboxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    let expected_project = crate::generate::sanitize_compose_project_name(name);
    let mut compose_projects = BTreeSet::new();
    compose_projects.insert(expected_project);

    let state = runtime.container_status(name)?;
    if state != ContainerState::Missing
        && let Ok(Some(actual_project)) = runtime.get_container_compose_project(name)
    {
        compose_projects.insert(actual_project);
    }

    let mut initial_project_containers = BTreeSet::new();
    for project in &compose_projects {
        for container in runtime.list_containers_by_compose_project(project)? {
            initial_project_containers.insert(container);
        }
    }

    if state == ContainerState::Missing && initial_project_containers.is_empty() {
        output::info("No runtime containers found");
        return Ok(());
    }

    output::info("Stopping and removing runtime containers...");
    runtime.compose_down(crate::config::COMPOSE_FILE)?;

    let mut removed = BTreeSet::new();
    for project in &compose_projects {
        let containers = runtime.list_containers_by_compose_project(project)?;
        if !containers.is_empty() {
            output::info(&format!(
                "Removing remaining containers in compose project '{}'...",
                project
            ));
        }
        for container in containers {
            runtime.remove_container_by_name(&container)?;
            removed.insert(container);
        }
    }

    if runtime.container_status(name)? != ContainerState::Missing && !removed.contains(name) {
        output::info("Removing stale same-name container outside current compose project...");
        runtime.remove_container_by_name(name)?;
    }
    output::ok(&format!("Runtime containers for '{}' removed", name));

    Ok(())
}

pub fn cmd_status(config_path: &Option<String>, format: crate::cli::OutputFormat) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    let state = runtime.container_status(name)?;
    let state_str = match state {
        ContainerState::Running => "running",
        ContainerState::Stopped => "stopped",
        ContainerState::Missing => "missing",
    };

    match format {
        crate::cli::OutputFormat::Json => {
            let obj = serde_json::json!({
                "container": name,
                "state": state_str,
            });
            println!("{}", serde_json::to_string_pretty(&obj)?);
        }
        crate::cli::OutputFormat::Yaml => {
            let obj = serde_json::json!({
                "container": name,
                "state": state_str,
            });
            print!("{}", serde_yaml::to_string(&obj)?);
        }
        crate::cli::OutputFormat::Table => match state {
            ContainerState::Running => {
                output::ok(&format!("Container '{}' is running", name));
            }
            ContainerState::Stopped => {
                output::warn(&format!("Container '{}' is stopped", name));
            }
            ContainerState::Missing => {
                output::warn(&format!("Container '{}' does not exist", name));
            }
        },
    }

    Ok(())
}

/// Serialize config to TOML with comprehensive comments.
pub(crate) fn serialize_config_with_comments(config: &AiboxConfig) -> String {
    let mut out = String::new();
    let sep = "# =============================================================================\n";

    // File header
    out.push_str(sep);
    out.push_str("# aibox.toml — single source of truth for your aibox project.\n");
    out.push_str(
        "# All .devcontainer/ files are generated from this. Edit here, run `aibox apply`.\n",
    );
    out.push_str(
        "# Reference: https://projectious-work.github.io/aibox/docs/reference/configuration\n",
    );
    out.push_str(sep);
    out.push('\n');

    // Object header + [aibox] section
    out.push_str("# Object identity. These root keys mirror Kubernetes-style resource files:\n");
    out.push_str(
        "# apiVersion selects the aibox config API; kind is currently always Workspace.\n",
    );
    out.push_str(&format!("apiVersion = \"{}\"\n", config.api_version));
    out.push_str(&format!("kind       = \"{}\"\n", config.kind));
    out.push('\n');
    out.push_str("[aibox]\n");
    out.push_str(&format!(
        "project_name = {:20} # Human-readable project name; defaults to container.name\n",
        format!("\"{}\"", config.aibox.project_name)
    ));
    out.push_str(&format!(
        "profile      = \"{}\"       # Usage profile. Options: human-dev, headless-runner\n",
        config.aibox.profile
    ));

    // [container] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [container] — runtime and build configuration\n");
    out.push_str(sep);
    out.push_str("[container]\n");
    out.push_str(&format!(
        "name     = {:20} # Container name used by docker/podman\n",
        format!("\"{}\"", config.container.name)
    ));
    out.push_str(&format!(
        "hostname = {:20} # Hostname visible inside the container\n",
        format!("\"{}\"", config.container.hostname)
    ));

    // user — active if non-root, commented if root
    if config.container.user != "root" {
        out.push_str(&format!(
            "user     = {:20} # User inside the container (controls mount paths)\n",
            format!("\"{}\"", config.container.user)
        ));
    } else {
        out.push_str("# user     = \"root\"               # User inside the container. Options: root, aibox, or any username\n");
        out.push_str("#                                  # Controls mount paths (e.g. /root vs /home/<user>/.vim)\n");
    }

    out.push('\n');
    out.push_str("[container.image]\n");
    out.push_str(&format!(
        "release_version = \"{}\"       # Target aibox image/CLI version. Use \"latest\" to resolve newest on apply.\n",
        config.container.image.version
    ));
    out.push_str(&format!(
        "base            = \"{}\"          # Published base image flavor. Options: debian\n",
        config.container.image.base
    ));

    out.push('\n');
    out.push_str("[container.paths]\n");
    out.push_str(&format!(
        "devcontainer_json     = \"{}\"\n",
        config.container.paths.devcontainer_json
    ));
    out.push_str(&format!(
        "docker_compose        = \"{}\"\n",
        config.container.paths.docker_compose
    ));
    out.push_str(&format!(
        "docker_compose_override = \"{}\"\n",
        config.container.paths.docker_compose_override
    ));
    out.push_str(&format!(
        "dockerfile            = \"{}\"\n",
        config.container.paths.dockerfile
    ));
    out.push_str(&format!(
        "dockerfile_local      = \"{}\"\n",
        config.container.paths.dockerfile_local
    ));
    out.push_str(&format!(
        "local_env             = \"{}\"  # Generated from .aibox-local.toml for docker compose env_file\n",
        config.container.paths.local_env
    ));

    // --- Lifecycle ---
    out.push_str("\n# --- Lifecycle ---\n");
    out.push_str("[container.lifecycle]\n");
    if let Some(cmd) = &config.container.lifecycle.post_create_command {
        out.push_str(&format!(
            "post_create_command = {:20} # Shell command run once after container first starts\n",
            format!("\"{}\"", cmd)
        ));
    } else {
        out.push_str("# post_create_command = \"npm install\"  # Shell command run once after container first starts\n");
    }
    if config.container.lifecycle.keepalive {
        out.push_str("keepalive = true               # Send periodic keepalive (prevents NAT idle dropout in OrbStack/VMs)\n");
    } else {
        out.push_str("# keepalive           = true           # Send periodic keepalive (prevents NAT idle dropout in OrbStack/VMs)\n");
    }
    out.push_str("\n# --- Resource pressure warnings (`aibox doctor`) ---\n");
    out.push_str("# [container.resource_thresholds]\n");
    out.push_str(
        "# memory_mib_warn = 4096       # Optional cgroup memory warning threshold in MiB\n",
    );
    out.push_str("# process_count_warn = 400     # Set to 0 to disable this warning\n");
    out.push_str(
        "# processkit_mcp_python_warn = 50  # Expected to drop after processkit gateway adoption\n",
    );
    out.push_str(
        "# oom_kill_warn = 0            # Warn when cgroup OOM kill count is greater than this\n",
    );
    if config
        .container
        .resource_thresholds
        .memory_mib_warn
        .is_some()
        || config.container.resource_thresholds.process_count_warn != Some(400)
        || config
            .container
            .resource_thresholds
            .processkit_mcp_python_warn
            != Some(50)
        || config.container.resource_thresholds.oom_kill_warn != Some(0)
    {
        out.push_str("[container.resource_thresholds]\n");
        if let Some(value) = config.container.resource_thresholds.memory_mib_warn {
            out.push_str(&format!("memory_mib_warn = {}\n", value));
        }
        if let Some(value) = config.container.resource_thresholds.process_count_warn {
            out.push_str(&format!("process_count_warn = {}\n", value));
        }
        if let Some(value) = config
            .container
            .resource_thresholds
            .processkit_mcp_python_warn
        {
            out.push_str(&format!("processkit_mcp_python_warn = {}\n", value));
        }
        if let Some(value) = config.container.resource_thresholds.oom_kill_warn {
            out.push_str(&format!("oom_kill_warn = {}\n", value));
        }
    }
    render_audio_section(&mut out, config, sep);
    if !config.container.environment.is_empty() {
        out.push_str(
            "\n# Team-shared environment variables. Put secrets in .aibox-local.toml instead.\n",
        );
        out.push_str("[container.environment]\n");
        let mut keys: Vec<_> = config.container.environment.keys().collect();
        keys.sort();
        for key in keys {
            out.push_str(&format!(
                "{} = \"{}\"\n",
                key, config.container.environment[key]
            ));
        }
    }
    if !config.container.extra_volumes.is_empty() {
        out.push_str(
            "\n# Additional team-shared bind mounts. Put personal mounts in .aibox-local.toml.\n",
        );
        for volume in &config.container.extra_volumes {
            out.push_str("[[container.extra_volumes]]\n");
            out.push_str(&format!("source = \"{}\"\n", volume.source));
            out.push_str(&format!("target = \"{}\"\n", volume.target));
            if volume.read_only {
                out.push_str("read_only = true\n");
            } else {
                out.push_str("# read_only = true\n");
            }
            out.push('\n');
        }
    }

    // [skills] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [skills] — processkit skill catalog\n");
    out.push_str(sep);
    out.push_str("# Fresh projects list the standard processkit operating skills explicitly.\n");
    out.push_str("# Use enabled[] for additions and disabled[] for explicit removals. Core\n");
    out.push_str("# skills are always installed; disabling one only triggers a doctor warning.\n");
    out.push_str("[skills]\n");
    let skill_catalog = skill_catalog_entries_for_comments(config);
    render_skill_array(
        &mut out,
        "enabled",
        &config.skills.include,
        &skill_catalog,
        "explicitly enable",
    );
    render_skill_array(
        &mut out,
        "disabled",
        &config.skills.exclude,
        &skill_catalog,
        "explicitly disable",
    );

    // [addons] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [addons] — language runtimes and tool bundles\n");
    out.push_str(sep);
    out.push_str("# Each addon installs a tool set into the container at build time.\n");
    out.push_str("# Selected addons land here pre-populated with default-enabled tools at\n");
    out.push_str("# their default versions; edit the version strings to switch.\n");
    out.push_str("#\n");
    out.push_str("# Version strings:\n");
    out.push_str("#   \"1.2.3\"  — pin to a specific version\n");
    out.push_str("#   \"latest\" — always install the newest version (skips pinning)\n");
    out.push_str("#   \"\"       — use the addon's built-in default version\n");
    out.push_str("#\n");
    out.push_str("# Run `aibox get addon` to see all available addons.\n");
    out.push_str(
        "# Run `aibox describe addon <name>` to see every supported tool/version per addon.\n",
    );
    out.push_str("#\n");
    out.push_str("# To add an addon after init, edit this file and re-run `aibox apply`,\n");
    out.push_str(
        "# or use `aibox set addon <name>` (which also pulls in transitive `requires`).\n",
    );
    out.push_str("#\n");
    out.push_str(
        "# Addon catalog — uncomment/comment one block header to enable or remove an addon.\n",
    );
    out.push_str(
        "# Inside an enabled addon, omitted default-enabled tools stay enabled. Uncomment\n",
    );
    out.push_str(
        "# a tool line to pin a version, enable an off-by-default tool, or disable a default-on tool.\n",
    );
    out.push_str("#\n");
    let mut catalog: Vec<_> = crate::addon_loader::all_addons().iter().collect();
    catalog.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.name.cmp(&b.name))
    });
    let selected_addons = &config.addons.addons;
    let mut catalog_names = std::collections::BTreeSet::new();
    let mut current_category: Option<&str> = None;
    for def in &catalog {
        catalog_names.insert(def.name.as_str());
        if is_internal_audio_addon_name(&def.name)
            && selected_addons
                .get(&def.name)
                .is_none_or(|addon_tools| addon_tools.tools.is_empty())
        {
            continue;
        }
        if is_ai_harness_addon_name(&def.name)
            || (def.category == "AI Providers" && !selected_addons.contains_key(&def.name))
        {
            continue;
        }
        if current_category != Some(def.category.as_str()) {
            current_category = Some(def.category.as_str());
            out.push_str(&format!(
                "\n# ---- {} ------------------------------------------------------------\n",
                def.category
            ));
        }
        if let Some(addon_tools) = selected_addons.get(&def.name) {
            render_active_addon_block(&mut out, def, addon_tools);
        } else {
            render_commented_addon_block(&mut out, def);
        }
    }
    let mut unknown_selected: Vec<_> = selected_addons
        .keys()
        .filter(|name| {
            !catalog_names.contains(name.as_str())
                && !is_ai_harness_addon_name(name)
                && !is_internal_audio_addon_name(name)
        })
        .collect();
    unknown_selected.sort();
    for addon_name in unknown_selected {
        render_unknown_active_addon_block(&mut out, addon_name, &selected_addons[addon_name]);
    }

    // [ai] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [ai] — AI agent harnesses and model providers\n");
    out.push_str(sep);
    out.push_str("# Harnesses: CLI tools installed in the container.\n");
    out.push_str("# Harness (CLI tool)          Config value   Provider (API key)\n");
    out.push_str("# Claude Code                 claude         Anthropic\n");
    out.push_str("# OpenAI Codex                codex          OpenAI\n");
    out.push_str("# Gemini CLI                  gemini         Google\n");
    out.push_str("# Aider                       aider          any (multi-provider)\n");
    out.push_str("# Continue                    continue       any (multi-provider)\n");
    out.push_str("# Cursor                      cursor         any (host-side IDE)\n");
    out.push_str("# GitHub Copilot              copilot        (uses GITHUB_TOKEN)\n");
    out.push_str("# OpenCode                    opencode       any (multi-provider)\n");
    out.push_str("# Hermes                      hermes         any (multi-provider)\n");
    out.push_str("#\n");
    out.push_str("# Model providers (optional): declare which API keys are available.\n");
    out.push_str("# Provider     Config value   Env var\n");
    out.push_str("# Anthropic    anthropic      ANTHROPIC_API_KEY\n");
    out.push_str("# OpenAI       openai         OPENAI_API_KEY\n");
    out.push_str("# Google       google         GEMINI_API_KEY\n");
    out.push_str("# Mistral      mistral        MISTRAL_API_KEY\n");
    out.push_str("[ai]\n");
    render_ai_harness_catalog(&mut out, &config.ai.harnesses);
    render_ai_model_provider_catalog(&mut out, &config.ai.model_providers);
    render_ai_harness_detail_catalog(&mut out, config);

    out.push('\n');
    out.push_str("[ai.agents]\n");
    out.push_str(&format!(
        "canonical     = \"{}\"\n",
        config.ai.agents.canonical
    ));
    let mode_str = match config.ai.agents.provider_mode {
        crate::config::AgentsProviderMode::Pointer => "pointer",
        crate::config::AgentsProviderMode::Full => "full",
    };
    out.push_str(&format!(
        "provider_mode = \"{}\"  # options: pointer, full\n",
        mode_str
    ));
    render_ai_mcp_section(&mut out, config, sep);

    // [processkit] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [processkit] — content layer source (skills, primitives, processes)\n");
    out.push_str(sep);
    out.push_str("# processkit ships the skills and primitives that aibox installs into the\n");
    out.push_str("# project. The default upstream is the canonical projectious-work/processkit\n");
    out.push_str(
        "# repo. Companies can fork processkit and have their projects consume the fork\n",
    );
    out.push_str("# by changing `source` to point at their fork.\n");
    out.push_str("#\n");
    out.push_str(
        "# `version` is the git tag of the processkit source to consume. Special values:\n",
    );
    out.push_str("#   \"unset\"  — no version pinned yet; processkit content is not installed.\n");
    out.push_str("#   \"latest\" — resolve to the newest available tag at every `aibox apply`.\n");
    out.push_str("[processkit]\n");
    out.push_str(&format!("source   = \"{}\"\n", config.processkit.source));
    out.push_str(&format!("version  = \"{}\"\n", config.processkit.version));
    out.push_str(&format!("src_path = \"{}\"\n", config.processkit.src_path));
    match &config.processkit.branch {
        Some(branch) => out.push_str(&format!("branch   = \"{}\"\n", branch)),
        None => out.push_str(
            "# branch = \"main\"   # optional — for tracking a moving branch (discouraged)\n",
        ),
    }
    out.push_str("#\n");
    out.push_str("# Optional release-asset URL template for non-GitHub hosts (Gitea, GitLab,\n");
    out.push_str("# self-hosted). When unset, the fetcher uses the GitHub-style default:\n");
    out.push_str("#   {source}/releases/download/{version}/{name}-{version}.tar.gz\n");
    out.push_str("# Placeholders: {source} (.git stripped), {version}, {org}, {name}.\n");
    match &config.processkit.release_asset_url_template {
        Some(t) => out.push_str(&format!("release_asset_url_template = \"{}\"\n", t)),
        None => out.push_str("# release_asset_url_template = \"https://gitea.example.com/{org}/{name}/releases/download/{version}/payload.tar.gz\"\n"),
    }
    out.push('\n');
    out.push_str("[processkit.context]\n");
    out.push_str(&format!(
        "schema_version = {:12} # Context schema version — updated automatically by `aibox apply`\n",
        format!("\"{}\"", config.processkit.context.schema_version)
    ));
    out.push_str(
        "# packages = [\"product\"]  # deprecated; use explicit [skills].enabled instead\n",
    );

    // [customization] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [customization] — color theme, shell prompt, and zellij layout\n");
    out.push_str(sep);
    out.push_str("# Theme is applied consistently across Zellij, Vim, Yazi, lazygit, and bat.\n");
    out.push_str("# Options: gruvbox-dark | catppuccin-mocha | catppuccin-latte | dracula | tokyo-night | nord | projectious\n");
    out.push_str("[customization]\n");
    out.push_str(&format!("theme  = \"{}\"\n", config.customization.theme));
    out.push_str("# Global mode overlay. `auto` preserves the selected concrete theme.\n");
    out.push_str("# Options: auto | light | dark\n");
    out.push_str(&format!("mode   = \"{}\"\n", config.customization.mode));
    out.push_str("# Starship prompt preset.\n");
    out.push_str("# Options: default | plain | minimal | nerd-font | pastel | bracketed | arrow\n");
    out.push_str(&format!("prompt = \"{}\"\n", config.customization.prompt));
    out.push_str(
        "# Default zellij layout. Options: dev | focus | cowork | cowork-swap | browse | ai\n",
    );
    out.push_str(&format!("layout = \"{}\"\n", config.customization.layout));
    out.push('\n');
    out.push_str("# Zellij status presentation. Options: native | shell | hidden\n");
    out.push_str("[customization.zellij_status]\n");
    out.push_str(&format!(
        "mode = \"{}\"\n",
        config.customization.zellij_status.mode
    ));

    out
}

fn render_audio_section(out: &mut String, config: &AiboxConfig, sep: &str) {
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [audio] — audio and voice feature support\n");
    out.push_str(sep);
    out.push_str("# Requires host-side setup: run `aibox apply audio` on the host first.\n");
    out.push_str("[audio]\n");
    out.push_str(&format!("enabled = {}\n", config.audio.enabled));
    out.push_str(&format!(
        "backend = \"{}\"  # options: pulseaudio\n",
        config.audio.backend
    ));
    out.push_str(&format!(
        "install = {}  # selects the internal audio-voice recipe\n",
        config.audio.install
    ));
    if config.audio.enabled {
        out.push_str(&format!(
            "pulse_server = \"{}\"  # PulseAudio TCP endpoint (default port: 4714)\n",
            config.audio.pulse_server
        ));
    } else {
        out.push_str("# pulse_server = \"tcp:host.docker.internal:4714\"  # PulseAudio TCP endpoint (default port: 4714)\n");
    }
}

fn render_ai_mcp_section(out: &mut String, config: &AiboxConfig, sep: &str) {
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [ai.mcp] — MCP gateway, permissions, and extra servers\n");
    out.push_str(sep);
    out.push_str("# Auto-allow / deny MCP tools by glob pattern. processkit's own MCP tools are\n");
    out.push_str(
        "# pre-approved separately via the skill-gate preauth spec — these patterns are\n",
    );
    out.push_str("# for user-added MCP servers. See:\n");
    out.push_str("# https://projectious-work.github.io/aibox/docs/reference/configuration#permission-configuration-mcppermissions\n");
    out.push_str("# [ai.mcp.permissions]\n");
    out.push_str("# default_mode   = \"ask\"\n");
    out.push_str("# allow_patterns = []\n");
    out.push_str("# deny_patterns  = []\n");
    if mcp_permissions_are_explicit(config) {
        let permissions = &config.ai.mcp.permissions;
        out.push_str("[ai.mcp.permissions]\n");
        out.push_str(&format!(
            "default_mode   = \"{}\"\n",
            permissions.default_mode
        ));
        out.push_str(&format!(
            "allow_patterns = {}\n",
            toml_string_array(&permissions.allow_patterns)
        ));
        out.push_str(&format!(
            "deny_patterns  = {}\n",
            toml_string_array(&permissions.deny_patterns)
        ));
        for (harness, override_cfg) in &permissions.harness {
            out.push('\n');
            out.push_str(&format!("[ai.mcp.permissions.harness.{}]\n", harness));
            out.push_str(&format!("enabled = {}\n", override_cfg.enabled));
            if let Some(mode) = &override_cfg.mode {
                out.push_str(&format!("mode = \"{}\"\n", mode));
            } else {
                out.push_str(
                    "# mode = \"ask\"          # optional harness-specific mode override\n",
                );
            }
            out.push_str(&format!(
                "extra_patterns = {}\n",
                toml_string_array(&override_cfg.extra_patterns)
            ));
            out.push_str(&format!(
                "deny_patterns  = {}\n",
                toml_string_array(&override_cfg.deny_patterns)
            ));
        }
    }
    out.push('\n');
    out.push_str("# [ai.mcp.gateway] — processkit MCP topology. Options for mode: auto | granular | stdio | daemon-proxy\n");
    out.push_str("[ai.mcp.gateway]\n");
    out.push_str(&format!(
        "mode = \"{}\"          # auto uses daemon-proxy when processkit-gateway is installed\n",
        mcp_gateway_mode_str(config.ai.mcp.gateway.mode)
    ));
    out.push_str(&format!(
        "lazy_catalog = {}    # Use processkit's lazy catalog where supported\n",
        config.ai.mcp.gateway.lazy_catalog
    ));
    out.push_str(&format!(
        "host = \"{}\"     # daemon-proxy is always localhost-only\n",
        config.ai.mcp.gateway.host
    ));
    out.push_str(&format!("port = {}\n", config.ai.mcp.gateway.port));
    out.push_str(&format!("path = \"{}\"\n", config.ai.mcp.gateway.path));
    if !config.ai.mcp.servers.is_empty() {
        out.push('\n');
        out.push_str(
            "# Extra team-shared MCP servers. Put personal MCP servers in .aibox-local.toml.\n",
        );
        for server in &config.ai.mcp.servers {
            out.push_str("[[ai.mcp.servers]]\n");
            out.push_str(&format!("name = \"{}\"\n", server.name));
            out.push_str(&format!("command = \"{}\"\n", server.command));
            out.push_str(&format!("args = {}\n", toml_string_array(&server.args)));
            if !server.env.is_empty() {
                let env_pairs = server
                    .env
                    .iter()
                    .map(|(key, value)| format!("{} = \"{}\"", key, value))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!("env = {{ {} }}\n", env_pairs));
            }
            out.push('\n');
        }
    }
}

fn toml_string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn mcp_permissions_are_explicit(config: &AiboxConfig) -> bool {
    let permissions = &config.ai.mcp.permissions;
    permissions.default_mode != "ask"
        || !permissions.allow_patterns.is_empty()
        || !permissions.deny_patterns.is_empty()
        || !permissions.harness.is_empty()
}

fn mcp_gateway_mode_str(mode: McpGatewayMode) -> &'static str {
    match mode {
        McpGatewayMode::Auto => "auto",
        McpGatewayMode::Granular => "granular",
        McpGatewayMode::Stdio => "stdio",
        McpGatewayMode::DaemonProxy => "daemon-proxy",
    }
}

fn render_ai_harness_catalog(out: &mut String, selected: &[crate::config::AiHarness]) {
    out.push_str("harnesses = [\n");
    for harness in crate::config::AiHarness::all() {
        let line = format!("    \"{}\",", harness);
        if selected.contains(harness) {
            out.push_str(&line);
        } else {
            out.push_str("# ");
            out.push_str(&line);
        }
        out.push_str(&format!(" # {}\n", harness.display_name()));
    }
    out.push_str("]\n");
}

fn render_ai_model_provider_catalog(out: &mut String, selected: &[crate::config::AiModelProvider]) {
    out.push_str("\nmodel_providers = [\n");
    for provider in crate::config::AiModelProvider::all() {
        let line = format!("    \"{}\",", provider);
        if selected.contains(provider) {
            out.push_str(&line);
        } else {
            out.push_str("# ");
            out.push_str(&line);
        }
        out.push_str(&format!(" # env: {}\n", provider.api_key_env()));
    }
    out.push_str("]\n");
}

fn render_ai_harness_detail_catalog(out: &mut String, config: &AiboxConfig) {
    out.push_str("\n# Per-harness install controls. `enabled` participates in generated\n");
    out.push_str("# agent/MCP config; `install` selects the in-container CLI recipe.\n");
    for harness in crate::config::AiHarness::all() {
        let selected = config.ai.harnesses.contains(harness);
        let install =
            config.ai.harness_install_enabled(harness) && !harness.addon_name().is_empty();
        let version = ai_harness_version_for_render(config, harness);
        if selected {
            out.push_str(&format!("\n[ai.harness.{}]\n", harness));
            out.push_str("enabled = true\n");
            out.push_str(&format!("install = {}\n", install));
            if let Some(version) = version {
                out.push_str(&format!("version = \"{}\"\n", version));
            } else {
                out.push_str("# version = \"latest\"\n");
            }
        } else {
            out.push_str(&format!("\n# [ai.harness.{}]\n", harness));
            out.push_str("# enabled = true\n");
            out.push_str(&format!("# install = {}\n", install));
            out.push_str("# version = \"latest\"\n");
        }
    }
}

fn ai_harness_version_for_render(
    config: &AiboxConfig,
    harness: &crate::config::AiHarness,
) -> Option<String> {
    if let Some(version) = config.ai.harness_version(harness) {
        return Some(version.to_string());
    }
    let addon_name = harness.addon_name();
    if addon_name.is_empty() {
        return None;
    }
    config
        .addons
        .get_addon(&addon_name)
        .and_then(|addon| addon.tools.get(harness.binary_name()))
        .and_then(|tool| tool.version.clone())
        .filter(|version| !version.is_empty())
}

fn is_ai_harness_addon_name(name: &str) -> bool {
    crate::config::AiHarness::all()
        .iter()
        .map(crate::config::AiHarness::addon_name)
        .any(|addon_name| addon_name == name)
}

fn is_internal_audio_addon_name(name: &str) -> bool {
    name == "audio-voice"
}

#[derive(Debug, Clone)]
struct SkillCatalogEntry {
    name: String,
    category: String,
    description: String,
    core: bool,
}

fn render_skill_array(
    out: &mut String,
    key: &str,
    active: &[String],
    catalog: &[SkillCatalogEntry],
    action: &str,
) {
    out.push_str(&format!("{key} = [\n"));
    let active_set: std::collections::BTreeSet<&str> = active.iter().map(String::as_str).collect();
    for name in active {
        let entry = catalog.iter().find(|entry| entry.name == *name);
        let comment = entry
            .map(skill_line_comment)
            .unwrap_or_else(|| "custom skill override".to_string());
        out.push_str(&format!("    \"{}\", # {}\n", name, comment));
    }
    for entry in catalog {
        if !active_set.contains(entry.name.as_str()) {
            out.push_str(&format!(
                "    # \"{}\", # {}; {}\n",
                entry.name,
                action,
                skill_line_comment(entry)
            ));
        }
    }
    out.push_str("]\n");
}

fn skill_line_comment(entry: &SkillCatalogEntry) -> String {
    let mut parts = vec![entry.category.clone()];
    if entry.core {
        parts.push("core".to_string());
    }
    if !entry.description.is_empty() {
        parts.push(entry.description.clone());
    }
    parts.join("; ")
}

fn skill_catalog_entries_for_comments(config: &AiboxConfig) -> Vec<SkillCatalogEntry> {
    let mut entries = std::collections::BTreeMap::<String, SkillCatalogEntry>::new();
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if let Some(version) = processkit_template_version_for_skill_catalog(config, &project_root) {
        let templates_root = project_root
            .join(crate::processkit_vocab::TEMPLATES_PROCESSKIT_DIR)
            .join(version)
            .join(crate::processkit_vocab::src::CONTEXT_DIR)
            .join(crate::processkit_vocab::src::SKILLS);
        collect_skill_catalog_entries(&templates_root, &mut entries);
    }
    collect_skill_catalog_entries(
        &project_root
            .join(crate::processkit_vocab::src::CONTEXT_DIR)
            .join(crate::processkit_vocab::src::SKILLS),
        &mut entries,
    );

    entries.into_values().collect()
}

fn processkit_template_version_for_skill_catalog(
    config: &AiboxConfig,
    project_root: &Path,
) -> Option<String> {
    match config.processkit.version.as_str() {
        crate::config::PROCESSKIT_VERSION_UNSET => None,
        crate::config::PROCESSKIT_VERSION_LATEST => crate::lock::read_lock(project_root)
            .ok()
            .flatten()
            .and_then(|lock| lock.processkit)
            .map(|processkit| processkit.version),
        version => Some(version.to_string()),
    }
}

fn collect_skill_catalog_entries(
    root: &Path,
    entries: &mut std::collections::BTreeMap<String, SkillCatalogEntry>,
) {
    let Ok(children) = std::fs::read_dir(root) else {
        return;
    };
    for child in children.flatten() {
        let path = child.path();
        if !path.is_dir() {
            continue;
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('_') || name == "lib")
        {
            continue;
        }
        let skill_file = path.join(crate::processkit_vocab::SKILL_FILENAME);
        if skill_file.is_file() {
            let fallback_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();
            let fallback_category = path
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("uncategorized")
                .to_string();
            if let Some(entry) =
                parse_skill_catalog_entry(&skill_file, &fallback_name, &fallback_category)
            {
                entries.insert(entry.name.clone(), entry);
            }
        } else {
            collect_skill_catalog_entries(&path, entries);
        }
    }
}

fn parse_skill_catalog_entry(
    path: &Path,
    fallback_name: &str,
    fallback_category: &str,
) -> Option<SkillCatalogEntry> {
    let body = std::fs::read_to_string(path).ok()?;
    let frontmatter = body
        .strip_prefix("---\n")
        .and_then(|rest| rest.find("\n---").map(|end| &rest[..end]));
    let parsed: Option<serde_yaml::Value> =
        frontmatter.and_then(|yaml| serde_yaml::from_str(yaml).ok());

    let name = parsed
        .as_ref()
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str())
        .unwrap_or(fallback_name)
        .to_string();
    let description = parsed
        .as_ref()
        .and_then(|value| value.get("description"))
        .and_then(|value| value.as_str())
        .map(short_comment)
        .unwrap_or_default();
    let processkit = parsed
        .as_ref()
        .and_then(|value| value.get("metadata"))
        .and_then(|value| value.get("processkit"));
    let category = processkit
        .and_then(|value| value.get("category"))
        .and_then(|value| value.as_str())
        .unwrap_or(fallback_category)
        .to_string();
    let core = processkit
        .and_then(|value| value.get("core"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    Some(SkillCatalogEntry {
        name,
        category,
        description,
        core,
    })
}

fn render_active_addon_block(
    out: &mut String,
    def: &crate::addon_loader::LoadedAddon,
    addon_tools: &crate::config::AddonToolsSection,
) {
    out.push('\n');
    out.push_str(&format!("# {}\n", addon_header_comment(def)));
    out.push_str(&format!("[addons.{}.tools]\n", def.name));

    for tool in &def.tools {
        if let Some(entry) = addon_tools.tools.get(&tool.name) {
            out.push_str(&format!(
                "{} # {}\n",
                render_tool_entry(&tool.name, entry),
                addon_tool_comment(tool)
            ));
        } else {
            out.push_str(&format!(
                "# {} # {}\n",
                active_default_tool_example(tool),
                addon_tool_comment(tool)
            ));
        }
    }

    let mut unknown_tools: Vec<_> = addon_tools
        .tools
        .keys()
        .filter(|tool_name| !def.tools.iter().any(|tool| tool.name == tool_name.as_str()))
        .collect();
    unknown_tools.sort();
    for tool_name in unknown_tools {
        out.push_str(&format!(
            "{} # custom/unknown tool entry\n",
            render_tool_entry(tool_name, &addon_tools.tools[tool_name])
        ));
    }
}

fn render_commented_addon_block(out: &mut String, def: &crate::addon_loader::LoadedAddon) {
    out.push('\n');
    out.push_str(&format!("# {}\n", addon_header_comment(def)));
    out.push_str(&format!("# [addons.{}.tools]\n", def.name));
    if def.tools.is_empty() {
        out.push_str("# # no tool switches; uncomment the header to select this addon\n");
    } else {
        for tool in &def.tools {
            out.push_str(&format!(
                "# {} # {}\n",
                default_tool_example(tool),
                addon_tool_comment(tool)
            ));
        }
    }
}

fn render_unknown_active_addon_block(
    out: &mut String,
    addon_name: &str,
    addon_tools: &crate::config::AddonToolsSection,
) {
    out.push('\n');
    out.push_str("# Selected addon not found in the loaded addon catalog.\n");
    out.push_str(&format!("[addons.{}.tools]\n", addon_name));
    let mut tool_names: Vec<_> = addon_tools.tools.keys().collect();
    tool_names.sort();
    for tool_name in tool_names {
        out.push_str(&format!(
            "{} # options: {{}}, {{ enabled = true|false }}, {{ version = \"x.y.z\" or \"latest\" }}\n",
            render_tool_entry(tool_name, &addon_tools.tools[tool_name])
        ));
    }
}

fn addon_header_comment(def: &crate::addon_loader::LoadedAddon) -> String {
    let mut text = format!("{}/{} — {}", def.category, def.name, def.description);
    if !def.requires.is_empty() {
        text.push_str(&format!("; requires {}", def.requires.join(", ")));
    }
    text
}

fn default_tool_example(tool: &crate::addon_loader::LoadedTool) -> String {
    if tool.default_enabled {
        format!("{} = {{}}", tool.name)
    } else {
        format!("{} = {{ enabled = true }}", tool.name)
    }
}

fn active_default_tool_example(tool: &crate::addon_loader::LoadedTool) -> String {
    if tool.default_enabled {
        format!("{} = {{ enabled = false }}", tool.name)
    } else {
        format!("{} = {{ enabled = true }}", tool.name)
    }
}

fn render_tool_entry(name: &str, entry: &crate::config::ToolEntry) -> String {
    match (&entry.version, entry.enabled) {
        (Some(version), Some(false)) => {
            format!("{name} = {{ version = \"{version}\", enabled = false }}")
        }
        (Some(version), _) => format!("{name} = {{ version = \"{version}\" }}"),
        (None, Some(false)) => format!("{name} = {{ enabled = false }}"),
        (None, Some(true)) => format!("{name} = {{ enabled = true }}"),
        (None, None) => format!("{name} = {{}}"),
    }
}

fn addon_tool_comment(tool: &crate::addon_loader::LoadedTool) -> String {
    let default = if tool.default_enabled {
        "default on"
    } else {
        "default off"
    };
    let version_options = if tool.supported_versions.is_empty() {
        "version = \"x.y.z\" or \"latest\"".to_string()
    } else {
        format!(
            "version = {} or \"latest\"",
            tool.supported_versions
                .iter()
                .map(|version| {
                    if *version == tool.default_version {
                        format!("\"{}\" (default)", version)
                    } else {
                        format!("\"{}\"", version)
                    }
                })
                .collect::<Vec<_>>()
                .join(" | ")
        )
    };
    format!("{default}; options: {{}}, {{ enabled = true|false }}, {{ {version_options} }}")
}

fn short_comment(value: &str) -> String {
    let mut normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    const LIMIT: usize = 110;
    if normalized.chars().count() > LIMIT {
        normalized = normalized.chars().take(LIMIT - 1).collect::<String>();
        normalized.push('…');
    }
    normalized
}

/// Init command: create a aibox.toml and generate files.
pub fn cmd_init(config_path: &Option<String>, params: InitParams) -> Result<()> {
    use crate::config::{
        AddonsSection, AiSection, AiboxConfig, AiboxSection, AudioSection, ContainerSection,
        ContextSection, CustomizationSection, ImageSection, MetadataSection, SkillsSection,
    };

    let toml_path = config_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("aibox.toml"));

    if toml_path.exists() {
        bail!(
            "Config file already exists: {}. Delete it first or edit it directly.",
            toml_path.display()
        );
    }

    let interactive = std::io::IsTerminal::is_terminal(&std::io::stdin());

    let resolved = resolve_init_values(
        params.name,
        params.base,
        params.profile,
        params.process,
        params.addons,
        interactive,
    )?;

    let container_user = params.user.unwrap_or_else(|| "aibox".to_string());
    let zellij_status_mode = match params.zellij_status {
        Some(mode) => mode,
        None if interactive => {
            let labels = [
                "shell — built-in Zellij bar plus aibox runtime status (recommended)",
                "native — experimental aibox WASM plugin",
                "hidden — no aibox-provided status rows",
            ];
            let modes = [
                ZellijStatusMode::Shell,
                ZellijStatusMode::Native,
                ZellijStatusMode::Hidden,
            ];
            let idx = dialoguer::Select::new()
                .with_prompt("Zellij status")
                .items(&labels)
                .default(0)
                .interact()?;
            modes[idx].clone()
        }
        None => ZellijStatusMode::default(),
    };
    let ai_providers = match params.ai {
        Some(providers) => providers,
        None if interactive => {
            let all_harnesses = AiHarness::all();
            let items: Vec<String> = all_harnesses
                .iter()
                .map(|h| h.display_name().to_string())
                .collect();
            // Claude Code is the first item and pre-selected by default.
            let defaults: Vec<bool> = all_harnesses
                .iter()
                .enumerate()
                .map(|(i, _)| i == 0)
                .collect();
            let selections = dialoguer::MultiSelect::new()
                .with_prompt("AI harnesses (space to select, enter to confirm)")
                .items(&items)
                .defaults(&defaults)
                .interact()?;
            if selections.is_empty() {
                vec![AiHarness::Claude]
            } else {
                selections
                    .into_iter()
                    .map(|i| all_harnesses[i].clone())
                    .collect()
            }
        }
        None => vec![AiHarness::Claude],
    };

    // Collect AI harness addon names before they're moved into the config
    // struct so we can include them in the dependency expansion and tool
    // population below.
    let ai_addon_names: Vec<String> = ai_providers
        .iter()
        .filter(|h| h.is_active())
        .map(|h| h.addon_name())
        .filter(|n| !n.is_empty())
        .collect();

    let mut config = AiboxConfig {
        api_version: "aibox.projectious.work/v1".to_string(),
        kind: "Workspace".to_string(),
        metadata: MetadataSection {
            name: resolved.project_name.clone(),
        },
        aibox: AiboxSection {
            config_schema: "1.0.0".to_string(),
            project_name: resolved.project_name.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            base: resolved.base_image.clone(),
            profile: resolved.profile,
        },
        image: ImageSection {
            version: env!("CARGO_PKG_VERSION").to_string(),
            base: resolved.base_image.clone(),
        },
        container: ContainerSection {
            name: resolved.project_name.clone(),
            hostname: resolved.project_name,
            user: container_user,
            post_create_command: None,
            keepalive: false,
            lifecycle: crate::config::ContainerLifecycleSection::default(),
            environment: std::collections::HashMap::new(),
            extra_volumes: vec![],
            resource_thresholds: crate::config::ResourceThresholdsSection::default(),
            image: ImageSection {
                version: env!("CARGO_PKG_VERSION").to_string(),
                base: resolved.base_image.clone(),
            },
            paths: crate::config::ContainerPathsSection::default(),
            audio: AudioSection::default(),
        },
        context: ContextSection {
            packages: resolved.process_packages,
            ..ContextSection::default()
        },
        ai: AiSection {
            harnesses: ai_providers,
            model_providers: Vec::new(),
            harness: std::collections::HashMap::new(),
            providers: Vec::new(),
            agents: crate::config::AgentsSection::default(),
            mcp: crate::config::McpSection::default(),
        },
        process: None,
        addons: {
            // Build the addon section in four steps:
            //   1. Combine user-selected addons with AI provider addons so
            //      their `requires` deps (e.g. ai-codex → node) are pulled
            //      in transitively alongside the rest.
            //   2. Transitively expand `requires` on the combined list.
            //   3. Parse the repeated --addon-tool flag values into a
            //      nested map (addon → tool → version).
            //   4. For each (now-complete) addon, populate its tools
            //      sub-table with default-enabled tools at the right
            //      version (CLI override > interactive pick > default).
            let all_initial: Vec<String> = resolved
                .addon_names
                .iter()
                .chain(ai_addon_names.iter())
                .cloned()
                .collect();
            let expanded_addons = expand_addon_requires(&all_initial);
            for added in &expanded_addons {
                if !all_initial.contains(added) {
                    output::info(&format!(
                        "Adding addon '{}' (transitively required by your selection)",
                        added
                    ));
                }
            }
            let tool_overrides = build_tool_overrides(&params.addon_tool)?;
            let mut section = AddonsSection::default();
            for name in &expanded_addons {
                let tools = populate_addon_tools(name, tool_overrides.get(name), interactive)?;
                section.addons.insert(name.clone(), tools);
            }
            section
        },
        skills: SkillsSection {
            include: crate::processkit_vocab::STANDARD_PROCESSKIT_SKILLS
                .iter()
                .map(|skill| (*skill).to_string())
                .collect(),
            exclude: Vec::new(),
        },
        processkit: resolve_processkit_section(
            params.processkit_source.as_deref(),
            params.processkit_version.as_deref(),
            params.processkit_branch.as_deref(),
            interactive,
        )?,
        customization: CustomizationSection {
            theme: params.theme.unwrap_or_default(),
            mode: ThemeMode::Auto,
            prompt: params.prompt.unwrap_or_default(),
            layout: crate::config::ConfigLayout::default(),
            zellij_status: crate::config::ZellijStatusSection {
                mode: zellij_status_mode,
            },
        },
        agents: crate::config::AgentsSection::default(),
        audio: AudioSection::default(),
        mcp: crate::config::McpSection::default(),
        local_env: std::collections::HashMap::new(),
        local_mcp_servers: vec![],
    };
    config.resolve_ai_provider_addons();

    config.validate()?;

    // --- summary page ---
    if interactive {
        println!();
        output::info("Configuration summary:");
        println!("  Project:     {}", config.container.name);
        println!("  Base:        {}", config.aibox.base);
        println!("  Profile:     {}", config.aibox.profile);
        println!("  Process:     {}", config.context.packages.join(", "));
        let addon_list: Vec<String> = config
            .addons
            .addons
            .keys()
            .filter(|k| !k.starts_with("ai-"))
            .cloned()
            .collect();
        if addon_list.is_empty() {
            println!("  Addons:      (none)");
        } else {
            println!("  Addons:      {}", addon_list.join(", "));
        }
        let harness_list: Vec<&str> = config
            .ai
            .harnesses
            .iter()
            .map(|h| h.display_name())
            .collect();
        println!("  Harnesses:   {}", harness_list.join(", "));
        println!(
            "  Theme:       {} (mode: {})",
            config.customization.theme, config.customization.mode
        );
        println!("  processkit:  {}", config.processkit.version);
        println!();
        let proceed = dialoguer::Confirm::new()
            .with_prompt("Generate project with these settings?")
            .default(true)
            .interact()?;
        if !proceed {
            bail!("Init cancelled by user.");
        }
    }

    let toml_str = serialize_config_with_comments(&config);

    std::fs::write(&toml_path, toml_str)
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", toml_path.display(), e))?;

    output::ok(&format!("Created {}", toml_path.display()));

    generate::generate_all(&config)?;
    context::scaffold_context(&config)?;
    seed::seed_root_dir(&config)?;

    // Install processkit content (A5). Runs last, after the rest of the
    // init pipeline has succeeded. Warn-and-continue on failure so a
    // network hiccup or bad processkit URL doesn't wedge the user's
    // whole init — they get a working aibox project either way and can
    // fix the [processkit] section then re-run `aibox apply`.
    output::info("Installing processkit content...");
    let project_root = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("failed to resolve current directory: {}", e))?;
    match crate::content_init::install_content_source(&project_root, &config) {
        Ok(report) if report.skipped_due_to_unset => {
            output::warn(&format!(
                "Skipped processkit install — [processkit] version is \"{}\". \
                 Edit aibox.toml and run `aibox apply` to install processkit content.",
                crate::config::PROCESSKIT_VERSION_UNSET
            ));
        }
        Ok(report) => {
            output::ok(&format!(
                "Installed {} files from processkit {}@{} ({} groups, {} skipped)",
                report.files_installed,
                report.fetched_from,
                report.fetched_version,
                report.groups_touched,
                report.files_skipped,
            ));
            // After install, regenerate per-harness MCP config files.
            // Best-effort: any failure is warned-and-continued so an
            // MCP-registration glitch doesn't break the rest of init.
            if let Err(e) = crate::mcp_registration::regenerate_mcp_configs(&config, &project_root)
            {
                output::warn(&format!("MCP registration failed: {}", e));
            }
            // Wire processkit enforcement hooks into harness config files.
            // Best-effort: a hook-registration failure must not abort init.
            if let Err(e) =
                crate::hook_registration::regenerate_hook_configs(&config, &project_root)
            {
                output::warn(&format!("Hook registration failed: {}", e));
            }
            // Merge processkit's preauth.json into .claude/settings.json
            // (pre-approve Bash patterns + MCP servers shipped by
            // processkit ≥ v0.22.0). Best-effort.
            if let Err(e) =
                crate::preauth::merge_processkit_preauth_into_claude_settings(&project_root)
            {
                output::warn(&format!("Preauth merge failed: {}", e));
            }
            // Surface the processkit compliance contract to each harness.
            // Best-effort.
            if let Err(e) =
                crate::compliance::regenerate_compliance_configs(&config, &project_root, false)
            {
                output::warn(&format!("Compliance config generation failed: {}", e));
            }
            // Sync processkit command adapter files to per-harness command
            // directories (Claude, Codex, Cursor, Gemini, OpenCode) so each
            // harness can tab-complete them as slash commands. Best-effort.
            if let Err(e) = crate::harness_commands::sync_harness_commands(&project_root, &config) {
                output::warn(&format!("Harness command sync failed: {}", e));
            }
        }
        Err(e) => {
            output::warn(&format!(
                "Processkit install failed: {}. The project is set up but processkit \
                 content was not installed. Run `aibox apply` to retry.",
                e
            ));
        }
    }

    // Ensure aibox.lock exists even if processkit install was skipped or failed.
    // If install_content_source succeeded, it already created the lock. If it was
    // skipped or failed, create a minimal lock with just the [aibox] section.
    let lock_path = project_root.join("aibox.lock");
    if !lock_path.exists() {
        let cli_version = env!("CARGO_PKG_VERSION").to_string();
        let synced_at = chrono::Utc::now().to_rfc3339();
        let minimal_lock = crate::lock::AiboxLock {
            aibox: crate::lock::AiboxLockSection {
                cli_version,
                synced_at,
            },
            processkit: None,
            addons: None,
            runtime_home: None,
        };
        if let Err(e) = crate::lock::write_lock(&project_root, &minimal_lock) {
            output::warn(&format!("Failed to write fallback aibox.lock: {}", e));
        }
    }

    output::ok("Project initialized. Edit aibox.toml to customize, then run: aibox up");

    Ok(())
}

/// Sync command: force-seed theme-dependent files, seed missing configs, regenerate .devcontainer/.
///
/// See `crate::sync_perimeter` for the documented set of files this
/// command is allowed to create, modify, or delete. The tripwire below
/// snapshots a small set of representative out-of-perimeter files
/// before the sync runs and verifies after that none of them were
/// touched — providing a runtime guarantee in addition to the static
/// `is_within_perimeter` check used by sync write helpers.
pub fn cmd_sync(
    config_path: &Option<String>,
    no_cache: bool,
    no_build: bool,
    fix_compliance_contract: bool,
    no_container: bool,
) -> Result<()> {
    // Snapshot out-of-perimeter sentinels before any sync work runs.
    // The tripwire is verified at the end of cmd_sync.
    let tripwire =
        crate::sync_perimeter::Tripwire::snapshot(std::env::current_dir().ok().as_deref());
    let pre_sync_cli_version = crate::lock::read_lock(std::path::Path::new("."))
        .ok()
        .flatten()
        .map(|lock| lock.aibox.cli_version)
        .filter(|v| !v.is_empty());

    // Check for version migration before any other sync steps
    crate::migration::check_and_generate_migration()?;

    let mut config = AiboxConfig::from_cli_option(config_path)?;

    // Resolve [processkit].version = "latest" to a concrete tag before any
    // further processing. The lock always stores a concrete version; "latest"
    // is an aibox.toml-only convenience that is never written to the lock.
    //
    // Semver-aware upgrade policy:
    //   - Fresh install (no lock): take absolute latest unconditionally.
    //   - Patch or minor upgrade (same major): apply automatically.
    //   - Major upgrade: block and warn; take best available within current major.
    //     User must pin an explicit version in aibox.toml to cross a major boundary.
    if config.processkit.version == crate::config::PROCESSKIT_VERSION_LATEST {
        match crate::content_source::list_versions(&config.processkit.source) {
            Ok(versions) if !versions.is_empty() => {
                // Read the currently installed version tag from the lock file.
                let installed_tag: Option<String> =
                    crate::lock::read_lock(std::path::Path::new("."))
                        .ok()
                        .flatten()
                        .and_then(|lock| lock.processkit)
                        .map(|pk| pk.version.clone());

                let absolute_latest = versions[0].clone();

                let resolved = if let Some(ref tag) = installed_tag {
                    let installed_sv = crate::content_source::parse_loose_semver(tag);
                    let latest_sv = crate::content_source::parse_loose_semver(&absolute_latest);
                    match (installed_sv, latest_sv) {
                        (Some(installed), Some(latest)) if latest.major > installed.major => {
                            // Major upgrade: block and find best within current major.
                            crate::output::warn(&format!(
                                "processkit 'latest' ({}) would be a major upgrade from \
                                 the installed version ({}). Major version upgrades are \
                                 not applied automatically — pin an explicit version in \
                                 aibox.toml to upgrade. Staying on the latest v{}.x release.",
                                absolute_latest, tag, installed.major
                            ));
                            let best_in_major = versions
                                .iter()
                                .filter_map(|v| {
                                    crate::content_source::parse_loose_semver(v)
                                        .map(|sv| (sv, v.clone()))
                                })
                                .filter(|(sv, _)| sv.major == installed.major)
                                .max_by_key(|(sv, _)| sv.clone())
                                .map(|(_, v)| v);
                            match best_in_major {
                                Some(v) => {
                                    output::info(&format!(
                                        "Resolved processkit 'latest' \u{2192} {} \
                                         (latest v{}.x)",
                                        v, installed.major
                                    ));
                                    v
                                }
                                None => {
                                    // No releases in current major — keep installed.
                                    output::info(&format!(
                                        "No v{}.x releases found; keeping installed \
                                         version {}.",
                                        installed.major, tag
                                    ));
                                    tag.clone()
                                }
                            }
                        }
                        _ => {
                            // Same or lower major, or unparseable: auto-apply latest.
                            output::info(&format!(
                                "Resolved processkit 'latest' \u{2192} {} (upgrade from {})",
                                absolute_latest, tag
                            ));
                            absolute_latest
                        }
                    }
                } else {
                    // Fresh install (no lock): take absolute latest unconditionally.
                    output::info(&format!(
                        "Resolved processkit 'latest' \u{2192} {} (fresh install)",
                        absolute_latest
                    ));
                    absolute_latest
                };
                config.processkit.version = resolved;
            }
            Ok(_) => {
                crate::output::warn(
                    "processkit.version = \"latest\" but no versions found at source; \
                     skipping processkit install. Set an explicit version in aibox.toml.",
                );
            }
            Err(e) => {
                crate::output::warn(&format!(
                    "processkit.version = \"latest\" but version resolution failed: {}. \
                     Skipping processkit install. Check your network or set an explicit version.",
                    e
                ));
            }
        }
    }

    // Snapshot the original pin before any resolution so the warning below
    // can distinguish "user wrote 'latest'" from "user wrote a concrete version".
    let original_pin = config.aibox.version.clone();

    // Resolve [aibox].version = "latest" to a concrete image tag before
    // Dockerfile generation. "latest" is never a valid Docker image tag in
    // our registry (tags are base-<flavor>-v<semver>), so generation must
    // fall back to a concrete value even when network resolution fails.
    resolve_aibox_image_version_for_generation(&mut config, Path::new("."));

    // Warn if running CLI version differs from the pinned target version.
    // Only fire when the user wrote a concrete version (not "latest" / "unset" / empty).
    let aibox_version_pin = &config.aibox.version;
    if !original_pin.is_empty()
        && original_pin != "latest"
        && original_pin != "unset"
        && aibox_version_pin != env!("CARGO_PKG_VERSION")
    {
        crate::output::warn(&format!(
            "aibox.toml pins version {} but you are running {} — consider updating [aibox].version",
            aibox_version_pin,
            env!("CARGO_PKG_VERSION")
        ));
    }

    // Warn if processkit version is below minimum for this aibox.
    // Skip when version was "latest" (already resolved to the newest available).
    let current_aibox = env!("CARGO_PKG_VERSION");
    if let Some(compat) = crate::compat::min_processkit_for(current_aibox)
        && !crate::compat::processkit_meets_minimum(
            &config.processkit.version,
            compat.processkit_version,
        )
    {
        crate::output::warn(&format!(
            "processkit {} is below the minimum recommended version {} for aibox v{} ({}). \
             Consider updating [processkit].version in aibox.toml.",
            config.processkit.version, compat.processkit_version, current_aibox, compat.note,
        ));
    }

    // Resolve "latest" addon tool versions to concrete versions.
    // The resolved versions are used in Dockerfile generation and recorded
    // in aibox.lock so builds are reproducible.
    let mut resolved_tools = std::collections::BTreeMap::new();
    for (addon_name, addon_tools) in &mut config.addons.addons {
        for (tool_name, tool_entry) in &mut addon_tools.tools {
            if tool_entry.version.as_deref() == Some("latest") {
                // Try upstream resolution for key tools
                if let Some(resolved) = crate::version_resolve::resolve_latest(tool_name) {
                    tool_entry.version = Some(resolved.clone());
                    resolved_tools.insert(tool_name.clone(), resolved);
                } else if let Some(addon) = crate::addon_loader::get_addon(addon_name)
                    && let Some(tool_def) = addon.tools.iter().find(|t| &t.name == tool_name)
                    && !tool_def.default_version.is_empty()
                {
                    // Fall back to addon's default_version
                    let ver = tool_def.default_version.clone();
                    tool_entry.version = Some(ver.clone());
                    resolved_tools.insert(tool_name.clone(), ver);
                }
            }
        }
    }
    if !resolved_tools.is_empty() {
        output::info(&format!(
            "Resolved {} 'latest' tool version(s) to concrete values",
            resolved_tools.len()
        ));
        // Write resolved versions to aibox.lock
        let project_root = std::env::current_dir().unwrap_or_default();
        if let Ok(Some(mut lock)) = crate::lock::read_lock(&project_root) {
            lock.addons = Some(crate::lock::AddonsLockSection {
                resolved_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                tools: resolved_tools,
            });
            if let Err(e) = crate::lock::write_lock(&project_root, &lock) {
                output::warn(&format!(
                    "Failed to update aibox.lock with resolved tool versions: {}",
                    e
                ));
            }
        }
    }

    let added_required_addons = complete_missing_required_addons(&mut config);
    if !added_required_addons.is_empty() {
        for (addon, required) in &added_required_addons {
            output::warn(&format!(
                "Addon '{}' requires '{}'; using '{}' for this apply. \
                 Add [addons.{}.tools] to aibox.toml to make the migration explicit.",
                addon, required, required, required
            ));
        }
        if let Ok(cwd) = std::env::current_dir()
            && let Err(e) =
                crate::migration::generate_addon_dependency_migration(&cwd, &added_required_addons)
        {
            output::warn(&format!(
                "Could not write addon dependency migration guidance: {}",
                e
            ));
        }
    }

    output::info("Scaffolding missing runtime directories...");
    seed::ensure_runtime_dirs(&config)?;
    let runtime_permission_updates = seed::sync_managed_runtime_permissions(&config)?;
    let runtime_cleanup_updates = seed::cleanup_disabled_runtime_files(&config)?;
    if !runtime_permission_updates.is_empty() {
        output::ok(&format!(
            "Updated {} runtime file permission(s)",
            runtime_permission_updates.len()
        ));
    }
    if !runtime_cleanup_updates.is_empty() {
        output::ok(&format!(
            "Removed {} disabled runtime file(s)",
            runtime_cleanup_updates.len()
        ));
    }
    generate::generate_all(&config)?;

    // Capture the pre-install lock before installing so the three-way diff
    // below uses the OLD snapshot as its reference baseline. Reading the lock
    // again after install_content_source would return the new version, making
    // the diff compare new-against-new and recording from_version == to_version.
    let pre_install_processkit_lock: Option<crate::lock::ProcessKitLockSection> =
        std::env::current_dir()
            .ok()
            .and_then(|cwd| crate::lock::read_lock(&cwd).ok().flatten())
            .and_then(|lock| lock.processkit);

    // Decide install / reinstall / skip based on lock+config drift AND
    // the live install-integrity check (WS-1). The integrity check is
    // best-effort — if it errors, fall back to Skip with a warning so a
    // corrupt live marker can't brick `aibox apply` outright.
    match std::env::current_dir() {
        Ok(cwd) => {
            let lock = crate::lock::read_lock(&cwd).ok().flatten();
            let decision =
                crate::integrity::decide_sync(&config, &cwd, &lock).unwrap_or_else(|e| {
                    output::warn(&format!(
                        "integrity check failed: {} — falling back to skip",
                        e
                    ));
                    crate::integrity::SyncDecision::Skip
                });
            match &decision {
                crate::integrity::SyncDecision::Skip => {}
                crate::integrity::SyncDecision::Install { reason } => {
                    output::info(&format!(
                        "Installing processkit {}@{} ({})",
                        config.processkit.source, config.processkit.version, reason
                    ));
                    run_install(&cwd, &config);
                }
                crate::integrity::SyncDecision::Reinstall {
                    reason,
                    prior_state,
                } => {
                    output::warn(&format!(
                        "Repairing processkit template mirror for {}@{}: {}. No manual action is required; aibox will reinstall the pinned processkit files now. Previous integrity state: {}",
                        config.processkit.source, config.processkit.version, reason, prior_state
                    ));
                    run_install(&cwd, &config);
                }
            }
        }
        Err(e) => output::warn(&format!(
            "Failed to determine working directory; skipping processkit install: {}",
            e
        )),
    }

    // Regenerate per-harness MCP config files (.mcp.json,
    // .cursor/mcp.json, .gemini/settings.json, .codex/config.toml,
    // .continue/mcpServers/*.json) based on the currently-pinned
    // processkit version and the [ai].harnesses list. Idempotent —
    // re-running on a stable (version, harnesses, skills) set
    // produces byte-identical output. Best-effort: any failure is
    // warned-and-continued. See DEC-033.
    if let Ok(cwd) = std::env::current_dir() {
        // ── Processkit-install fingerprint drift check ───────────────────
        // WS-7: broadened from the narrow `mcp_config_hash` to cover the
        // full processkit-shipped install payload (skill source,
        // schemas, processes, state-machines, _lib). Any edit under
        // those paths between syncs invalidates the hash here.
        let stored_hash = crate::lock::read_lock(&cwd)
            .ok()
            .flatten()
            .and_then(|l| l.processkit)
            .and_then(|p| p.processkit_install_hash);

        // If processkit ships a manifest with an expected hash, warn on drift.
        // The manifest still publishes a narrow `mcp_config_hash` value, so
        // this is currently a best-effort cross-check rather than a strict
        // equality. A future processkit release that publishes the broad
        // hash will tighten this comparison automatically.
        let manifest_hash = crate::mcp_registration::read_processkit_mcp_manifest_hash(&cwd);
        #[allow(clippy::collapsible_if)]
        if let (Some(mh), Some(sh)) = (&manifest_hash, &stored_hash) {
            if mh != sh {
                output::warn(
                    "processkit MCP manifest hash differs from last sync — \
                     per-skill configs may have changed; regenerating .mcp.json",
                );
            }
        }

        // ── Regenerate (already unconditional) ───────────────────────────
        if let Err(e) = crate::mcp_registration::regenerate_mcp_configs(&config, &cwd) {
            output::warn(&format!("MCP registration failed: {}", e));
        }

        // ── Update fingerprint in lock ───────────────────────────────────
        let new_hash = crate::mcp_registration::compute_processkit_install_fingerprint(&cwd);
        #[allow(clippy::collapsible_if)]
        if new_hash != stored_hash {
            // Fingerprint changed — update the lock so future runs have a fresh baseline.
            if let Ok(Some(mut lock)) = crate::lock::read_lock(&cwd) {
                if let Some(pk) = lock.processkit.as_mut() {
                    pk.processkit_install_hash = new_hash.clone();
                    // Clear the deprecated narrow field so old values
                    // don't linger past the first post-WS-7 sync.
                    #[allow(deprecated)]
                    {
                        pk.mcp_config_hash = None;
                    }
                    if let Err(e) = crate::lock::write_lock(&cwd, &lock) {
                        output::warn(&format!(
                            "Failed to update processkit_install_hash in lock: {}",
                            e
                        ));
                    }
                }
            }
        }

        // Wire processkit enforcement hooks into harness config files.
        // Best-effort: a hook-registration failure must not abort sync.
        if let Err(e) = crate::hook_registration::regenerate_hook_configs(&config, &cwd) {
            output::warn(&format!("Hook registration failed: {}", e));
        }
        // Merge processkit's preauth.json into .claude/settings.json
        // (pre-approve Bash patterns + MCP servers shipped by
        // processkit ≥ v0.22.0). Best-effort.
        if let Err(e) = crate::preauth::merge_processkit_preauth_into_claude_settings(&cwd) {
            output::warn(&format!("Preauth merge failed: {}", e));
        }
        // Surface the processkit compliance contract to each harness
        // (drift check, Cursor rules, Aider conf). Best-effort.
        if let Err(e) =
            crate::compliance::regenerate_compliance_configs(&config, &cwd, fix_compliance_contract)
        {
            output::warn(&format!("Compliance config generation failed: {}", e));
        }
        // Sync processkit command adapter files to per-harness command
        // directories (Claude, Codex, Cursor, Gemini, OpenCode) so each
        // harness can tab-complete them as slash commands. Best-effort.
        if let Err(e) = crate::harness_commands::sync_harness_commands(&cwd, &config) {
            output::warn(&format!("Harness command sync failed: {}", e));
        }
    }

    // Three-way runtime diff for managed .aibox-home files.
    match std::env::current_dir() {
        Ok(cwd) => {
            let current_cli_version = env!("CARGO_PKG_VERSION");
            match crate::runtime_sync::run_runtime_sync(
                &cwd,
                pre_sync_cli_version.as_deref(),
                current_cli_version,
                &config,
            ) {
                Ok(report) => {
                    if report.summary.has_user_relevant_changes() {
                        output::info(&format!(
                            ".aibox-home changes detected: {} upstream-only, {} conflicts, {} new, {} removed",
                            report.summary.changed_upstream_only,
                            report.summary.conflict,
                            report.summary.new_upstream,
                            report.summary.removed_upstream,
                        ));
                        if let Some(path) = report.migration_document_path {
                            output::ok(&format!("Wrote migration document: {}", path.display()));
                        }
                    } else {
                        output::ok(
                            "Managed .aibox-home runtime files are in sync — no migration needed",
                        );
                    }
                }
                Err(e) => output::warn(&format!("Runtime config diff failed: {}", e)),
            }
        }
        Err(e) => output::warn(&format!("Failed to determine working directory: {}", e)),
    }

    // Three-way processkit diff (A6).
    //
    // If the project doesn't yet have an aibox.lock (i.e. nobody has run
    // `aibox init` against this project after A5 landed, OR the version is
    // "unset"), skip — there's nothing to compare against. Any failure is
    // warned-and-continued so a network glitch doesn't break the rest of
    // sync's work.
    match (std::env::current_dir(), pre_install_processkit_lock) {
        (Ok(cwd), Some(from_pk)) => {
            output::info("Comparing processkit cache against project...");
            match crate::content_diff::run_content_sync(&cwd, &from_pk, &config) {
                Ok(report) => {
                    if report.summary.has_user_relevant_changes() {
                        output::info(&format!(
                            "Processkit changes detected: {} upstream-only, {} conflicts, {} new, {} removed",
                            report.summary.changed_upstream_only,
                            report.summary.conflict,
                            report.summary.new_upstream,
                            report.summary.removed_upstream,
                        ));
                        if let Some(path) = report.migration_document_path {
                            output::ok(&format!("Wrote migration document: {}", path.display()));
                        }
                    } else {
                        output::ok("Processkit cache is in sync — no migration needed");
                    }
                    match crate::model_migration::write_legacy_model_spec_migration(&cwd) {
                        Ok(Some(path)) => output::ok(&format!(
                            "Wrote legacy model-spec migration: {}",
                            path.display()
                        )),
                        Ok(None) => {}
                        Err(e) => output::warn(&format!(
                            "Legacy model-spec migration check failed: {}",
                            e
                        )),
                    }
                }
                Err(e) => output::warn(&format!("Processkit diff failed: {}", e)),
            }
        }
        (Ok(_), None) => { /* No pre-install processkit lock — nothing to diff against. */ }
        (Err(e), _) => output::warn(&format!("Failed to determine working directory: {}", e)),
    }

    // Verify the perimeter tripwire BEFORE the (potentially long) image
    // build, so a perimeter violation aborts as fast as possible.
    tripwire.verify()?;

    // Build container image (if a container runtime is available).
    //
    // Three mutually exclusive completion paths so test/log assertions
    // can disambiguate:
    //   * --no-container: never touches Runtime::detect() at all.
    //   * --config-only:     same effect, but the older flag — kept for
    //                     back-compat. Distinct message so the two
    //                     paths can be told apart in tests.
    //   * default:        probe runtime, build image (or warn-skip).
    if no_container {
        output::ok("Sync complete (--no-container: skipped runtime probe and image build)");
    } else if no_build {
        output::ok("Sync complete (build skipped)");
    } else {
        perform_container_build(no_cache, &config)?;
    }

    Ok(())
}

pub fn cmd_apply_generated_runtime(config_path: &Option<String>) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let tripwire = crate::sync_perimeter::Tripwire::snapshot(Some(project_root.as_path()));

    crate::migration::check_and_generate_migration()?;

    let mut config = AiboxConfig::from_cli_option(config_path)?;
    resolve_aibox_image_version_for_generation(&mut config, &project_root);

    let added_required_addons = complete_missing_required_addons(&mut config);
    if !added_required_addons.is_empty() {
        for (addon, required) in &added_required_addons {
            output::warn(&format!(
                "Addon '{}' requires '{}'; using '{}' for this apply. \
                 Add [addons.{}.tools] to aibox.toml to make the migration explicit.",
                addon, required, required, required
            ));
        }
        if let Err(e) = crate::migration::generate_addon_dependency_migration(
            &project_root,
            &added_required_addons,
        ) {
            output::warn(&format!(
                "Could not write addon dependency migration guidance: {}",
                e
            ));
        }
    }

    output::info("Generating devcontainer files...");
    generate::generate_all(&config)?;

    let version = env!("CARGO_PKG_VERSION");
    crate::runtime_sync::copy_runtime_templates(&project_root, version, &config)?;
    crate::runtime_sync::refresh_runtime_home_template_lock(&project_root, &config)?;

    tripwire.verify()?;
    output::ok(
        "Generated runtime files refreshed (skipped processkit, harness config, live runtime, runtime probe, and image build)",
    );

    Ok(())
}

fn resolve_aibox_image_version_for_generation(config: &mut AiboxConfig, project_root: &Path) {
    if config.aibox.version != "latest" {
        return;
    }

    let flavor = config.aibox.base.to_string();
    match crate::update::fetch_latest_image_version(&flavor) {
        Ok(v) => {
            let resolved = format!("{}.{}.{}", v.major, v.minor, v.patch);
            output::info(&format!(
                "Resolved aibox image 'latest' \u{2192} v{}",
                resolved
            ));
            config.aibox.version = resolved;
            config.image.version = config.aibox.version.clone();
        }
        Err(e) => {
            if let Some(previous) = previous_concrete_aibox_version(project_root) {
                output::warn(&format!(
                    "[aibox].version = \"latest\" but image version resolution failed: {}. \
                     Reusing previously resolved aibox version {} from aibox.lock.",
                    e, previous
                ));
                config.aibox.version = previous;
                config.image.version = config.aibox.version.clone();
            } else {
                let current = env!("CARGO_PKG_VERSION").to_string();
                output::warn(&format!(
                    "[aibox].version = \"latest\" but image version resolution failed: {}. \
                     Falling back to the running CLI version {}.",
                    e, current
                ));
                config.aibox.version = current;
                config.image.version = config.aibox.version.clone();
            }
        }
    }
}

fn previous_concrete_aibox_version(project_root: &Path) -> Option<String> {
    crate::lock::read_lock(project_root)
        .ok()
        .flatten()
        .map(|lock| lock.aibox.cli_version)
        .filter(|version| !version.is_empty() && version != "latest" && version != "unset")
}

/// Probe for a container runtime and build the project image.
///
/// Extracted from `cmd_sync` so the runtime-touching step can be
/// short-circuited cleanly by `--no-container` / `AIBOX_NO_CONTAINER`.
/// Preserves the original semantics exactly: a successful probe builds
/// the image and warns if a running container lags the freshly-built
/// image; a failed probe degrades to a warn-and-skip with a "config
/// files only" success message.
fn perform_container_build(no_cache: bool, config: &AiboxConfig) -> Result<()> {
    match Runtime::detect() {
        Ok(runtime) => {
            output::info("Building container image...");
            runtime.compose_build(crate::config::COMPOSE_FILE, no_cache)?;
            output::ok("Sync complete — image built");
            warn_if_container_lags_image(&runtime, config);
        }
        Err(_) => {
            output::warn("No container runtime found — skipping image build");
            output::ok("Sync complete (config files only)");
        }
    }
    Ok(())
}

/// Warn the user if a container exists for this project AND its image
/// label disagrees with the just-built image. This catches the
/// "I synced but my old container is still running on the old image"
/// situation BEFORE the user runs `aibox up` and gets a hard error.
///
/// Best-effort: any failure (runtime probe, label read) is silently
/// swallowed. The warning is informational, not load-bearing — its
/// only job is to surface a stale runtime so the next `aibox up`
/// isn't a surprise.
fn warn_if_container_lags_image(runtime: &Runtime, config: &AiboxConfig) {
    let name = &config.container.name;
    let Ok(state) = runtime.container_status(name) else {
        return;
    };
    if state == ContainerState::Missing {
        return;
    }
    let Ok(Some(container_version)) = runtime.get_container_image_version(name) else {
        return;
    };
    if container_version == config.aibox.version {
        return;
    }
    output::warn(&format!(
        "Container '{}' is still running on image v{} but the freshly-built image is v{}.\n    \
         The current container will keep running on the old image until you recreate it. To upgrade:\n    \
         \n        aibox delete runtime && aibox up\n    \
         \n    Existing in-flight work in the container (open editors, running processes) will be lost \
         on recreation; project files under /workspace are mounted from the host and survive.",
        name, container_version, config.aibox.version
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_init_values_non_interactive_defaults() {
        let resolved =
            resolve_init_values(None, None, None, None, None, false).expect("should succeed");

        // Name defaults to current directory name (or "my-project" fallback)
        assert!(
            !resolved.project_name.is_empty(),
            "name should not be empty"
        );

        assert_eq!(resolved.base_image, BaseImage::Debian);
        assert_eq!(resolved.profile, AiboxProfile::HumanDev);
        assert_eq!(resolved.process_packages, vec!["product".to_string()]);
        assert!(resolved.addon_names.is_empty());
    }

    #[test]
    fn serialized_config_groups_nested_sections_near_parent_sections() {
        let config = crate::config::test_config();
        let body = serialize_config_with_comments(&config);
        let container = body.find("[container]").unwrap();
        let audio = body.find("[audio]").unwrap();
        let skills = body.find("[skills]").unwrap();
        let ai = body.find("[ai]").unwrap();
        let ai_mcp = body.find("[ai.mcp.gateway]").unwrap();
        let processkit = body.find("[processkit]").unwrap();

        assert!(
            container < audio && audio < skills,
            "[audio] should stay near the container section"
        );
        assert!(
            ai < ai_mcp && ai_mcp < processkit,
            "[ai.mcp] should stay with the ai section"
        );
    }

    #[test]
    fn serialized_config_places_ai_harness_install_controls_under_ai() {
        let mut config = crate::config::test_config();
        config.ai.harnesses = vec![
            crate::config::AiHarness::Claude,
            crate::config::AiHarness::Codex,
        ];
        config.addons.addons.insert(
            "ai-codex".to_string(),
            crate::config::AddonToolsSection {
                tools: [(
                    "codex".to_string(),
                    crate::config::ToolEntry {
                        version: Some("1.2.3".to_string()),
                        enabled: None,
                    },
                )]
                .into_iter()
                .collect(),
            },
        );

        let body = serialize_config_with_comments(&config);
        assert!(body.contains("[ai.harness.claude]"));
        assert!(body.contains("[ai.harness.codex]"));
        assert!(body.contains("version = \"1.2.3\""));
        assert!(!body.contains("[addons.ai-claude.tools]"));
        assert!(!body.contains("[addons.ai-codex.tools]"));
    }

    #[test]
    fn serialized_config_places_default_audio_install_under_audio() {
        let mut config = crate::config::test_config();
        config.audio.enabled = true;
        config.container.audio = config.audio.clone();
        config.addons.addons.insert(
            "audio-voice".to_string(),
            crate::config::AddonToolsSection::default(),
        );

        let body = serialize_config_with_comments(&config);
        assert!(body.contains("[audio]"));
        assert!(body.contains("backend = \"pulseaudio\""));
        assert!(body.contains("install = true"));
        assert!(!body.contains("[container.audio]"));
        assert!(!body.contains("[addons.audio-voice.tools]"));
    }

    #[test]
    fn standard_processkit_skills_are_serialized_when_enabled() {
        let mut config = crate::config::test_config();
        config.skills.include = crate::processkit_vocab::STANDARD_PROCESSKIT_SKILLS
            .iter()
            .map(|skill| (*skill).to_string())
            .collect();
        let body = serialize_config_with_comments(&config);
        assert!(body.contains("enabled = ["));
        assert!(body.contains("\"pk-doctor\""));
        assert!(body.contains("\"status-briefing\""));
        assert!(body.contains("\"workitem-management\""));
    }

    #[test]
    fn resolve_init_values_explicit_args_override() {
        // Even with interactive=true, explicit values should be used without prompting
        let resolved = resolve_init_values(
            Some("my-app".to_string()),
            Some(BaseImage::Debian),
            Some(AiboxProfile::HeadlessRunner),
            Some(vec!["research".to_string()]),
            Some(vec!["latex".to_string()]),
            true,
        )
        .expect("should succeed with explicit args");

        assert_eq!(resolved.project_name, "my-app");
        assert_eq!(resolved.base_image, BaseImage::Debian);
        assert_eq!(resolved.profile, AiboxProfile::HeadlessRunner);
        assert_eq!(resolved.process_packages, vec!["research".to_string()]);
        assert_eq!(resolved.addon_names, vec!["latex".to_string()]);
    }

    // ── parse_addon_tool_override / build_tool_overrides ────────────────────

    #[test]
    fn parse_addon_tool_override_happy_path() {
        let (a, t, v) = parse_addon_tool_override("python:python=3.14").unwrap();
        assert_eq!(a, "python");
        assert_eq!(t, "python");
        assert_eq!(v, "3.14");
    }

    #[test]
    fn parse_addon_tool_override_handles_dotted_versions() {
        let (a, t, v) = parse_addon_tool_override("node:pnpm=10.5.0").unwrap();
        assert_eq!(a, "node");
        assert_eq!(t, "pnpm");
        assert_eq!(v, "10.5.0");
    }

    #[test]
    fn parse_addon_tool_override_rejects_missing_equals() {
        let err = parse_addon_tool_override("python:python").unwrap_err();
        assert!(format!("{}", err).contains("=<version>"));
    }

    #[test]
    fn parse_addon_tool_override_rejects_missing_colon() {
        let err = parse_addon_tool_override("python=3.14").unwrap_err();
        assert!(format!("{}", err).contains("addon prefix"));
    }

    #[test]
    fn parse_addon_tool_override_rejects_empty_components() {
        assert!(parse_addon_tool_override(":python=3.14").is_err());
        assert!(parse_addon_tool_override("python:=3.14").is_err());
        assert!(parse_addon_tool_override("python:python=").is_err());
    }

    #[test]
    fn build_tool_overrides_groups_by_addon() {
        let raw = vec![
            "python:python=3.14".to_string(),
            "python:uv=0.8".to_string(),
            "node:node=20".to_string(),
        ];
        let map = build_tool_overrides(&raw).unwrap();
        assert_eq!(map["python"]["python"], "3.14");
        assert_eq!(map["python"]["uv"], "0.8");
        assert_eq!(map["node"]["node"], "20");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn build_tool_overrides_propagates_parse_error() {
        let raw = vec!["bogus".to_string()];
        assert!(build_tool_overrides(&raw).is_err());
    }

    // ── expand_addon_requires ───────────────────────────────────────────────
    //
    // The full transitive expansion is exercised by the e2e tests
    // (cli/tests/e2e/) which load the real addon registry. The unit
    // tests below cover the dedupe and ordering invariants without
    // depending on the registry.

    #[test]
    fn expand_addon_requires_preserves_initial_order() {
        // No addon registry initialized in unit tests → expansion is a
        // no-op (get_addon returns None for everything). The function
        // must still preserve the input order and not duplicate.
        let input = vec!["python".to_string(), "node".to_string()];
        let out = expand_addon_requires(&input);
        assert_eq!(out, vec!["python", "node"]);
    }

    #[test]
    fn expand_addon_requires_handles_empty() {
        let out = expand_addon_requires(&[]);
        assert!(out.is_empty());
    }

    // ── sync_should_install_processkit ──────────────────────────────────────

    #[test]
    fn sync_install_skipped_when_version_is_unset() {
        // The "unset" sentinel always disables the auto-install — even if a
        // stale lock from an earlier real version exists, sync should leave
        // it alone. The user has explicitly opted out by typing "unset".
        assert!(!sync_should_install_processkit(
            crate::config::PROCESSKIT_VERSION_UNSET,
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            None,
        ));
        assert!(!sync_should_install_processkit(
            crate::config::PROCESSKIT_VERSION_UNSET,
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            Some((
                crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
                crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION
            )),
        ));
    }

    #[test]
    fn sync_install_runs_when_version_pinned_and_no_lock() {
        // User pinned a real version but no lock exists yet — sync must install.
        assert!(sync_should_install_processkit(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            None,
        ));
    }

    #[test]
    fn sync_install_skipped_when_lock_matches_config() {
        // Steady state: the install already ran, lock matches config →
        // sync should NOT re-install. The downstream three-way diff path
        // handles drift detection from here on.
        assert!(!sync_should_install_processkit(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            Some((
                crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
                crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION
            )),
        ));
    }

    #[test]
    fn sync_install_runs_when_lock_version_stale() {
        // User bumped processkit.version in aibox.toml from v0.5.1 → v0.6.0.
        // Sync must re-install so the new version's content lands.
        assert!(sync_should_install_processkit(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            Some((crate::processkit_vocab::PROCESSKIT_GIT_SOURCE, "v0.5.1")),
        ));
    }

    #[test]
    fn sync_install_runs_when_lock_source_changed() {
        // User switched from upstream processkit to a fork (or vice versa).
        // Sync must re-install from the new source even if the version tag
        // happens to match.
        assert!(sync_should_install_processkit(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            "https://github.com/acme/processkit-acme.git",
            Some((
                crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
                crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION
            )),
        ));
    }

    #[test]
    fn resolve_init_values_explicit_args_non_interactive() {
        let resolved = resolve_init_values(
            Some("test-proj".to_string()),
            Some(BaseImage::Debian),
            Some(AiboxProfile::HeadlessRunner),
            Some(vec!["minimal".to_string()]),
            Some(vec!["python".to_string(), "latex".to_string()]),
            false,
        )
        .expect("should succeed");

        assert_eq!(resolved.project_name, "test-proj");
        assert_eq!(resolved.base_image, BaseImage::Debian);
        assert_eq!(resolved.profile, AiboxProfile::HeadlessRunner);
        assert_eq!(resolved.process_packages, vec!["minimal".to_string()]);
        assert_eq!(
            resolved.addon_names,
            vec!["python".to_string(), "latex".to_string()]
        );
    }

    // -- FIX 2: version-pin warning is gated on original_pin -----------------

    fn should_warn_version_mismatch(original_pin: &str, resolved_pin: &str) -> bool {
        let cargo_ver = env!("CARGO_PKG_VERSION");
        !original_pin.is_empty()
            && original_pin != "latest"
            && original_pin != "unset"
            && resolved_pin != cargo_ver
    }

    #[test]
    fn version_pin_warning_suppressed_when_original_is_latest() {
        assert!(
            !should_warn_version_mismatch("latest", "1.2.3"),
            "warning must be suppressed when user wrote 'latest'"
        );
    }

    #[test]
    fn version_pin_warning_suppressed_when_original_is_unset() {
        assert!(
            !should_warn_version_mismatch("unset", "1.2.3"),
            "warning must be suppressed when user wrote 'unset'"
        );
    }

    #[test]
    fn version_pin_warning_suppressed_when_original_is_empty() {
        assert!(
            !should_warn_version_mismatch("", ""),
            "warning must be suppressed when original_pin is empty"
        );
    }

    #[test]
    fn version_pin_warning_fires_when_concrete_pin_mismatches_cli() {
        // Use a version that is certainly not the compiled CARGO_PKG_VERSION.
        assert!(
            should_warn_version_mismatch("0.0.1", "0.0.1"),
            "warning must fire when user pinned a concrete version that differs from CLI"
        );
    }
}
