use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// ExtraVolume — user-defined bind mount
// ---------------------------------------------------------------------------

/// A user-defined bind mount entry, from `[[container.extra_volumes]]` in
/// `aibox.toml` or `.aibox-local.toml`.
///
/// In TOML:
/// ```toml
/// [[container.extra_volumes]]
/// source = "~/.config/gh"
/// target = "/home/aibox/.config/gh"
///
/// [[container.extra_volumes]]
/// source = "~/.aws"
/// target = "/home/aibox/.aws"
/// read_only = true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtraVolume {
    /// Host-side path. Supports `~` expansion by Docker Compose. No `..` allowed.
    pub source: String,
    /// Container-side absolute path. Must start with `/`. No `..` allowed.
    pub target: String,
    /// Mount read-only. Defaults to false.
    #[serde(default)]
    pub read_only: bool,
}

/// Container image registry base URL.
pub const IMAGE_REGISTRY: &str = "ghcr.io/projectious-work/aibox";

/// Standard devcontainer directory name.
pub const DEVCONTAINER_DIR: &str = ".devcontainer";
/// Standard compose file name within devcontainer dir.
pub const COMPOSE_FILE: &str = ".devcontainer/docker-compose.yml";
/// Standard Dockerfile name within devcontainer dir.
pub const DOCKERFILE: &str = ".devcontainer/Dockerfile";
/// Standard devcontainer.json name.
pub const DEVCONTAINER_JSON: &str = ".devcontainer/devcontainer.json";
/// Standard container-side project mount used by generated aibox runtimes.
pub const CONTAINER_WORKSPACE_DIR: &str = "/workspace";

// ---------------------------------------------------------------------------
// Base image
// ---------------------------------------------------------------------------

/// Base image for the aibox container. Currently only Debian is supported;
/// Alpine is planned for later.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum BaseImage {
    #[default]
    Debian,
    // Alpine, // planned
}

impl std::fmt::Display for BaseImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseImage::Debian => write!(f, "debian"),
        }
    }
}

// ---------------------------------------------------------------------------
// aibox profile
// ---------------------------------------------------------------------------

/// Container usage profile. `human-dev` remains the default; `headless-runner`
/// is a warning-mode contract used to classify automation-safe addons before
/// a dedicated runner image exists.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum AiboxProfile {
    #[default]
    HumanDev,
    HeadlessRunner,
}

impl AiboxProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            AiboxProfile::HumanDev => "human-dev",
            AiboxProfile::HeadlessRunner => "headless-runner",
        }
    }
}

impl std::fmt::Display for AiboxProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// [aibox] section
// ---------------------------------------------------------------------------

fn default_config_schema() -> String {
    "1.0.0".to_string()
}

fn default_image_version() -> String {
    "latest".to_string()
}

fn default_api_version() -> String {
    "aibox.projectious.work/v1".to_string()
}

fn default_kind() -> String {
    "Workspace".to_string()
}

/// Root metadata for the workspace object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetadataSection {
    #[serde(default, alias = "project_name")]
    pub name: String,
}

/// Published base image selector.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageSection {
    #[serde(default = "default_image_version", alias = "release_version")]
    pub version: String,
    #[serde(default)]
    pub base: BaseImage,
}

impl Default for ImageSection {
    fn default() -> Self {
        Self {
            version: default_image_version(),
            base: BaseImage::Debian,
        }
    }
}

/// Top-level [aibox] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiboxSection {
    #[serde(default = "default_config_schema")]
    pub config_schema: String,
    #[serde(default, alias = "name")]
    pub project_name: String,
    #[serde(default = "default_image_version")]
    pub version: String,
    #[serde(default)]
    pub base: BaseImage,
    #[serde(default)]
    pub profile: AiboxProfile,
}

impl Default for AiboxSection {
    fn default() -> Self {
        Self {
            config_schema: default_config_schema(),
            project_name: String::new(),
            version: default_image_version(),
            base: BaseImage::Debian,
            profile: AiboxProfile::HumanDev,
        }
    }
}

// ---------------------------------------------------------------------------
// [container] section — UNCHANGED
// ---------------------------------------------------------------------------

/// [container] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSection {
    pub name: String,
    #[serde(default = "default_hostname")]
    pub hostname: String,
    /// Container user (default: "aibox"). Determines mount paths inside container.
    #[serde(default = "default_user")]
    pub user: String,
    #[serde(default)]
    pub post_create_command: Option<String>,
    /// Network keepalive — prevents OrbStack/VM NAT from dropping idle connections.
    #[serde(default)]
    pub keepalive: bool,
    /// Lifecycle commands. New configs use `[container.lifecycle]`; legacy
    /// root-level `post_create_command` and `keepalive` are migrated in.
    #[serde(default)]
    pub lifecycle: ContainerLifecycleSection,
    /// Extra environment variables injected into the container.
    /// Committed entries go in `aibox.toml`; secrets go in `.aibox-local.toml`.
    ///
    /// ```toml
    /// [container.environment]
    /// AWS_DEFAULT_REGION = "eu-west-1"
    /// ```
    #[serde(default)]
    pub environment: HashMap<String, String>,
    /// Additional bind mounts beyond the aibox defaults.
    /// Committed entries (shared caches) go in `aibox.toml`; personal credential
    /// directories go in `.aibox-local.toml` (gitignored).
    ///
    /// ```toml
    /// [[container.extra_volumes]]
    /// source = "~/.config/gh"
    /// target = "/home/aibox/.config/gh"
    /// ```
    #[serde(default)]
    pub extra_volumes: Vec<ExtraVolume>,
    /// Runtime resource warning thresholds used by `aibox doctor`.
    #[serde(default)]
    pub resource_thresholds: ResourceThresholdsSection,
    /// Published image selector. New configs use `[container.image]`; legacy
    /// `[image]` and `[aibox].version/base` are migrated into this section.
    #[serde(default)]
    pub image: ImageSection,
    /// Generated/input file paths used by the devcontainer generator.
    #[serde(default)]
    pub paths: ContainerPathsSection,
    /// Legacy audio bridge settings. New configs use top-level `[audio]`.
    #[serde(default)]
    pub audio: AudioSection,
}

fn default_user() -> String {
    "aibox".to_string()
}

fn default_hostname() -> String {
    "aibox".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ContainerLifecycleSection {
    #[serde(default)]
    pub post_create_command: Option<String>,
    #[serde(default)]
    pub keepalive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContainerPathsSection {
    #[serde(default = "default_devcontainer_json_path")]
    pub devcontainer_json: String,
    #[serde(default = "default_docker_compose_path")]
    pub docker_compose: String,
    #[serde(default = "default_docker_compose_override_path")]
    pub docker_compose_override: String,
    #[serde(default = "default_dockerfile_path")]
    pub dockerfile: String,
    #[serde(default = "default_dockerfile_local_path")]
    pub dockerfile_local: String,
    #[serde(default = "default_local_env_path")]
    pub local_env: String,
}

impl Default for ContainerPathsSection {
    fn default() -> Self {
        Self {
            devcontainer_json: default_devcontainer_json_path(),
            docker_compose: default_docker_compose_path(),
            docker_compose_override: default_docker_compose_override_path(),
            dockerfile: default_dockerfile_path(),
            dockerfile_local: default_dockerfile_local_path(),
            local_env: default_local_env_path(),
        }
    }
}

fn default_devcontainer_json_path() -> String {
    ".devcontainer/devcontainer.json".to_string()
}

fn default_docker_compose_path() -> String {
    ".devcontainer/docker-compose.yml".to_string()
}

fn default_docker_compose_override_path() -> String {
    ".devcontainer/docker-compose.override.yml".to_string()
}

fn default_dockerfile_path() -> String {
    ".devcontainer/Dockerfile".to_string()
}

fn default_dockerfile_local_path() -> String {
    ".devcontainer/Dockerfile.local".to_string()
}

fn default_local_env_path() -> String {
    ".aibox-local.env".to_string()
}

/// Runtime resource pressure warning thresholds.
///
/// These are intentionally warnings only. They help surface pressure before the
/// operating system kills processes, but they must not block normal workflows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceThresholdsSection {
    /// Warn when current cgroup memory usage exceeds this many MiB.
    #[serde(default)]
    pub memory_mib_warn: Option<u64>,
    /// Warn when total process count exceeds this number.
    #[serde(default = "default_process_count_warn")]
    pub process_count_warn: Option<usize>,
    /// Warn when processkit MCP Python process count exceeds this number.
    #[serde(default = "default_processkit_mcp_python_warn")]
    pub processkit_mcp_python_warn: Option<usize>,
    /// Warn when cgroup `oom_kill` exceeds this count.
    #[serde(default = "default_oom_kill_warn")]
    pub oom_kill_warn: Option<u64>,
}

impl Default for ResourceThresholdsSection {
    fn default() -> Self {
        Self {
            memory_mib_warn: None,
            process_count_warn: default_process_count_warn(),
            processkit_mcp_python_warn: default_processkit_mcp_python_warn(),
            oom_kill_warn: default_oom_kill_warn(),
        }
    }
}

fn default_process_count_warn() -> Option<usize> {
    Some(400)
}

fn default_processkit_mcp_python_warn() -> Option<usize> {
    Some(50)
}

fn default_oom_kill_warn() -> Option<u64> {
    Some(0)
}

// ---------------------------------------------------------------------------
// [context] section — merged with former [process]
// ---------------------------------------------------------------------------

/// [context] section — context system versioning and process packages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextSection {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default = "default_context_packages")]
    pub packages: Vec<String>,
}

fn default_schema_version() -> String {
    "1.0.0".to_string()
}

fn default_context_packages() -> Vec<String> {
    vec!["product".to_string()]
}

impl Default for ContextSection {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            packages: default_context_packages(),
        }
    }
}

/// Legacy [process] section for backward compatibility.
/// If present, packages are merged into [context].packages during load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyProcessSection {
    #[serde(default)]
    pub packages: Vec<String>,
}

// ---------------------------------------------------------------------------
// [ai] section
// ---------------------------------------------------------------------------

/// AI tool providers supported in aibox containers.
///
/// MCP-capable providers (have a built-in MCP client and a project-level
/// config file aibox can write to):
/// - `Claude` — `.mcp.json` at project root
/// - `Cursor` — `.cursor/mcp.json` at project root (host-side IDE only)
/// - `Gemini` — `.gemini/settings.json` (Gemini CLI)
/// - `OpenAI` — `.codex/config.toml` (OpenAI Codex CLI, binary: `codex`)
/// - `Continue` — `.continue/mcpServers/<name>.json` (Continue CLI, binary: `cn`)
/// - `Copilot` — `.mcp.json` at project root (GitHub Copilot CLI, binary: `copilot`)
///
/// Special MCP routing:
/// - `Mistral` — has MCP client capability via Python SDK and Le Chat,
///   but no local file-based config. aibox writes `.mcp.json` (the
///   Claude shape) when Mistral is selected so a custom Mistral
///   SDK-based CLI tool can read MCP server registrations from there.
///
/// Non-MCP providers (no built-in MCP client; aibox cannot register
/// processkit MCP servers; sync emits a warning):
/// - `Aider` — no native MCP client. Third-party experimental bridges
///   exist but are not yet stable.
///
/// Note: `Cursor` is a host-side IDE extension only — it has no container
/// An AI agent harness — the CLI tool installed in the container.
///
/// Cursor is a host-side IDE extension with MCP registration but no container
/// CLI binary and no in-container persistence directory. All other harnesses
/// have a corresponding `ai-<name>` addon that installs their CLI in the image.
///
/// The deserializer still accepts a few older config spellings so existing
/// aibox.toml files can be read and normalized by `aibox apply`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum AiHarness {
    Claude,
    Aider,
    Gemini,
    Cursor,
    Continue,
    Copilot,
    /// OpenAI Codex CLI.
    #[serde(rename = "codex", alias = "openai")]
    #[clap(name = "codex", alias = "openai")]
    Codex,
    /// Open-source multi-provider harness (Go-based).
    #[serde(rename = "opencode")]
    #[clap(name = "opencode")]
    OpenCode,
    /// Nous Research autonomous agent.
    Hermes,
    /// Mistral has no CLI harness; retained only so old config can be parsed.
    #[serde(rename = "mistral")]
    #[clap(skip)]
    Mistral,
}

impl std::fmt::Display for AiHarness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiHarness::Claude => write!(f, "claude"),
            AiHarness::Codex => write!(f, "codex"),
            AiHarness::Gemini => write!(f, "gemini"),
            AiHarness::Aider => write!(f, "aider"),
            AiHarness::Continue => write!(f, "continue"),
            AiHarness::Cursor => write!(f, "cursor"),
            AiHarness::Copilot => write!(f, "copilot"),
            AiHarness::OpenCode => write!(f, "opencode"),
            AiHarness::Hermes => write!(f, "hermes"),
            AiHarness::Mistral => write!(f, "mistral"),
        }
    }
}

