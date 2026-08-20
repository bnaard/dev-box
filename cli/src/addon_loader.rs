//! Loads addon definitions from YAML files and renders Dockerfile templates.
//!
//! Canonical addon YAML files are embedded in the executable. Optional files
//! in `$XDG_CONFIG_HOME/aibox/addons/` (or `AIBOX_ADDONS_DIR`) override matching
//! canonical definitions and may add custom definitions.
//!
//! Each YAML file defines:
//! - Metadata (name, version, builder_weight)
//! - Tools with version selection
//! - Optional builder stage (Dockerfile template)
//! - Runtime commands (Dockerfile template)
//!
//! Templates use minijinja syntax with `tools.<name>.version` and
//! `tools.<name>.enabled` as context variables.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use crate::addon_registry::{ToolConfig, ToolDef};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_addons.rs"));
}

pub const ADDON_CATALOG_SCHEMA_VERSION: &str = "aibox.addon-catalog.v0";

// ---------------------------------------------------------------------------
// YAML data model
// ---------------------------------------------------------------------------

/// Deserialized from a single addon YAML file.
#[derive(Debug, Deserialize)]
pub struct AddonYaml {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub profile_intent: Option<AddonProfileIntent>,
    #[serde(default)]
    pub usage_class: Option<AddonUsageClass>,
    #[serde(default)]
    pub profiles: Vec<AddonProfile>,
    #[serde(default)]
    pub exported_surfaces: Vec<AddonExportSurface>,
    #[serde(default)]
    pub builder_weight: Option<String>,
    #[serde(default)]
    pub tools: Vec<ToolYaml>,
    #[serde(default)]
    pub requires: Vec<String>,
    /// Nested configuration aliases exposed below `[addons.<language>]`.
    /// Values are canonical addon names, e.g. `supply-chain: supply-chain`.
    #[serde(default)]
    pub groups: HashMap<String, String>,
    /// Tolerated for backwards-compat with addon YAML files that still
    /// declare a `skills:` block. Ignored since v0.16.0 — skills are
    /// owned by processkit and installed via the content-source pipeline.
    #[serde(default)]
    #[allow(dead_code)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub builder: Option<String>,
    #[serde(default)]
    pub runtime: Option<String>,
}

/// Draft LivelyMoss profile intent vocabulary for addon-spec alignment.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AddonProfileIntent {
    Interactive,
    Runtime,
    Build,
    Mcp,
    ProviderCli,
    Debug,
    PreviewMedia,
}

impl AddonProfileIntent {
    pub fn as_str(self) -> &'static str {
        match self {
            AddonProfileIntent::Interactive => "interactive",
            AddonProfileIntent::Runtime => "runtime",
            AddonProfileIntent::Build => "build",
            AddonProfileIntent::Mcp => "mcp",
            AddonProfileIntent::ProviderCli => "provider-cli",
            AddonProfileIntent::Debug => "debug",
            AddonProfileIntent::PreviewMedia => "preview-media",
        }
    }
}

/// Whether tools exported by an addon may be invoked automatically.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AddonUsageClass {
    Automated,
    ManualEscalationOnly,
}

impl AddonUsageClass {
    pub fn as_str(self) -> &'static str {
        match self {
            AddonUsageClass::Automated => "automated",
            AddonUsageClass::ManualEscalationOnly => "manual-escalation-only",
        }
    }
}

/// aibox image profiles an addon is compatible with.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AddonProfile {
    HumanDev,
    HeadlessRunner,
}

impl AddonProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            AddonProfile::HumanDev => "human-dev",
            AddonProfile::HeadlessRunner => "headless-runner",
        }
    }
}

/// User-visible surfaces an addon exports into the generated environment.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AddonExportSurface {
    CliBinary,
    ConfigFiles,
    ShellIntegration,
    LanguageRuntime,
    BuildToolchain,
    RuntimeService,
    Previewer,
    McpServer,
}

impl AddonExportSurface {
    pub fn as_str(self) -> &'static str {
        match self {
            AddonExportSurface::CliBinary => "cli-binary",
            AddonExportSurface::ConfigFiles => "config-files",
            AddonExportSurface::ShellIntegration => "shell-integration",
            AddonExportSurface::LanguageRuntime => "language-runtime",
            AddonExportSurface::BuildToolchain => "build-toolchain",
            AddonExportSurface::RuntimeService => "runtime-service",
            AddonExportSurface::Previewer => "previewer",
            AddonExportSurface::McpServer => "mcp-server",
        }
    }
}

/// A tool entry in the YAML file.
#[derive(Debug, Deserialize)]
pub struct ToolYaml {
    pub name: String,
    /// Short one-line purpose shown in generated aibox.toml comments.
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_true")]
    pub default_enabled: bool,
    #[serde(default)]
    pub default_version: Option<String>,
    #[serde(default)]
    pub supported_versions: Vec<String>,
}

fn default_true() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Loaded addon storage
// ---------------------------------------------------------------------------

/// A fully loaded addon with owned data (not static references).
#[derive(Debug)]
pub struct LoadedAddon {
    pub name: String,
    pub addon_version: String,
    /// Short one-line description from the YAML `description:` field.
    pub description: String,
    /// Draft processkit addon-spec intent. Warning-only until processkit
    /// publishes the canonical `Artifact{kind=addon-spec}` schema.
    pub profile_intent: Option<AddonProfileIntent>,
    /// Draft automation policy for tools exported by this addon.
    pub usage_class: Option<AddonUsageClass>,
    /// aibox profiles this addon can participate in.
    pub profiles: Vec<AddonProfile>,
    /// User-visible surfaces this addon contributes to the environment.
    pub exported_surfaces: Vec<AddonExportSurface>,
    /// Category derived from the addon's parent directory (ai/, languages/, tools/, docs/).
    pub category: String,
    pub builder_weight: Option<String>,
    pub tools: Vec<LoadedTool>,
    pub requires: Vec<String>,
    pub groups: HashMap<String, String>,
    pub builder_template: Option<String>,
    pub runtime_template: Option<String>,
}

#[derive(Debug)]
pub struct LoadedTool {
    pub name: String,
    pub description: String,
    pub default_enabled: bool,
    pub default_version: String,
    pub supported_versions: Vec<String>,
}

/// Stable generated index of the addon catalog for downstream consumers.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct AddonCatalogIndex {
    pub schema_version: &'static str,
    pub aibox_version: &'static str,
    pub addons: Vec<AddonCatalogEntry>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct AddonCatalogEntry {
    pub name: String,
    pub addon_version: String,
    pub category: String,
    pub description: String,
    pub profile_intent: Option<String>,
    pub usage_class: Option<String>,
    pub profiles: Vec<String>,
    pub exported_surfaces: Vec<String>,
    pub requires: Vec<String>,
    pub groups: HashMap<String, String>,
    pub tools: Vec<AddonCatalogTool>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct AddonCatalogTool {
    pub name: String,
    pub description: String,
    pub default_enabled: bool,
    pub default_version: String,
    pub supported_versions: Vec<String>,
}

/// Global addon store, initialized once.
static ADDONS: OnceLock<Vec<LoadedAddon>> = OnceLock::new();

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Load all addon YAML files from the addons directory.
/// Walks subdirectories (languages/, tools/, docs/, ai/).
fn load_from_dir(dir: &Path) -> Result<Vec<LoadedAddon>> {
    if !dir.exists() {
        bail!(
            "Addon definitions not found at {}\n\
             Run the install script to set them up:\n\
             curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash",
            dir.display()
        );
    }

    let mut addons = Vec::new();

    // Walk subdirectories
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Category subdirectory — read YAML files inside
            for file_entry in
                fs::read_dir(&path).with_context(|| format!("Failed to read {}", path.display()))?
            {
                let file_entry = file_entry?;
                let file_path = file_entry.path();
                if file_path
                    .extension()
                    .is_some_and(|e| e == "yaml" || e == "yml")
                {
                    let addon = load_yaml_file(&file_path)?;
                    addons.push(addon);
                }
            }
        } else if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            // Top-level YAML file (for flexibility)
            let addon = load_yaml_file(&path)?;
            addons.push(addon);
        }
    }

    if addons.is_empty() {
        bail!(
            "No addon YAML files found in {}\n\
             The directory exists but contains no .yaml files.",
            dir.display()
        );
    }

    // Sort by name for consistent ordering
    addons.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(addons)
}

