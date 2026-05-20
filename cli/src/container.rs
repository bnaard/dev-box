use anyhow::{Context, Result, bail};
use semver::Version;
use std::path::{Path, PathBuf};

use crate::config::{
    AiHarness, AiProvider, AiboxConfig, AiboxProfile, BaseImage, McpGatewayMode, StarshipPreset,
    ThemeFamily, ThemeMode, TmuxStatusMode,
};
use crate::context;
use crate::generate;
use crate::output;
use crate::runtime::{ContainerState, Runtime};
use crate::seed;

fn default_image_version_for_new_config() -> String {
    "latest".to_string()
}

fn toml_string_list(items: &[String]) -> String {
    items
        .iter()
        .map(|item| {
            let escaped = item.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{escaped}\"")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn toml_string_value(item: &str) -> String {
    let escaped = item.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Parameters for the init command, grouping all optional CLI arguments.
pub struct InitParams {
    pub name: Option<String>,
    pub base: Option<BaseImage>,
    pub profile: Option<AiboxProfile>,
    pub process: Option<Vec<String>>,
    pub ai: Option<Vec<AiProvider>>,
    pub user: Option<String>,
    pub theme: Option<ThemeFamily>,
    pub prompt: Option<StarshipPreset>,
    pub tmux_status: Option<TmuxStatusMode>,
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
///      - Interactive: show a `dialoguer::Select` with a literal "latest"
///        tracking entry as the default, followed by the 10 newest concrete
///        tags. Includes an "unset (skip processkit install)" entry as the
///        escape hatch when the user explicitly wants no install.
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
        let visible_versions = processkit_wizard_visible_versions(&versions);
        // Build the menu with a literal "latest" tracking entry at the top,
        // followed by a bounded set of concrete pin choices and an explicit
        // "skip" escape hatch at the bottom.
        let mut items: Vec<String> = vec!["latest (always track newest)".to_string()];
        items.extend(visible_versions.iter().cloned());
        items.push(format!(
            "{} — skip processkit install (configure later)",
            PROCESSKIT_VERSION_UNSET
        ));
        let idx = dialoguer::Select::new()
            .with_prompt("processkit version")
            .items(&items)
            .default(0)
            .interact()?;
        section.version = selected_processkit_wizard_version(idx, &visible_versions);
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

fn processkit_wizard_visible_versions(versions: &[String]) -> Vec<String> {
    const MAX_CONCRETE_VERSIONS: usize = 10;

    versions
        .iter()
        .filter(|version| crate::content_source::parse_loose_semver(version).is_some())
        .take(MAX_CONCRETE_VERSIONS)
        .cloned()
        .collect()
}

fn selected_processkit_wizard_version(idx: usize, visible_versions: &[String]) -> String {
    if idx == 0 {
        crate::config::PROCESSKIT_VERSION_LATEST.to_string()
    } else if idx == visible_versions.len() + 1 {
        crate::config::PROCESSKIT_VERSION_UNSET.to_string()
    } else {
        visible_versions[idx - 1].clone()
    }
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

fn persist_missing_required_addons(
    config_path: &Option<String>,
    added_required_addons: &[(String, String)],
) -> Result<Vec<String>> {
    if added_required_addons.is_empty() {
        return Ok(Vec::new());
    }

    let toml_path = resolve_aibox_toml_path(config_path);
    if !toml_path.is_file() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("Failed to read {}", toml_path.display()))?;
    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;

    let mut persisted = Vec::new();
    let mut required_addons: Vec<_> = added_required_addons
        .iter()
        .map(|(_, required)| required.clone())
        .collect();
    required_addons.sort();
    required_addons.dedup();

    for required in required_addons {
        let has_required_tools = doc
            .get("addons")
            .and_then(|item| item.get(&required))
            .and_then(|item| item.get("tools"))
            .and_then(|item| item.as_table())
            .is_some();
        if has_required_tools {
            continue;
        }

        if !doc
            .get("addons")
            .and_then(|item| item.as_table())
            .is_some_and(|table| table.contains_key(&required))
        {
            doc["addons"][&required] = toml_edit::table();
        }
        doc["addons"][&required]["tools"] = toml_edit::table();
        persisted.push(required);
    }

    if persisted.is_empty() {
        return Ok(Vec::new());
    }

    std::fs::write(&toml_path, doc.to_string())
        .with_context(|| format!("Failed to write {}", toml_path.display()))?;
    Ok(persisted)
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
pub fn cmd_start(
    config_path: &Option<String>,
    layout: &str,
    forget_tmux_state: bool,
) -> Result<()> {
    let mut config = AiboxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;

    let added_required_addons = complete_missing_required_addons(&mut config);
    if !added_required_addons.is_empty() {
        match persist_missing_required_addons(config_path, &added_required_addons) {
            Ok(persisted) if !persisted.is_empty() => output::ok(&format!(
                "Added required addon section(s) to aibox.toml: {}",
                persisted.join(", ")
            )),
            Ok(_) => {
                for (addon, required) in &added_required_addons {
                    output::warn(&format!(
                        "Addon '{}' requires '{}'; using '{}' for this start.",
                        addon, required, required
                    ));
                }
            }
            Err(e) => output::warn(&format!(
                "Could not persist required addon section(s) to aibox.toml: {}",
                e
            )),
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
    // image label != [container.image].release_version) but have different fixes:
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
        && config.container.image.version != "latest"
        && let Ok(Some(container_version)) = runtime.get_container_image_version(name)
        && container_version != config.container.image.version
    {
        bail!(
            "Version mismatch: the existing container was built from image v{} \
             but aibox.toml pins v{}.\n\n\
             Likely cause: an old container survived an aibox upgrade. Recreate it:\n\
             \n    aibox delete runtime && aibox up\n\n\
             If you have not yet rebuilt the image at the new version, run \
            `aibox apply` first to rebuild it, then the recreate command above.",
            container_version,
            config.container.image.version
        );
    }
    if state != ContainerState::Missing {
        let mount_problems = runtime_home_read_only_mounts(&runtime, &config)?;
        if !mount_problems.is_empty() {
            bail!(
                "Runtime .aibox-home mounts are stale or not writable in the existing container:\n  - {}\n\n\
                 Yazi, Codex, Bash/Starship, PowerKit, and tmux status all require the managed \
                 .config, .cache, and .local runtime-home mounts to be writable. Recreate the \
                 container from the generated compose files:\n\
                 \n    aibox delete runtime && aibox up",
                mount_problems.join("\n  - ")
            );
        }
    }

    let session_name = config.tmux_session_name();
    let should_recreate_tmux_session = forget_tmux_state || state != ContainerState::Running;
    let refreshed_runtime_files = crate::seed::sync_theme_files(&config)?;
    if !refreshed_runtime_files.is_empty() {
        output::ok(&format!(
            "Refreshed {} managed runtime file(s)",
            refreshed_runtime_files.len()
        ));
    }
    if state != ContainerState::Missing {
        let restored_runtime_files = crate::seed::restore_missing_managed_runtime_files(&config)?;
        if !restored_runtime_files.is_empty() {
            output::ok(&format!(
                "Restored {} missing managed runtime file(s)",
                restored_runtime_files.len()
            ));
        }
    }
    if should_recreate_tmux_session && state != ContainerState::Missing {
        let refreshed_tmux_files = crate::tmux::sync_tmux_runtime_files(&config)?;
        if !refreshed_tmux_files.is_empty() {
            output::ok(&format!(
                "Refreshed {} managed tmux runtime file(s)",
                refreshed_tmux_files.len()
            ));
        }
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

    let diagnostics_service = format!("{name}-diagnostics");
    if let Err(error) =
        runtime.compose_up_no_deps(crate::config::COMPOSE_FILE, &diagnostics_service)
    {
        output::warn(&format!(
            "Diagnostics sidecar could not be started: {error}. Continuing with main container attach."
        ));
    }

    output::info(&format!("Attaching via tmux (layout: {})...", layout));

    // When explicitly requested or after starting a non-running container, discard
    // stale tmux session state so the configured managed layout wins. Plain
    // re-entry into a running container preserves the user's live session.
    if should_recreate_tmux_session {
        let kill_cmd = tmux_kill_session_command(&session_name);
        let kill_args: Vec<&str> = kill_cmd.iter().map(|arg| arg.as_str()).collect();
        let _ = runtime.exec_status(name, &config.container.user, &kill_args);
    }

    let attach_cmd = tmux_attach_command(layout, &session_name, should_recreate_tmux_session);
    let attach_args: Vec<&str> = attach_cmd.iter().map(|arg| arg.as_str()).collect();
    runtime.exec_interactive(name, &config.container.user, &attach_args)?;

    Ok(())
}

fn tmux_attach_command(layout: &str, session_name: &str, _recreate_session: bool) -> Vec<String> {
    vec![
        "aibox-tmux-session".to_string(),
        layout.to_string(),
        session_name.to_string(),
    ]
}

fn tmux_kill_session_command(session_name: &str) -> Vec<String> {
    vec![
        "sh".to_string(),
        "-lc".to_string(),
        r#"socket="${AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}"; tmux -S "$socket" kill-session -t "$1" >/dev/null 2>&1 || true"#.to_string(),
        "aibox-tmux-kill".to_string(),
        session_name.to_string(),
    ]
}

fn runtime_home_read_only_mounts(runtime: &Runtime, config: &AiboxConfig) -> Result<Vec<String>> {
    let mounts = runtime.get_container_mounts(&config.container.name)?;
    if mounts.is_empty() {
        return Ok(Vec::new());
    }

    let mut problems = Vec::new();
    for destination in crate::runtime_home::writable_runtime_home_destinations(config) {
        match mounts.iter().find(|mount| mount.destination == destination) {
            Some(mount) if !mount.rw => {
                problems.push(format!(
                    "{} from {} is read-only",
                    mount.destination, mount.source
                ));
            }
            Some(_) => {}
            None => {
                problems.push(format!(
                    "{destination} is not mounted from .aibox-home (legacy or shadowed runtime layout)"
                ));
            }
        }
    }
    for destination in crate::runtime_home::legacy_runtime_home_destinations(config) {
        if let Some(mount) = mounts.iter().find(|mount| mount.destination == destination)
            && !mount.rw
        {
            problems.push(format!(
                "{} from {} is read-only legacy runtime-home mount",
                mount.destination, mount.source
            ));
        }
    }
    Ok(problems)
}

pub fn cmd_emergency(config_path: &Option<String>, harness: AiHarness) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    match runtime.container_status(name)? {
        ContainerState::Running => {
            output::info("Container already running");
        }
        state @ (ContainerState::Stopped | ContainerState::Missing) => {
            let action = if state == ContainerState::Stopped {
                "Starting stopped"
            } else {
                "Creating and starting"
            };
            output::info(&format!("{} main container...", action));
            runtime.compose_up(crate::config::COMPOSE_FILE, name)?;
            runtime.wait_for_running(name, 7500)?;
            output::ok("Container started");
        }
    }

    let home = config.container_home();
    let briefing_path = format!("{home}/.cache/aibox/emergency-briefing.md");
    let briefing = emergency_briefing(&config, &harness, &briefing_path);
    let write_script = format!(
        "umask 077; mkdir -p {}; printf '%s' {} > {}",
        shell_single_quote(&format!("{home}/.cache/aibox")),
        shell_single_quote(&briefing),
        shell_single_quote(&briefing_path)
    );
    if !runtime.exec_status(name, &config.container.user, &["sh", "-lc", &write_script])? {
        output::warn("Could not write emergency briefing inside the container; printing inline");
    }

    output::info(&format!(
        "Launching emergency session for {} without tmux...",
        harness.display_name()
    ));
    let launch_script = emergency_launch_script(&harness, &briefing_path, &briefing);
    runtime.exec_interactive(
        name,
        &config.container.user,
        &["bash", "-lc", &launch_script],
    )?;

    Ok(())
}

fn emergency_briefing(config: &AiboxConfig, harness: &AiHarness, briefing_path: &str) -> String {
    format!(
        r#"# aibox emergency recovery briefing

You are running inside the main aibox container `{container}` via `aibox emergency {harness}`.
This session intentionally bypassed tmux, Yazi, and aibox status tooling.

First checks:
- Read recent aibox logs: `/workspace/.aibox/aibox.log` and rotated `/workspace/.aibox/aibox.log.*` files if present.
- Inspect runtime diagnostics snapshots if available: `/workspace/.aibox/diagnostics/` and `/tmp/aibox-diagnostics/`.
- Check process pressure: `cat /sys/fs/cgroup/pids.current /sys/fs/cgroup/pids.max`; inspect `/proc` for process counts and zombies.
- Check memory pressure: `cat /sys/fs/cgroup/memory.current /sys/fs/cgroup/memory.max /sys/fs/cgroup/memory.events`.
- Inspect tmux only as evidence: `{home}/.config/tmux`, `{home}/.tmux`, and running `tmux` processes.
- Avoid launching tmux, Yazi, or status helpers until the runtime is stable.

Workspace: `/workspace`
Container home: `{home}`
Selected harness: `{display_name}`
Harness binary: `{binary}`
Briefing file: `{briefing_path}`
"#,
        container = config.container.name,
        harness = harness,
        home = config.container_home(),
        display_name = harness.display_name(),
        binary = harness.binary_name(),
        briefing_path = briefing_path,
    )
}

fn emergency_launch_script(harness: &AiHarness, briefing_path: &str, briefing: &str) -> String {
    let binary = harness.binary_name();
    format!(
        r#"if [ -r {briefing_path} ]; then
  printf '\n'
  sed -n '1,220p' {briefing_path}
  printf '\n'
else
  printf '\nEmergency briefing was not found at %s; printing fallback briefing.\n\n' {briefing_path}
  printf '%s\n' {briefing}
fi

if command -v {binary} >/dev/null 2>&1; then
  resolved="$(command -v {binary})"
  printf 'Launching %s at %s. Briefing: %s\n\n' {binary} "$resolved" {briefing_path}
  exec {binary}
fi

printf 'Harness binary %s is not available in this container. Briefing: %s\n' {binary} {briefing_path}
printf 'Starting a plain recovery shell instead.\n\n'
exec "${{SHELL:-/bin/bash}}"
"#,
        briefing_path = shell_single_quote(briefing_path),
        briefing = shell_single_quote(briefing),
        binary = shell_single_quote(binary),
    )
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
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
        "release_version = \"{}\"       # Target base image version. Use \"latest\" to resolve newest published image on apply.\n",
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
        out.push_str("keepalive = true               # Send periodic DNS keepalive for idle network timeouts\n");
    } else {
        out.push_str("# keepalive           = true           # Send periodic DNS keepalive for idle network timeouts\n");
    }
    out.push_str("\n# --- Resource pressure warnings (`aibox doctor`) ---\n");
    out.push_str("# [container.resource_thresholds]\n");
    out.push_str(
        "# memory_mib_warn = 4096       # Optional warning limit for cgroup memory usage in MiB\n",
    );
    out.push_str(
        "# process_count_warn = 400     # Optional warning limit for total live processes; 0 disables\n",
    );
    out.push_str(
        "# processkit_mcp_python_warn = 50  # Optional warning limit for live Python MCP server processes; 0 disables\n",
    );
    out.push_str(
        "# oom_kill_warn = 0            # Optional warning threshold for cgroup OOM kill count\n",
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
    out.push_str("# Each known skill appears once. Uncomment a line to enable that skill;\n");
    out.push_str("# comment it out (leading `#`) to disable. Core skills are always\n");
    out.push_str("# installed; disabling one only triggers a doctor warning.\n");
    out.push_str("[skills]\n");
    let skill_catalog = skill_catalog_entries_for_comments(config);
    render_skill_array(&mut out, "enabled", &config.skills.include, &skill_catalog);

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
    out.push_str("# Harnesses are configured by the ordered `harnesses` list below.\n");
    out.push_str("# The list order is the tmux/layout order: 1st, 2nd, 3rd harness.\n");
    out.push_str("#\n");
    out.push_str(
        "# Model providers (optional): declare which API key/base URL env vars are available.\n",
    );
    out.push_str("# Provider     Config value   API key env         Base URL env\n");
    out.push_str("# Anthropic    anthropic      ANTHROPIC_API_KEY   ANTHROPIC_BASE_URL\n");
    out.push_str("# OpenAI       openai         OPENAI_API_KEY      OPENAI_BASE_URL\n");
    out.push_str("# Google       google         GEMINI_API_KEY      GEMINI_BASE_URL\n");
    out.push_str("# Mistral      mistral        MISTRAL_API_KEY     MISTRAL_BASE_URL\n");
    out.push_str("#\n");
    out.push_str("# Alias used by some tools: OPENAI_API_BASE (OpenAI).\n");
    out.push_str("[ai]\n");
    render_ai_model_provider_catalog(&mut out, &config.ai.model_providers);
    render_ai_harness_detail_catalog(&mut out, config);
    render_ai_execution_section(&mut out, config);

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

    // [customization] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [customization] — color theme, shell prompt, and tmux layout\n");
    out.push_str(sep);
    out.push_str("# Theme is applied consistently across tmux, Vim, Yazi, lazygit, and bat.\n");
    out.push_str("# Theme families (31 total):\n");
    out.push_str("#   Multi-variant: ayu, catppuccin, dracula, everforest, github, gruvbox,\n");
    out.push_str("#     kanagawa, material, min, night-owl, one-dark, rose-pine, slack,\n");
    out.push_str("#     solarized, tokyo-night, vitesse, vscode\n");
    out.push_str("#   Solo (mode ignored): andromeeda, aurora-x, houston, laserwave,\n");
    out.push_str("#     monokai, moonlight, nord, plastic, poimandres, projectious, red,\n");
    out.push_str("#     snazzy, synthwave-84, vesper\n");
    out.push_str("[customization]\n");
    // Always emit the family form. If the user had a legacy concrete name and
    // has not yet run `aibox apply --standardize-config`, the family is already
    // correct (the deserializer derived it). The legacy lock is runtime-only.
    out.push_str(&format!("theme  = \"{}\"\n", config.customization.theme));
    out.push_str("# Light/dark variant. `auto` follows host OS appearance when detectable.\n");
    out.push_str("# Solo families (see list above) ignore mode.\n");
    out.push_str("# Options: auto | light | dark\n");
    out.push_str(&format!("mode   = \"{}\"\n", config.customization.mode));
    out.push_str("# Optional alternate variant override (per family). Default = unset.\n");
    out.push_str("#   ayu: \"mirage\"           catppuccin: \"macchiato\" | \"frappe\"\n");
    out.push_str(
        "#   dracula: \"soft\"         github: \"dimmed\" | \"high-contrast-dark\" | \"high-contrast-light\"\n",
    );
    out.push_str(
        "#   kanagawa: \"dragon\"      material: \"ocean\" | \"palenight\" | \"darker\"\n",
    );
    out.push_str("#   rose-pine: \"moon\"       slack: \"ochin\"\n");
    out.push_str("#   tokyo-night: \"storm\"    vitesse: \"black\"\n");
    if let Some(ref v) = config.customization.variant {
        out.push_str(&format!("variant = \"{v}\"\n"));
    } else {
        out.push_str("# variant = \"<name>\"\n");
    }
    out.push_str("# Starship prompt preset.\n");
    out.push_str("# Options: default | plain | minimal | nerd-font | pastel | powerline-pastel | bracketed | arrow\n");
    out.push_str("# ASCII sketches:\n");
    out.push_str("#   default          ~/repo main +2  py3.13  2s\n");
    out.push_str("#                    >\n");
    out.push_str("#   plain            ~/repo main +2\n");
    out.push_str("#                    >\n");
    out.push_str("#   minimal          ~/repo main\n");
    out.push_str("#                    >\n");
    out.push_str("#   nerd-font        [os] ~/repo main +2 py rs js go 2s\n");
    out.push_str("#                    >\n");
    out.push_str("#   pastel           ( ~/repo )>( main +2 )>( py rs js go ) 2s >\n");
    out.push_str("#   powerline-pastel ( ~/repo )>( main +2 )>( py rs js go ) 2s >\n");
    out.push_str("#   bracketed        ~/repo [main] [+2] [py3.13]\n");
    out.push_str("#                    >\n");
    out.push_str("#   arrow            > ~/repo > main +2 > 2s\n");
    out.push_str("#                    >\n");
    out.push_str(&format!("prompt = \"{}\"\n", config.customization.prompt));
    out.push_str("# Default tmux layout. Options: dev | focus | cowork | ai\n");
    out.push_str("# Layout sketches, one screen each:\n");
    out.push_str("# +---- ai ----+  +--- dev ----+  +-- focus --+  +-- cowork -+\n");
    out.push_str("# |files|ai1  |  |files|shell|  |  files     |  |files|shell|\n");
    out.push_str("# |     |     |  |-----|     |  +-----------+  |     |     |\n");
    out.push_str("# |     |     |  |ai1  |     |  ai1 ai2 ... |  +-----------+\n");
    out.push_str("# +-----------+  +-----------+  +-----------+  ai1 ai2 ...\n");
    out.push_str(
        "# Extra windows: ai holds additional harnesses; lazygit and shell open when enabled.\n",
    );
    out.push_str(&format!("layout = \"{}\"\n", config.customization.layout));
    out.push('\n');
    out.push_str(
        "# tmux runtime options. `layout` may override [customization].layout for tmux only.\n",
    );
    out.push_str("[customization.tmux]\n");
    out.push_str(&format!(
        "prefix = \"{}\"\n",
        config.customization.tmux.prefix
    ));
    out.push_str(&format!(
        "session_name = \"{}\"\n",
        config.tmux_session_name()
    ));
    out.push('\n');
    out.push_str("# tmux status presentation.\n");
    out.push_str("# - extended: full aibox themed status (legacy alias: powerline)\n");
    out.push_str("# - plain: minimal tmux-native status text\n");
    out.push_str("# - disabled: turn the tmux status line off\n");
    out.push_str("[customization.tmux.status]\n");
    out.push_str(&format!(
        "mode = \"{}\"  # extended | plain | disabled (legacy: powerline -> extended)\n",
        config.customization.tmux.status.mode
    ));
    out.push('\n');
    out.push_str("[customization.tmux.status.layout]\n");
    out.push_str("# Row lists are ordered. Removing a name disables that status element.\n");
    out.push_str("# Allowed line1-left entries:\n");
    out.push_str("# - session: current tmux session name and prefix/copy-mode state\n");
    out.push_str("# - windows: tmux window list\n");
    out.push_str("#\n");
    out.push_str("# Allowed line1-right / line2-left / line2-right entries:\n");
    out.push_str("# - aibox_log: aibox log health counts\n");
    out.push_str("# - aibox_oom: cgroup OOM kill counters\n");
    out.push_str("# - aibox_proc: live process count versus configured process warning limit\n");
    out.push_str("# - aibox_ai: detected AI-agent/runtime process count\n");
    out.push_str("# - aibox_mcp: processkit/MCP daemon and server process status\n");
    out.push_str("# - aibox_mig: pending processkit migration count\n");
    out.push_str("# - weather: weather segment from tmux-powerkit\n");
    out.push_str("# - uptime: container uptime\n");
    out.push_str("# - datetime: local date/time\n");
    out.push_str("# - git: current repository branch/status\n");
    out.push_str("# - github: GitHub/repository integration status\n");
    out.push_str("# - kubernetes: Kubernetes context/status\n");
    out.push_str("# - terraform: Terraform/OpenTofu workspace/status\n");
    out.push_str("# - cloud: local cloud CLI/context status\n");
    out.push_str(
        "# - cloudstatus: networked public provider status checks; opt-in, not enabled by default\n",
    );
    out.push_str("# - hostname: container hostname\n");
    out.push_str("# - externalip: detected external IP\n");
    out.push_str("# - ssh: SSH agent/key status\n");
    out.push_str("# - netspeed: network throughput\n");
    out.push_str("# - ping: network latency\n");
    out.push_str("# - cpu: CPU usage\n");
    out.push_str("# - loadavg: system load average\n");
    out.push_str("# - memory: memory usage\n");
    out.push_str("# - swap: swap usage\n");
    out.push_str("# - disk: disk usage\n");
    out.push_str("# - gpu: GPU status when available\n");
    out.push_str("# - modelstatus_<provider>: per-provider AI status segment; explicit layout entries render even when model-provider auto-add is off\n");
    let layout = crate::tmux::resolved_tmux_status_layout(config);
    out.push_str(&format!(
        "line1-left = [{}]\n",
        toml_string_list(&layout.line1_left)
    ));
    out.push_str(&format!(
        "line1-right = [{}]\n",
        toml_string_list(&layout.line1_right)
    ));
    out.push_str(&format!(
        "line2-left = [{}]\n",
        toml_string_list(&layout.line2_left)
    ));
    out.push_str(&format!(
        "line2-right = [{}]\n",
        toml_string_list(&layout.line2_right)
    ));
    out.push('\n');
    out.push_str("[customization.tmux.status.labels]\n");
    out.push_str(
        "# Visible headers/icons for status segments. Layout controls which segments appear;\n",
    );
    out.push_str("# this section controls how those segments are labeled once rendered.\n");
    out.push_str(
        "# Values may be plain ASCII labels or symbols. ASCII is safest across terminals;\n",
    );
    out.push_str(
        "# Nerd Font / Powerline symbols are compact but require the user's terminal font.\n",
    );
    out.push_str("# Practical symbol candidates from Nerd Fonts. Keep icons distinct across\n");
    out.push_str("# configured PowerKit segments so adjacent status cells remain scannable.\n");
    out.push_str("# aibox-log: aibox log info/warn/error counter header.\n");
    out.push_str("# aibox-oom: cgroup OOM event/kill counter header.\n");
    out.push_str("# aibox-proc: process/thread count header.\n");
    out.push_str("# aibox-ai: active AI-agent process count header.\n");
    out.push_str("# aibox-mcp: processkit/MCP topology header.\n");
    out.push_str("# aibox-mig: pending processkit migration count header.\n");
    out.push_str("# kubernetes: Kubernetes segment icon/header.\n");
    out.push_str(
        "# cloud/cloud-aws/cloud-gcp/cloud-azure/cloud-multi: local cloud context icons.\n",
    );
    out.push_str("# uptime: container uptime icon/header.\n");
    out.push_str("# netspeed/netspeed-download/netspeed-upload: network throughput icons.\n");
    let labels = &config.customization.tmux.status.labels;
    out.push_str(&format!(
        "aibox-log = {}\n",
        toml_string_value(&labels.aibox_log)
    ));
    out.push_str(&format!(
        "aibox-oom = {}\n",
        toml_string_value(&labels.aibox_oom)
    ));
    out.push_str(&format!(
        "aibox-proc = {}\n",
        toml_string_value(&labels.aibox_proc)
    ));
    out.push_str(&format!(
        "aibox-ai = {}\n",
        toml_string_value(&labels.aibox_ai)
    ));
    out.push_str(&format!(
        "aibox-mcp = {}\n",
        toml_string_value(&labels.aibox_mcp)
    ));
    out.push_str(&format!(
        "aibox-mig = {}\n",
        toml_string_value(&labels.aibox_mig)
    ));
    out.push_str(&format!(
        "kubernetes = {}\n",
        toml_string_value(&labels.kubernetes)
    ));
    out.push_str(&format!("cloud = {}\n", toml_string_value(&labels.cloud)));
    out.push_str(&format!(
        "cloud-aws = {}\n",
        toml_string_value(&labels.cloud_aws)
    ));
    out.push_str(&format!(
        "cloud-gcp = {}\n",
        toml_string_value(&labels.cloud_gcp)
    ));
    out.push_str(&format!(
        "cloud-azure = {}\n",
        toml_string_value(&labels.cloud_azure)
    ));
    out.push_str(&format!(
        "cloud-multi = {}\n",
        toml_string_value(&labels.cloud_multi)
    ));
    out.push_str(&format!("uptime = {}\n", toml_string_value(&labels.uptime)));
    out.push_str(&format!(
        "netspeed = {}\n",
        toml_string_value(&labels.netspeed)
    ));
    out.push_str(&format!(
        "netspeed-download = {}\n",
        toml_string_value(&labels.netspeed_download)
    ));
    out.push_str(&format!(
        "netspeed-upload = {}\n",
        toml_string_value(&labels.netspeed_upload)
    ));
    out.push('\n');
    out.push_str("[customization.tmux.status.separators]\n");
    out.push_str("# PowerKit separator style. Options: normal | rounded | slant | slantup | trapezoid | flame | pixel | honeycomb | none\n");
    out.push_str(&format!(
        "style = \"{}\"\n",
        config.customization.tmux.status.separators.style
    ));
    out.push_str("# Edge separators may use a different style at status boundaries.\n");
    out.push_str(&format!(
        "edge-style = \"{}\"\n",
        config.customization.tmux.status.separators.edge_style
    ));
    out.push_str("# Spacing between elements. Options: false | true | both | windows | plugins\n");
    out.push_str(&format!(
        "elements-spacing = \"{}\"\n",
        config.customization.tmux.status.separators.elements_spacing
    ));
    out.push('\n');
    out.push_str("[customization.tmux.status.refresh]\n");
    out.push_str("# Refresh/caching controls for extended tmux status.\n");
    out.push_str(
        "# interval-seconds: tmux redraw cadence. Higher values reduce shell process churn.\n",
    );
    out.push_str("# aibox-metrics-cache-ttl-seconds: LOG/OOM/PROC/AI/MCP/MIG cache TTL.\n");
    out.push_str("#   These metrics are useful but not worth refreshing every redraw; a 30s TTL\n");
    out.push_str(
        "#   still surfaces runtime problems quickly while avoiding repeated aibox-status calls.\n",
    );
    out.push_str(
        "# netspeed-cache-ttl-seconds: network throughput cache TTL. Keep near the redraw\n",
    );
    out.push_str("#   cadence if you want live-ish rates; increase for quieter laptops.\n");
    out.push_str(
        "# kubernetes-cache-ttl-seconds: local kubeconfig context cache TTL; this should\n",
    );
    out.push_str("#   not poll live clusters and does not need second-level freshness.\n");
    out.push_str("# cloud-cache-ttl-seconds: local cloud CLI/context cache TTL; this avoids auth/network probes.\n");
    out.push_str("# github-cache-ttl-seconds: local repo + GitHub issue/PR count cache TTL.\n");
    let refresh = &config.customization.tmux.status.refresh;
    out.push_str(&format!(
        "interval-seconds = {}\n",
        refresh.interval_seconds
    ));
    out.push_str(&format!(
        "aibox-metrics-cache-ttl-seconds = {}\n",
        refresh.aibox_metrics_cache_ttl_seconds
    ));
    out.push_str(&format!(
        "netspeed-cache-ttl-seconds = {}\n",
        refresh.netspeed_cache_ttl_seconds
    ));
    out.push_str(&format!(
        "kubernetes-cache-ttl-seconds = {}\n",
        refresh.kubernetes_cache_ttl_seconds
    ));
    out.push_str(&format!(
        "cloud-cache-ttl-seconds = {}\n",
        refresh.cloud_cache_ttl_seconds
    ));
    out.push_str(&format!(
        "github-cache-ttl-seconds = {}\n",
        refresh.github_cache_ttl_seconds
    ));
    out.push('\n');
    out.push_str("[customization.tmux.status.model-providers]\n");
    out.push_str(
        "# Optional networked model-provider health segments for the extended tmux status line.\n",
    );
    out.push_str("# Each configured provider becomes one PowerKit segment when enabled, for example OAI ✓ or ANT 󰚌.\n");
    out.push_str("# enabled: false avoids auto-adding all configured providers; explicit layout entries still render.\n");
    out.push_str(
        "# cache-ttl-seconds: minimum time between provider status requests per provider.\n",
    );
    out.push_str(
        "# timeout-seconds: per-request HTTP timeout so status rendering cannot hang tmux.\n",
    );
    out.push_str("# show-ok: true shows healthy providers with ✓; false hides healthy providers and only shows degraded/unknown/outage.\n");
    out.push_str(&format!(
        "enabled = {}\n",
        config.customization.tmux.status.model_providers.enabled
    ));
    out.push_str(&format!(
        "cache-ttl-seconds = {}\n",
        config
            .customization
            .tmux
            .status
            .model_providers
            .cache_ttl_seconds
    ));
    out.push_str(&format!(
        "timeout-seconds = {}\n",
        config
            .customization
            .tmux
            .status
            .model_providers
            .timeout_seconds
    ));
    out.push_str(&format!(
        "show-ok = {}\n",
        config.customization.tmux.status.model_providers.show_ok
    ));
    out.push_str("# Provider entries:\n");
    out.push_str("# - provider: stable key from the model roster (openai, anthropic, google, mistral, deepseek, cohere, xai, alibaba, aws, meta, microsoft, minimax, moonshot, nvidia, xiaomi, zai)\n");
    out.push_str("# - label: short category header shown in the status segment; use text or a symbol that your font supports\n");
    out.push_str("# - checks: any of overall, models, harness; worst status wins (outage > degraded > unknown > ok)\n");
    out.push_str("# - status-url: JSON status endpoint; Statuspage summary APIs are supported, Google uses incidents.json\n");
    out.push_str("# - overall-components/model-components/harness-components: optional component-name filters for providers with componentized status APIs\n");
    out.push_str("#   Symbols: ✓ ok, 󰀦 degraded, 󰚌 outage, ? unknown.\n");
    for provider in &config.customization.tmux.status.model_providers.providers {
        out.push_str("\n[[customization.tmux.status.model-providers.providers]]\n");
        out.push_str(&format!(
            "provider = {}\n",
            toml_string_value(&provider.provider)
        ));
        out.push_str(&format!("label = {}\n", toml_string_value(&provider.label)));
        out.push_str(&format!(
            "checks = [{}]\n",
            toml_string_list(&provider.checks)
        ));
        if let Some(status_url) = &provider.status_url {
            out.push_str(&format!("status-url = {}\n", toml_string_value(status_url)));
        } else {
            out.push_str("# status-url intentionally omitted: no stable public JSON status API is configured yet\n");
        }
        if !provider.overall_components.is_empty() {
            out.push_str(&format!(
                "overall-components = [{}]\n",
                toml_string_list(&provider.overall_components)
            ));
        }
        if !provider.model_components.is_empty() {
            out.push_str(&format!(
                "model-components = [{}]\n",
                toml_string_list(&provider.model_components)
            ));
        }
        if !provider.harness_components.is_empty() {
            out.push_str(&format!(
                "harness-components = [{}]\n",
                toml_string_list(&provider.harness_components)
            ));
        }
    }

    // [security] section — only emitted when non-default (avoid noise in normal projects)
    if config.security.acknowledge_seccomp_unconfined {
        out.push('\n');
        out.push_str(sep);
        out.push_str("# [security] — explicit consent for security-sensitive runtime options\n");
        out.push_str(sep);
        out.push_str("[security]\n");
        out.push_str(
            "# Codex bubblewrap sandboxing requires seccomp=unconfined in docker-compose.yml.\n",
        );
        out.push_str(
            "# Set to true to acknowledge the trade-off and allow `aibox apply` to emit it.\n",
        );
        out.push_str(&format!(
            "acknowledge_seccomp_unconfined = {}\n",
            config.security.acknowledge_seccomp_unconfined
        ));
    }

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
            if let Some(mode) = &override_cfg.default_mode {
                out.push_str(&format!("default_mode = \"{}\"\n", mode));
            } else {
                out.push_str(
                    "# default_mode = \"ask\"  # optional harness-specific default override\n",
                );
            }
            out.push_str(&format!(
                "allow_patterns = {}\n",
                toml_string_array(&override_cfg.allow_patterns)
            ));
            out.push_str(&format!(
                "deny_patterns  = {}\n",
                toml_string_array(&override_cfg.deny_patterns)
            ));
        }
    }
    out.push('\n');
    out.push_str("# [ai.mcp.gateway] — processkit MCP topology. Options for mode: auto | daemon | stdio | separate\n");
    out.push_str("[ai.mcp.gateway]\n");
    out.push_str(&format!(
        "mode = \"{}\"          # auto uses daemon when processkit-gateway is installed\n",
        mcp_gateway_mode_str(config.ai.mcp.gateway.mode)
    ));
    out.push_str(&format!(
        "lazy_catalog = {}    # Use processkit's lazy catalog where supported\n",
        config.ai.mcp.gateway.lazy_catalog
    ));
    out.push_str(&format!(
        "host = \"{}\"     # daemon is always localhost-only\n",
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
        McpGatewayMode::Granular => "separate",
        McpGatewayMode::Stdio => "stdio",
        McpGatewayMode::DaemonProxy => "daemon",
        McpGatewayMode::Aggregate => "aggregate",
        McpGatewayMode::LazyAggregate => "lazy-aggregate",
    }
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
        out.push_str(&format!(
            " # env: {}, {}\n",
            provider.api_key_env(),
            provider.endpoint_env()
        ));
    }
    out.push_str("]\n");
}

fn render_ai_harness_detail_catalog(out: &mut String, config: &AiboxConfig) {
    out.push_str("\n# Ordered harness list. Supported harness values:\n");
    out.push_str("# claude, codex, gemini, aider, continue, cursor, copilot, opencode, hermes.\n");
    out.push_str(
        "# Each one-line entry is directly uncommentable; list order is tmux/layout order.\n",
    );
    out.push_str("# `enable = true` includes the harness in generated agent/MCP/runtime config.\n");
    out.push_str(
        "# `install = true` installs the matching in-container CLI recipe when one exists.\n",
    );
    out.push_str("# Defaults for both are false when omitted. Cursor has no container CLI, so\n");
    out.push_str("# keep `install = false` for cursor even when `enable = true`.\n");
    out.push_str("# `version` optionally pins the CLI recipe; omit it to use the addon default.\n");
    out.push_str("harnesses = [\n");
    let mut rendered = Vec::new();
    for harness in &config.ai.harnesses {
        if !rendered.contains(harness) {
            render_ai_harness_detail_catalog_entry(out, config, harness);
            rendered.push(harness.clone());
        }
    }
    for harness in crate::config::AiHarness::all() {
        if rendered.contains(harness) {
            continue;
        }
        render_ai_harness_detail_catalog_entry(out, config, harness);
    }
    out.push_str("]\n");
}

fn render_ai_execution_section(out: &mut String, config: &AiboxConfig) {
    out.push_str("\n# AI harness execution policy. These are aibox-level intent settings;\n");
    out.push_str("# aibox maps them to each harness where supported.\n");
    out.push_str("# Optional per-harness overrides use `[ai.execution.<harness>]`.\n");
    out.push_str("# filesystem: read-only | workspace-write | container-full\n");
    out.push_str("# approval:   ask | on-request | never\n");
    out.push_str("# network:    deny | ask | allow\n");
    out.push_str("[ai.execution]\n");
    out.push_str(&format!(
        "filesystem = \"{}\"\n",
        config.ai.execution.filesystem
    ));
    out.push_str(&format!(
        "approval   = \"{}\"\n",
        config.ai.execution.approval
    ));
    out.push_str(&format!(
        "network    = \"{}\"\n",
        config.ai.execution.network
    ));

    out.push_str("\n# Per-harness execution overrides. Uncomment a section and only the axes\n");
    out.push_str("# you want to override; omitted axes inherit `[ai.execution]`.\n");
    let mut rendered = Vec::new();
    for harness in &config.ai.harnesses {
        render_ai_harness_execution_override(out, config, harness, &mut rendered);
    }
    for harness in crate::config::AiHarness::all() {
        render_ai_harness_execution_override(out, config, harness, &mut rendered);
    }
}

fn render_ai_harness_execution_override(
    out: &mut String,
    config: &AiboxConfig,
    harness: &crate::config::AiHarness,
    rendered: &mut Vec<crate::config::AiHarness>,
) {
    if rendered.contains(harness) {
        return;
    }
    rendered.push(harness.clone());

    if let Some(execution) = config
        .ai
        .harness
        .get(harness)
        .and_then(|harness_config| harness_config.execution.as_ref())
    {
        out.push('\n');
        out.push_str(&format!("[ai.execution.{}]\n", harness));
        if let Some(filesystem) = execution.filesystem {
            out.push_str(&format!("filesystem = \"{}\"\n", filesystem));
        }
        if let Some(approval) = execution.approval {
            out.push_str(&format!("approval   = \"{}\"\n", approval));
        }
        if let Some(network) = execution.network {
            out.push_str(&format!("network    = \"{}\"\n", network));
        }
    } else {
        out.push('\n');
        out.push_str(&format!("# [ai.execution.{}]\n", harness));
        out.push_str("# filesystem = \"workspace-write\"\n");
        out.push_str("# approval   = \"on-request\"\n");
        out.push_str("# network    = \"ask\"\n");
    }
}

fn render_ai_harness_detail_catalog_entry(
    out: &mut String,
    config: &AiboxConfig,
    harness: &crate::config::AiHarness,
) {
    let selected = config.ai.harnesses.contains(harness);
    let install = config.ai.harness_install_enabled(harness) && !harness.addon_name().is_empty();
    let version = ai_harness_version_for_render(config, harness);
    if selected {
        out.push_str("    { ");
        out.push_str(&format!(
            "harness = \"{}\", enable = true, install = {}",
            harness, install
        ));
        if let Some(version) = version {
            out.push_str(&format!(", version = \"{}\"", version));
        }
        out.push_str(" },\n");
    } else {
        out.push_str(&format!(
            "#   {{ harness = \"{}\", enable = true, install = {} }},\n",
            harness, install
        ));
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
                "    # \"{}\", # {}\n",
                entry.name,
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
    let purpose = short_comment(&tool.description);
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
    let details =
        format!("{default}; options: {{}}, {{ enabled = true|false }}, {{ {version_options} }}");
    if purpose.is_empty() {
        details
    } else {
        format!("{purpose}; {details}")
    }
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
    let tmux_status_mode = match params.tmux_status {
        Some(mode) => mode,
        None if interactive => {
            let labels = [
                "extended — themed tmux bar with aibox runtime status (recommended)",
                "plain — minimal tmux-native status text",
                "disabled — tmux status line off",
            ];
            let modes = [
                TmuxStatusMode::Extended,
                TmuxStatusMode::Plain,
                TmuxStatusMode::Disabled,
            ];
            let idx = dialoguer::Select::new()
                .with_prompt("tmux status")
                .items(&labels)
                .default(0)
                .interact()?;
            modes[idx].clone()
        }
        None => TmuxStatusMode::default(),
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
            version: default_image_version_for_new_config(),
            base: resolved.base_image.clone(),
            profile: resolved.profile,
        },
        image: ImageSection {
            version: default_image_version_for_new_config(),
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
                version: default_image_version_for_new_config(),
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
            harness_order: Vec::new(),
            model_providers: Vec::new(),
            execution: crate::config::AiExecutionPolicy::default(),
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
            variant: None,
            prompt: params.prompt.unwrap_or_default(),
            layout: crate::config::ConfigLayout::default(),
            tmux: crate::config::TmuxSection {
                status: crate::config::TmuxStatusSection {
                    mode: tmux_status_mode,
                    ..crate::config::TmuxStatusSection::default()
                },
                ..crate::config::TmuxSection::default()
            },
            legacy_theme: None,
        },
        agents: crate::config::AgentsSection::default(),
        audio: AudioSection::default(),
        apply: crate::config::ApplySection::default(),
        mcp: crate::config::McpSection::default(),
        // S5 — BR-SEC-HARDEN: Codex consent is plumbed in after struct init
        // (ai_providers has been moved into config.ai.harnesses by this point).
        security: crate::config::SecuritySection::default(),
        local_env: std::collections::HashMap::new(),
        local_mcp_servers: vec![],
    };
    config.resolve_ai_provider_addons();
    // Resolve tmux session name from project name (and sync other grouped sections).
    config.migrate_legacy_sections();

    // S5 — BR-SEC-HARDEN: when Codex is selected during `init`, automatically
    // set acknowledge_seccomp_unconfined = true.  The user has consciously
    // chosen Codex as a harness, so they're implicitly accepting the bubblewrap
    // user-namespace seccomp fallback that ships with it.  This avoids an
    // immediate error on first `aibox apply` and keeps the consent documented
    // in aibox.toml for git history.
    if config
        .ai
        .harnesses
        .contains(&crate::config::AiHarness::Codex)
    {
        config.security.acknowledge_seccomp_unconfined = true;
    }

    config.validate()?;

    // --- summary page ---
    if interactive {
        println!();
        output::info("Configuration summary:");
        println!("  Project:     {}", config.container.name);
        println!("  Base:        {}", config.container.image.base);
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
            // Merge processkit's preauth.json into enabled harness settings.
            // Best-effort and provider-scoped: Codex-only projects should not
            // create or warn about Claude-specific config files.
            if let Err(e) =
                crate::preauth::merge_processkit_preauth_for_config(&project_root, &config)
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
            harnesses: None,
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
/// Warn (once, to stderr) when `aibox.toml` uses the legacy `powerline`
/// alias for `[customization.tmux.status] mode`.
///
/// The alias still works (it maps to `Extended` at parse time), but it was
/// deprecated in v0.25.5 and the canonical name is now `extended`.
/// LINT-CODE: `LINT-POWERLINE-ALIAS`
fn warn_if_legacy_powerline_mode(config_path: &Option<String>) {
    let path = config_path
        .as_deref()
        .map(std::path::Path::new)
        .unwrap_or_else(|| std::path::Path::new("aibox.toml"));
    let Ok(body) = std::fs::read_to_string(path) else {
        return;
    };
    // Scan for `mode = "powerline"` (or `mode= "powerline"`, etc.) in the
    // [customization.tmux.status] section. We only look for the literal
    // string `"powerline"` as a value to avoid false-positives in comments.
    if body
        .lines()
        .any(|line| line.trim_start().starts_with("mode") && line.contains("\"powerline\""))
    {
        output::warn(
            "[LINT-POWERLINE-ALIAS] customization.tmux.status.mode = \"powerline\" is \
             a deprecated alias for \"extended\". Update aibox.toml to use mode = \"extended\" \
             to suppress this warning.",
        );
    }
}

/// Outcome of evaluating whether the seccomp consent gate must fire.
///
/// `Codex` is enabled, the project would emit `seccomp=unconfined`, and
/// `[security].acknowledge_seccomp_unconfined` is not yet set.
#[derive(Debug, PartialEq, Eq)]
enum SeccompConsentDecision {
    /// Gate does not apply (Codex disabled, or override declares seccomp,
    /// or the flag is already set).
    NotRequired,
    /// Interactive: caller should prompt the user.
    PromptInteractive,
    /// Non-interactive: caller must refuse with a remediation message.
    RefuseNonInteractive,
}

fn evaluate_seccomp_consent(
    config: &AiboxConfig,
    project_root: &Path,
    interactive: bool,
) -> SeccompConsentDecision {
    if config.security.acknowledge_seccomp_unconfined {
        return SeccompConsentDecision::NotRequired;
    }
    if !config.ai.harnesses.contains(&AiHarness::Codex) {
        return SeccompConsentDecision::NotRequired;
    }
    let override_path = project_root.join(&config.container.paths.docker_compose_override);
    if generate::compose_override_declares_codex_seccomp_pub(&override_path, &config.container.name)
    {
        return SeccompConsentDecision::NotRequired;
    }
    if interactive {
        SeccompConsentDecision::PromptInteractive
    } else {
        SeccompConsentDecision::RefuseNonInteractive
    }
}

/// Human-facing explanation of the seccomp=unconfined trade-off, shown
/// both in the interactive prompt and in the non-interactive refusal.
fn seccomp_consent_explanation() -> &'static str {
    "Codex is enabled in this project. Codex's shell-tool calls run inside an additional \
     bubblewrap user-namespace sandbox, which requires the outer container to run with \
     `seccomp=unconfined`. This widens the syscall surface the container can issue to the \
     host kernel — slightly weaker container-host isolation in exchange for a working \
     Codex sandbox. This is not a no-op decision; it must be explicitly acknowledged."
}

fn seccomp_consent_refusal_message(toml_path: &Path) -> String {
    format!(
        "Codex requires seccomp=unconfined in the generated docker-compose.yml, but \
         `[security].acknowledge_seccomp_unconfined` is not set in {}.\n\n{}\n\n\
         To proceed non-interactively, add to {}:\n\n  [security]\n  \
         acknowledge_seccomp_unconfined = true\n\n\
         Or re-run `aibox apply` from an interactive terminal to be prompted.",
        toml_path.display(),
        seccomp_consent_explanation(),
        toml_path.display(),
    )
}

fn write_seccomp_consent_to_toml(toml_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(toml_path)
        .with_context(|| format!("Failed to read {}", toml_path.display()))?;
    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;
    let security = doc
        .entry("security")
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
    if let Some(table) = security.as_table_mut() {
        table.insert("acknowledge_seccomp_unconfined", toml_edit::value(true));
    } else {
        bail!(
            "[security] in {} is not a table; cannot set acknowledge_seccomp_unconfined",
            toml_path.display()
        );
    }
    std::fs::write(toml_path, doc.to_string())
        .with_context(|| format!("Failed to write {}", toml_path.display()))?;
    Ok(())
}

/// Resolve which TOML file backs the loaded config, defaulting to
/// `aibox.toml` in the project root.
fn resolve_aibox_toml_path(config_path: &Option<String>) -> PathBuf {
    config_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("aibox.toml"))
}

/// v0.25.6 S3 — gate `aibox apply` on explicit consent for seccomp=unconfined
/// when Codex is enabled. Interactive prompt persists the flag to aibox.toml
/// and updates the in-memory config. Non-interactive runs refuse with a
/// remediation message pointing at the flag location.
fn ensure_seccomp_consent(config: &mut AiboxConfig, config_path: &Option<String>) -> Result<()> {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let interactive = std::io::IsTerminal::is_terminal(&std::io::stdin());
    match evaluate_seccomp_consent(config, &project_root, interactive) {
        SeccompConsentDecision::NotRequired => Ok(()),
        SeccompConsentDecision::RefuseNonInteractive => {
            let toml_path = resolve_aibox_toml_path(config_path);
            bail!("{}", seccomp_consent_refusal_message(&toml_path));
        }
        SeccompConsentDecision::PromptInteractive => {
            let toml_path = resolve_aibox_toml_path(config_path);
            eprintln!();
            output::warn("Codex seccomp consent required");
            eprintln!("{}", seccomp_consent_explanation());
            eprintln!();
            let accepted = dialoguer::Confirm::new()
                .with_prompt("Set [security].acknowledge_seccomp_unconfined = true and continue?")
                .default(false)
                .interact()?;
            if !accepted {
                bail!(
                    "Apply cancelled: seccomp=unconfined consent declined. Re-run when ready, \
                     or set `[security].acknowledge_seccomp_unconfined = true` in {} manually.",
                    toml_path.display()
                );
            }
            write_seccomp_consent_to_toml(&toml_path)?;
            config.security.acknowledge_seccomp_unconfined = true;
            output::ok(&format!(
                "Recorded seccomp=unconfined consent in {}",
                toml_path.display()
            ));
            Ok(())
        }
    }
}

/// command is allowed to create, modify, or delete. The tripwire below
/// snapshots a small set of representative out-of-perimeter files
/// before the sync runs and verifies after that none of them were
/// touched — providing a runtime guarantee in addition to the static
/// `is_within_perimeter` check used by sync write helpers.
pub fn cmd_sync(
    config_path: &Option<String>,
    no_cache: bool,
    no_build: bool,
    standardize_config: bool,
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
    if standardize_config {
        if let Some(path) = config_path.as_deref() {
            crate::migration::standardize_aibox_toml_file(Path::new(path))?;
        } else {
            crate::migration::standardize_aibox_toml(Path::new("."))?;
        }
    }

    // BR-TEST-GAPS H2: warn when [customization.tmux.status] mode uses the
    // legacy "powerline" alias.  "powerline" maps to Extended at parse time,
    // so we scan the raw TOML text before full deserialization.
    warn_if_legacy_powerline_mode(config_path);

    let mut config = AiboxConfig::from_cli_option(config_path)?;

    // v0.25.6 S3 — gate apply on explicit seccomp=unconfined consent for
    // Codex projects before any generation runs. Fires before doctor and
    // generate so the user is prompted (or refused) at the earliest point
    // a downstream step would have failed.
    ensure_seccomp_consent(&mut config, config_path)?;

    crate::context::update_gitignore(&config.addons)?;

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

    // Resolve [container.image].release_version = "latest" to a concrete image tag before
    // Dockerfile generation. "latest" is never a valid Docker image tag in
    // our registry (tags are base-<flavor>-v<semver>), so generation must
    // fall back to a concrete value even when network resolution fails.
    resolve_aibox_image_version_for_generation(&mut config, Path::new("."));

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
            let preserved_previous_selection = lock
                .addons
                .as_ref()
                .map(|a| a.previous_selection.clone())
                .unwrap_or_default();
            lock.addons = Some(crate::lock::AddonsLockSection {
                resolved_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                tools: resolved_tools,
                previous_selection: preserved_previous_selection,
            });
            if let Err(e) = crate::lock::write_lock(&project_root, &lock) {
                output::warn(&format!(
                    "Failed to update aibox.lock with resolved tool versions: {}",
                    e
                ));
            }
        }
    }

    // v0.25.6 BR-CLEANUP-ARCH item 1 (DEC-20260508_1515-SilentAsh):
    // Backfill addon/harness `previous_selection` on the lock so future
    // applies can compute a removal diff when a tool or harness is
    // disabled. Idempotent — only acts once per project (subsequent
    // applies see the fields populated and short-circuit). Emits a
    // pending Migration the first time it runs.
    if let Ok(cwd) = std::env::current_dir() {
        match crate::lock::backfill_lock_selection(&cwd, &config) {
            Ok(Some(path)) => output::ok(&format!(
                "Backfilled lock previous_selection; wrote migration: {}",
                path.display()
            )),
            Ok(None) => {}
            Err(e) => output::warn(&format!("Lock selection backfill failed: {}", e)),
        }
    }

    let added_required_addons = complete_missing_required_addons(&mut config);
    if !added_required_addons.is_empty() {
        match persist_missing_required_addons(config_path, &added_required_addons) {
            Ok(persisted) if !persisted.is_empty() => output::ok(&format!(
                "Added required addon section(s) to aibox.toml: {}",
                persisted.join(", ")
            )),
            Ok(_) => {
                for (addon, required) in &added_required_addons {
                    output::warn(&format!(
                        "Addon '{}' requires '{}'; using '{}' for this apply.",
                        addon, required, required
                    ));
                }
            }
            Err(e) => output::warn(&format!(
                "Could not persist required addon section(s) to aibox.toml: {}",
                e
            )),
        }
    }

    output::info("Scaffolding missing runtime directories...");
    seed::ensure_runtime_dirs(&config)?;
    let runtime_theme_updates = seed::sync_theme_files(&config)?;
    let runtime_permission_updates = seed::sync_managed_runtime_permissions(&config)?;
    let runtime_cleanup_updates = seed::cleanup_disabled_runtime_files(&config)?;
    if !runtime_theme_updates.is_empty() {
        output::ok(&format!(
            "Updated {} runtime theme/config file(s)",
            runtime_theme_updates.len()
        ));
    }
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
                    // The `install_hash_mismatch` reason is self-healing
                    // by design — the install machinery silently restores
                    // the pinned upstream payload. Demote it to an info
                    // line so derived projects don't see a scary
                    // recurring warn for a non-issue. Other prior states
                    // (MissingProvenance, MismatchedVersion, …) signal
                    // genuinely unusual conditions and keep the warn.
                    if matches!(
                        &prior_state,
                        crate::integrity::IntegrityStatus::Stale { reason: r, .. }
                            if r == "install_hash_mismatch"
                    ) {
                        output::info(&format!(
                            "Refreshing pinned processkit install for {}@{} (detected drift in upstream-shipped files since the last sync; reinstalling).",
                            config.processkit.source, config.processkit.version
                        ));
                    } else {
                        output::warn(&format!(
                            "Repairing processkit template mirror for {}@{}: {}. No manual action is required; aibox will reinstall the pinned processkit files now.",
                            config.processkit.source, config.processkit.version, reason
                        ));
                    }
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
    // processkit version and the enabled AI harness list. Idempotent —
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

        // ── Per-skill drift attribution (GitHub #54) ─────────────────────
        // Complement to the coarse fingerprint: identify *which* skill's
        // mcp-config.json has drifted from the current .mcp.json so the
        // user sees an actionable message, not just "something changed".
        // Best-effort — a failure here must not abort sync.
        {
            let dot_mcp = cwd.join(".mcp.json");
            match crate::mcp_registration::detect_per_skill_mcp_config_drift(&cwd, &dot_mcp) {
                Ok(drifts) if !drifts.is_empty() => {
                    let sample = drifts
                        .iter()
                        .take(5)
                        .map(|d| d.skill_name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let suffix = if drifts.len() > 5 { ", ..." } else { "" };
                    output::ok(&format!(
                        "Auto-repaired processkit MCP drift for {} server(s): {}{}",
                        drifts.len(),
                        sample,
                        suffix
                    ));
                }
                Ok(_) => {}
                Err(e) => {
                    output::warn(&format!("per-skill MCP drift check failed: {e}"));
                }
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
        // Merge processkit's preauth.json into enabled harness settings.
        // Best-effort and provider-scoped.
        if let Err(e) = crate::preauth::merge_processkit_preauth_for_config(&cwd, &config) {
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
                        if let Some(path) = report.drift_migration_path {
                            output::warn(&format!(
                                "Drifted runtime files detected — review pending migration: {}",
                                path.display()
                            ));
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
                    // AmberThorn (v0.25.7): emit v1→v2 Migration documents for every
                    // cutover whose upstream_release falls in the interval
                    // (from_pk.version, config.processkit.version].
                    match crate::v1_v2_migration::emit_v1_v2_migrations(
                        &from_pk.version,
                        &config.processkit.version,
                        &cwd,
                    ) {
                        Ok(emissions) if !emissions.is_empty() => {
                            for emission in &emissions {
                                output::ok(&format!(
                                    "Emitted v1→v2 migration: {} ({})",
                                    emission.id,
                                    emission.path.display()
                                ));
                            }
                            output::info(&format!(
                                "{} pending v1→v2 migration(s) written — review and apply via `apply_migration`",
                                emissions.len()
                            ));
                        }
                        Ok(_) => {}
                        Err(e) => output::warn(&format!("v1→v2 migration emission failed: {}", e)),
                    }
                }
                Err(e) => output::warn(&format!("Processkit diff failed: {}", e)),
            }
        }
        (Ok(_), None) => { /* No pre-install processkit lock — nothing to diff against. */ }
        (Err(e), _) => output::warn(&format!("Failed to determine working directory: {}", e)),
    }

    // EagerDew (v0.25.7): when a docs addon is enabled and the project
    // ships a `<docs-dir>/package.json`, run a project-local
    // `npm install --prefix <docs-dir>` so site-local deps like
    // `prism-react-renderer` are present before any docs build/deploy.
    // The addons themselves only install global tooling. Best-effort —
    // failures are warned-and-continued inside the helper.
    if let Ok(cwd) = std::env::current_dir() {
        crate::docs_install::maybe_install_project_docs_deps(&config, &cwd);
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
        match persist_missing_required_addons(config_path, &added_required_addons) {
            Ok(persisted) if !persisted.is_empty() => output::ok(&format!(
                "Added required addon section(s) to aibox.toml: {}",
                persisted.join(", ")
            )),
            Ok(_) => {
                for (addon, required) in &added_required_addons {
                    output::warn(&format!(
                        "Addon '{}' requires '{}'; using '{}' for this apply.",
                        addon, required, required
                    ));
                }
            }
            Err(e) => output::warn(&format!(
                "Could not persist required addon section(s) to aibox.toml: {}",
                e
            )),
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
    if config.container.image.version != "latest" {
        return;
    }

    let flavor = config.container.image.base.to_string();
    match crate::update::fetch_latest_image_version(&flavor) {
        Ok(v) => {
            let resolved = crate::generate::image_version_for_generation(v);
            output::info(&format!(
                "Resolved aibox image 'latest' \u{2192} v{}",
                resolved
            ));
            config.container.image.version = resolved;
            config.image.version = config.container.image.version.clone();
        }
        Err(e) => {
            if let Some(previous) = previous_concrete_image_version(project_root) {
                output::warn(&format!(
                    "[container.image].release_version = \"latest\" but image version resolution failed: {}. \
                     Reusing previously generated image version {}.",
                    e, previous
                ));
                config.container.image.version = previous;
                config.image.version = config.container.image.version.clone();
            } else {
                let current = env!("CARGO_PKG_VERSION").to_string();
                output::warn(&format!(
                    "[container.image].release_version = \"latest\" but image version resolution failed: {}. \
                     Falling back to the running CLI version {}.",
                    e, current
                ));
                config.container.image.version = current;
                config.image.version = config.container.image.version.clone();
            }
        }
    }
}

fn previous_concrete_image_version(project_root: &Path) -> Option<String> {
    let dockerfile = project_root.join(crate::config::DOCKERFILE);
    let content = std::fs::read_to_string(dockerfile).ok()?;
    let base = crate::config::BaseImage::Debian.to_string();
    content
        .lines()
        .find_map(|line| parse_dockerfile_image_version(line, &base))
}

fn parse_dockerfile_image_version(line: &str, base: &str) -> Option<String> {
    let image_ref = line.split_whitespace().find(|part| part.contains(':'))?;
    let (_, tag) = image_ref.rsplit_once(':')?;
    let prefixes = [
        format!("base-{}-runtime-v", base),
        format!("base-{}-v", base),
    ];

    for prefix in &prefixes {
        let Some(version_str) = tag.strip_prefix(prefix) else {
            continue;
        };
        let version = version_str
            .split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
            .next()
            .unwrap_or_default();
        if version.is_empty() || version == "latest" || version == "unset" {
            return None;
        }
        if Version::parse(version).is_err() {
            return None;
        }
        return Some(version.to_string());
    }

    None
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
            runtime.compose_build(
                crate::config::COMPOSE_FILE,
                &config.container.name,
                no_cache,
            )?;
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
    if container_version == config.container.image.version {
        return;
    }
    output::warn(&format!(
        "Container '{}' is still running on image v{} but the freshly-built image is v{}.\n    \
         The current container will keep running on the old image until you recreate it. To upgrade:\n    \
         \n        aibox delete runtime && aibox up\n    \
         \n    Existing in-flight work in the container (open editors, running processes) will be lost \
         on recreation; project files under /workspace are mounted from the host and survive.",
        name, container_version, config.container.image.version
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
    fn processkit_wizard_versions_keep_ten_newest_concrete_tags() {
        let versions = vec![
            "v0.27.0".to_string(),
            "v0.26.18".to_string(),
            "v0.26.17".to_string(),
            "v0.26.16".to_string(),
            "v0.26.15".to_string(),
            "v0.26.14".to_string(),
            "v0.26.13".to_string(),
            "v0.26.12".to_string(),
            "v0.26.11".to_string(),
            "v0.26.10".to_string(),
            "v0.26.9".to_string(),
            "v0.25.8".to_string(),
        ];

        let visible = processkit_wizard_visible_versions(&versions);

        assert_eq!(
            visible,
            vec![
                "v0.27.0", "v0.26.18", "v0.26.17", "v0.26.16", "v0.26.15", "v0.26.14", "v0.26.13",
                "v0.26.12", "v0.26.11", "v0.26.10"
            ]
        );
    }

    #[test]
    fn processkit_wizard_versions_filter_non_semver_entries() {
        let versions = vec![
            "v1.0.1".to_string(),
            "latest".to_string(),
            "v1.0.0".to_string(),
            "main".to_string(),
            "not-a-version".to_string(),
            "v0.99.4".to_string(),
        ];

        let visible = processkit_wizard_visible_versions(&versions);

        assert_eq!(visible, vec!["v1.0.1", "v1.0.0", "v0.99.4"]);
    }

    #[test]
    fn processkit_wizard_selection_writes_literal_latest_or_selected_pin() {
        let visible = vec!["v0.27.0".to_string(), "v0.26.18".to_string()];

        assert_eq!(selected_processkit_wizard_version(0, &visible), "latest");
        assert_eq!(selected_processkit_wizard_version(1, &visible), "v0.27.0");
        assert_eq!(selected_processkit_wizard_version(2, &visible), "v0.26.18");
        assert_eq!(selected_processkit_wizard_version(3, &visible), "unset");
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
    fn serialized_config_comments_include_theme_family_catalog() {
        let config = crate::config::test_config();
        let body = serialize_config_with_comments(&config);

        // New family-form catalog.
        for family in [
            "ayu",
            "catppuccin",
            "dracula",
            "github",
            "gruvbox",
            "material",
            "moonlight",
            "night-owl",
            "nord",
            "projectious",
            "rose-pine",
            "solarized",
            "tokyo-night",
        ] {
            assert!(
                body.contains(family),
                "missing theme family comment entry: {family}"
            );
        }
        // Variant hint comments.
        assert!(
            body.contains("\"mirage\""),
            "ayu mirage variant hint missing"
        );
        assert!(
            body.contains("\"storm\""),
            "tokyo-night storm variant hint missing"
        );
        assert!(body.contains("`auto` follows host OS appearance"));
    }

    #[test]
    fn serialized_config_prompt_comments_include_ascii_examples() {
        let config = crate::config::test_config();
        let body = serialize_config_with_comments(&config);

        assert!(body.contains("powerline-pastel"));
        assert!(body.contains("# ASCII sketches:"));
        assert!(body.contains("( ~/repo )>( main +2 )>( py rs js go ) 2s >"));
    }

    #[test]
    fn serialized_config_addon_tool_comments_include_purpose() {
        let _ = crate::addon_loader::init();
        let config = crate::config::test_config();
        let body = serialize_config_with_comments(&config);

        assert!(body.contains("Terminal image renderer used by Yazi image and SVG previews"));
        assert!(
            body.contains("Markdown, JSON, RST, and notebook terminal rendering for Yazi previews")
        );
    }

    #[test]
    fn tmux_attach_command_recreates_with_managed_layout_script() {
        assert_eq!(
            tmux_attach_command("ai", "aibox", true),
            vec!["aibox-tmux-session", "ai", "aibox"]
        );
    }

    #[test]
    fn tmux_attach_command_preserves_existing_session_by_default() {
        assert_eq!(
            tmux_attach_command("ai", "aibox", false),
            vec!["aibox-tmux-session", "ai", "aibox"]
        );
    }

    #[test]
    fn tmux_kill_session_uses_managed_socket_quietly() {
        let cmd = tmux_kill_session_command("aibox");

        assert_eq!(cmd[0], "sh");
        assert!(cmd[2].contains("AIBOX_TMUX_SOCKET"));
        assert!(cmd[2].contains("tmux -S \"$socket\" kill-session"));
        assert!(cmd[2].contains(">/dev/null 2>&1 || true"));
        assert_eq!(cmd[4], "aibox");
        assert!(!cmd[2].contains("/tmp/tmux-1000/default"));
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
        assert!(body.contains("harnesses = ["));
        assert!(body.contains("{ harness = \"claude\", enable = true, install = true }"));
        assert!(body.contains(
            "{ harness = \"codex\", enable = true, install = true, version = \"1.2.3\" }"
        ));
        assert!(body.contains("[ai.execution]"));
        assert!(body.contains("filesystem = \"workspace-write\""));
        assert!(body.contains("approval   = \"on-request\""));
        assert!(body.contains("network    = \"ask\""));
        assert!(body.contains("version = \"1.2.3\""));
        assert!(!body.contains("[[ai.harnesses]]"));
        assert!(!body.contains("[addons.ai-claude.tools]"));
        assert!(!body.contains("[addons.ai-codex.tools]"));
    }

    #[test]
    fn serialized_config_preserves_enabled_harness_order() {
        let mut config = crate::config::test_config();
        config.ai.harnesses = vec![
            crate::config::AiHarness::Codex,
            crate::config::AiHarness::Claude,
            crate::config::AiHarness::Gemini,
        ];

        let body = serialize_config_with_comments(&config);
        let codex = body.find(r#"harness = "codex", enable = true"#).unwrap();
        let claude = body.find(r#"harness = "claude", enable = true"#).unwrap();
        let gemini = body.find(r#"harness = "gemini", enable = true"#).unwrap();

        assert!(
            codex < claude && claude < gemini,
            "enabled harness entries must keep user/layout order:\n{body}"
        );
    }

    #[test]
    fn serialized_config_exposes_per_harness_execution_overrides() {
        let mut config = crate::config::test_config();
        config.ai.harnesses = vec![crate::config::AiHarness::Codex];
        config.ai.harness.insert(
            crate::config::AiHarness::Codex,
            crate::config::AiHarnessConfig {
                enabled: Some(true),
                install: Some(true),
                version: None,
                execution: Some(crate::config::AiHarnessExecutionOverride {
                    filesystem: Some(crate::config::AiExecutionFilesystem::ContainerFull),
                    approval: None,
                    network: None,
                }),
            },
        );

        let body = serialize_config_with_comments(&config);
        assert!(body.contains("[ai.execution.codex]"));
        assert!(body.contains("filesystem = \"container-full\""));
        assert!(body.contains("# [ai.execution.claude]"));
        assert!(!body.contains("trust_level"));
    }

    #[test]
    fn serialized_config_model_provider_catalog_includes_api_key_and_base_url_env_hints() {
        let mut config = crate::config::test_config();
        config.ai.model_providers = vec![crate::config::AiModelProvider::Anthropic];

        let body = serialize_config_with_comments(&config);
        assert!(body.contains("# Provider     Config value   API key env         Base URL env"));
        assert!(body.contains("# env: ANTHROPIC_API_KEY, ANTHROPIC_BASE_URL"));
        assert!(body.contains("# env: OPENAI_API_KEY, OPENAI_BASE_URL"));
        assert!(body.contains("# env: GEMINI_API_KEY, GEMINI_BASE_URL"));
        assert!(body.contains("# env: MISTRAL_API_KEY, MISTRAL_BASE_URL"));
    }

    #[test]
    fn serialized_config_exposes_tmux_status_separators() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.separators.style =
            crate::config::TmuxStatusSeparatorStyle::Flame;
        config.customization.tmux.status.separators.edge_style =
            crate::config::TmuxStatusSeparatorStyle::Honeycomb;
        config.customization.tmux.status.separators.elements_spacing =
            crate::config::TmuxStatusElementsSpacing::Plugins;

        let body = serialize_config_with_comments(&config);

        assert!(body.contains("[customization.tmux.status.separators]"));
        assert!(body.contains(
            "# PowerKit separator style. Options: normal | rounded | slant | slantup | trapezoid | flame | pixel | honeycomb | none"
        ));
        assert!(body.contains("style = \"flame\""));
        assert!(body.contains("edge-style = \"honeycomb\""));
        assert!(body.contains("elements-spacing = \"plugins\""));
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

    #[test]
    fn persist_missing_required_addons_writes_absent_tools_table() {
        let tmp = tempfile::tempdir().unwrap();
        let toml_path = tmp.path().join("aibox.toml");
        std::fs::write(
            &toml_path,
            r#"apiVersion = "aibox.projectious.work/v1"
kind = "Workspace"

[metadata]
name = "demo"

[aibox]
profile = "human-dev"

[image]
version = "0.25.8"
base = "debian"

[container]
name = "demo"

[addons.preview-enhanced.tools]
rich = {}
"#,
        )
        .unwrap();

        let persisted = persist_missing_required_addons(
            &Some(toml_path.to_string_lossy().to_string()),
            &[(
                "preview-enhanced".to_string(),
                "preview-archive".to_string(),
            )],
        )
        .unwrap();

        assert_eq!(persisted, vec!["preview-archive"]);
        let written = std::fs::read_to_string(&toml_path).unwrap();
        assert!(written.contains("[addons.preview-enhanced.tools]"));
        assert!(written.contains("[addons.preview-archive.tools]"));
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

    #[test]
    fn parse_dockerfile_image_version_supports_legacy_and_runtime_tag_families() {
        assert_eq!(
            parse_dockerfile_image_version(
                "FROM ghcr.io/projectious-work/aibox:base-debian-runtime-v0.27.3 AS aibox",
                "debian"
            ),
            Some("0.27.3".to_string())
        );
        assert_eq!(
            parse_dockerfile_image_version(
                "FROM ghcr.io/projectious-work/aibox:base-debian-v0.26.3 AS aibox",
                "debian"
            ),
            Some("0.26.3".to_string())
        );
    }

    #[test]
    fn parse_dockerfile_image_version_ignores_non_versioned_aliases() {
        assert_eq!(
            parse_dockerfile_image_version(
                "FROM ghcr.io/projectious-work/aibox:base-debian-runtime-latest AS aibox",
                "debian"
            ),
            None
        );
        assert_eq!(
            parse_dockerfile_image_version(
                "FROM ghcr.io/projectious-work/aibox:base-debian-latest AS aibox",
                "debian"
            ),
            None
        );
    }

    #[test]
    fn previous_concrete_image_version_recovers_from_legacy_and_runtime_dockerfile_tags() {
        let tmp = tempfile::tempdir().unwrap();
        let dockerfile = tmp.path().join(".devcontainer/Dockerfile");
        std::fs::create_dir_all(dockerfile.parent().unwrap()).unwrap();
        std::fs::write(
            &dockerfile,
            "FROM ghcr.io/projectious-work/aibox:base-debian-runtime-v0.27.1 AS aibox\n",
        )
        .unwrap();

        assert_eq!(
            previous_concrete_image_version(tmp.path()),
            Some("0.27.1".to_string())
        );
    }

    // ---------------------------------------------------------------------
    // v0.25.6 S3 — seccomp=unconfined consent gate
    // ---------------------------------------------------------------------

    fn config_with_codex_no_consent() -> AiboxConfig {
        let mut config = crate::config::test_config();
        config.ai.harnesses = vec![AiHarness::Codex];
        config.security.acknowledge_seccomp_unconfined = false;
        config
    }

    #[test]
    fn seccomp_consent_skipped_when_flag_already_true() {
        let mut config = config_with_codex_no_consent();
        config.security.acknowledge_seccomp_unconfined = true;
        let tmp = tempfile::tempdir().unwrap();
        // Both interactive and non-interactive must skip when already acknowledged.
        assert_eq!(
            evaluate_seccomp_consent(&config, tmp.path(), true),
            SeccompConsentDecision::NotRequired
        );
        assert_eq!(
            evaluate_seccomp_consent(&config, tmp.path(), false),
            SeccompConsentDecision::NotRequired
        );
    }

    #[test]
    fn seccomp_consent_skipped_when_codex_disabled() {
        let mut config = config_with_codex_no_consent();
        config.ai.harnesses = vec![AiHarness::Claude];
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(
            evaluate_seccomp_consent(&config, tmp.path(), false),
            SeccompConsentDecision::NotRequired
        );
    }

    #[test]
    fn seccomp_consent_refuses_non_interactive_when_unset() {
        let config = config_with_codex_no_consent();
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(
            evaluate_seccomp_consent(&config, tmp.path(), false),
            SeccompConsentDecision::RefuseNonInteractive
        );
    }

    #[test]
    fn seccomp_consent_prompts_interactive_when_unset() {
        let config = config_with_codex_no_consent();
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(
            evaluate_seccomp_consent(&config, tmp.path(), true),
            SeccompConsentDecision::PromptInteractive
        );
    }

    #[test]
    fn seccomp_consent_skipped_when_override_already_declares_unconfined() {
        let config = config_with_codex_no_consent();
        let tmp = tempfile::tempdir().unwrap();
        // Write a docker-compose.override.yml that already declares seccomp=unconfined
        // for the configured service name. The gate must defer to the override.
        let override_path = tmp
            .path()
            .join(&config.container.paths.docker_compose_override);
        if let Some(parent) = override_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let body = format!(
            "services:\n  {}:\n    security_opt:\n      - seccomp=unconfined\n",
            config.container.name
        );
        std::fs::write(&override_path, body).unwrap();
        assert_eq!(
            evaluate_seccomp_consent(&config, tmp.path(), false),
            SeccompConsentDecision::NotRequired,
        );
    }

    #[test]
    fn seccomp_consent_refusal_message_points_at_flag_and_path() {
        let path = std::path::Path::new("aibox.toml");
        let msg = seccomp_consent_refusal_message(path);
        assert!(msg.contains("acknowledge_seccomp_unconfined"));
        assert!(msg.contains("[security]"));
        assert!(msg.contains("aibox.toml"));
        assert!(msg.to_lowercase().contains("re-run"));
    }

    #[test]
    fn write_seccomp_consent_inserts_security_section_in_existing_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let toml_path = tmp.path().join("aibox.toml");
        std::fs::write(
            &toml_path,
            "[aibox]\nversion = \"0.25.6\"\n\n[ai]\nharnesses = [\"codex\"]\n",
        )
        .unwrap();

        write_seccomp_consent_to_toml(&toml_path).expect("write should succeed");

        let written = std::fs::read_to_string(&toml_path).unwrap();
        assert!(written.contains("[security]"));
        assert!(written.contains("acknowledge_seccomp_unconfined = true"));
        // Existing keys must be preserved.
        assert!(written.contains("version = \"0.25.6\""));
        assert!(written.contains("harnesses = [\"codex\"]"));
    }

    #[test]
    fn write_seccomp_consent_updates_existing_security_section() {
        let tmp = tempfile::tempdir().unwrap();
        let toml_path = tmp.path().join("aibox.toml");
        std::fs::write(
            &toml_path,
            "[security]\nacknowledge_seccomp_unconfined = false\n",
        )
        .unwrap();

        write_seccomp_consent_to_toml(&toml_path).expect("write should succeed");

        let written = std::fs::read_to_string(&toml_path).unwrap();
        assert!(written.contains("acknowledge_seccomp_unconfined = true"));
        assert!(!written.contains("acknowledge_seccomp_unconfined = false"));
    }
}