impl AiHarness {
    /// Returns the actual CLI binary name for this harness.
    pub fn binary_name(&self) -> &'static str {
        match self {
            AiHarness::Claude => "claude",
            AiHarness::Codex => "codex",
            AiHarness::Gemini => "gemini",
            AiHarness::Aider => "aider",
            AiHarness::Continue => "cn",
            AiHarness::Cursor => "cursor",
            AiHarness::Copilot => "copilot",
            AiHarness::OpenCode => "opencode",
            AiHarness::Hermes => "hermes",
            AiHarness::Mistral => "mistral",
        }
    }

    /// Human-friendly display name, e.g. "Claude Code (claude)".
    pub fn display_name(&self) -> &'static str {
        match self {
            AiHarness::Claude => "Claude Code (claude)",
            AiHarness::Codex => "OpenAI Codex (codex)",
            AiHarness::Gemini => "Gemini CLI (gemini)",
            AiHarness::Aider => "Aider (aider)",
            AiHarness::Continue => "Continue (continue)",
            AiHarness::Cursor => "Cursor (cursor)",
            AiHarness::Copilot => "GitHub Copilot (copilot)",
            AiHarness::OpenCode => "OpenCode (opencode)",
            AiHarness::Hermes => "Hermes (hermes)",
            AiHarness::Mistral => "Mistral (mistral, legacy)",
        }
    }

    /// Addon name for this harness (e.g. "ai-claude").
    ///
    /// Host-only or legacy SDK entries return an empty string because they
    /// have no in-container CLI addon.
    pub fn addon_name(&self) -> String {
        match self {
            AiHarness::Cursor | AiHarness::Mistral => String::new(),
            _ => format!("ai-{}", self),
        }
    }

    pub fn from_addon_name(name: &str) -> Option<Self> {
        Self::all()
            .iter()
            .find(|harness| harness.addon_name() == name)
            .cloned()
    }

    /// Config directory mounted into the container (e.g. ".claude").
    pub fn config_dir(&self) -> Option<&'static str> {
        match self {
            AiHarness::Claude => Some(".claude"),
            AiHarness::Codex => Some(".codex"),
            AiHarness::Gemini => Some(".gemini"),
            AiHarness::Aider => Some(".aider"),
            AiHarness::Continue => Some(".continue"),
            AiHarness::Cursor => Some(".cursor"),
            AiHarness::Copilot => Some(".copilot"),
            AiHarness::OpenCode => Some(".opencode"),
            AiHarness::Hermes => Some(".hermes"),
            AiHarness::Mistral => None,
        }
    }

    /// Whether this is a real harness (vs legacy placeholder).
    pub fn is_active(&self) -> bool {
        !matches!(self, AiHarness::Mistral)
    }

    /// Returns all active harness variants (excluding legacy).
    pub fn all() -> &'static [AiHarness] {
        &[
            AiHarness::Claude,
            AiHarness::Codex,
            AiHarness::Gemini,
            AiHarness::Aider,
            AiHarness::Continue,
            AiHarness::Cursor,
            AiHarness::Copilot,
            AiHarness::OpenCode,
            AiHarness::Hermes,
        ]
    }
}

/// Backward-compatible alias — all existing code that references `AiProvider`
/// continues to compile. The `OpenAI` variant maps to `AiHarness::Codex` via
/// serde alias. New code should use `AiHarness` directly.
pub type AiProvider = AiHarness;

/// An AI model provider — the organization whose API key may be needed.
/// Declaring a provider is optional; it hints which API keys are available.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum AiModelProvider {
    Anthropic,
    #[serde(rename = "openai")]
    #[clap(name = "openai")]
    OpenAI,
    Google,
    Mistral,
}

impl std::fmt::Display for AiModelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiModelProvider::Anthropic => write!(f, "anthropic"),
            AiModelProvider::OpenAI => write!(f, "openai"),
            AiModelProvider::Google => write!(f, "google"),
            AiModelProvider::Mistral => write!(f, "mistral"),
        }
    }
}

#[allow(dead_code)]
impl AiModelProvider {
    /// The environment variable name for this provider's API key.
    pub fn api_key_env(&self) -> &'static str {
        match self {
            AiModelProvider::Anthropic => "ANTHROPIC_API_KEY",
            AiModelProvider::OpenAI => "OPENAI_API_KEY",
            AiModelProvider::Google => "GEMINI_API_KEY",
            AiModelProvider::Mistral => "MISTRAL_API_KEY",
        }
    }

    /// Human-friendly display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            AiModelProvider::Anthropic => "Anthropic (anthropic)",
            AiModelProvider::OpenAI => "OpenAI (openai)",
            AiModelProvider::Google => "Google (google)",
            AiModelProvider::Mistral => "Mistral (mistral)",
        }
    }

    /// Returns all model provider variants.
    pub fn all() -> &'static [AiModelProvider] {
        &[
            AiModelProvider::Anthropic,
            AiModelProvider::OpenAI,
            AiModelProvider::Google,
            AiModelProvider::Mistral,
        ]
    }
}

fn default_ai_harnesses() -> Vec<AiHarness> {
    vec![AiHarness::Claude]
}

/// Per-harness install controls under `[ai.harness.<name>]`.
///
/// `enabled` controls whether the harness participates in generated config.
/// `install` controls whether the corresponding in-container CLI addon is
/// selected. Host-only harnesses such as Cursor can be enabled without install.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AiHarnessConfig {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub install: Option<bool>,
    #[serde(default)]
    pub version: Option<String>,
}

/// [ai] section — AI harness and model provider configuration.
///
/// `harnesses` controls which CLI tools are installed in the container.
/// `model_providers` is optional — declares which API keys are available.
/// Legacy `providers` field is accepted for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSection {
    /// Which AI coding tools to install (determines addons, volumes, MCP config).
    /// Serde default is empty; `migrate_legacy()` applies the real default
    /// ([Claude]) when neither harnesses nor providers was explicitly set.
    #[serde(default)]
    pub harnesses: Vec<AiHarness>,

    /// Which model provider API keys are available (optional hint).
    #[serde(default)]
    pub model_providers: Vec<AiModelProvider>,

    /// Per-harness install controls. New configs should prefer this for
    /// version/install overrides while `harnesses` remains a compact selector.
    #[serde(default)]
    pub harness: HashMap<AiHarness, AiHarnessConfig>,

    /// Legacy field — accepted for backward compatibility during migration.
    /// When present and `harnesses` is empty, auto-migrated to `harnesses`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<AiHarness>,

    /// Canonical AGENTS.md and provider-specific pointer behavior. New configs
    /// write this as `[ai.agents]`; legacy top-level `[agents]` is migrated in.
    #[serde(default)]
    pub agents: AgentsSection,

    /// MCP gateway, permissions, and team-shared servers. New configs write
    /// this as `[ai.mcp]`; legacy top-level `[mcp]` is migrated in.
    #[serde(default)]
    pub mcp: McpSection,
}

impl AiSection {
    /// Migrate legacy `providers` → `harnesses` if needed.
    /// Call after deserialization and before any code reads `harnesses`.
    pub fn migrate_legacy(&mut self) {
        if self.harnesses.is_empty() && !self.providers.is_empty() {
            // Legacy format: move providers → harnesses
            self.harnesses = self.providers.drain(..).collect();
        }
        for (harness, config) in &self.harness {
            if config.enabled.unwrap_or(true) {
                if !self.harnesses.contains(harness) {
                    self.harnesses.push(harness.clone());
                }
            } else {
                self.harnesses.retain(|candidate| candidate != harness);
            }
        }
    }

    /// The effective harness list (after migration).
    #[allow(dead_code)]
    pub fn effective_harnesses(&self) -> &[AiHarness] {
        if self.harnesses.is_empty() && !self.providers.is_empty() {
            &self.providers
        } else {
            &self.harnesses
        }
    }

    pub fn harness_install_enabled(&self, harness: &AiHarness) -> bool {
        self.harness
            .get(harness)
            .and_then(|config| config.install)
            .unwrap_or(true)
    }

    pub fn harness_version(&self, harness: &AiHarness) -> Option<&str> {
        self.harness
            .get(harness)
            .and_then(|config| config.version.as_deref())
            .filter(|version| !version.is_empty())
    }
}

impl Default for AiSection {
    fn default() -> Self {
        Self {
            harnesses: default_ai_harnesses(),
            model_providers: Vec::new(),
            harness: HashMap::new(),
            providers: Vec::new(),
            agents: AgentsSection::default(),
            mcp: McpSection::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// [addons] section — REWRITTEN
// ---------------------------------------------------------------------------

/// Configuration for a single tool within an addon.
///
/// In TOML this appears as e.g. `python = { version = "3.13" }`,
/// `clippy = {}`, or `lazygit = { enabled = false }`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolEntry {
    /// Tool version. `None` when no version is specified (e.g. `clippy = {}`).
    pub version: Option<String>,
    /// Explicitly disable a default-enabled tool while keeping the addon.
    #[serde(default)]
    pub enabled: Option<bool>,
}

/// The `tools` sub-table of an addon, e.g. `[addons.python.tools]`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AddonToolsSection {
    #[serde(default)]
    pub tools: HashMap<String, ToolEntry>,
}

/// [addons] section — each key is an addon name mapping to its tools table.
///
/// In TOML:
/// ```toml
/// [addons.python.tools]
/// python = { version = "3.13" }
/// uv = { version = "0.7" }
/// ```
///
/// Deserialized as `HashMap<String, AddonToolsSection>` where the outer key
/// is the addon name (e.g. "python") and the inner map contains tool entries.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct AddonsSection {
    pub addons: HashMap<String, AddonToolsSection>,
}

// Custom deserialization: the TOML section `[addons]` is a table where each
// key is an addon name and each value is an `AddonToolsSection`. Serde by
// default would look for a field called `addons` inside the `[addons]` table,
// but in our TOML the addon names ARE the keys of the `[addons]` table. We
// use `deserialize_with` at the AiboxConfig level via a transparent wrapper.
impl<'de> Deserialize<'de> for AddonsSection {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let addons = HashMap::<String, AddonToolsSection>::deserialize(deserializer)?;
        Ok(AddonsSection { addons })
    }
}

#[allow(dead_code)]
impl AddonsSection {
    /// Check whether a specific addon is configured.
    pub fn has_addon(&self, name: &str) -> bool {
        self.addons.contains_key(name)
    }

    /// Get the tools section for a specific addon, if present.
    pub fn get_addon(&self, name: &str) -> Option<&AddonToolsSection> {
        self.addons.get(name)
    }

    /// Iterate over all configured addons.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &AddonToolsSection)> {
        self.addons.iter()
    }

    /// Check whether a specific addon contains a specific tool.
    pub fn has_tool(&self, addon: &str, tool: &str) -> bool {
        self.addons
            .get(addon)
            .and_then(|a| a.tools.get(tool))
            .is_some_and(|t| t.enabled.unwrap_or(true))
    }

    /// Get the version of a specific tool in an addon, if configured.
    pub fn tool_version(&self, addon: &str, tool: &str) -> Option<&str> {
        self.addons
            .get(addon)
            .and_then(|a| a.tools.get(tool))
            .and_then(|t| t.version.as_deref())
    }

    /// Convenience: check if the python addon is configured.
    pub fn has_python(&self) -> bool {
        self.has_addon("python")
    }

    /// Convenience: check if the rust addon is configured.
    pub fn has_rust(&self) -> bool {
        self.has_addon("rust")
    }

    /// Convenience: check if the node addon is configured.
    pub fn has_node(&self) -> bool {
        self.has_addon("node")
    }

    /// Convenience: check if the latex addon is configured.
    pub fn has_latex(&self) -> bool {
        self.has_addon("latex")
    }
}

// ---------------------------------------------------------------------------
// [skills] section — NEW
// ---------------------------------------------------------------------------

/// [skills] section — include/exclude overrides for skill deployment.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SkillsSection {
    #[serde(default, alias = "enabled")]
    pub include: Vec<String>,
    #[serde(default, alias = "disabled")]
    pub exclude: Vec<String>,
}

// ---------------------------------------------------------------------------
// [appearance] section — UNCHANGED
// ---------------------------------------------------------------------------

/// Color themes available across all tools (Zellij, Vim, Yazi, lazygit).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    GruvboxDark,
    CatppuccinMocha,
    CatppuccinLatte,
    Dracula,
    TokyoNight,
    Nord,
    Projectious,
}

/// Global light/dark preference applied on top of the selected theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum ThemeMode {
    /// Preserve the selected concrete theme.
    #[default]
    Auto,
    /// Prefer a light concrete palette. Falls back to Catppuccin Latte until
    /// more theme families provide first-class light variants.
    Light,
    /// Prefer a dark concrete palette. Keeps dark themes unchanged and maps
    /// known light variants to their dark counterpart.
    Dark,
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeMode::Auto => write!(f, "auto"),
            ThemeMode::Light => write!(f, "light"),
            ThemeMode::Dark => write!(f, "dark"),
        }
    }
}