/// Map a parent directory name to a display category string.
fn category_from_dir_name(dir_name: &str) -> &'static str {
    match dir_name {
        "ai" => "AI Providers",
        "languages" => "Languages",
        "tools" => "Tools",
        "docs" => "Documentation",
        _ => "Other",
    }
}

/// Parse a single YAML file into a LoadedAddon.
/// The category is derived from the file's parent directory name.
fn load_yaml_file(path: &Path) -> Result<LoadedAddon> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read addon file: {}", path.display()))?;
    let category = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(category_from_dir_name)
        .unwrap_or("Other")
        .to_string();

    load_yaml_content(&content, &category, &path.display().to_string())
}

fn load_yaml_content(content: &str, category: &str, source: &str) -> Result<LoadedAddon> {
    let yaml: AddonYaml = serde_yaml::from_str(content)
        .with_context(|| format!("Failed to parse addon YAML: {source}"))?;

    Ok(LoadedAddon {
        name: yaml.name,
        addon_version: yaml.version,
        description: yaml.description,
        profile_intent: yaml.profile_intent,
        usage_class: yaml.usage_class,
        profiles: yaml.profiles,
        exported_surfaces: yaml.exported_surfaces,
        category: category.to_string(),
        builder_weight: yaml.builder_weight,
        requires: yaml.requires,
        groups: yaml.groups,
        tools: yaml
            .tools
            .into_iter()
            .map(|t| LoadedTool {
                name: t.name,
                description: t.description,
                default_enabled: t.default_enabled,
                default_version: t.default_version.unwrap_or_default(),
                supported_versions: t.supported_versions,
            })
            .collect(),
        builder_template: yaml.builder,
        runtime_template: yaml.runtime,
    })
}

fn load_embedded_catalog() -> Result<Vec<LoadedAddon>> {
    let mut addons = embedded::EMBEDDED_ADDON_YAMLS
        .iter()
        .map(|(category, source, content)| {
            load_yaml_content(content, category_from_dir_name(category), source)
        })
        .collect::<Result<Vec<_>>>()?;
    addons.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(addons)
}

fn merge_catalogs(embedded: Vec<LoadedAddon>, overrides: Vec<LoadedAddon>) -> Vec<LoadedAddon> {
    let mut by_name: HashMap<String, LoadedAddon> = embedded
        .into_iter()
        .map(|addon| (addon.name.clone(), addon))
        .collect();
    for addon in overrides {
        by_name.insert(addon.name.clone(), addon);
    }
    let mut addons: Vec<_> = by_name.into_values().collect();
    addons.sort_by(|a, b| a.name.cmp(&b.name));
    addons
}

// ---------------------------------------------------------------------------
// Global access
// ---------------------------------------------------------------------------

/// Initialize the embedded addon store plus optional filesystem overrides.
pub fn init() -> Result<()> {
    if let Ok(dir) = std::env::var("AIBOX_ADDONS_DIR") {
        let addons = merge_catalogs(load_embedded_catalog()?, load_from_dir(Path::new(&dir))?);
        ADDONS
            .set(addons)
            .map_err(|_| anyhow::anyhow!("Addon store already initialized"))?;
        return Ok(());
    }

    let repo_addons = Path::new(env!("CARGO_MANIFEST_DIR")).join("../addons");
    let addons = if repo_addons.is_dir() {
        load_from_dir(&repo_addons)?
    } else {
        let embedded = load_embedded_catalog()?;
        let installed = crate::dirs::config_dir()
            .map(|dir| dir.join("addons"))
            .filter(|dir| dir.is_dir())
            .map(|dir| load_from_dir(&dir))
            .transpose()?
            .unwrap_or_default();
        merge_catalogs(embedded, installed)
    };
    ADDONS
        .set(addons)
        .map_err(|_| anyhow::anyhow!("Addon store already initialized"))?;
    Ok(())
}

/// Initialize the addon store from a specific directory.
/// Used by tests to point at the repo's addons/ directory.
#[cfg(test)]
pub fn init_from_dir(dir: &Path) -> Result<()> {
    let addons = load_from_dir(dir)?;
    ADDONS
        .set(addons)
        .map_err(|_| anyhow::anyhow!("Addon store already initialized"))?;
    Ok(())
}

/// Get all loaded addons.
pub fn all_addons() -> &'static [LoadedAddon] {
    ADDONS.get().map(|v| v.as_slice()).unwrap_or(&[])
}

