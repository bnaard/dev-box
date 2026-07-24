//! Loads addon definitions from YAML files and renders Dockerfile templates.
//!
//! Addon YAML files are stored in `$XDG_CONFIG_HOME/aibox/addons/` with
//! category subdirectories (languages/, tools/, docs/, ai/).
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
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::addon_registry::{ToolConfig, ToolDef};

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

/// Get the addons directory path.
///
/// Checks `AIBOX_ADDONS_DIR` first. In source checkouts, fall back to the
/// repository's bundled `addons/` directory so `cargo run ... -- doctor` works
/// without requiring a separate global install. Packaged binaries still use the
/// XDG install path from `scripts/install.sh`.
pub fn addons_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("AIBOX_ADDONS_DIR") {
        return Ok(PathBuf::from(dir));
    }

    let repo_addons = Path::new(env!("CARGO_MANIFEST_DIR")).join("../addons");
    if repo_addons.is_dir() {
        return Ok(repo_addons);
    }

    crate::dirs::config_dir()
        .map(|d| d.join("addons"))
        .ok_or_else(|| anyhow::anyhow!("Could not determine XDG config directory"))
}

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
    let yaml: AddonYaml = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse addon YAML: {}", path.display()))?;

    let category = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(category_from_dir_name)
        .unwrap_or("Other")
        .to_string();

    Ok(LoadedAddon {
        name: yaml.name,
        addon_version: yaml.version,
        description: yaml.description,
        profile_intent: yaml.profile_intent,
        usage_class: yaml.usage_class,
        profiles: yaml.profiles,
        exported_surfaces: yaml.exported_surfaces,
        category,
        builder_weight: yaml.builder_weight,
        requires: yaml.requires,
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

// ---------------------------------------------------------------------------
// Global access
// ---------------------------------------------------------------------------

/// Initialize the addon store from the default XDG path. Call once at startup.
pub fn init() -> Result<()> {
    let dir = addons_dir()?;
    init_from_dir(&dir)
}

/// Initialize the addon store from a specific directory.
/// Used by tests to point at the repo's addons/ directory.
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
    fn addons_dir_defaults_to_repo_addons_in_source_checkout() {
        unsafe {
            std::env::remove_var("AIBOX_ADDONS_DIR");
        }
        let dir = addons_dir().unwrap();
        assert!(
            dir.ends_with("addons") && dir.is_dir(),
            "source checkout should resolve bundled addons dir, got {}",
            dir.display()
        );
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
        assert!(rendered.contains("pip3 uninstall -y ansible"));
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
        assert!(rendered.contains(
            "grep \" ${HUGO_ASSET}$\" /tmp/hugo_checksums.txt | sed 's#  .*#  /tmp/hugo.tar.gz#' | sha256sum -c"
        ));
    }
}