fn default_theme_mode() -> ThemeMode {
    ThemeMode::default()
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::GruvboxDark => write!(f, "gruvbox-dark"),
            Theme::CatppuccinMocha => write!(f, "catppuccin-mocha"),
            Theme::CatppuccinLatte => write!(f, "catppuccin-latte"),
            Theme::Dracula => write!(f, "dracula"),
            Theme::TokyoNight => write!(f, "tokyo-night"),
            Theme::Nord => write!(f, "nord"),
            Theme::Projectious => write!(f, "projectious"),
        }
    }
}

fn default_theme() -> Theme {
    Theme::default()
}

/// Starship prompt presets.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum StarshipPreset {
    #[default]
    Default, // Clean, informative — dir, git, language, duration
    Plain,     // ASCII only — no Nerd Font needed
    Minimal,   // Just directory + git branch
    NerdFont,  // Full Nerd Font symbols
    Pastel,    // Soft powerline segments
    Bracketed, // [segments] in brackets
    Arrow,     // Powerline-style chevron/arrow segments (airline-style)
}

impl std::fmt::Display for StarshipPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StarshipPreset::Default => write!(f, "default"),
            StarshipPreset::Plain => write!(f, "plain"),
            StarshipPreset::Minimal => write!(f, "minimal"),
            StarshipPreset::NerdFont => write!(f, "nerd-font"),
            StarshipPreset::Pastel => write!(f, "pastel"),
            StarshipPreset::Bracketed => write!(f, "bracketed"),
            StarshipPreset::Arrow => write!(f, "arrow"),
        }
    }
}

fn default_prompt() -> StarshipPreset {
    StarshipPreset::default()
}

/// Default zellij layout for `aibox up`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum ConfigLayout {
    /// VS Code-like: Yazi sidebar, Vim editor, stacked terminals
    #[default]
    Dev,
    /// One tool per tab, fullscreen, zero distraction
    Focus,
    /// Side-by-side coding with AI: yazi+vim left (50%), claude right (50%)
    Cowork,
    /// Cowork swapped: yazi+ai left (40%), vim editor right (60%)
    CoworkSwap,
    /// Yazi-focused with large preview and AI pane
    Browse,
    /// AI-first: Yazi left (60%), AI agent right (40%), no editor on first screen
    Ai,
}

impl std::fmt::Display for ConfigLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLayout::Dev => write!(f, "dev"),
            ConfigLayout::Focus => write!(f, "focus"),
            ConfigLayout::Cowork => write!(f, "cowork"),
            ConfigLayout::CoworkSwap => write!(f, "cowork-swap"),
            ConfigLayout::Browse => write!(f, "browse"),
            ConfigLayout::Ai => write!(f, "ai"),
        }
    }
}

fn default_layout() -> ConfigLayout {
    ConfigLayout::default()
}

/// Zellij runtime status presentation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum ZellijStatusMode {
    /// Native aibox key-hint plugin plus native aibox runtime status plugin.
    Native,
    /// Legacy shell fallback: built-in Zellij status bar plus `aibox-status --watch`.
    #[default]
    Shell,
    /// Hide aibox-provided status rows from generated layouts.
    Hidden,
}

impl std::fmt::Display for ZellijStatusMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZellijStatusMode::Native => write!(f, "native"),
            ZellijStatusMode::Shell => write!(f, "shell"),
            ZellijStatusMode::Hidden => write!(f, "hidden"),
        }
    }
}

fn default_zellij_status_mode() -> ZellijStatusMode {
    ZellijStatusMode::default()
}

/// [customization.zellij_status] section — Zellij status/keybar presentation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZellijStatusSection {
    #[serde(default = "default_zellij_status_mode")]
    pub mode: ZellijStatusMode,
}

impl Default for ZellijStatusSection {
    fn default() -> Self {
        Self {
            mode: default_zellij_status_mode(),
        }
    }
}

/// [customization] section — color theme, shell prompt, and zellij layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomizationSection {
    #[serde(default = "default_theme")]
    pub theme: Theme,
    #[serde(default = "default_theme_mode")]
    pub mode: ThemeMode,
    #[serde(default = "default_prompt")]
    pub prompt: StarshipPreset,
    #[serde(default = "default_layout")]
    pub layout: ConfigLayout,
    #[serde(default)]
    pub zellij_status: ZellijStatusSection,
}

impl CustomizationSection {
    /// Resolve the concrete palette rendered into tool config files.
    ///
    /// `theme` remains the user's selected concrete/default palette for
    /// backward compatibility. `mode` is a global override layered on top.
    pub fn resolved_theme(&self) -> Theme {
        match self.mode {
            ThemeMode::Auto => self.theme.clone(),
            ThemeMode::Light => Theme::CatppuccinLatte,
            ThemeMode::Dark => match self.theme {
                Theme::CatppuccinLatte => Theme::CatppuccinMocha,
                _ => self.theme.clone(),
            },
        }
    }
}

impl Default for CustomizationSection {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            mode: default_theme_mode(),
            prompt: default_prompt(),
            layout: default_layout(),
            zellij_status: ZellijStatusSection::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// [processkit] section — content layer source (skills, primitives, processes)
// ---------------------------------------------------------------------------

/// [processkit] section — configures the processkit-compatible source
/// the project consumes content from.
///
/// processkit ships skills and primitives that aibox installs into the
/// project. The default upstream is the canonical projectious-work/processkit
/// repo. Companies can fork processkit and have projects consume the fork by
/// changing `source` to point at their fork.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessKitSection {
    /// Git URL of the processkit-compatible source.
    #[serde(default = "default_processkit_source")]
    pub source: String,
    /// Semver tag of the processkit source to consume. The sentinel value
    /// `"unset"` means "no version pinned yet" — downstream code can detect
    /// this and skip processkit fetching until a real version is set.
    #[serde(default = "default_processkit_version")]
    pub version: String,
    /// Subdirectory within the source repo containing the processkit content.
    #[serde(default = "default_processkit_src_path")]
    pub src_path: String,
    /// Optional branch name. If set, tracks a moving branch instead of a
    /// pinned tag (discouraged but supported).
    #[serde(default)]
    pub branch: Option<String>,
    /// URL template for the release-asset tarball, with `{source}`,
    /// `{version}`, `{org}`, and `{name}` placeholders. When unset, the
    /// fetcher uses the GitHub-style default
    /// `{source}/releases/download/{version}/{name}-{version}.tar.gz`.
    /// Set this to point at non-GitHub hosts (Gitea, GitLab, self-hosted)
    /// that serve release assets at a different URL shape.
    #[serde(default)]
    pub release_asset_url_template: Option<String>,
    /// Context schema metadata. New configs write `[processkit.context]`;
    /// legacy top-level `[context]` is migrated in.
    #[serde(default)]
    pub context: ContextSection,
}

fn default_processkit_source() -> String {
    crate::processkit_vocab::PROCESSKIT_GIT_SOURCE.to_string()
}

fn default_processkit_version() -> String {
    "unset".to_string()
}

fn default_processkit_src_path() -> String {
    "src".to_string()
}

/// Sentinel version value meaning "no processkit version pinned yet".
pub const PROCESSKIT_VERSION_UNSET: &str = "unset";
/// Sentinel value meaning "resolve to the latest available tag at sync time".
pub const PROCESSKIT_VERSION_LATEST: &str = "latest";

// ---------------------------------------------------------------------------
// .aibox-local.toml — gitignored personal overlay
// ---------------------------------------------------------------------------

/// Personal, gitignored overlay that merges on top of `aibox.toml`.
///
/// Only a subset of the config is overridable locally — specifically the fields
/// that vary per developer (credentials, personal mount paths). Shared settings
/// like container name, aibox version, and addon list stay in `aibox.toml`.
///
/// Location: `.aibox-local.toml` in the project root (same dir as `aibox.toml`).
/// This file is added to `.gitignore` by `aibox init` / `aibox apply`.
///
/// Example `.aibox-local.toml`:
/// ```toml
/// [container.environment]
/// GH_TOKEN            = "ghp_..."
/// ANTHROPIC_API_KEY   = "sk-ant-..."
///
/// [[container.extra_volumes]]
/// source = "~/.config/gh"
/// target = "/home/aibox/.config/gh"
///
/// [[container.extra_volumes]]
/// source = "~/.aws"
/// target = "/home/aibox/.aws"
/// ```
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AiboxLocalConfig {
    #[serde(default)]
    pub container: LocalContainerSection,
    /// Personal MCP servers — never committed to git.
    #[serde(default)]
    pub mcp: McpSection,
}

/// The `[container]` sub-section of `.aibox-local.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LocalContainerSection {
    /// Environment variables to inject — merged with (and override) `aibox.toml` entries.
    #[serde(default)]
    pub environment: HashMap<String, String>,
    /// Additional bind mounts — appended after `aibox.toml` extra_volumes.
    #[serde(default)]
    pub extra_volumes: Vec<ExtraVolume>,
}

/// One extra MCP server entry defined in `aibox.toml` (team-shared) or
/// `.aibox-local.toml` (personal). Supplements the processkit-managed
/// servers that aibox discovers from the installed skills.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtraMcpServer {
    /// Server name — used as the key in the `mcpServers` JSON object.
    pub name: String,
    /// Executable to spawn (e.g. `uv`, `npx`, `python3`).
    pub command: String,
    /// Arguments passed to `command`.
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional environment variables injected into the server process.
    #[serde(default)]
    pub env: std::collections::BTreeMap<String, String>,
}

/// `[mcp]` section shared by `aibox.toml` (team-shared servers) and
/// `.aibox-local.toml` (personal servers). Same shape, different semantics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpSection {
    /// Extra MCP servers to register alongside processkit-managed ones.
    #[serde(default)]
    pub servers: Vec<ExtraMcpServer>,

    /// processkit MCP gateway selection. `auto` uses the gateway daemon proxy
    /// when the installed processkit release ships it and falls back to
    /// per-skill MCP servers otherwise.
    #[serde(default)]
    pub gateway: McpGatewaySection,

    /// MCP permissions configuration: global allow/deny patterns and per-harness overrides.
    /// Controls which MCP tools are available to each harness via allow/deny lists.
    #[serde(default)]
    pub permissions: crate::mcp_registration::McpConfig,
}

/// Gateway mode for processkit-managed MCP servers.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum McpGatewayMode {
    /// Prefer the processkit-gateway daemon proxy when available, otherwise use granular servers.
    #[default]
    Auto,
    /// Always write one server per processkit skill.
    Granular,
    /// Spawn processkit-gateway directly as a stdio MCP server per harness.
    Stdio,
    /// Use a managed local HTTP daemon plus one stdio proxy per harness.
    DaemonProxy,
}

/// `[mcp.gateway]` controls how aibox exposes processkit MCP tools to
/// harnesses. It applies only to processkit-managed servers; team and local
/// MCP servers remain independent entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpGatewaySection {
    #[serde(default)]
    pub mode: McpGatewayMode,
    #[serde(default)]
    pub lazy_catalog: bool,
    #[serde(default = "default_mcp_gateway_host")]
    pub host: String,
    #[serde(default = "default_mcp_gateway_port")]
    pub port: u16,
    #[serde(default = "default_mcp_gateway_path")]
    pub path: String,
}

impl Default for McpGatewaySection {
    fn default() -> Self {
        Self {
            mode: McpGatewayMode::Auto,
            lazy_catalog: false,
            host: default_mcp_gateway_host(),
            port: default_mcp_gateway_port(),
            path: default_mcp_gateway_path(),
        }
    }
}

impl McpGatewaySection {
    pub fn url(&self) -> String {
        format!("http://{}:{}{}", self.host, self.port, self.path)
    }
}

fn default_mcp_gateway_host() -> String {
    "127.0.0.1".to_string()
}

fn default_mcp_gateway_port() -> u16 {
    8765
}

fn default_mcp_gateway_path() -> String {
    "/mcp".to_string()
}