/// Stable fingerprint of the loaded addon option catalog.
///
/// Generated `aibox.toml` files record this value so a newly shipped addon or
/// tool causes the comment catalog to be refreshed even when the config schema
/// itself did not change.
pub fn catalog_fingerprint() -> String {
    let mut addons: Vec<_> = all_addons().iter().collect();
    addons.sort_by(|a, b| a.name.cmp(&b.name));

    let mut hasher = Sha256::new();
    hasher.update(ADDON_CATALOG_SCHEMA_VERSION.as_bytes());
    for addon in addons {
        hasher.update([0]);
        hasher.update(addon.name.as_bytes());
        hasher.update([0]);
        hasher.update(addon.addon_version.as_bytes());
        for tool in &addon.tools {
            hasher.update([0]);
            hasher.update(tool.name.as_bytes());
            hasher.update([tool.default_enabled as u8]);
            hasher.update(tool.default_version.as_bytes());
            for version in &tool.supported_versions {
                hasher.update([0]);
                hasher.update(version.as_bytes());
            }
        }
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Find an addon by name.
pub fn get_addon(name: &str) -> Option<&'static LoadedAddon> {
    all_addons().iter().find(|a| a.name == name)
}

pub fn addon_catalog_index(addons: &[LoadedAddon]) -> AddonCatalogIndex {
    let mut entries: Vec<AddonCatalogEntry> = addons
        .iter()
        .map(|addon| AddonCatalogEntry {
            name: addon.name.clone(),
            addon_version: addon.addon_version.clone(),
            category: addon.category.clone(),
            description: addon.description.clone(),
            profile_intent: addon.profile_intent.map(|v| v.as_str().to_string()),
            usage_class: addon.usage_class.map(|v| v.as_str().to_string()),
            profiles: addon
                .profiles
                .iter()
                .map(|p| p.as_str().to_string())
                .collect(),
            exported_surfaces: addon
                .exported_surfaces
                .iter()
                .map(|surface| surface.as_str().to_string())
                .collect(),
            requires: addon.requires.clone(),
            groups: addon.groups.clone(),
            tools: addon
                .tools
                .iter()
                .map(|tool| AddonCatalogTool {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    default_enabled: tool.default_enabled,
                    default_version: tool.default_version.clone(),
                    supported_versions: tool.supported_versions.clone(),
                })
                .collect(),
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    AddonCatalogIndex {
        schema_version: ADDON_CATALOG_SCHEMA_VERSION,
        aibox_version: env!("CARGO_PKG_VERSION"),
        addons: entries,
    }
}

/// Warning-mode validation for the draft LivelyMoss addon metadata.
///
/// These checks intentionally do not fail loading: processkit has not yet
/// published the canonical addon-spec schema, so aibox treats the fields as
/// early compatibility metadata until that schema is available.
pub fn addon_metadata_warnings(addons: &[LoadedAddon]) -> Vec<String> {
    let mut warnings = Vec::new();
    for addon in addons {
        if addon.profile_intent.is_none() {
            warnings.push(format!(
                "addon-metadata-missing: {} has no profile_intent",
                addon.name
            ));
        }
        if addon.usage_class.is_none() {
            warnings.push(format!(
                "addon-metadata-missing: {} has no usage_class",
                addon.name
            ));
        }
        if addon.profiles.is_empty() {
            warnings.push(format!(
                "addon-metadata-missing: {} has no profiles",
                addon.name
            ));
        }
        if addon.exported_surfaces.is_empty() {
            warnings.push(format!(
                "addon-metadata-missing: {} has no exported_surfaces",
                addon.name
            ));
        }
        if addon.builder_template.is_none() && addon.runtime_template.is_none() {
            warnings.push(format!(
                "addon-without-install-steps: {} has no builder or runtime install template",
                addon.name
            ));
        }
        if addon.profile_intent == Some(AddonProfileIntent::ProviderCli)
            && addon.profiles.contains(&AddonProfile::HeadlessRunner)
        {
            warnings.push(format!(
                "subscription-cli-headless-leak: {} is provider-cli but allows headless-runner",
                addon.name
            ));
        }
    }
    warnings
}

/// Warning-mode validation for selected addons against the configured profile.
pub fn addon_profile_compatibility_warnings(
    addons: &[LoadedAddon],
    selected_addons: &[String],
    profile: &str,
) -> Vec<String> {
    let mut selected = selected_addons.to_vec();
    selected.sort();

    let mut warnings = Vec::new();
    for addon_name in selected {
        let Some(addon) = addons.iter().find(|candidate| candidate.name == addon_name) else {
            continue;
        };
        if !addon.profiles.iter().any(|p| p.as_str() == profile) {
            warnings.push(format!(
                "addon-profile-incompatible: {} is selected but does not support {}",
                addon.name, profile
            ));
        }
    }
    warnings
}

// ---------------------------------------------------------------------------
// Conversion to legacy types (for backward compat with addons.rs)
// ---------------------------------------------------------------------------

impl LoadedAddon {
    /// Convert to an AddonDef for backward compatibility.
    /// Note: returns owned ToolDefs, not static references.
    pub fn to_tool_defs(&self) -> Vec<ToolDef> {
        self.tools
            .iter()
            .map(|t| ToolDef {
                name: leak_str(&t.name),
                default_enabled: t.default_enabled,
                supported_versions: leak_str_slice(&t.supported_versions),
                default_version: leak_str(&t.default_version),
            })
            .collect()
    }

    /// Builder weight as a numeric sort key for ordering.
    /// heavy=0, medium=1, light=2, none=3
    pub fn builder_order_key(&self) -> usize {
        match self.builder_weight.as_deref() {
            Some("heavy") => 0,
            Some("medium") => 1,
            Some("light") => 2,
            _ => 3,
        }
    }
}

// Leak strings to get &'static str — these live for the program's lifetime
// since addons are loaded once at startup via OnceLock.
pub fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

fn leak_str_slice(v: &[String]) -> &'static [&'static str] {
    let leaked: Vec<&'static str> = v.iter().map(|s| leak_str(s)).collect();
    Box::leak(leaked.into_boxed_slice())
}

// ---------------------------------------------------------------------------
// Template rendering
// ---------------------------------------------------------------------------

/// Build the minijinja context for an addon's templates.
fn build_template_context(
    addon: &LoadedAddon,
    tools: &HashMap<String, ToolConfig>,
) -> minijinja::Value {
    let mut tool_map = HashMap::new();

    for tool_def in &addon.tools {
        let enabled = tools.get(&tool_def.name).is_some_and(|t| t.enabled);

        // "latest" is a sentinel that explicitly opts out of version pinning.
        // It maps to an empty version string so templates that check
        // `{% if tools.X.version %}` skip the pinned install path.
        let version = match tools.get(&tool_def.name) {
            Some(t) if t.version == "latest" => String::new(),
            Some(t) if !t.version.is_empty() => t.version.clone(),
            _ => tool_def.default_version.clone(),
        };
        let pinned = !version.is_empty();

        let mut entry = HashMap::new();
        entry.insert("enabled".to_string(), minijinja::Value::from(enabled));
        entry.insert("version".to_string(), minijinja::Value::from(version));
        entry.insert("pinned".to_string(), minijinja::Value::from(pinned));
        tool_map.insert(
            tool_def.name.clone(),
            minijinja::Value::from_serialize(&entry),
        );
    }

    minijinja::Value::from_serialize(HashMap::from([("tools", tool_map)]))
}

/// Render the builder stage template for an addon. Returns None if no builder.
pub fn render_builder(
    addon: &LoadedAddon,
    tools: &HashMap<String, ToolConfig>,
) -> Result<Option<String>> {
    let template_str = match &addon.builder_template {
        Some(t) => t,
        None => return Ok(None),
    };

    let mut env = minijinja::Environment::new();
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);
    env.add_template("builder", template_str)
        .with_context(|| format!("Invalid builder template for addon '{}'", addon.name))?;

    let tmpl = env.get_template("builder").unwrap();
    let ctx = build_template_context(addon, tools);
    let rendered = tmpl.render(&ctx).with_context(|| {
        format!(
            "Failed to render builder template for addon '{}'",
            addon.name
        )
    })?;

    Ok(Some(rendered))
}

/// Render the runtime commands template for an addon.
pub fn render_runtime(addon: &LoadedAddon, tools: &HashMap<String, ToolConfig>) -> Result<String> {
    let template_str = match &addon.runtime_template {
        Some(t) => t,
        None => return Ok(String::new()),
    };

    let mut env = minijinja::Environment::new();
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);
    env.add_template("runtime", template_str)
        .with_context(|| format!("Invalid runtime template for addon '{}'", addon.name))?;

    let tmpl = env.get_template("runtime").unwrap();
    let ctx = build_template_context(addon, tools);
    let rendered = tmpl.render(&ctx).with_context(|| {
        format!(
            "Failed to render runtime template for addon '{}'",
            addon.name
        )
    })?;

    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_test_yaml(dir: &Path, category: &str, name: &str, content: &str) {
        let cat_dir = dir.join(category);
        fs::create_dir_all(&cat_dir).unwrap();
        let mut f = fs::File::create(cat_dir.join(format!("{}.yaml", name))).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn load_from_dir_finds_yaml_files() {
        let dir = tempfile::tempdir().unwrap();
        write_test_yaml(
            dir.path(),
            "languages",
            "test-addon",
            r#"
name: test-addon
version: "1.0.0"
tools:
  - name: test-tool
    default_enabled: true
    default_version: "1.0"
    supported_versions: ["1.0", "2.0"]
runtime: |
  RUN echo "hello"
"#,
        );

        let addons = load_from_dir(dir.path()).unwrap();
        assert_eq!(addons.len(), 1);
        assert_eq!(addons[0].name, "test-addon");
        assert_eq!(addons[0].tools.len(), 1);
        assert_eq!(addons[0].tools[0].name, "test-tool");
        assert_eq!(addons[0].tools[0].default_version, "1.0");
        assert!(addons[0].requires.is_empty());
    }

    #[test]
    fn embedded_catalog_contains_release_addons() {
        let addons = load_embedded_catalog().unwrap();
        for name in [
            "browser-testing",
            "cloudflare",
            "go-quality",
            "release",
            "supply-chain",
        ] {
            assert!(
                addons.iter().any(|addon| addon.name == name),
                "embedded catalog should contain {name}"
            );
        }
    }

    #[test]
    fn incomplete_installed_catalog_overrides_without_hiding_embedded_addons() {
        let dir = tempfile::tempdir().unwrap();
        write_test_yaml(
            dir.path(),
            "tools",
            "git-ui",
            r#"
name: git-ui
version: "override"
tools:
  - name: gh
    default_enabled: true
runtime: |
  RUN echo override
"#,
        );

        let merged = merge_catalogs(
            load_embedded_catalog().unwrap(),
            load_from_dir(dir.path()).unwrap(),
        );
        assert_eq!(
            merged
                .iter()
                .find(|addon| addon.name == "git-ui")
                .unwrap()
                .addon_version,
            "override"
        );
        assert!(
            merged.iter().any(|addon| addon.name == "supply-chain"),
            "a stale installed catalog must not hide embedded canonical addons"
        );
    }

    #[test]
    fn load_addon_with_requires() {
        let dir = tempfile::tempdir().unwrap();
        write_test_yaml(
            dir.path(),
            "docs",
            "test-docs",
            r#"
name: test-docs
version: "1.0.0"
requires:
  - node
tools:
  - name: docusaurus
    default_enabled: true
    default_version: "3"
    supported_versions: ["3"]
runtime: |
  RUN npm install -g create-docusaurus@latest
"#,
        );

        let addons = load_from_dir(dir.path()).unwrap();
        assert_eq!(addons[0].requires, vec!["node"]);
    }

    #[test]
    fn load_addon_with_language_groups() {
        let addon = load_repo_addon("go");
        assert_eq!(
            addon.groups.get("quality").map(String::as_str),
            Some("go-quality")
        );
        assert_eq!(
            addon.groups.get("supply-chain").map(String::as_str),
            Some("supply-chain")
        );
        assert_eq!(
            addon.groups.get("release").map(String::as_str),
            Some("go-release")
        );
    }

    #[test]
    fn browser_testing_addon_pins_stack_and_defaults_to_full_chromium() {
        let addon = load_repo_addon("browser-testing");
        assert_eq!(addon.requires, vec!["node"]);

        let playwright = addon
            .tools
            .iter()
            .find(|tool| tool.name == "playwright")
            .expect("browser-testing should define the Playwright tool");
        assert_eq!(playwright.default_version, "1.62.1");
        assert_eq!(playwright.supported_versions, vec!["1.62.1"]);

        let axe = addon
            .tools
            .iter()
            .find(|tool| tool.name == "axe-playwright")
            .expect("browser-testing should define the axe adapter tool");
        assert_eq!(axe.default_version, "4.13.0");
        assert_eq!(axe.supported_versions, vec!["4.13.0"]);

        let chromium = addon
            .tools
            .iter()
            .find(|tool| tool.name == "chromium")
            .expect("browser-testing should define Chromium");
        assert!(chromium.default_enabled);
        assert!(
            !addon
                .tools
                .iter()
                .find(|tool| tool.name == "firefox")
                .unwrap()
                .default_enabled
        );
        assert!(
            !addon
                .tools
                .iter()
                .find(|tool| tool.name == "webkit")
                .unwrap()
                .default_enabled
        );

        let rendered = render_runtime(&addon, &all_enabled_tools(&addon)).unwrap();
        assert!(rendered.contains("@playwright/test@1.62.1"));
        assert!(rendered.contains("@axe-core/playwright@4.13.0"));
        assert!(rendered.contains("axe-core@4.13.0"));
        assert!(rendered.contains("--no-shell"));
        assert!(
            rendered.contains("PLAYWRIGHT_BROWSERS_PATH=/ms-playwright"),
            "full Chromium must be installed in a shared browser path"
        );
        assert!(rendered.contains("chromium"));
        assert!(rendered.contains("firefox"));
        assert!(rendered.contains("webkit"));
        assert!(!rendered.contains("chromium-headless-shell"));

        let rendered_defaults = render_runtime(
            &addon,
            &addon
                .tools
                .iter()
                .map(|tool| {
                    (
                        tool.name.clone(),
                        ToolConfig {
                            enabled: tool.default_enabled,
                            version: tool.default_version.clone(),
                        },
                    )
                })
                .collect(),
        )
        .unwrap();
        assert!(rendered_defaults.contains("chromium \\"));
        assert!(!rendered_defaults.contains("      firefox \\"));
        assert!(!rendered_defaults.contains("      webkit \\"));

        let rendered_disabled = render_runtime(&addon, &all_disabled_tools(&addon)).unwrap();
        assert!(rendered_disabled.contains("npm uninstall -g @playwright/test playwright"));
        assert!(rendered_disabled.contains("npm uninstall -g @axe-core/playwright axe-core"));
        assert!(!rendered_disabled.contains("playwright install --with-deps"));
    }

    #[test]
    fn latex_addon_uses_reachable_immutable_texlive_archive() {
        let addon = load_repo_addon("latex");
        let rendered = render_builder(&addon, &all_enabled_tools(&addon))
            .unwrap()
            .expect("latex should define a builder stage");

        assert!(rendered.contains(
            "https://ftp.tu-chemnitz.de/pub/tug/historic/systems/texlive/2025/tlnet-final"
        ));
        assert!(!rendered.contains("https://texlive.info/historic/"));
        assert!(rendered.contains("--repository \"${CTAN_MIRROR}\""));
    }

    #[test]
    fn every_language_exposes_consistent_shared_groups() {
        for language in ["go", "rust", "python", "node", "typst", "latex"] {
            let addon = load_repo_addon(language);
            assert_eq!(
                addon.groups.get("infrastructure").map(String::as_str),
                Some("infrastructure"),
                "{language} infrastructure group"
            );
            assert_eq!(
                addon.groups.get("security").map(String::as_str),
                Some("supply-chain"),
                "{language} security group"
            );
            assert_eq!(
                addon.groups.get("supply-chain").map(String::as_str),
                Some("supply-chain"),
                "{language} supply-chain group"
            );
            assert!(
                addon.groups.contains_key("release"),
                "{language} release group"
            );
        }
    }

    #[test]
    fn render_runtime_substitutes_versions() {
        let addon = LoadedAddon {
            name: "test".to_string(),
            addon_version: "1.0.0".to_string(),
            description: String::new(),
            profile_intent: Some(AddonProfileIntent::Runtime),
            usage_class: Some(AddonUsageClass::Automated),
            profiles: vec![AddonProfile::HumanDev, AddonProfile::HeadlessRunner],
            exported_surfaces: vec![AddonExportSurface::CliBinary],
            category: "Other".to_string(),
            builder_weight: None,
            requires: vec![],
            tools: vec![LoadedTool {
                name: "mytool".to_string(),
                description: String::new(),
                default_enabled: true,
                default_version: "3.0".to_string(),
                supported_versions: vec!["3.0".to_string()],
            }],
            groups: HashMap::new(),
            builder_template: None,
            runtime_template: Some("RUN install mytool={{ tools.mytool.version }}".to_string()),
        };

        let mut tools = HashMap::new();
        tools.insert(
            "mytool".to_string(),
            ToolConfig {
                enabled: true,
                version: "3.0".to_string(),
            },
        );

        let result = render_runtime(&addon, &tools).unwrap();
        assert!(result.contains("mytool=3.0"), "got: {}", result);
    }

    #[test]
    fn render_runtime_handles_conditionals() {
        let addon = LoadedAddon {
            name: "test".to_string(),
            addon_version: "1.0.0".to_string(),
            description: String::new(),
            profile_intent: Some(AddonProfileIntent::Runtime),
            usage_class: Some(AddonUsageClass::Automated),
            profiles: vec![AddonProfile::HumanDev],
            exported_surfaces: vec![AddonExportSurface::CliBinary],
            category: "Other".to_string(),
            builder_weight: None,
            requires: vec![],
            tools: vec![
                LoadedTool {
                    name: "required".to_string(),
                    description: String::new(),
                    default_enabled: true,
                    default_version: "1.0".to_string(),
                    supported_versions: vec![],
                },
                LoadedTool {
                    name: "optional".to_string(),
                    description: String::new(),
                    default_enabled: false,
                    default_version: "2.0".to_string(),
                    supported_versions: vec![],
                },
            ],
            groups: HashMap::new(),
            builder_template: None,
            runtime_template: Some(
                "RUN install required\n\
                 {% if tools.optional.enabled %}RUN install optional{% endif %}"
                    .to_string(),
            ),
        };

        // Only required is enabled
        let mut tools = HashMap::new();
        tools.insert(
            "required".to_string(),
            ToolConfig {
                enabled: true,
                version: "1.0".to_string(),
            },
        );

        let result = render_runtime(&addon, &tools).unwrap();
        assert!(result.contains("install required"));
        assert!(!result.contains("install optional"));
    }

    #[test]
    fn builder_order_key_sorts_correctly() {
        let heavy = LoadedAddon {
            name: "a".to_string(),
            addon_version: "1.0.0".to_string(),
            description: String::new(),
            profile_intent: Some(AddonProfileIntent::Build),
            usage_class: Some(AddonUsageClass::Automated),
            profiles: vec![AddonProfile::HumanDev, AddonProfile::HeadlessRunner],
            exported_surfaces: vec![AddonExportSurface::BuildToolchain],
            category: "Other".to_string(),
            builder_weight: Some("heavy".to_string()),
            tools: vec![],
            requires: vec![],
            groups: HashMap::new(),
            builder_template: Some("FROM debian".to_string()),
            runtime_template: None,
        };
        let medium = LoadedAddon {
            name: "b".to_string(),
            addon_version: "1.0.0".to_string(),
            description: String::new(),
            profile_intent: Some(AddonProfileIntent::Build),
            usage_class: Some(AddonUsageClass::Automated),
            profiles: vec![AddonProfile::HumanDev, AddonProfile::HeadlessRunner],
            exported_surfaces: vec![AddonExportSurface::BuildToolchain],
            category: "Other".to_string(),
            builder_weight: Some("medium".to_string()),
            tools: vec![],
            requires: vec![],
            groups: HashMap::new(),
            builder_template: Some("FROM debian".to_string()),
            runtime_template: None,
        };
        let none = LoadedAddon {
            name: "c".to_string(),
            addon_version: "1.0.0".to_string(),
            description: String::new(),
            profile_intent: Some(AddonProfileIntent::Runtime),
            usage_class: Some(AddonUsageClass::Automated),
            profiles: vec![AddonProfile::HumanDev],
            exported_surfaces: vec![AddonExportSurface::RuntimeService],
            category: "Other".to_string(),
            builder_weight: None,
            tools: vec![],
            requires: vec![],
            groups: HashMap::new(),
            builder_template: None,
            runtime_template: None,
        };

        assert!(heavy.builder_order_key() < medium.builder_order_key());
        assert!(medium.builder_order_key() < none.builder_order_key());
    }

    #[test]
    fn missing_dir_gives_clear_error() {
        let result = load_from_dir(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("install script"),
            "error should mention install: {}",
            err
        );
    }

    #[test]
    fn latest_version_disables_pinning() {
        let addon = LoadedAddon {
            name: "test".to_string(),
            addon_version: "1.0.0".to_string(),
            description: String::new(),
            profile_intent: Some(AddonProfileIntent::Runtime),
            usage_class: Some(AddonUsageClass::Automated),
            profiles: vec![AddonProfile::HumanDev],
            exported_surfaces: vec![AddonExportSurface::CliBinary],
            category: "Other".to_string(),
            builder_weight: None,
            requires: vec![],
            tools: vec![LoadedTool {
                name: "mytool".to_string(),
                description: String::new(),
                default_enabled: true,
                default_version: "1.0".to_string(),
                supported_versions: vec![],
            }],
            groups: HashMap::new(),
            builder_template: None,
            runtime_template: Some(
                "{% if tools.mytool.version %}RUN install mytool={{ tools.mytool.version }}{% else %}RUN install mytool{% endif %}"
                    .to_string(),
            ),
        };

        // When version = "latest", the template should get an empty version
        // and take the unpinned branch.
        let mut tools = HashMap::new();
        tools.insert(
            "mytool".to_string(),
            ToolConfig {
                enabled: true,
                version: "latest".to_string(),
            },
        );

        let result = render_runtime(&addon, &tools).unwrap();
        assert_eq!(result.trim(), "RUN install mytool", "got: {}", result);

        // Verify `pinned` is also false
        let ctx = build_template_context(&addon, &tools);
        let pinned = ctx
            .get_attr("tools")
            .unwrap()
            .get_attr("mytool")
            .unwrap()
            .get_attr("pinned")
            .unwrap();
        assert!(!pinned.is_true(), "pinned should be false for 'latest'");
    }

    #[test]
    fn category_derived_from_dir_name() {
        let dir = tempfile::tempdir().unwrap();
        write_test_yaml(
            dir.path(),
            "ai",
            "test-ai",
            r#"
name: test-ai
version: "1.0.0"
description: "Test AI addon"
tools: []
"#,
        );

        let addons = load_from_dir(dir.path()).unwrap();
        assert_eq!(addons[0].category, "AI Providers");
        assert_eq!(addons[0].description, "Test AI addon");
    }

    #[test]
    fn addon_metadata_fields_are_loaded() {
        let dir = tempfile::tempdir().unwrap();
        write_test_yaml(
            dir.path(),
            "ai",
            "test-ai",
            r#"
name: test-ai
version: "1.0.0"
description: "Test AI addon"
profile_intent: provider-cli
usage_class: manual-escalation-only
profiles: ["human-dev"]
exported_surfaces: ["cli-binary", "config-files"]
tools: []
runtime: |
  RUN true
"#,
        );

        let addons = load_from_dir(dir.path()).unwrap();
        assert_eq!(
            addons[0].profile_intent,
            Some(AddonProfileIntent::ProviderCli)
        );
        assert_eq!(
            addons[0].usage_class,
            Some(AddonUsageClass::ManualEscalationOnly)
        );
        assert_eq!(addons[0].profiles, vec![AddonProfile::HumanDev]);
        assert_eq!(
            addons[0].exported_surfaces,
            vec![
                AddonExportSurface::CliBinary,
                AddonExportSurface::ConfigFiles
            ]
        );
        assert!(addon_metadata_warnings(&addons).is_empty());
    }

    #[test]
    fn addon_metadata_warnings_detect_missing_and_headless_provider_cli() {
        let addons = vec![
            LoadedAddon {
                name: "untagged".to_string(),
                addon_version: "1.0.0".to_string(),
                description: String::new(),
                profile_intent: None,
                usage_class: None,
                profiles: vec![],
                exported_surfaces: vec![],
                category: "Other".to_string(),
                builder_weight: None,
                tools: vec![],
                requires: vec![],
                groups: HashMap::new(),
                builder_template: None,
                runtime_template: None,
            },
            LoadedAddon {
                name: "bad-provider".to_string(),
                addon_version: "1.0.0".to_string(),
                description: String::new(),
                profile_intent: Some(AddonProfileIntent::ProviderCli),
                usage_class: Some(AddonUsageClass::ManualEscalationOnly),
                profiles: vec![AddonProfile::HumanDev, AddonProfile::HeadlessRunner],
                exported_surfaces: vec![AddonExportSurface::CliBinary],
                category: "Other".to_string(),
                builder_weight: None,
                tools: vec![],
                requires: vec![],
                groups: HashMap::new(),
                builder_template: None,
                runtime_template: None,
            },
        ];

        let warnings = addon_metadata_warnings(&addons);
        assert!(warnings.iter().any(|w| w.contains("no profile_intent")));
        assert!(warnings.iter().any(|w| w.contains("no usage_class")));
        assert!(warnings.iter().any(|w| w.contains("no profiles")));
        assert!(warnings.iter().any(|w| w.contains("no exported_surfaces")));
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("addon-without-install-steps"))
        );
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("subscription-cli-headless-leak"))
        );
    }

    #[test]
    fn addon_profile_compatibility_warnings_detect_selected_mismatch() {
        let addons = vec![
            LoadedAddon {
                name: "ai-cli".to_string(),
                addon_version: "1.0.0".to_string(),
                description: String::new(),
                profile_intent: Some(AddonProfileIntent::ProviderCli),
                usage_class: Some(AddonUsageClass::ManualEscalationOnly),
                profiles: vec![AddonProfile::HumanDev],
                exported_surfaces: vec![AddonExportSurface::CliBinary],
                category: "Other".to_string(),
                builder_weight: None,
                tools: vec![],
                requires: vec![],
                groups: HashMap::new(),
                builder_template: None,
                runtime_template: None,
            },
            LoadedAddon {
                name: "runtime".to_string(),
                addon_version: "1.0.0".to_string(),
                description: String::new(),
                profile_intent: Some(AddonProfileIntent::Runtime),
                usage_class: Some(AddonUsageClass::Automated),
                profiles: vec![AddonProfile::HumanDev, AddonProfile::HeadlessRunner],
                exported_surfaces: vec![AddonExportSurface::RuntimeService],
                category: "Other".to_string(),
                builder_weight: None,
                tools: vec![],
                requires: vec![],
                groups: HashMap::new(),
                builder_template: None,
                runtime_template: None,
            },
        ];

        let selected = vec![
            "runtime".to_string(),
            "missing-addon".to_string(),
            "ai-cli".to_string(),
        ];
        let warnings = addon_profile_compatibility_warnings(&addons, &selected, "headless-runner");

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("ai-cli"));
        assert!(warnings[0].contains("headless-runner"));
    }

    #[test]
    fn addon_catalog_index_is_sorted_and_includes_metadata() {
        let addons = vec![
            LoadedAddon {
                name: "runtime".to_string(),
                addon_version: "1.0.0".to_string(),
                description: "Runtime addon".to_string(),
                profile_intent: Some(AddonProfileIntent::Runtime),
                usage_class: Some(AddonUsageClass::Automated),
                profiles: vec![AddonProfile::HumanDev, AddonProfile::HeadlessRunner],
                exported_surfaces: vec![
                    AddonExportSurface::RuntimeService,
                    AddonExportSurface::CliBinary,
                ],
                category: "Other".to_string(),
                builder_weight: None,
                tools: vec![LoadedTool {
                    name: "runner".to_string(),
                    description: "Runs the sample runtime".to_string(),
                    default_enabled: true,
                    default_version: "2.0".to_string(),
                    supported_versions: vec!["2.0".to_string()],
                }],
                requires: vec!["base".to_string()],
                groups: HashMap::new(),
                builder_template: None,
                runtime_template: None,
            },
            LoadedAddon {
                name: "ai-cli".to_string(),
                addon_version: "1.0.0".to_string(),
                description: "Provider CLI".to_string(),
                profile_intent: Some(AddonProfileIntent::ProviderCli),
                usage_class: Some(AddonUsageClass::ManualEscalationOnly),
                profiles: vec![AddonProfile::HumanDev],
                exported_surfaces: vec![AddonExportSurface::CliBinary],
                category: "Other".to_string(),
                builder_weight: None,
                tools: vec![],
                requires: vec![],
                groups: HashMap::new(),
                builder_template: None,
                runtime_template: None,
            },
        ];

        let index = addon_catalog_index(&addons);

        assert_eq!(index.schema_version, ADDON_CATALOG_SCHEMA_VERSION);
        assert_eq!(index.addons[0].name, "ai-cli");
        assert_eq!(index.addons[1].name, "runtime");
        assert_eq!(
            index.addons[0].usage_class.as_deref(),
            Some("manual-escalation-only")
        );
        assert_eq!(
            index.addons[1].profiles,
            vec!["human-dev".to_string(), "headless-runner".to_string()]
        );
        assert_eq!(
            index.addons[1].exported_surfaces,
            vec!["runtime-service".to_string(), "cli-binary".to_string()]
        );
        assert_eq!(index.addons[1].requires, vec!["base".to_string()]);
        assert_eq!(index.addons[1].tools[0].name, "runner");
        assert_eq!(
            index.addons[1].tools[0].description,
            "Runs the sample runtime"
        );
    }

    // -----------------------------------------------------------------------
    // BR-CLEANUP-ARCH item 3 — disable-then-purge generalization tests.
    // These tests load the real `addons/` tree from the repo so they catch
    // regressions in the YAML purge_template templates as well as the
    // template renderer.
    // -----------------------------------------------------------------------

    fn repo_addons_dir() -> std::path::PathBuf {
        // CARGO_MANIFEST_DIR points at /workspace/cli; addons/ lives one up.
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("addons")
    }

    fn load_repo_addon(name: &str) -> LoadedAddon {
        let addons = load_from_dir(&repo_addons_dir()).unwrap();
        addons
            .into_iter()
            .find(|a| a.name == name)
            .unwrap_or_else(|| panic!("addon '{}' not found in repo addons dir", name))
    }

    #[test]
    fn repo_addon_tools_have_user_visible_descriptions() {
        let addons = load_from_dir(&repo_addons_dir()).unwrap();
        let missing: Vec<String> = addons
            .iter()
            .flat_map(|addon| {
                addon.tools.iter().filter_map(|tool| {
                    if tool.description.trim().is_empty() {
                        Some(format!("{}.{}", addon.name, tool.name))
                    } else {
                        None
                    }
                })
            })
            .collect();

        assert!(
            missing.is_empty(),
            "addon tools must describe their purpose for generated aibox.toml comments: {missing:?}"
        );
    }

    #[test]
    fn production_tool_addons_render_enabled_and_disabled_paths() {
        for name in ["go-quality", "supply-chain", "release", "go-release"] {
            let addon = load_repo_addon(name);
            let enabled = all_enabled_tools(&addon);
            if addon.builder_template.is_some() {
                let rendered = render_builder(&addon, &enabled).unwrap().unwrap();
                assert!(!rendered.trim().is_empty(), "{name} builder must render");
            }
            let rendered = render_runtime(&addon, &enabled).unwrap();
            assert!(!rendered.trim().is_empty(), "{name} runtime must render");

            let disabled = render_runtime(&addon, &all_disabled_tools(&addon)).unwrap();
            for tool in &addon.tools {
                assert!(
                    disabled.contains(&format!("rm -f /usr/local/bin/{}", tool.name))
                        || disabled.contains(&format!("rm -f /usr/local/bin/{} ", tool.name)),
                    "{name}.{} must purge its disabled binary: {disabled}",
                    tool.name
                );
            }
        }
    }

    fn all_disabled_tools(addon: &LoadedAddon) -> HashMap<String, ToolConfig> {
        addon
            .tools
            .iter()
            .map(|t| {
                (
                    t.name.clone(),
                    ToolConfig {
                        enabled: false,
                        version: t.default_version.clone(),
                    },
                )
            })
            .collect()
    }

    fn all_enabled_tools(addon: &LoadedAddon) -> HashMap<String, ToolConfig> {
        addon
            .tools
            .iter()
            .map(|t| {
                (
                    t.name.clone(),
                    ToolConfig {
                        enabled: true,
                        version: t.default_version.clone(),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn purge_kubernetes_emits_rm_when_kubectl_disabled() {
        let addon = load_repo_addon("kubernetes");
        let tools = all_disabled_tools(&addon);
        let rendered = render_runtime(&addon, &tools).unwrap();
        // Disabled tools must NOT install
        assert!(
            !rendered.contains("COPY --from=k8s-builder /build/bin/kubectl"),
            "disabled kubectl must not COPY: {rendered}"
        );
        // ... and MUST hard-purge
        assert!(
            rendered.contains("rm -f /usr/local/bin/kubectl"),
            "disabled kubectl must purge binary: {rendered}"
        );
        assert!(
            rendered.contains("rm -f /usr/local/bin/helm"),
            "disabled helm must purge binary: {rendered}"
        );
    }

    #[test]
    fn purge_kubernetes_skips_purge_when_enabled() {
        let addon = load_repo_addon("kubernetes");
        let tools = all_enabled_tools(&addon);
        let rendered = render_runtime(&addon, &tools).unwrap();
        assert!(
            rendered.contains("COPY --from=k8s-builder /build/bin/kubectl"),
            "enabled kubectl must COPY: {rendered}"
        );
        assert!(
            !rendered.contains("rm -f /usr/local/bin/kubectl"),
            "enabled kubectl must not be purged: {rendered}"
        );
    }

    #[test]
    fn kubernetes_builder_verifies_archives_using_upstream_filenames() {
        let addon = load_repo_addon("kubernetes");
        let tools = all_enabled_tools(&addon);
        let rendered = render_builder(&addon, &tools).unwrap().unwrap();

        for expected in [
            r#"HELM_ARCHIVE="helm-v${HELM_VERSION}-linux-${ARCH}.tar.gz""#,
            r#"(cd /tmp && sha256sum -c "${HELM_ARCHIVE}.sha256sum")"#,
            r#"KUSTOMIZE_ARCHIVE="kustomize_v${KUSTOMIZE_VERSION}_linux_${ARCH}.tar.gz""#,
            r#"(cd /tmp && grep "${KUSTOMIZE_ARCHIVE}$" kustomize.checksums.txt | sha256sum -c -)"#,
            r#"K9S_ARCHIVE="k9s_Linux_${ARCH}.tar.gz""#,
            r#"(cd /tmp && grep "${K9S_ARCHIVE}$" k9s_checksums.sha256 | sha256sum -c -)"#,
        ] {
            assert!(
                rendered.contains(expected),
                "Kubernetes builder must verify the downloaded upstream filename ({expected}): {rendered}"
            );
        }

        for stale in [
            "sha256sum -c /tmp/helm.tar.gz.sha256sum",
            "kustomize.checksums.txt | sha256sum -c &&",
            "k9s_checksums.sha256 | sha256sum -c &&",
        ] {
            assert!(
                !rendered.contains(stale),
                "Kubernetes builder must not verify a renamed archive ({stale}): {rendered}"
            );
        }
    }

    #[test]
    fn infrastructure_builder_verifies_archives_using_upstream_filenames() {
        let addon = load_repo_addon("infrastructure");
        let tools = all_enabled_tools(&addon);
        let rendered = render_builder(&addon, &tools).unwrap().unwrap();

        for expected in [
            r#"TOFU_ARCHIVE="tofu_${TOFU_VERSION}_linux_${ARCH}.tar.gz""#,
            r#"(cd /tmp && grep "${TOFU_ARCHIVE}$" tofu_SHA256SUMS | sha256sum -c -)"#,
            r#"PACKER_ARCHIVE="packer_${PACKER_VERSION}_linux_${ARCH}.zip""#,
            r#"(cd /tmp && grep "${PACKER_ARCHIVE}$" packer_SHA256SUMS | sha256sum -c -)"#,
        ] {
            assert!(
                rendered.contains(expected),
                "Infrastructure builder must verify the downloaded upstream filename ({expected}): {rendered}"
            );
        }

        for stale in [
            "grep \"tofu_${TOFU_VERSION}_linux_${ARCH}.tar.gz\" /tmp/tofu_SHA256SUMS | sha256sum -c",
            "grep \"packer_${PACKER_VERSION}_linux_${ARCH}.zip\" /tmp/packer_SHA256SUMS | sha256sum -c",
        ] {
            assert!(
                !rendered.contains(stale),
                "Infrastructure builder must not verify a renamed archive ({stale}): {rendered}"
            );
        }
    }

    #[test]
    fn infrastructure_runtime_installs_ansible_in_isolated_venv() {
        let addon = load_repo_addon("infrastructure");
        let tools = all_enabled_tools(&addon);
        let rendered = render_runtime(&addon, &tools).unwrap();

        let venv_install = rendered
            .find("python3-venv")
            .expect("enabled Ansible must install python3-venv: {rendered}");
        let ansible_install = rendered
            .find("/opt/aibox/ansible/bin/pip install --no-cache-dir 'ansible==")
            .expect("enabled Ansible must be installed in its virtual environment: {rendered}");
        assert!(
            venv_install < ansible_install,
            "python3-venv must be installed before the Ansible virtual environment: {rendered}"
        );
        assert!(
            rendered.contains("ln -sf \"$bin\" \"/usr/local/bin/$(basename \"$bin\")\""),
            "Ansible commands must be exposed on PATH: {rendered}"
        );
    }

    #[test]
    fn infrastructure_runtime_installs_rootless_podman_prerequisites() {
        let addon = load_repo_addon("infrastructure");
        let tools = all_enabled_tools(&addon);
        let rendered = render_runtime(&addon, &tools).unwrap();

        for expected in [
            "podman-compose",
            "fuse-overlayfs",
            "slirp4netns",
            "uidmap",
            "aibox:100000:65536",
            "/etc/containers/containers.conf",
            "cgroup_manager = \"cgroupfs\"",
        ] {
            assert!(
                rendered.contains(expected),
                "Podman runtime must include {expected}: {rendered}"
            );
        }
    }

    #[test]
    fn pip_packaged_runtime_tools_use_isolated_venvs() {
        for (addon_name, venv, command) in [
            ("cloud-azure", "azure-cli", "az"),
            ("python", "poetry", "poetry"),
            ("python", "pdm", "pdm"),
        ] {
            let addon = load_repo_addon(addon_name);
            let tools = all_enabled_tools(&addon);
            let rendered = render_runtime(&addon, &tools).unwrap();
            let venv_path = format!("/opt/aibox/{venv}/bin/pip install");
            let command_path = format!("/opt/aibox/{venv}/bin/{command}");
            assert!(
                rendered.contains("python3-venv")
                    && rendered.contains(&venv_path)
                    && rendered.contains(&command_path),
                "{addon_name} must install {command} through the {venv} virtual environment: {rendered}"
            );
            assert!(
                !rendered.contains("RUN pip3 install"),
                "{addon_name} must not install Python packages into Debian's externally managed Python: {rendered}"
            );
        }
    }

    #[test]
    fn purge_cloud_aws_uninstalls_when_disabled() {
        let addon = load_repo_addon("cloud-aws");
        let tools = all_disabled_tools(&addon);
        let rendered = render_runtime(&addon, &tools).unwrap();
        assert!(
            rendered.contains("rm -rf /usr/local/aws-cli"),
            "disabled aws-cli must purge install dir: {rendered}"
        );
    }

    #[test]
    fn cloud_aws_installer_vendors_and_verifies_documented_signing_key() {
        let addon = load_repo_addon("cloud-aws");
        let rendered = render_runtime(&addon, &all_enabled_tools(&addon)).unwrap();

        assert!(
            rendered.contains("gpg gpg-agent")
                && rendered.contains("AWS_CLI_PGP_KEY_BASE64=")
                && rendered.contains("FB5DB77FD5C118B80511ADA8A6310ACC4672475C")
                && rendered.contains("gpg --batch --import /tmp/aws-cli-public-key.asc"),
            "AWS CLI verification must use the fingerprint-checked key from AWS's install guide: {rendered}"
        );
        assert!(rendered.contains("gpg --verify /tmp/awscli.sig /tmp/awscli.zip"));
        assert!(
            !rendered.contains("gpg --keyserver"),
            "AWS CLI builds must not depend on external keyserver availability: {rendered}"
        );
    }

    #[test]
    fn purge_cloud_azure_removes_venv_when_disabled() {
        let addon = load_repo_addon("cloud-azure");
        let tools = all_disabled_tools(&addon);
        let rendered = render_runtime(&addon, &tools).unwrap();
        assert!(
            rendered.contains("rm -rf /opt/aibox/azure-cli"),
            "disabled azure-cli must remove its virtual environment: {rendered}"
        );
    }

    #[test]
    fn purge_cloud_gcp_apt_purges_when_disabled() {
        let addon = load_repo_addon("cloud-gcp");
        let tools = all_disabled_tools(&addon);
        let rendered = render_runtime(&addon, &tools).unwrap();
        assert!(
            rendered.contains("apt-get purge -y --auto-remove google-cloud-cli"),
            "disabled gcloud-cli must apt purge: {rendered}"
        );
    }

    #[test]
    fn purge_infrastructure_handles_each_tool() {
        let addon = load_repo_addon("infrastructure");
        let tools = all_disabled_tools(&addon);
        let rendered = render_runtime(&addon, &tools).unwrap();
        assert!(rendered.contains("rm -f /usr/local/bin/tofu"));
        assert!(rendered.contains("rm -f /usr/local/bin/packer"));
        assert!(rendered.contains("apt-get purge -y podman podman-compose"));
        assert!(rendered.contains("rm -rf /opt/aibox/ansible"));
        assert!(rendered.contains("rm -f /usr/local/bin/ansible*"));
    }

    #[test]
    fn purge_audio_voice_uses_dpkg_query() {
        let addon = load_repo_addon("audio-voice");
        let tools = all_disabled_tools(&addon);
        let rendered = render_runtime(&addon, &tools).unwrap();
        assert!(
            rendered.contains("dpkg-query -W -f='${Status}' sox"),
            "disabled sox must use dpkg-query guard: {rendered}"
        );
        assert!(rendered.contains("apt-get purge -y --auto-remove sox"));
    }

    #[test]
    fn purge_preview_addons_each_handle_tool_disable() {
        for name in ["preview-archive", "preview-enhanced", "data-preview"] {
            let addon = load_repo_addon(name);
            let tools = all_disabled_tools(&addon);
            let rendered = render_runtime(&addon, &tools).unwrap();
            assert!(
                rendered.contains("apt-get purge") || rendered.contains("rm -f /usr/local/bin/"),
                "addon '{name}' must emit a purge step when all tools disabled: {rendered}"
            );
        }
    }

    #[test]
    fn docs_hugo_is_default_non_installed() {
        let addon = load_repo_addon("docs-hugo");
        let hugo = addon
            .tools
            .iter()
            .find(|tool| tool.name == "hugo")
            .expect("docs-hugo should define a hugo tool");
        assert!(
            !hugo.default_enabled,
            "Hugo should be opt-in within the docs-hugo addon"
        );

        let rendered = render_runtime(&addon, &all_disabled_tools(&addon)).unwrap();
        assert!(
            !rendered.contains("hugo_extended_"),
            "disabled Hugo must not download/install Hugo: {rendered}"
        );
        assert!(rendered.contains("rm -f /usr/local/bin/hugo"));
    }

    #[test]
    fn docs_hugo_installer_uses_existing_release_assets() {
        let addon = load_repo_addon("docs-hugo");
        let mut tools = all_disabled_tools(&addon);
        tools.insert(
            "hugo".to_string(),
            ToolConfig {
                enabled: true,
                version: String::new(),
            },
        );

        let rendered = render_runtime(&addon, &tools).unwrap();
        assert!(
            rendered
                .contains("HUGO_ASSET=\"hugo_extended_${HUGO_VERSION}_linux-${HUGO_ARCH}.tar.gz\"")
        );
        assert!(rendered.contains("hugo_${HUGO_VERSION}_checksums.txt"));
        assert_eq!(
            rendered
                .matches(
                    "curl -fsSL --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 30"
                )
                .count(),
            2,
            "both Hugo release downloads must retry transient GitHub connection failures: {rendered}"
        );
        assert!(rendered.contains(
            "grep \" ${HUGO_ASSET}$\" /tmp/hugo_checksums.txt | sed 's#  .*#  /tmp/hugo.tar.gz#' | sha256sum -c"
        ));
    }

    #[test]
    fn opencode_installer_retries_transient_release_download_failures() {
        let addon = load_repo_addon("ai-opencode");
        let rendered = render_runtime(&addon, &all_enabled_tools(&addon)).unwrap();

        assert_eq!(
            rendered
                .matches(
                    "curl -fsSL --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 30"
                )
                .count(),
            2,
            "both pinned OpenCode release downloads must retry transient GitHub failures: {rendered}"
        );
        assert!(rendered.contains("/checksums.txt"));
        assert!(rendered.contains("sha256sum -c"));
    }

    #[test]
    fn node_installer_uses_verified_official_release_archive() {
        let addon = load_repo_addon("node");
        let rendered = render_runtime(&addon, &all_enabled_tools(&addon)).unwrap();

        assert!(rendered.contains("https://nodejs.org/dist/latest-v26.x"));
        assert!(rendered.contains("SHASUMS256.txt"));
        assert!(rendered.contains("sha256sum -c -"));
        assert!(
            rendered.contains("libatomic1"),
            "official Node.js ARM64 archives require libatomic.so.1: {rendered}"
        );
        assert!(
            !rendered.contains("deb.nodesource.com"),
            "Node installation must not depend on the retired NodeSource key endpoint: {rendered}"
        );
    }

    #[test]
    fn go_installer_uses_published_archive_digests() {
        let addon = load_repo_addon("go");
        let rendered = render_runtime(&addon, &all_enabled_tools(&addon)).unwrap();

        assert!(rendered.contains(
            r#"1.26.6:amd64) GO_SHA256="708effb774be8237570d0add163225abbdfaf4fca28b2611df167beba4feef89""#
        ));
        assert!(rendered.contains(
            r#"1.26.6:arm64) GO_SHA256="d0507e9e9d7fe012aae570108cbd76c15de879e17130ab8cb90d4d7445cb1f2e""#
        ));
        assert!(rendered.contains(r#"echo "${GO_SHA256}  /tmp/go.tar.gz" | sha256sum -c -"#));
        assert!(
            !rendered.contains(".tar.gz.sha256"),
            "go.dev does not publish per-archive checksum sidecars: {rendered}"
        );
        assert_eq!(
            addon.tools[0].supported_versions,
            ["1.25.12", "1.26.3", "1.26.4", "1.26.5", "1.26.6"]
        );
    }

    #[test]
    fn typst_installer_uses_published_archive_digests() {
        let addon = load_repo_addon("typst");
        let rendered = render_runtime(&addon, &all_enabled_tools(&addon)).unwrap();

        assert!(rendered.contains(
            r#"0.15.0:aarch64) TYPST_SHA256="cdf50ffc7b8ba759ed02200632eda3d78eb8b99aacb6611f4f75684990647620""#
        ));
        assert!(rendered.contains(
            r#"0.15.0:x86_64) TYPST_SHA256="59b207df01be2dab9f13e80f73d04d7ff8273ffd46b3dd1b9eef5c60f3eeabea""#
        ));
        assert!(rendered.contains(r#"echo "${TYPST_SHA256}  /tmp/typst.tar.xz" | sha256sum -c -"#));
        assert!(
            !rendered.contains(".tar.xz.sha256"),
            "Typst does not publish per-archive checksum sidecars: {rendered}"
        );
    }

    #[test]
    fn docs_mdbook_installer_uses_published_asset_digests() {
        let addon = load_repo_addon("docs-mdbook");
        let mut tools = all_disabled_tools(&addon);
        tools.insert(
            "mdbook".to_string(),
            ToolConfig {
                enabled: true,
                version: String::new(),
            },
        );
        let rendered = render_runtime(&addon, &tools).unwrap();

        assert!(rendered.contains(
            "aarch64) MDBOOK_SHA256=\"753e5c5c363ee8a56972344dcf91466f005a51db84a7aeffe427ae3ef83d6d44\""
        ));
        assert!(rendered.contains(
            "x86_64) MDBOOK_SHA256=\"5222beabd3e37dc5be0d18ff99b79058469354db5c220153a1b92db5ba12be89\""
        ));
        assert!(
            rendered.contains("echo \"${MDBOOK_SHA256}  /tmp/mdbook.tar.gz\" | sha256sum -c -")
        );
        assert!(
            !rendered.contains(".tar.gz.sha256"),
            "mdBook 0.5.4 does not publish separate checksum assets: {rendered}"
        );
    }
}