impl Default for ProcessKitSection {
    fn default() -> Self {
        Self {
            source: default_processkit_source(),
            version: default_processkit_version(),
            src_path: default_processkit_src_path(),
            branch: None,
            release_asset_url_template: None,
            context: ContextSection::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// [agents] section — canonical AGENTS.md + provider pointer files
// ---------------------------------------------------------------------------

/// How aibox scaffolds provider-specific agent entry files (e.g.
/// `CLAUDE.md`) when both `AGENTS.md` and a provider file exist.
///
/// - `Pointer` (default): provider files are thin pointers that say
///   "see `AGENTS.md`". Canonical instructions live exclusively in
///   `AGENTS.md`. This is the recommended mode and matches the
///   `agents.md` ecosystem convention.
/// - `Full`: provider files contain the rich, provider-flavoured
///   content. Use only when a project genuinely needs different
///   instructions per harness.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AgentsProviderMode {
    #[default]
    Pointer,
    Full,
}

/// `[agents]` section — controls how aibox scaffolds the canonical
/// agent entry file (`AGENTS.md`) and the provider-specific pointer
/// files (`CLAUDE.md`, future `CODEX.md`, …).
///
/// The principle is provider neutrality: every agent harness reads the
/// same `AGENTS.md` so projects don't have to keep N versions of the
/// same instructions in sync. Provider files exist only to satisfy the
/// auto-load convention of specific harnesses (Claude Code auto-loads
/// `CLAUDE.md` at startup, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentsSection {
    /// Filename of the canonical agent entry document. Almost no one
    /// should override this — the default `AGENTS.md` matches the
    /// growing ecosystem convention at <https://agents.md/>.
    #[serde(default = "default_agents_canonical")]
    pub canonical: String,

    /// How provider-specific files are scaffolded. See [`AgentsProviderMode`].
    #[serde(default)]
    pub provider_mode: AgentsProviderMode,
}

fn default_agents_canonical() -> String {
    crate::processkit_vocab::AGENTS_FILENAME.to_string()
}

impl Default for AgentsSection {
    fn default() -> Self {
        Self {
            canonical: default_agents_canonical(),
            provider_mode: AgentsProviderMode::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// [audio] section
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AudioBackend {
    #[default]
    Pulseaudio,
}

impl std::fmt::Display for AudioBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioBackend::Pulseaudio => write!(f, "pulseaudio"),
        }
    }
}

/// [audio] section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudioSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub backend: AudioBackend,
    #[serde(default = "default_audio_install")]
    pub install: bool,
    #[serde(default = "default_pulse_server")]
    pub pulse_server: String,
}

fn default_audio_install() -> bool {
    true
}

fn default_pulse_server() -> String {
    "tcp:host.docker.internal:4714".to_string()
}

impl Default for AudioSection {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: AudioBackend::Pulseaudio,
            install: true,
            pulse_server: default_pulse_server(),
        }
    }
}

fn audio_section_is_explicit(audio: &AudioSection) -> bool {
    audio != &AudioSection::default()
}

fn mcp_section_is_explicit(mcp: &McpSection) -> bool {
    !mcp.servers.is_empty()
        || mcp.gateway != McpGatewaySection::default()
        || mcp.permissions.default_mode != "ask"
        || !mcp.permissions.allow_patterns.is_empty()
        || !mcp.permissions.deny_patterns.is_empty()
        || !mcp.permissions.harness.is_empty()
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/// Check that a string is a safe container/hostname identifier.
/// Must start with alphanumeric and contain only [a-zA-Z0-9._-].
fn is_safe_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

/// Check that an addon/tool/skill name uses only safe characters: [a-zA-Z0-9_-].
fn is_safe_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Check that a tool version string is safe: non-empty and contains only
/// alphanumeric characters or [.-_+].
fn is_safe_version(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
}

fn addon_tool_owner_hint(tool_name: &str, configured_addon_name: &str) -> Option<String> {
    crate::addon_loader::all_addons()
        .iter()
        .find(|addon| {
            addon.name != configured_addon_name
                && addon.tools.iter().any(|tool| tool.name == tool_name)
        })
        .map(|addon| {
            format!(
                "'{}' is provided by [addons.{}.tools]",
                tool_name, addon.name
            )
        })
}

// ---------------------------------------------------------------------------
// Root config — AiboxConfig
// ---------------------------------------------------------------------------

/// Root config structure mapping aibox.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiboxConfig {
    #[serde(rename = "apiVersion", default = "default_api_version")]
    pub api_version: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default)]
    pub metadata: MetadataSection,
    #[serde(default, rename = "aibox")]
    pub aibox: AiboxSection,
    #[serde(default)]
    pub image: ImageSection,
    pub container: ContainerSection,
    #[serde(default)]
    pub context: ContextSection,
    #[serde(default)]
    pub ai: AiSection,
    #[serde(default)]
    pub addons: AddonsSection,
    #[serde(default)]
    pub skills: SkillsSection,
    #[serde(default)]
    pub processkit: ProcessKitSection,
    #[serde(default)]
    pub agents: AgentsSection,
    #[serde(default, alias = "appearance")]
    pub customization: CustomizationSection,
    #[serde(default)]
    pub audio: AudioSection,

    /// Legacy [process] section — if present, packages are merged into [context].
    #[serde(default, skip_serializing)]
    pub(crate) process: Option<LegacyProcessSection>,

    /// Team-shared custom MCP servers from `aibox.toml [mcp.servers]`.
    #[serde(default)]
    pub mcp: McpSection,

    /// Environment variables from `.aibox-local.toml` only — tracked separately
    /// so `generate.rs` can write them to `.aibox-local.env` rather than
    /// embedding literal credential values in `docker-compose.yml`.
    /// Not part of the TOML schema; populated programmatically at load time.
    #[serde(skip)]
    pub local_env: HashMap<String, String>,

    /// Personal MCP servers from `.aibox-local.toml [mcp.servers]` — tracked
    /// separately so they are never committed to git (same principle as
    /// `local_env` / `.aibox-local.env`). Not part of the TOML schema;
    /// populated programmatically at load time.
    #[serde(skip)]
    pub local_mcp_servers: Vec<ExtraMcpServer>,
}

impl AiboxConfig {
    /// Load configuration from a specific file path.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let mut config: AiboxConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        config.migrate_legacy_sections();
        config.resolve_ai_provider_addons();
        config.validate()?;
        Ok(config)
    }

    /// Migrate legacy [process] section into [context].packages.
    fn migrate_legacy_sections(&mut self) {
        if let Some(legacy) = self.process.take()
            && !legacy.packages.is_empty()
        {
            // Only override if context.packages is still at the default
            if self.context.packages == default_context_packages() {
                self.context.packages = legacy.packages;
            }
            crate::output::warn(
                "Deprecated: [process] section found in aibox.toml. \
                 Please move 'packages' into the [context] section.",
            );
        }
        self.sync_grouped_sections();
        self.sync_legacy_aibox_image_fields();
    }

    /// Keep the new grouped schema and legacy top-level sections in sync.
    fn sync_grouped_sections(&mut self) {
        if self.aibox.project_name.is_empty() {
            self.aibox.project_name = if self.metadata.name.is_empty() {
                self.container.name.clone()
            } else {
                self.metadata.name.clone()
            };
        }
        self.metadata.name = self.aibox.project_name.clone();

        if self.context != ContextSection::default() {
            self.processkit.context = self.context.clone();
        } else {
            self.context = self.processkit.context.clone();
        }
        self.processkit.context = self.context.clone();

        if self.agents != AgentsSection::default() {
            self.ai.agents = self.agents.clone();
        } else {
            self.agents = self.ai.agents.clone();
        }
        self.ai.agents = self.agents.clone();

        if mcp_section_is_explicit(&self.mcp) {
            self.ai.mcp = self.mcp.clone();
        } else {
            self.mcp = self.ai.mcp.clone();
        }
        self.ai.mcp = self.mcp.clone();

        if self.container.post_create_command.is_some() || self.container.keepalive {
            self.container.lifecycle.post_create_command =
                self.container.post_create_command.clone();
            self.container.lifecycle.keepalive = self.container.keepalive;
        } else {
            self.container.post_create_command =
                self.container.lifecycle.post_create_command.clone();
            self.container.keepalive = self.container.lifecycle.keepalive;
        }

        if audio_section_is_explicit(&self.audio) {
            self.container.audio = self.audio.clone();
        } else {
            self.audio = self.container.audio.clone();
        }
        self.container.audio = self.audio.clone();
    }

    /// Keep the new `[container.image]` section plus legacy `[image]` and
    /// `[aibox].version/base` fields in sync. Generation still reads
    /// `config.aibox.*`; this bridge keeps old and new files behaviorally
    /// identical during migration.
    fn sync_legacy_aibox_image_fields(&mut self) {
        if self.container.image == ImageSection::default() {
            if self.image != ImageSection::default() {
                self.container.image = self.image.clone();
            } else if self.aibox.version != default_image_version()
                || self.aibox.base != BaseImage::Debian
            {
                self.container.image.version = self.aibox.version.clone();
                self.container.image.base = self.aibox.base.clone();
            }
        }

        self.image = self.container.image.clone();
        self.aibox.version = self.container.image.version.clone();
        self.aibox.base = self.container.image.base.clone();

        if self.metadata.name.is_empty() {
            self.metadata.name = self.aibox.project_name.clone();
        }
    }

    /// Load config from an optional CLI path argument.
    pub fn from_cli_option(config_path: &Option<String>) -> Result<Self> {
        match config_path {
            Some(path) => Self::load(&PathBuf::from(path)),
            None => Self::load_or_default(),
        }
    }

    /// Load from ./aibox.toml, then merge .aibox-local.toml if present.
    ///
    /// `.aibox-local.toml` is a gitignored personal overlay for per-developer
    /// settings (credentials, personal mount paths). Its `[container.environment]`
    /// entries are merged into the base config (local wins on key conflicts) and its
    /// `[[container.extra_volumes]]` entries are appended after the base ones.
    pub fn load_or_default() -> Result<Self> {
        let path = PathBuf::from("aibox.toml");
        if !path.exists() {
            bail!("No aibox.toml found in the current directory. Run 'aibox init' to create one.");
        }
        let mut config = Self::load(&path)?;

        // Merge .aibox-local.toml if present (gitignored personal overlay).
        let local_path = PathBuf::from(".aibox-local.toml");
        if local_path.exists() {
            let local_content =
                std::fs::read_to_string(&local_path).context("Failed to read .aibox-local.toml")?;
            let local: AiboxLocalConfig =
                toml::from_str(&local_content).context("Failed to parse .aibox-local.toml")?;
            // Capture local env vars before merging so generate.rs can write
            // them to .aibox-local.env instead of embedding them in compose.
            config.local_env = local.container.environment.clone();
            // Environment: local wins on key conflicts.
            config
                .container
                .environment
                .extend(local.container.environment);
            // Extra volumes: additive.
            config
                .container
                .extra_volumes
                .extend(local.container.extra_volumes);
            // Validate merged extra_volumes from both sources.
            config.validate_extra_volumes()?;
            // Personal MCP servers — stored separately so they never get
            // embedded in a committed file (same principle as local_env).
            config.local_mcp_servers = local.mcp.servers;
        }

        Ok(config)
    }

    /// Validate all `[[container.extra_volumes]]` entries for path safety.
    /// Called after merging `.aibox-local.toml` so both sources are covered.
    fn validate_extra_volumes(&self) -> Result<()> {
        for vol in &self.container.extra_volumes {
            if vol.source.is_empty() {
                bail!("container.extra_volumes entry has an empty 'source'");
            }
            if vol.target.is_empty() {
                bail!("container.extra_volumes entry has an empty 'target'");
            }
            if vol.source.contains("..") {
                bail!(
                    "container.extra_volumes source '{}' must not contain '..'",
                    vol.source
                );
            }
            if vol.target.contains("..") {
                bail!(
                    "container.extra_volumes target '{}' must not contain '..'",
                    vol.target
                );
            }
            if !vol.target.starts_with('/') {
                bail!(
                    "container.extra_volumes target '{}' must be an absolute path (start with '/')",
                    vol.target
                );
            }
        }
        Ok(())
    }

    /// Parse config from a TOML string. Useful for testing and programmatic use.
    #[allow(dead_code)]
    pub fn from_str(toml_str: &str) -> Result<Self> {
        let mut config: AiboxConfig =
            toml::from_str(toml_str).context("Failed to parse TOML config")?;
        config.migrate_legacy_sections();
        config.resolve_ai_provider_addons();
        config.validate()?;
        Ok(config)
    }

    /// Report unknown `aibox.toml` keys that serde would otherwise ignore.
    ///
    /// Normal loading remains backward-compatible; `aibox doctor` uses this
    /// stricter pass to catch misspelled sections such as `[customisation]` or
    /// `container.nmae` before they silently fall back to defaults.
    pub fn schema_mismatches(toml_str: &str) -> Result<Vec<String>> {
        let value: toml::Value = toml::from_str(toml_str).context("Failed to parse TOML config")?;
        let Some(root) = value.as_table() else {
            return Ok(vec!["aibox.toml root must be a TOML table".to_string()]);
        };

        let mut mismatches = Vec::new();
        check_unknown_keys(
            "aibox.toml",
            root,
            &[
                "apiVersion",
                "kind",
                "metadata",
                "aibox",
                "image",
                "container",
                "context",
                "process",
                "ai",
                "addons",
                "skills",
                "processkit",
                "agents",
                "appearance",
                "customization",
                "audio",
                "mcp",
            ],
            &mut mismatches,
        );

        check_child_table(root, "metadata", &["name", "project_name"], &mut mismatches);
        check_child_table(
            root,
            "aibox",
            &[
                "config_schema",
                "project_name",
                "name",
                "version",
                "base",
                "profile",
            ],
            &mut mismatches,
        );
        check_child_table(
            root,
            "image",
            &["version", "release_version", "base"],
            &mut mismatches,
        );
        check_child_table(
            root,
            "container",
            &[
                "name",
                "hostname",
                "user",
                "post_create_command",
                "keepalive",
                "lifecycle",
                "environment",
                "extra_volumes",
                "resource_thresholds",
                "image",
                "paths",
                "audio",
            ],
            &mut mismatches,
        );
        if let Some(container) = table_child(root, "container") {
            check_child_table(
                container,
                "lifecycle",
                &["post_create_command", "keepalive"],
                &mut mismatches,
            );
            check_child_table(
                container,
                "image",
                &["version", "release_version", "base"],
                &mut mismatches,
            );
            check_child_table(
                container,
                "paths",
                &[
                    "devcontainer_json",
                    "docker_compose",
                    "docker_compose_override",
                    "dockerfile",
                    "dockerfile_local",
                    "local_env",
                ],
                &mut mismatches,
            );
            check_child_table(
                container,
                "audio",
                &["enabled", "backend", "install", "pulse_server"],
                &mut mismatches,
            );
            check_child_table(
                container,
                "resource_thresholds",
                &[
                    "memory_mib_warn",
                    "process_count_warn",
                    "processkit_mcp_python_warn",
                    "oom_kill_warn",
                ],
                &mut mismatches,
            );
            check_extra_volume_entries(container, &mut mismatches);
        }
        check_child_table(
            root,
            "context",
            &["schema_version", "packages"],
            &mut mismatches,
        );
        check_child_table(root, "process", &["packages"], &mut mismatches);
        check_child_table(
            root,
            "ai",
            &[
                "harnesses",
                "model_providers",
                "harness",
                "providers",
                "agents",
                "mcp",
            ],
            &mut mismatches,
        );
        if let Some(ai) = table_child(root, "ai") {
            if let Some(harnesses) = table_child(ai, "harness") {
                for (harness, value) in harnesses {
                    if let Some(table) = value.as_table() {
                        check_unknown_keys(
                            &format!("[ai.harness.{harness}]"),
                            table,
                            &["enabled", "install", "version"],
                            &mut mismatches,
                        );
                    }
                }
            }
            check_child_table(
                ai,
                "agents",
                &["canonical", "provider_mode"],
                &mut mismatches,
            );
            check_mcp_table(ai, &mut mismatches);
        }
        check_child_table(
            root,
            "skills",
            &["include", "exclude", "enabled", "disabled"],
            &mut mismatches,
        );
        check_child_table(
            root,
            "processkit",
            &[
                "source",
                "version",
                "src_path",
                "branch",
                "release_asset_url_template",
                "context",
            ],
            &mut mismatches,
        );
        if let Some(processkit) = table_child(root, "processkit") {
            check_child_table(
                processkit,
                "context",
                &["schema_version", "packages"],
                &mut mismatches,
            );
        }
        check_child_table(
            root,
            "agents",
            &["canonical", "provider_mode"],
            &mut mismatches,
        );
        check_customization_table(root, "appearance", &mut mismatches);
        check_customization_table(root, "customization", &mut mismatches);
        check_child_table(
            root,
            "audio",
            &["enabled", "backend", "install", "pulse_server"],
            &mut mismatches,
        );
        check_mcp_table(root, &mut mismatches);
        check_addons_table(root, &mut mismatches);

        Ok(mismatches)
    }

    /// Validate the config values. Called internally by `load`, but also
    /// available for validating programmatically-constructed configs.
    pub fn validate(&self) -> Result<()> {
        if self.api_version != default_api_version() {
            bail!(
                "apiVersion '{}' is not supported by this aibox version; expected '{}'",
                self.api_version,
                default_api_version()
            );
        }
        if self.kind != default_kind() {
            bail!(
                "kind '{}' is not supported by this aibox version; expected '{}'",
                self.kind,
                default_kind()
            );
        }
        semver::Version::parse(&self.aibox.config_schema).with_context(|| {
            format!(
                "Invalid aibox.config_schema '{}': must be valid semver",
                self.aibox.config_schema
            )
        })?;

        // Validate version is valid semver (allow "latest" sentinel)
        if self.aibox.version != "latest" {
            semver::Version::parse(&self.aibox.version).with_context(|| {
                format!(
                    "Invalid version '{}': must be valid semver",
                    self.aibox.version
                )
            })?;
        }

        // Validate schema_version is valid semver
        semver::Version::parse(&self.context.schema_version).with_context(|| {
            format!(
                "Invalid schema_version '{}': must be valid semver",
                self.context.schema_version
            )
        })?;

        // Validate container name is non-empty and safe
        if self.container.name.is_empty() {
            bail!("container.name must not be empty");
        }
        if !is_safe_identifier(&self.container.name) {
            bail!(
                "container.name '{}' contains invalid characters. \
                 Must start with alphanumeric and contain only [a-zA-Z0-9._-]",
                self.container.name
            );
        }
        if !self.container.hostname.is_empty() && !is_safe_identifier(&self.container.hostname) {
            bail!(
                "container.hostname '{}' contains invalid characters. \
                 Must start with alphanumeric and contain only [a-zA-Z0-9._-]",
                self.container.hostname
            );
        }

        // Validate context packages have safe names
        if self.context.packages.is_empty() {
            bail!("context.packages must not be empty (at minimum ['core'] is required)");
        }
        for pkg in &self.context.packages {
            if !is_safe_name(pkg) {
                bail!(
                    "context.packages entry '{}' contains invalid characters. \
                     Must contain only [a-zA-Z0-9_-]",
                    pkg
                );
            }
        }

        // Validate addon names and tool names are safe identifiers
        for (addon_name, addon_tools) in &self.addons.addons {
            if !is_safe_name(addon_name) {
                bail!(
                    "addon name '{}' contains invalid characters. \
                     Must contain only [a-zA-Z0-9_-]",
                    addon_name
                );
            }
            for (tool_name, tool_entry) in &addon_tools.tools {
                if !is_safe_name(tool_name) {
                    bail!(
                        "tool name '{}' in addon '{}' contains invalid characters. \
                         Must contain only [a-zA-Z0-9_-]",
                        tool_name,
                        addon_name
                    );
                }
                if let Some(version) = &tool_entry.version
                    && !version.is_empty() // empty string means "use addon default" — valid
                    && !is_safe_version(version)
                {
                    bail!(
                        "tool version '{}' for '{}' in addon '{}' contains invalid characters. \
                         Must contain only [a-zA-Z0-9._\\-+]",
                        version,
                        tool_name,
                        addon_name
                    );
                }
            }
        }
        self.validate_known_addon_tools()?;

        // Validate skill names are safe
        for skill in &self.skills.include {
            if !is_safe_name(skill) {
                bail!(
                    "skills.include entry '{}' contains invalid characters. \
                     Must contain only [a-zA-Z0-9_-]",
                    skill
                );
            }
        }
        for skill in &self.skills.exclude {
            if !is_safe_name(skill) {
                bail!(
                    "skills.exclude entry '{}' contains invalid characters. \
                     Must contain only [a-zA-Z0-9_-]",
                    skill
                );
            }
        }

        // Validate [processkit]
        self.validate_processkit()?;

        // Validate processkit gateway runtime exposure stays local. The
        // daemon is a developer-workstation helper, not a network service.
        self.validate_mcp_gateway()?;

        // Validate extra volumes path safety
        self.validate_extra_volumes()?;
        self.validate_container_paths()?;

        Ok(())
    }

    fn validate_known_addon_tools(&self) -> Result<()> {
        if crate::addon_loader::all_addons().is_empty() {
            return Ok(());
        }

        for (addon_name, addon_tools) in &self.addons.addons {
            let Some(addon) = crate::addon_loader::get_addon(addon_name) else {
                continue;
            };
            let known_tools: BTreeSet<&str> =
                addon.tools.iter().map(|tool| tool.name.as_str()).collect();
            for tool_name in addon_tools.tools.keys() {
                if known_tools.contains(tool_name.as_str()) {
                    continue;
                }

                let mut message = format!(
                    "unknown tool '{}' in [addons.{}.tools]; '{}' supports: {}",
                    tool_name,
                    addon_name,
                    addon_name,
                    known_tools.iter().copied().collect::<Vec<_>>().join(", ")
                );
                if let Some(suggestion) = addon_tool_owner_hint(tool_name, addon_name) {
                    message.push_str(&format!(". {suggestion}"));
                }
                bail!(message);
            }
        }

        Ok(())
    }

    fn validate_container_paths(&self) -> Result<()> {
        let paths = [
            (
                "container.paths.devcontainer_json",
                &self.container.paths.devcontainer_json,
            ),
            (
                "container.paths.docker_compose",
                &self.container.paths.docker_compose,
            ),
            (
                "container.paths.docker_compose_override",
                &self.container.paths.docker_compose_override,
            ),
            (
                "container.paths.dockerfile",
                &self.container.paths.dockerfile,
            ),
            (
                "container.paths.dockerfile_local",
                &self.container.paths.dockerfile_local,
            ),
            ("container.paths.local_env", &self.container.paths.local_env),
        ];
        for (name, value) in paths {
            if value.trim().is_empty() {
                bail!("{name} must not be empty");
            }
            let path = std::path::Path::new(value);
            if path.is_absolute() || value.contains("..") {
                bail!("{name} must be a project-relative path without '..'");
            }
        }
        Ok(())
    }

    /// Validate the [processkit] section. Split out for testability.
    fn validate_processkit(&self) -> Result<()> {
        let pk = &self.processkit;

        // source must be a non-empty URL-ish string
        if pk.source.trim().is_empty() {
            bail!("processkit.source must not be empty");
        }
        if !(pk.source.starts_with("http://")
            || pk.source.starts_with("https://")
            || pk.source.starts_with("git@")
            || pk.source.starts_with("file://")
            || pk.source.starts_with("ssh://"))
        {
            bail!(
                "processkit.source '{}' does not look like a URL. \
                 Expected one of: http://, https://, git@, ssh://, file://",
                pk.source
            );
        }

        // version: allow the "unset" sentinel, OR a leading-`v` semver-ish
        // tag, OR a bare semver string. We don't pin to strict semver because
        // git tags vary; just sanity check.
        if pk.version != PROCESSKIT_VERSION_UNSET && pk.version != PROCESSKIT_VERSION_LATEST {
            let stripped = pk.version.strip_prefix('v').unwrap_or(&pk.version);
            // Either parses as semver, or matches a relaxed `numbers + dots`
            // shape (e.g. "0.4", "1.0.0-rc1").
            let semver_ok = semver::Version::parse(stripped).is_ok();
            let relaxed_ok = !stripped.is_empty()
                && stripped
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
                && stripped.chars().any(|c| c.is_ascii_digit());
            if !semver_ok && !relaxed_ok {
                bail!(
                    "processkit.version '{}' is not a valid version tag. \
                     Use the sentinel \"unset\", \"latest\", a semver string like \"0.4.0\", \
                     or a tag like \"v0.4.0\".",
                    pk.version
                );
            }
        }

        // src_path: no traversal, no absolute paths
        if pk.src_path.contains("..") {
            bail!(
                "processkit.src_path '{}' must not contain '..'",
                pk.src_path
            );
        }
        if pk.src_path.starts_with('/') {
            bail!(
                "processkit.src_path '{}' must not be an absolute path",
                pk.src_path
            );
        }

        // branch: if set, must not be empty
        if let Some(branch) = &pk.branch
            && branch.trim().is_empty()
        {
            bail!("processkit.branch is set but empty; remove it or provide a name");
        }

        Ok(())
    }

    fn validate_mcp_gateway(&self) -> Result<()> {
        let gateway = &self.mcp.gateway;
        if gateway.host != "127.0.0.1" && gateway.host != "localhost" {
            bail!(
                "mcp.gateway.host must be localhost or 127.0.0.1; got '{}'",
                gateway.host
            );
        }
        if gateway.path.is_empty() || !gateway.path.starts_with('/') {
            bail!(
                "mcp.gateway.path must be an absolute HTTP path starting with '/'; got '{}'",
                gateway.path
            );
        }
        Ok(())
    }

    /// Resolve config sections into implicit addon entries so the addon
    /// pipeline handles tool installation. Called before `validate()` during load.
    /// Idempotent — won't overwrite if the user already configured the addon
    /// explicitly in `[addons]`.
    pub fn resolve_ai_provider_addons(&mut self) {
        // Migrate legacy providers → harnesses if needed.
        self.ai.migrate_legacy();

        // Migrate legacy addon name: ai-openai → ai-codex.
        // Before v0.18.1 the Codex harness addon was named "ai-openai".
        // Users with [addons.ai-openai.tools] in their aibox.toml need the
        // tools carried over to the new name.
        if let Some(legacy) = self.addons.addons.remove("ai-openai") {
            self.addons
                .addons
                .entry("ai-codex".to_string())
                .or_insert(legacy);
        }

        let legacy_ai_harnesses: Vec<AiHarness> = self
            .addons
            .addons
            .keys()
            .filter_map(|name| AiHarness::from_addon_name(name))
            .collect();
        for harness in legacy_ai_harnesses {
            let Some(harness_config) = self.ai.harness.get(&harness) else {
                if !self.ai.harnesses.contains(&harness) {
                    self.ai.harnesses.push(harness);
                }
                continue;
            };
            if harness_config.enabled.unwrap_or(true)
                && harness_config.install.unwrap_or(true)
                && !self.ai.harnesses.contains(&harness)
            {
                self.ai.harnesses.push(harness);
            }
        }

        for harness in &self.ai.harnesses {
            if !harness.is_active() {
                continue;
            }
            if !self.ai.harness_install_enabled(harness) {
                continue;
            }
            let addon_name = harness.addon_name();
            if addon_name.is_empty() {
                continue;
            }
            let addon_tools =
                self.addons
                    .addons
                    .entry(addon_name)
                    .or_insert_with(|| AddonToolsSection {
                        tools: HashMap::new(),
                    });
            if let Some(version) = self.ai.harness_version(harness) {
                addon_tools
                    .tools
                    .entry(harness.binary_name().to_string())
                    .or_insert_with(|| ToolEntry {
                        version: Some(version.to_string()),
                        enabled: None,
                    });
            }
        }

        if self.audio.enabled && self.audio.install {
            self.addons
                .addons
                .entry("audio-voice".to_string())
                .or_insert_with(|| AddonToolsSection {
                    tools: HashMap::new(),
                });
        }
    }

    /// Get the host root path (.aibox-home/ directory), respecting env override.
    /// Falls back to `.root/` if that directory exists (backward compatibility).
    pub fn host_root_dir(&self) -> PathBuf {
        if let Ok(val) = std::env::var("AIBOX_HOST_ROOT") {
            return PathBuf::from(val);
        }
        // Backward compat: use .root/ if it exists and .aibox-home/ doesn't
        let new_path = PathBuf::from(".aibox-home");
        let old_path = PathBuf::from(".root");
        if old_path.exists() && !new_path.exists() {
            old_path
        } else {
            new_path
        }
    }

    /// Get the container-side home directory based on the configured user.
    pub fn container_home(&self) -> String {
        if self.container.user == "root" {
            "/root".to_string()
        } else {
            format!("/home/{}", self.container.user)
        }
    }

    /// Get the workspace directory, respecting env override.
    pub fn workspace_dir(&self) -> String {
        std::env::var("AIBOX_WORKSPACE_DIR").unwrap_or_else(|_| "..".to_string())
    }
}

fn allowed_set(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|key| (*key).to_string()).collect()
}

fn check_unknown_keys(
    label: &str,
    table: &toml::map::Map<String, toml::Value>,
    allowed: &[&str],
    mismatches: &mut Vec<String>,
) {
    let allowed = allowed_set(allowed);
    for key in table.keys() {
        if !allowed.contains(key) {
            mismatches.push(format!("{label}: unknown key `{key}`"));
        }
    }
}

fn table_child<'a>(
    table: &'a toml::map::Map<String, toml::Value>,
    key: &str,
) -> Option<&'a toml::map::Map<String, toml::Value>> {
    table.get(key).and_then(toml::Value::as_table)
}

fn check_child_table(
    parent: &toml::map::Map<String, toml::Value>,
    key: &str,
    allowed: &[&str],
    mismatches: &mut Vec<String>,
) {
    if let Some(table) = table_child(parent, key) {
        check_unknown_keys(&format!("[{key}]"), table, allowed, mismatches);
    }
}

fn check_customization_table(
    root: &toml::map::Map<String, toml::Value>,
    key: &str,
    mismatches: &mut Vec<String>,
) {
    check_child_table(
        root,
        key,
        &["theme", "mode", "prompt", "layout", "zellij_status"],
        mismatches,
    );
    if let Some(customization) = table_child(root, key) {
        check_child_table(customization, "zellij_status", &["mode"], mismatches);
    }
}

fn check_extra_volume_entries(
    container: &toml::map::Map<String, toml::Value>,
    mismatches: &mut Vec<String>,
) {
    let Some(extra_volumes) = container.get("extra_volumes") else {
        return;
    };
    let Some(entries) = extra_volumes.as_array() else {
        return;
    };
    for (index, entry) in entries.iter().enumerate() {
        if let Some(table) = entry.as_table() {
            check_unknown_keys(
                &format!("[[container.extra_volumes]][{index}]"),
                table,
                &["source", "target", "read_only"],
                mismatches,
            );
        }
    }
}

fn check_addons_table(root: &toml::map::Map<String, toml::Value>, mismatches: &mut Vec<String>) {
    let Some(addons) = table_child(root, "addons") else {
        return;
    };
    for (addon_name, addon_value) in addons {
        let Some(addon_table) = addon_value.as_table() else {
            continue;
        };
        check_unknown_keys(
            &format!("[addons.{addon_name}]"),
            addon_table,
            &["tools"],
            mismatches,
        );
        let Some(tools) = table_child(addon_table, "tools") else {
            continue;
        };
        for (tool_name, tool_value) in tools {
            let Some(tool_table) = tool_value.as_table() else {
                continue;
            };
            check_unknown_keys(
                &format!("[addons.{addon_name}.tools.{tool_name}]"),
                tool_table,
                &["version", "enabled"],
                mismatches,
            );
        }
    }
}

fn check_mcp_table(root: &toml::map::Map<String, toml::Value>, mismatches: &mut Vec<String>) {
    check_child_table(
        root,
        "mcp",
        &["servers", "gateway", "permissions"],
        mismatches,
    );
    let Some(mcp) = table_child(root, "mcp") else {
        return;
    };
    check_child_table(
        mcp,
        "gateway",
        &["mode", "lazy_catalog", "host", "port", "path"],
        mismatches,
    );
    check_child_table(
        mcp,
        "permissions",
        &["default_mode", "allow_patterns", "deny_patterns", "harness"],
        mismatches,
    );
    if let Some(permissions) = table_child(mcp, "permissions")
        && let Some(harnesses) = table_child(permissions, "harness")
    {
        for (harness, value) in harnesses {
            if let Some(table) = value.as_table() {
                check_unknown_keys(
                    &format!("[mcp.permissions.harness.{harness}]"),
                    table,
                    &["enabled", "mode", "extra_patterns", "deny_patterns"],
                    mismatches,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Test helper
// ---------------------------------------------------------------------------

/// Create a `AiboxConfig` for testing with sensible defaults.
/// Only available in test builds to reduce boilerplate across test modules.
#[cfg(test)]
pub fn test_config() -> AiboxConfig {
    let mut config = AiboxConfig {
        api_version: default_api_version(),
        kind: default_kind(),
        metadata: MetadataSection {
            name: "test-proj".to_string(),
        },
        aibox: AiboxSection {
            config_schema: default_config_schema(),
            project_name: "test-proj".to_string(),
            version: "0.9.0".to_string(),
            base: BaseImage::Debian,
            profile: AiboxProfile::HumanDev,
        },
        image: ImageSection {
            version: "0.9.0".to_string(),
            base: BaseImage::Debian,
        },
        container: ContainerSection {
            name: "test-proj".to_string(),
            hostname: "test-proj".to_string(),
            user: "root".to_string(),
            post_create_command: None,
            keepalive: false,
            lifecycle: ContainerLifecycleSection::default(),
            environment: HashMap::new(),
            extra_volumes: vec![],
            resource_thresholds: ResourceThresholdsSection::default(),
            image: ImageSection {
                version: "0.9.0".to_string(),
                base: BaseImage::Debian,
            },
            paths: ContainerPathsSection::default(),
            audio: AudioSection::default(),
        },
        context: ContextSection::default(),
        ai: AiSection::default(),
        addons: AddonsSection::default(),
        skills: SkillsSection::default(),
        processkit: ProcessKitSection::default(),
        agents: AgentsSection::default(),
        customization: CustomizationSection::default(),
        audio: AudioSection::default(),
        process: None,
        mcp: McpSection::default(),
        local_env: HashMap::new(),
        local_mcp_servers: vec![],
    };
    config.resolve_ai_provider_addons();
    config
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;

    // -- TOML fixtures ------------------------------------------------------

    fn full_toml() -> &'static str {
        r#"
[aibox]
version = "0.9.0"
base = "debian"
profile = "headless-runner"

[container]
name = "my-project"
hostname = "my-project"
user = "root"
keepalive = false
post_create_command = "npm install"

[container.resource_thresholds]
memory_mib_warn = 4096
process_count_warn = 375
processkit_mcp_python_warn = 45
oom_kill_warn = 0

[context]
schema_version = "2.0.0"

[ai]
harnesses = ["claude", "aider", "mistral"]

[process]
packages = ["managed", "code", "documentation"]

[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.7" }

[addons.node.tools]
node = { version = "22" }
pnpm = { version = "10" }

[addons.rust.tools]
rustc = { version = "1.89" }
clippy = {}
rustfmt = {}

[addons.latex.tools]
texlive-core = {}
texlive-recommended = {}

[addons.infrastructure.tools]
opentofu = {}
ansible = {}

[addons.kubernetes.tools]
kubectl = {}
helm = {}

[addons.cloud-aws.tools]
aws-cli = {}

[addons.docs-docusaurus.tools]
docusaurus = { version = "3" }

[skills]
exclude = ["standup-context"]
include = ["flutter-development"]

[appearance]
theme = "gruvbox-dark"
mode = "auto"
prompt = "default"

[appearance.zellij_status]
mode = "shell"

[audio]
enabled = false

[mcp.gateway]
mode = "daemon-proxy"
lazy_catalog = true
host = "127.0.0.1"
port = 8765
path = "/mcp"
"#
    }

    fn minimal_toml() -> &'static str {
        r#"
[aibox]
version = "0.9.0"

[container]
name = "my-project"
"#
    }

    fn new_shape_toml() -> &'static str {
        r#"
apiVersion = "aibox.projectious.work/v1"
kind = "Workspace"

[metadata]
name = "my-project"

[aibox]
config_schema = "1.0.0"
profile = "human-dev"

[image]
version = "0.23.8"
base = "debian"

[container]
name = "my-project"
"#
    }

    fn parse_toml(s: &str) -> Result<AiboxConfig> {
        let mut config: AiboxConfig = toml::from_str(s).context("Failed to parse TOML")?;
        config.migrate_legacy_sections();
        config.validate()?;
        Ok(config)
    }

    fn init_repo_addons_for_validation() {
        let addons_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("addons");
        let _ = crate::addon_loader::init_from_dir(&addons_dir);
    }

    // -- Full config parsing ------------------------------------------------

    #[test]
    fn parse_full_toml_all_fields() {
        let config = parse_toml(full_toml()).expect("should parse full toml");

        // [aibox]
        assert_eq!(config.aibox.version, "0.9.0");
        assert_eq!(config.aibox.base, BaseImage::Debian);
        assert_eq!(config.aibox.profile, AiboxProfile::HeadlessRunner);

        // [container]
        assert_eq!(config.container.name, "my-project");
        assert_eq!(config.container.hostname, "my-project");
        assert_eq!(config.container.user, "root");
        assert_eq!(
            config.container.post_create_command.as_deref(),
            Some("npm install")
        );
        assert!(!config.container.keepalive);
        assert_eq!(
            config.container.resource_thresholds.memory_mib_warn,
            Some(4096)
        );
        assert_eq!(
            config.container.resource_thresholds.process_count_warn,
            Some(375)
        );
        assert_eq!(
            config
                .container
                .resource_thresholds
                .processkit_mcp_python_warn,
            Some(45)
        );
        assert_eq!(config.container.resource_thresholds.oom_kill_warn, Some(0));

        // [context]
        assert_eq!(config.context.schema_version, "2.0.0");

        // [ai]
        assert_eq!(config.ai.harnesses.len(), 3);
        assert_eq!(config.ai.harnesses[0], AiProvider::Claude);
        assert_eq!(config.ai.harnesses[1], AiProvider::Aider);
        assert_eq!(config.ai.harnesses[2], AiProvider::Mistral);

        // [context].packages (migrated from legacy [process])
        assert_eq!(
            config.context.packages,
            vec!["managed", "code", "documentation"]
        );

        // [addons]
        assert_eq!(config.addons.addons.len(), 8);
        assert!(config.addons.has_addon("python"));
        assert!(config.addons.has_addon("node"));
        assert!(config.addons.has_addon("rust"));
        assert!(config.addons.has_addon("latex"));
        assert!(config.addons.has_addon("infrastructure"));
        assert!(config.addons.has_addon("kubernetes"));
        assert!(config.addons.has_addon("cloud-aws"));

        // Check specific tool versions
        assert_eq!(config.addons.tool_version("python", "python"), Some("3.13"));
        assert_eq!(config.addons.tool_version("python", "uv"), Some("0.7"));
        assert_eq!(config.addons.tool_version("rust", "rustc"), Some("1.89"));
        assert_eq!(config.addons.tool_version("rust", "clippy"), None);
        assert_eq!(config.addons.tool_version("rust", "rustfmt"), None);
        assert!(config.addons.has_tool("kubernetes", "kubectl"));
        assert!(config.addons.has_tool("kubernetes", "helm"));
        assert_eq!(
            config.addons.tool_version("docs-docusaurus", "docusaurus"),
            Some("3")
        );

        // [skills]
        assert_eq!(config.skills.exclude, vec!["standup-context"]);
        assert_eq!(config.mcp.gateway.mode, McpGatewayMode::DaemonProxy);
        assert!(config.mcp.gateway.lazy_catalog);
        assert_eq!(config.mcp.gateway.url(), "http://127.0.0.1:8765/mcp");
        assert_eq!(config.skills.include, vec!["flutter-development"]);

        // [customization] (parsed from legacy [appearance] via serde alias)
        assert_eq!(config.customization.theme, Theme::GruvboxDark);
        assert_eq!(config.customization.mode, ThemeMode::Auto);
        assert_eq!(config.customization.prompt, StarshipPreset::Default);
        assert_eq!(
            config.customization.zellij_status.mode,
            ZellijStatusMode::Shell
        );

        // [audio]
        assert!(!config.audio.enabled);
    }

    // -- Minimal config with defaults ---------------------------------------

    #[test]
    fn parse_minimal_toml_defaults() {
        let config = parse_toml(minimal_toml()).expect("should parse minimal toml");
        assert_eq!(config.aibox.base, BaseImage::Debian);
        assert_eq!(config.aibox.profile, AiboxProfile::HumanDev);
        assert_eq!(config.container.name, "my-project");
        assert_eq!(config.container.hostname, "aibox");
        assert_eq!(config.context.schema_version, "1.0.0");
        assert_eq!(config.ai.harnesses, vec![AiProvider::Claude]);
        assert_eq!(config.context.packages, vec!["product"]);
        assert!(config.addons.addons.is_empty());
        assert!(config.skills.include.is_empty());
        assert!(config.skills.exclude.is_empty());
        assert!(!config.audio.enabled);
        assert_eq!(config.audio.pulse_server, "tcp:host.docker.internal:4714");
    }

    #[test]
    fn parse_new_shape_toml_syncs_image_to_generation_fields() {
        let config = parse_toml(new_shape_toml()).expect("should parse new shape toml");
        assert_eq!(config.api_version, "aibox.projectious.work/v1");
        assert_eq!(config.kind, "Workspace");
        assert_eq!(config.metadata.name, "my-project");
        assert_eq!(config.image.version, "0.23.8");
        assert_eq!(config.aibox.version, "0.23.8");
        assert_eq!(config.aibox.base, BaseImage::Debian);
    }

    #[test]
    fn schema_mismatches_accepts_known_full_config_shape() {
        let mismatches = AiboxConfig::schema_mismatches(full_toml()).unwrap();
        assert!(
            mismatches.is_empty(),
            "full known config shape should not report unknown keys: {mismatches:?}"
        );
    }

    #[test]
    fn schema_mismatches_reports_unknown_nested_keys() {
        let toml = r#"
[aibox]
version = "0.23.5"

[container]
name = "test"
nmae = "typo"

[customization.zellij_status]
mod = "typo"

[addons.git-ui.tools.lazygit]
enabled = false
enabld = true
"#;
        let mismatches = AiboxConfig::schema_mismatches(toml).unwrap();

        assert!(mismatches.contains(&"[container]: unknown key `nmae`".to_string()));
        assert!(mismatches.contains(&"[zellij_status]: unknown key `mod`".to_string()));
        assert!(
            mismatches.contains(&"[addons.git-ui.tools.lazygit]: unknown key `enabld`".to_string())
        );
    }

    // -- Validation errors --------------------------------------------------

    #[test]
    fn parse_invalid_semver_version() {
        let toml = r#"
[aibox]
version = "not-a-version"

[container]
name = "test"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject invalid semver");
    }

    #[test]
    fn parse_aibox_version_latest_sentinel() {
        let toml = r#"
[aibox]
version = "latest"

[container]
name = "test"
"#;
        let result = parse_toml(toml);
        assert!(result.is_ok(), "should accept 'latest' as aibox version");
        assert_eq!(result.unwrap().aibox.version, "latest");
    }

    #[test]
    fn parse_empty_container_name() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = ""
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject empty container name");
    }

    #[test]
    fn invalid_schema_version_semver() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[context]
schema_version = "bad"
"#;
        let result = parse_toml(toml);
        assert!(
            result.is_err(),
            "should reject invalid schema_version semver"
        );
    }

    #[test]
    fn invalid_container_name_chars() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "my project!"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject invalid container name");
    }

    #[test]
    fn empty_context_packages_rejected() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[context]
packages = []
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject empty context packages");
    }

    #[test]
    fn legacy_process_section_migrated() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[process]
packages = ["managed", "code"]

[context]
schema_version = "2.0.0"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.context.packages, vec!["managed", "code"]);
        assert_eq!(config.context.schema_version, "2.0.0");
    }

    #[test]
    fn legacy_appearance_alias_works() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[appearance]
theme = "dracula"
mode = "dark"
prompt = "minimal"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.customization.theme, Theme::Dracula);
        assert_eq!(config.customization.mode, ThemeMode::Dark);
        assert_eq!(config.customization.prompt, StarshipPreset::Minimal);
        assert_eq!(
            config.customization.zellij_status.mode,
            ZellijStatusMode::Shell
        );
    }

    #[test]
    fn customization_zellij_status_mode_parses() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "my-project"

[customization.zellij_status]
mode = "hidden"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.customization.zellij_status.mode,
            ZellijStatusMode::Hidden
        );
    }

    #[test]
    fn invalid_skill_name_rejected() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[skills]
include = ["valid-skill", "bad skill!"]
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject invalid skill name");
    }

    #[test]
    fn invalid_addon_name_rejected() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[addons."bad addon!".tools]
tool = {}
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject invalid addon name");
    }

    #[test]
    #[serial]
    fn unknown_tool_under_known_addon_is_rejected_with_owner_hint() {
        init_repo_addons_for_validation();
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[addons.python.tools]
gh = { enabled = true }
"#;
        let err = parse_toml(toml).unwrap_err().to_string();
        assert!(err.contains("unknown tool 'gh' in [addons.python.tools]"));
        assert!(err.contains("[addons.git-ui.tools]"));
    }

    // -- AI providers -------------------------------------------------------

    #[test]
    fn ai_provider_display() {
        assert_eq!(format!("{}", AiProvider::Claude), "claude");
        assert_eq!(format!("{}", AiProvider::Aider), "aider");
        assert_eq!(format!("{}", AiProvider::Gemini), "gemini");
        assert_eq!(format!("{}", AiProvider::Mistral), "mistral");
        assert_eq!(format!("{}", AiProvider::Codex), "codex");
        assert_eq!(format!("{}", AiProvider::Continue), "continue");
        assert_eq!(format!("{}", AiProvider::Copilot), "copilot");
    }

    #[test]
    fn ai_provider_binary_name() {
        // Most providers: binary name matches display name.
        assert_eq!(AiProvider::Claude.binary_name(), "claude");
        assert_eq!(AiProvider::Aider.binary_name(), "aider");
        assert_eq!(AiProvider::Copilot.binary_name(), "copilot");
        // Continue is the exception: display = "continue", binary = "cn".
        assert_eq!(AiProvider::Continue.binary_name(), "cn");
        // Codex: both display and binary are "codex".
        assert_eq!(AiProvider::Codex.binary_name(), "codex");
    }

    #[test]
    fn parse_all_ai_providers() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[ai]
harnesses = ["claude", "aider", "gemini", "mistral"]
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.ai.harnesses.len(), 4);
        assert_eq!(config.ai.harnesses[3], AiProvider::Mistral);
    }

    #[test]
    fn parse_new_ai_providers() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[ai]
providers = ["openai", "copilot", "continue"]
"#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert_eq!(config.ai.harnesses.len(), 3);
        assert_eq!(config.ai.harnesses[0], AiProvider::Codex);
        assert_eq!(config.ai.harnesses[1], AiProvider::Copilot);
        assert_eq!(config.ai.harnesses[2], AiProvider::Continue);
        assert!(config.addons.has_addon("ai-codex"));
        assert!(config.addons.has_addon("ai-copilot"));
        assert!(config.addons.has_addon("ai-continue"));
    }

    #[test]
    fn parse_empty_ai_providers() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[ai]
harnesses = []
"#;
        let config = parse_toml(toml).unwrap();
        assert!(config.ai.harnesses.is_empty());
    }

    #[test]
    fn default_ai_providers_is_claude() {
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.ai.harnesses, vec![AiProvider::Claude]);
    }

    // -- Base image ---------------------------------------------------------

    #[test]
    fn base_image_display() {
        assert_eq!(format!("{}", BaseImage::Debian), "debian");
    }

    #[test]
    fn base_image_default_is_debian() {
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.aibox.base, BaseImage::Debian);
    }

    #[test]
    fn aibox_profile_display() {
        assert_eq!(format!("{}", AiboxProfile::HumanDev), "human-dev");
        assert_eq!(
            format!("{}", AiboxProfile::HeadlessRunner),
            "headless-runner"
        );
    }

    // -- Addons helpers -----------------------------------------------------

    #[test]
    fn addons_convenience_methods() {
        let config = parse_toml(full_toml()).unwrap();
        assert!(config.addons.has_python());
        assert!(config.addons.has_rust());
        assert!(config.addons.has_node());
        assert!(config.addons.has_latex());
    }

    #[test]
    fn addons_tool_lookup() {
        let config = parse_toml(full_toml()).unwrap();
        assert!(config.addons.has_tool("python", "python"));
        assert!(config.addons.has_tool("python", "uv"));
        assert!(!config.addons.has_tool("python", "poetry"));
        assert_eq!(config.addons.tool_version("node", "node"), Some("22"));
        assert_eq!(config.addons.tool_version("node", "pnpm"), Some("10"));
        assert_eq!(config.addons.tool_version("cloud-aws", "aws-cli"), None);
    }

    #[test]
    fn addons_empty_by_default() {
        let config = parse_toml(minimal_toml()).unwrap();
        assert!(config.addons.addons.is_empty());
        assert!(!config.addons.has_python());
        assert!(!config.addons.has_rust());
    }

    #[test]
    fn addon_with_only_versionless_tools() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[addons.rust.tools]
clippy = {}
rustfmt = {}
"#;
        let config = parse_toml(toml).unwrap();
        assert!(config.addons.has_rust());
        assert!(config.addons.has_tool("rust", "clippy"));
        assert_eq!(config.addons.tool_version("rust", "clippy"), None);
    }

    // -- Context packages ---------------------------------------------------

    #[test]
    fn context_packages_default_is_product() {
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.context.packages, vec!["product"]);
    }

    #[test]
    fn context_packages_custom_via_legacy_process() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[process]
packages = ["managed", "code", "research"]
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.context.packages, vec!["managed", "code", "research"]);
    }

    // -- Skills section -----------------------------------------------------

    #[test]
    fn skills_default_empty() {
        let config = parse_toml(minimal_toml()).unwrap();
        assert!(config.skills.include.is_empty());
        assert!(config.skills.exclude.is_empty());
    }

    #[test]
    fn skills_include_only() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[skills]
include = ["flutter-development", "rust-conventions"]
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.skills.include,
            vec!["flutter-development", "rust-conventions"]
        );
        assert!(config.skills.exclude.is_empty());
    }

    #[test]
    fn skills_exclude_only() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[skills]
exclude = ["standup-context"]
"#;
        let config = parse_toml(toml).unwrap();
        assert!(config.skills.include.is_empty());
        assert_eq!(config.skills.exclude, vec!["standup-context"]);
    }

    // -- Appearance ---------------------------------------------------------

    #[test]
    fn appearance_all_themes() {
        for (input, expected) in [
            ("gruvbox-dark", Theme::GruvboxDark),
            ("catppuccin-mocha", Theme::CatppuccinMocha),
            ("catppuccin-latte", Theme::CatppuccinLatte),
            ("dracula", Theme::Dracula),
            ("tokyo-night", Theme::TokyoNight),
            ("nord", Theme::Nord),
        ] {
            let toml = format!(
                r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[appearance]
theme = "{input}"
"#
            );
            let config = parse_toml(&toml).unwrap();
            assert_eq!(config.customization.theme, expected);
        }
    }

    #[test]
    fn appearance_mode_resolves_concrete_theme() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[customization]
theme = "dracula"
mode = "light"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.customization.theme, Theme::Dracula);
        assert_eq!(config.customization.mode, ThemeMode::Light);
        assert_eq!(
            config.customization.resolved_theme(),
            Theme::CatppuccinLatte
        );

        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[customization]
theme = "catppuccin-latte"
mode = "dark"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.customization.resolved_theme(),
            Theme::CatppuccinMocha
        );
    }

    // -- File loading -------------------------------------------------------

    #[test]
    fn load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aibox.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(minimal_toml().as_bytes()).unwrap();
        let config = AiboxConfig::load(&path).expect("should load from file");
        assert_eq!(config.container.name, "my-project");
    }

    #[test]
    fn load_missing_file() {
        let result = AiboxConfig::load(Path::new("/nonexistent/aibox.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn from_str_parses_and_validates() {
        let config = AiboxConfig::from_str(minimal_toml()).unwrap();
        assert_eq!(config.container.name, "my-project");
    }

    #[test]
    fn mcp_gateway_rejects_non_localhost_host() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "my-project"

[mcp.gateway]
host = "0.0.0.0"
"#;
        let err = AiboxConfig::from_str(toml).unwrap_err();
        assert!(err.to_string().contains("mcp.gateway.host"));
    }

    #[test]
    fn audio_enabled_selects_audio_voice_addon() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "audio-project"

[audio]
enabled = true
"#;

        let config = AiboxConfig::from_str(toml).unwrap();

        assert!(
            config.addons.addons.contains_key("audio-voice"),
            "audio.enabled should select the audio-voice addon now that audio packages are optional"
        );
    }

    #[test]
    fn audio_install_false_skips_audio_voice_addon() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "audio-project"

[audio]
enabled = true
install = false
"#;

        let config = AiboxConfig::from_str(toml).unwrap();

        assert!(config.audio.enabled);
        assert!(!config.audio.install);
        assert!(
            !config.addons.addons.contains_key("audio-voice"),
            "audio.install=false should not select the internal audio-voice addon"
        );
    }

    #[test]
    fn legacy_container_audio_is_accepted_and_promoted() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "audio-project"

[container.audio]
enabled = true
pulse_server = "tcp:localhost:4714"
"#;

        let config = AiboxConfig::from_str(toml).unwrap();

        assert!(config.audio.enabled);
        assert_eq!(config.audio.pulse_server, "tcp:localhost:4714");
        assert_eq!(config.container.audio, config.audio);
        assert!(config.addons.addons.contains_key("audio-voice"));
    }

    // -- Host/container path helpers ----------------------------------------

    #[test]
    #[serial]
    fn host_root_dir_default() {
        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.host_root_dir(), PathBuf::from(".aibox-home"));
    }

    #[test]
    #[serial]
    fn host_root_dir_env_override() {
        unsafe {
            std::env::set_var("AIBOX_HOST_ROOT", "/custom/root");
        }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.host_root_dir(), PathBuf::from("/custom/root"));
        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
    }

    #[test]
    #[serial]
    fn workspace_dir_default() {
        unsafe {
            std::env::remove_var("AIBOX_WORKSPACE_DIR");
        }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.workspace_dir(), "..");
    }

    #[test]
    #[serial]
    fn workspace_dir_env_override() {
        unsafe {
            std::env::set_var("AIBOX_WORKSPACE_DIR", "/my/workspace");
        }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.workspace_dir(), "/my/workspace");
        unsafe {
            std::env::remove_var("AIBOX_WORKSPACE_DIR");
        }
    }

    // -- test_config helper -------------------------------------------------

    #[test]
    fn test_config_validates() {
        let config = test_config();
        config.validate().expect("test_config should be valid");
    }

    // -- AI provider → addon resolution ------------------------------------

    #[test]
    fn resolve_ai_providers_creates_addon_entries() {
        let config = test_config(); // default: harnesses = [Claude]
        assert!(
            config.addons.has_addon("ai-claude"),
            "ai-claude addon should be auto-resolved from [ai].harnesses"
        );
    }

    #[test]
    fn resolve_ai_providers_multiple() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            harnesses = ["claude", "aider", "gemini"]
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert!(config.addons.has_addon("ai-claude"));
        assert!(config.addons.has_addon("ai-aider"));
        assert!(config.addons.has_addon("ai-gemini"));
    }

    #[test]
    fn resolve_ai_providers_skips_host_only_cursor_addon() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            harnesses = ["cursor", "codex"]
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert!(!config.addons.has_addon("ai-cursor"));
        assert!(config.addons.has_addon("ai-codex"));
    }

    #[test]
    fn resolve_ai_providers_empty_creates_no_addons() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            harnesses = []
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert!(!config.addons.has_addon("ai-claude"));
        assert!(!config.addons.has_addon("ai-aider"));
    }

    #[test]
    fn ai_harness_table_enables_harness_and_sets_cli_version() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai.harness.codex]
            enabled = true
            install = true
            version = "1.2.3"
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert!(config.ai.harnesses.contains(&AiProvider::Codex));
        assert_eq!(
            config.addons.tool_version("ai-codex", "codex"),
            Some("1.2.3")
        );
    }

    #[test]
    fn ai_harness_table_can_disable_legacy_array_selection() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            harnesses = ["claude", "codex"]
            [ai.harness.claude]
            enabled = false
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert!(!config.ai.harnesses.contains(&AiProvider::Claude));
        assert!(!config.addons.has_addon("ai-claude"));
        assert!(config.ai.harnesses.contains(&AiProvider::Codex));
        assert!(config.addons.has_addon("ai-codex"));
    }

    #[test]
    fn ai_harness_table_can_enable_without_installing_cli_addon() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai.harness.gemini]
            enabled = true
            install = false
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert!(config.ai.harnesses.contains(&AiProvider::Gemini));
        assert!(!config.addons.has_addon("ai-gemini"));
    }

    // -- ProcessKit section -------------------------------------------------

    #[test]
    fn processkit_section_default_values() {
        let pk = ProcessKitSection::default();
        assert_eq!(pk.source, crate::processkit_vocab::PROCESSKIT_GIT_SOURCE);
        assert_eq!(pk.version, "unset");
        assert_eq!(pk.src_path, "src");
        assert_eq!(pk.branch, None);
    }

    #[test]
    fn processkit_section_parses_from_toml() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://example.com/forks/processkit.git"
version = "v0.4.0"
src_path = "content"
branch = "develop"
"#;
        let config = parse_toml(toml).expect("should parse processkit section");
        assert_eq!(
            config.processkit.source,
            "https://example.com/forks/processkit.git"
        );
        assert_eq!(config.processkit.version, "v0.4.0");
        assert_eq!(config.processkit.src_path, "content");
        assert_eq!(config.processkit.branch.as_deref(), Some("develop"));
    }

    #[test]
    fn processkit_section_parses_with_only_source() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://example.com/forks/processkit.git"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.processkit.source,
            "https://example.com/forks/processkit.git"
        );
        assert_eq!(config.processkit.version, "unset");
        assert_eq!(config.processkit.src_path, "src");
        assert_eq!(config.processkit.branch, None);
    }

    #[test]
    fn processkit_section_parses_when_section_missing() {
        // An old-style aibox.toml with no [processkit] block should parse
        // cleanly with all defaults filled in.
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(
            config.processkit.source,
            "https://github.com/projectious-work/processkit.git"
        );
        assert_eq!(config.processkit.version, "unset");
        assert_eq!(config.processkit.src_path, "src");
        assert_eq!(config.processkit.branch, None);
    }

    #[test]
    fn processkit_validate_rejects_empty_source() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = ""
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject empty source");
    }

    #[test]
    fn processkit_validate_rejects_non_url_source() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "not-a-url"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject non-URL source");
    }

    #[test]
    fn processkit_validate_rejects_path_traversal_in_src_path() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://github.com/projectious-work/processkit.git"
src_path = "../etc"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject path traversal in src_path");
    }

    #[test]
    fn processkit_validate_rejects_absolute_src_path() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://github.com/projectious-work/processkit.git"
src_path = "/etc"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject absolute src_path");
    }

    #[test]
    fn processkit_validate_accepts_unset_version() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://github.com/projectious-work/processkit.git"
version = "unset"
"#;
        let config = parse_toml(toml).expect("unset sentinel should validate");
        assert_eq!(config.processkit.version, "unset");
    }

    #[test]
    fn processkit_validate_accepts_semver_version() {
        for ver in ["v0.4.0", "0.4.0", "v1.0.0-rc1", "v0.4"] {
            let toml = format!(
                r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://github.com/projectious-work/processkit.git"
version = "{ver}"
"#
            );
            parse_toml(&toml)
                .unwrap_or_else(|e| panic!("version {ver} should validate, but got error: {e}"));
        }
    }

    #[test]
    fn processkit_validate_rejects_empty_branch() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://github.com/projectious-work/processkit.git"
branch = ""
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject empty branch");
    }

    #[test]
    fn resolve_ai_providers_does_not_overwrite_explicit_addon() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            harnesses = ["aider"]
            [addons.ai-aider.tools]
            aider = { version = "custom" }
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        // Should keep the user's explicit config, not overwrite with empty tools
        let aider_tools = &config.addons.get_addon("ai-aider").unwrap().tools;
        assert!(
            aider_tools.contains_key("aider"),
            "should preserve user-configured tool entry"
        );
    }

    // -- ExtraVolume / .aibox-local.toml tests --------------------------------

    #[test]
    fn extra_volumes_parse_from_toml() {
        let toml = r#"
[aibox]
version = "0.9.0"
[container]
name = "test"

[[container.extra_volumes]]
source = "~/.config/gh"
target = "/home/aibox/.config/gh"

[[container.extra_volumes]]
source = "~/.aws"
target = "/home/aibox/.aws"
read_only = true
"#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert_eq!(config.container.extra_volumes.len(), 2);
        assert_eq!(config.container.extra_volumes[0].source, "~/.config/gh");
        assert_eq!(
            config.container.extra_volumes[0].target,
            "/home/aibox/.config/gh"
        );
        assert!(!config.container.extra_volumes[0].read_only);
        assert!(config.container.extra_volumes[1].read_only);
    }

    #[test]
    fn environment_parses_from_toml() {
        let toml = r#"
[aibox]
version = "0.9.0"
[container]
name = "test"

[container.environment]
GH_TOKEN = "ghp_abc"
MY_VAR = "hello"
"#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert_eq!(
            config
                .container
                .environment
                .get("GH_TOKEN")
                .map(|s| s.as_str()),
            Some("ghp_abc")
        );
        assert_eq!(
            config
                .container
                .environment
                .get("MY_VAR")
                .map(|s| s.as_str()),
            Some("hello")
        );
    }

    #[test]
    fn extra_volumes_rejects_dotdot_in_source() {
        let toml = r#"
[aibox]
version = "0.9.0"
[container]
name = "test"
[[container.extra_volumes]]
source = "../../../etc/passwd"
target = "/home/aibox/passwd"
"#;
        let result = AiboxConfig::from_str(toml);
        // from_str calls validate() which calls validate_extra_volumes()
        assert!(result.is_err(), "should reject .. in source");
    }

    #[test]
    fn extra_volumes_rejects_relative_target() {
        let toml = r#"
[aibox]
version = "0.9.0"
[container]
name = "test"
[[container.extra_volumes]]
source = "~/.config/gh"
target = "home/aibox/.config/gh"
"#;
        let result = AiboxConfig::from_str(toml);
        assert!(result.is_err(), "should reject non-absolute target");
    }

    #[test]
    fn aibox_local_toml_merges_environment_and_volumes() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        // Write aibox.toml
        std::fs::write(
            dir.join("aibox.toml"),
            r#"
[aibox]
version = "0.9.0"
[container]
name = "test"
[container.environment]
SHARED = "from-main"
"#,
        )
        .unwrap();

        // Write .aibox-local.toml
        std::fs::write(
            dir.join(".aibox-local.toml"),
            r#"
[container.environment]
GH_TOKEN = "ghp_secret"
SHARED = "local-wins"

[[container.extra_volumes]]
source = "~/.config/gh"
target = "/home/aibox/.config/gh"
"#,
        )
        .unwrap();

        // Load from the temp dir
        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir).unwrap();
        let config = AiboxConfig::load_or_default().unwrap();
        std::env::set_current_dir(orig).unwrap();

        // Local env merged: GH_TOKEN added, SHARED overridden by local
        assert_eq!(
            config
                .container
                .environment
                .get("GH_TOKEN")
                .map(|s| s.as_str()),
            Some("ghp_secret")
        );
        assert_eq!(
            config
                .container
                .environment
                .get("SHARED")
                .map(|s| s.as_str()),
            Some("local-wins")
        );
        // Volume appended
        assert_eq!(config.container.extra_volumes.len(), 1);
        assert_eq!(config.container.extra_volumes[0].source, "~/.config/gh");
    }
}
