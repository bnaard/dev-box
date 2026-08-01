use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[cfg(all(
    not(test),
    any(target_os = "macos", target_os = "linux", target_os = "windows")
))]
use std::process::Command;

#[cfg(test)]
thread_local! {
    static TEST_HOST_ROOT: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn set_test_host_root(path: Option<PathBuf>) {
    TEST_HOST_ROOT.with(|cell| {
        *cell.borrow_mut() = path;
    });
}

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

/// Context content mode.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum ContextMode {
    #[default]
    Processkit,
    HarnessOnly,
}

impl std::fmt::Display for ContextMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextMode::Processkit => write!(f, "processkit"),
            ContextMode::HarnessOnly => write!(f, "harness-only"),
        }
    }
}

/// [context] section — context system versioning and package selection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextSection {
    #[serde(default)]
    pub mode: ContextMode,
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
            mode: ContextMode::default(),
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
    /// Tau educational multi-provider coding agent.
    Tau,
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
            AiHarness::Tau => write!(f, "tau"),
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
            AiHarness::Tau => "tau",
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
            AiHarness::Tau => "Tau (tau)",
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
            AiHarness::Tau => Some(".tau"),
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
            AiHarness::Tau,
        ]
    }
}

/// Backward-compatible alias — all existing code that references `AiProvider`
/// continues to compile. The `OpenAI` variant maps to `AiHarness::Codex` via
/// serde alias. New code should use `AiHarness` directly.
pub type AiProvider = AiHarness;

/// An AI model provider — the organization whose API credentials may be needed.
/// Declaring a provider is optional; it hints which API key/base URL env vars are available.
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

    /// The environment variable name for this provider's endpoint/base URL.
    pub fn endpoint_env(&self) -> &'static str {
        match self {
            AiModelProvider::Anthropic => "ANTHROPIC_BASE_URL",
            AiModelProvider::OpenAI => "OPENAI_BASE_URL",
            AiModelProvider::Google => "GEMINI_BASE_URL",
            AiModelProvider::Mistral => "MISTRAL_BASE_URL",
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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AiExecutionFilesystem {
    ReadOnly,
    #[default]
    WorkspaceWrite,
    ContainerFull,
}

impl AiExecutionFilesystem {
    pub fn as_str(self) -> &'static str {
        match self {
            AiExecutionFilesystem::ReadOnly => "read-only",
            AiExecutionFilesystem::WorkspaceWrite => "workspace-write",
            AiExecutionFilesystem::ContainerFull => "container-full",
        }
    }
}

impl std::fmt::Display for AiExecutionFilesystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AiExecutionApproval {
    Ask,
    #[default]
    OnRequest,
    Never,
}

impl AiExecutionApproval {
    pub fn as_str(self) -> &'static str {
        match self {
            AiExecutionApproval::Ask => "ask",
            AiExecutionApproval::OnRequest => "on-request",
            AiExecutionApproval::Never => "never",
        }
    }
}

impl std::fmt::Display for AiExecutionApproval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AiExecutionNetwork {
    Deny,
    #[default]
    Ask,
    Allow,
}

impl AiExecutionNetwork {
    pub fn as_str(self) -> &'static str {
        match self {
            AiExecutionNetwork::Deny => "deny",
            AiExecutionNetwork::Ask => "ask",
            AiExecutionNetwork::Allow => "allow",
        }
    }
}

impl std::fmt::Display for AiExecutionNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Global AI execution policy under `[ai.execution]`.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiExecutionPolicy {
    #[serde(default)]
    pub filesystem: AiExecutionFilesystem,
    #[serde(default)]
    pub approval: AiExecutionApproval,
    #[serde(default)]
    pub network: AiExecutionNetwork,
}

/// Optional per-harness execution policy overrides under
/// `[ai.execution.<name>]` or the legacy `[ai.harness.<name>.execution]`.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiHarnessExecutionOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filesystem: Option<AiExecutionFilesystem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval: Option<AiExecutionApproval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<AiExecutionNetwork>,
}

impl AiExecutionPolicy {
    pub fn with_harness_override(self, override_policy: AiHarnessExecutionOverride) -> Self {
        Self {
            filesystem: override_policy.filesystem.unwrap_or(self.filesystem),
            approval: override_policy.approval.unwrap_or(self.approval),
            network: override_policy.network.unwrap_or(self.network),
        }
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<AiHarnessExecutionOverride>,
}

/// [ai] section — AI harness and model provider configuration.
///
/// New configs select AI harnesses via `[ai.harness.<name>]` tables. The
/// `harnesses` list remains accepted as a legacy/internal effective selector.
/// `model_providers` is optional — declares which model-provider credentials
/// are available (API keys and optional base URLs).
/// Legacy `providers` field is accepted for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum AiHarnessListEntry {
    Legacy(AiHarness),
    Detailed {
        harness: AiHarness,
        #[serde(default, alias = "enable")]
        enabled: Option<bool>,
        #[serde(default)]
        install: Option<bool>,
        #[serde(default)]
        version: Option<String>,
    },
}

/// Raw serde shape for `[ai.execution]`. The scalar fields are the global
/// defaults; nested `[ai.execution.<harness>]` tables are per-harness
/// overrides.
#[derive(Debug, Clone, Deserialize, Default)]
struct RawAiExecutionSection {
    #[serde(default)]
    filesystem: AiExecutionFilesystem,
    #[serde(default)]
    approval: AiExecutionApproval,
    #[serde(default)]
    network: AiExecutionNetwork,
    #[serde(default, flatten)]
    harness: HashMap<AiHarness, AiHarnessExecutionOverride>,
}

impl RawAiExecutionSection {
    fn global_policy(&self) -> AiExecutionPolicy {
        AiExecutionPolicy {
            filesystem: self.filesystem,
            approval: self.approval,
            network: self.network,
        }
    }
}

/// Raw serde shape for `[ai]`. `harnesses` accepts both the legacy
/// `["codex"]` list and the canonical ordered list of dictionaries.
#[derive(Debug, Clone, Deserialize, Default)]
struct RawAiSection {
    #[serde(default)]
    harnesses: Vec<AiHarnessListEntry>,
    #[serde(default)]
    harness_order: Vec<AiHarness>,
    #[serde(default)]
    model_providers: Vec<AiModelProvider>,
    #[serde(default)]
    execution: RawAiExecutionSection,
    #[serde(default)]
    harness: HashMap<AiHarness, AiHarnessConfig>,
    #[serde(default)]
    providers: Vec<AiHarness>,
    #[serde(default)]
    agents: AgentsSection,
    #[serde(default)]
    mcp: McpSection,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiSection {
    /// Which AI coding tools to install (determines addons, volumes, MCP config).
    /// Serde default is empty; `migrate_legacy()` applies the real default
    /// ([Claude]) when neither harnesses nor providers was explicitly set.
    #[serde(default)]
    pub harnesses: Vec<AiHarness>,

    /// Explicit tmux/layout order for enabled harnesses. The 1st, 2nd, 3rd
    /// harness in layout semantics are resolved from this list; enabled
    /// harnesses omitted here are appended in canonical order.
    #[serde(default)]
    pub harness_order: Vec<AiHarness>,

    /// Which model provider API key/base URL env vars are available (optional hint).
    #[serde(default)]
    pub model_providers: Vec<AiModelProvider>,

    /// Global AI execution policy. Per-harness tables may override individual
    /// axes through `[ai.execution.<name>]`; the legacy
    /// `[ai.harness.<name>.execution]` form remains accepted.
    #[serde(default)]
    pub execution: AiExecutionPolicy,

    /// Per-harness enable/install/version controls. New configs should prefer
    /// this as the user-facing selector.
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

impl<'de> Deserialize<'de> for AiSection {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawAiSection::deserialize(deserializer)?;
        let mut harnesses = Vec::new();
        let mut harness_config = raw.harness;

        for entry in raw.harnesses {
            match entry {
                AiHarnessListEntry::Legacy(harness) => {
                    if !harnesses.contains(&harness) {
                        harnesses.push(harness);
                    }
                }
                AiHarnessListEntry::Detailed {
                    harness,
                    enabled,
                    install,
                    version,
                } => {
                    let enabled = enabled.unwrap_or(false);
                    let install = install.unwrap_or(false);
                    let version = version.filter(|value| !value.is_empty());
                    harness_config.insert(
                        harness.clone(),
                        AiHarnessConfig {
                            enabled: Some(enabled),
                            install: Some(install),
                            version,
                            execution: None,
                        },
                    );
                    if enabled && !harnesses.contains(&harness) {
                        harnesses.push(harness);
                    }
                }
            }
        }

        let execution_policy = raw.execution.global_policy();
        for (harness, execution) in raw.execution.harness {
            harness_config.entry(harness).or_default().execution = Some(execution);
        }

        Ok(Self {
            harnesses,
            harness_order: raw.harness_order,
            model_providers: raw.model_providers,
            execution: execution_policy,
            harness: harness_config,
            providers: raw.providers,
            agents: raw.agents,
            mcp: raw.mcp,
        })
    }
}

impl AiSection {
    /// Migrate legacy `providers` → `harnesses` if needed.
    /// Call after deserialization and before any code reads `harnesses`.
    pub fn migrate_legacy(&mut self) {
        if self.harnesses.is_empty() && !self.providers.is_empty() {
            // Legacy format: move providers → harnesses
            self.harnesses = self.providers.drain(..).collect();
        }
        for harness in AiHarness::all() {
            let Some(config) = self.harness.get(harness) else {
                continue;
            };
            match config.enabled {
                Some(true) => {
                    if !self.harnesses.contains(harness) {
                        self.harnesses.push(harness.clone());
                    }
                }
                Some(false) => {
                    self.harnesses.retain(|candidate| candidate != harness);
                }
                None => {
                    if !self.harnesses.contains(harness) && config.has_controls() {
                        self.harnesses.push(harness.clone());
                    }
                }
            }
        }
        self.apply_harness_order();
    }

    pub fn apply_harness_order(&mut self) {
        if self.harnesses.len() <= 1 || self.harness_order.is_empty() {
            return;
        }

        let mut ordered = Vec::with_capacity(self.harnesses.len());
        for harness in &self.harness_order {
            if self.harnesses.contains(harness) && !ordered.contains(harness) {
                ordered.push(harness.clone());
            }
        }
        for harness in AiHarness::all() {
            if self.harnesses.contains(harness) && !ordered.contains(harness) {
                ordered.push(harness.clone());
            }
        }
        self.harnesses = ordered;
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

    pub fn execution_for_harness(&self, harness: &AiHarness) -> AiExecutionPolicy {
        self.harness
            .get(harness)
            .and_then(|config| config.execution)
            .map(|override_policy| self.execution.with_harness_override(override_policy))
            .unwrap_or(self.execution)
    }
}

impl AiHarnessConfig {
    fn has_controls(&self) -> bool {
        self.install.is_some()
            || self
                .version
                .as_deref()
                .is_some_and(|version| !version.is_empty())
    }
}

impl Default for AiSection {
    fn default() -> Self {
        Self {
            harnesses: default_ai_harnesses(),
            harness_order: Vec::new(),
            model_providers: Vec::new(),
            execution: AiExecutionPolicy::default(),
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
/// In TOML this appears as e.g. `python = { version = "3.14" }`,
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
/// python = { version = "3.14" }
/// uv = { version = "0.12.0" }
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

/// Color themes available across all tools (tmux, Vim, Yazi, lazygit).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    GruvboxDark,
    GruvboxLight,
    CatppuccinMocha,
    CatppuccinMacchiato,
    CatppuccinFrappe,
    CatppuccinLatte,
    Dracula,
    DraculaSoft,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightDay,
    Nord,
    RosePine,
    RosePineMoon,
    RosePineDawn,
    Material,
    MaterialOcean,
    MaterialPalenight,
    MaterialLighter,
    MaterialDarker,
    SolarizedDark,
    SolarizedLight,
    GithubDark,
    GithubLight,
    GithubDarkDimmed,
    GithubDarkHighContrast,
    GithubLightHighContrast,
    AyuDark,
    AyuMirage,
    AyuLight,
    NightOwl,
    NightOwlLight,
    Moonlight,
    Projectious,
    // NEW families
    Andromeeda,
    AuroraX,
    EverforestDark,
    EverforestLight,
    Houston,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Laserwave,
    MinDark,
    MinLight,
    Monokai,
    OneDarkPro,
    OneLight,
    Plastic,
    Poimandres,
    Red,
    SlackDark,
    SlackOchin,
    SnazzyLight,
    Synthwave84,
    Vesper,
    VitesseDark,
    VitesseLight,
    VitesseBlack,
    VsCodeDarkPlus,
    VsCodeLightPlus,
}

/// User-facing theme family selector. Pairs with `ThemeMode` (and optional
/// `variant`) to resolve to a concrete `Theme`. Solo families (dracula, moonlight,
/// nord, projectious) have no light/dark partner and ignore mode.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum ThemeFamily {
    Andromeeda,
    AuroraX,
    Ayu,
    Catppuccin,
    Dracula,
    Everforest,
    Github,
    #[default]
    Gruvbox,
    Houston,
    Kanagawa,
    Laserwave,
    Material,
    Min,
    Monokai,
    Moonlight,
    NightOwl,
    Nord,
    OneDark,
    Plastic,
    Poimandres,
    Projectious,
    Red,
    RosePine,
    Slack,
    Snazzy,
    Solarized,
    Synthwave84,
    TokyoNight,
    Vesper,
    Vitesse,
    #[clap(alias = "vscode")]
    #[serde(alias = "vscode")]
    VsCode,
}

impl std::fmt::Display for ThemeFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeFamily::Andromeeda => write!(f, "andromeeda"),
            ThemeFamily::AuroraX => write!(f, "aurora-x"),
            ThemeFamily::Ayu => write!(f, "ayu"),
            ThemeFamily::Catppuccin => write!(f, "catppuccin"),
            ThemeFamily::Dracula => write!(f, "dracula"),
            ThemeFamily::Everforest => write!(f, "everforest"),
            ThemeFamily::Github => write!(f, "github"),
            ThemeFamily::Gruvbox => write!(f, "gruvbox"),
            ThemeFamily::Houston => write!(f, "houston"),
            ThemeFamily::Kanagawa => write!(f, "kanagawa"),
            ThemeFamily::Laserwave => write!(f, "laserwave"),
            ThemeFamily::Material => write!(f, "material"),
            ThemeFamily::Min => write!(f, "min"),
            ThemeFamily::Monokai => write!(f, "monokai"),
            ThemeFamily::Moonlight => write!(f, "moonlight"),
            ThemeFamily::NightOwl => write!(f, "night-owl"),
            ThemeFamily::Nord => write!(f, "nord"),
            ThemeFamily::OneDark => write!(f, "one-dark"),
            ThemeFamily::Plastic => write!(f, "plastic"),
            ThemeFamily::Poimandres => write!(f, "poimandres"),
            ThemeFamily::Projectious => write!(f, "projectious"),
            ThemeFamily::Red => write!(f, "red"),
            ThemeFamily::RosePine => write!(f, "rose-pine"),
            ThemeFamily::Slack => write!(f, "slack"),
            ThemeFamily::Snazzy => write!(f, "snazzy"),
            ThemeFamily::Solarized => write!(f, "solarized"),
            ThemeFamily::Synthwave84 => write!(f, "synthwave-84"),
            ThemeFamily::TokyoNight => write!(f, "tokyo-night"),
            ThemeFamily::Vesper => write!(f, "vesper"),
            ThemeFamily::Vitesse => write!(f, "vitesse"),
            ThemeFamily::VsCode => write!(f, "vscode"),
        }
    }
}

/// Family that owns a concrete Theme, for legacy migration and resolution.
pub fn family_of(theme: &Theme) -> ThemeFamily {
    match theme {
        Theme::AyuDark | Theme::AyuMirage | Theme::AyuLight => ThemeFamily::Ayu,
        Theme::CatppuccinMocha
        | Theme::CatppuccinMacchiato
        | Theme::CatppuccinFrappe
        | Theme::CatppuccinLatte => ThemeFamily::Catppuccin,
        Theme::Dracula | Theme::DraculaSoft => ThemeFamily::Dracula,
        Theme::GithubDark
        | Theme::GithubLight
        | Theme::GithubDarkDimmed
        | Theme::GithubDarkHighContrast
        | Theme::GithubLightHighContrast => ThemeFamily::Github,
        Theme::GruvboxDark | Theme::GruvboxLight => ThemeFamily::Gruvbox,
        Theme::Material
        | Theme::MaterialOcean
        | Theme::MaterialPalenight
        | Theme::MaterialLighter
        | Theme::MaterialDarker => ThemeFamily::Material,
        Theme::Moonlight => ThemeFamily::Moonlight,
        Theme::NightOwl | Theme::NightOwlLight => ThemeFamily::NightOwl,
        Theme::Nord => ThemeFamily::Nord,
        Theme::Projectious => ThemeFamily::Projectious,
        Theme::RosePine | Theme::RosePineMoon | Theme::RosePineDawn => ThemeFamily::RosePine,
        Theme::SolarizedDark | Theme::SolarizedLight => ThemeFamily::Solarized,
        Theme::TokyoNight | Theme::TokyoNightStorm | Theme::TokyoNightDay => {
            ThemeFamily::TokyoNight
        }
        // New families
        Theme::Andromeeda => ThemeFamily::Andromeeda,
        Theme::AuroraX => ThemeFamily::AuroraX,
        Theme::EverforestDark | Theme::EverforestLight => ThemeFamily::Everforest,
        Theme::Houston => ThemeFamily::Houston,
        Theme::KanagawaWave | Theme::KanagawaDragon | Theme::KanagawaLotus => ThemeFamily::Kanagawa,
        Theme::Laserwave => ThemeFamily::Laserwave,
        Theme::MinDark | Theme::MinLight => ThemeFamily::Min,
        Theme::Monokai => ThemeFamily::Monokai,
        Theme::OneDarkPro | Theme::OneLight => ThemeFamily::OneDark,
        Theme::Plastic => ThemeFamily::Plastic,
        Theme::Poimandres => ThemeFamily::Poimandres,
        Theme::Red => ThemeFamily::Red,
        Theme::SlackDark | Theme::SlackOchin => ThemeFamily::Slack,
        Theme::SnazzyLight => ThemeFamily::Snazzy,
        Theme::Synthwave84 => ThemeFamily::Synthwave84,
        Theme::Vesper => ThemeFamily::Vesper,
        Theme::VitesseDark | Theme::VitesseLight | Theme::VitesseBlack => ThemeFamily::Vitesse,
        Theme::VsCodeDarkPlus | Theme::VsCodeLightPlus => ThemeFamily::VsCode,
    }
}

/// The alternate variant name of a concrete Theme, if any.
/// Returns `Some("mirage")` for AyuMirage, `Some("frappe")` for CatppuccinFrappe, etc.
/// Returns `None` for canonical dark/light/solo variants.
pub fn variant_name_of(theme: &Theme) -> Option<&'static str> {
    match theme {
        Theme::AyuMirage => Some("mirage"),
        Theme::CatppuccinMacchiato => Some("macchiato"),
        Theme::CatppuccinFrappe => Some("frappe"),
        Theme::DraculaSoft => Some("soft"),
        Theme::GithubDarkDimmed => Some("dimmed"),
        Theme::GithubDarkHighContrast => Some("high-contrast-dark"),
        Theme::GithubLightHighContrast => Some("high-contrast-light"),
        Theme::KanagawaDragon => Some("dragon"),
        Theme::MaterialOcean => Some("ocean"),
        Theme::MaterialPalenight => Some("palenight"),
        Theme::MaterialDarker => Some("darker"),
        Theme::RosePineMoon => Some("moon"),
        Theme::SlackOchin => Some("ochin"),
        Theme::TokyoNightStorm => Some("storm"),
        Theme::VitesseBlack => Some("black"),
        _ => None,
    }
}

/// Resolve (family, effective_mode, optional variant) → concrete Theme.
///
/// `mode` must already be resolved (not Auto); callers are responsible for
/// collapsing Auto → Light | Dark before calling this.
pub(crate) fn resolve_theme_from_family(
    family: &ThemeFamily,
    mode: ThemeMode,
    variant: Option<&str>,
) -> Theme {
    // Solo families ignore both mode and variant.
    match family {
        ThemeFamily::Andromeeda => return Theme::Andromeeda,
        ThemeFamily::AuroraX => return Theme::AuroraX,
        ThemeFamily::Houston => return Theme::Houston,
        ThemeFamily::Laserwave => return Theme::Laserwave,
        ThemeFamily::Monokai => return Theme::Monokai,
        ThemeFamily::Moonlight => return Theme::Moonlight,
        ThemeFamily::Nord => return Theme::Nord,
        ThemeFamily::Plastic => return Theme::Plastic,
        ThemeFamily::Poimandres => return Theme::Poimandres,
        ThemeFamily::Projectious => return Theme::Projectious,
        ThemeFamily::Red => return Theme::Red,
        ThemeFamily::Snazzy => return Theme::SnazzyLight,
        ThemeFamily::Synthwave84 => return Theme::Synthwave84,
        ThemeFamily::Vesper => return Theme::Vesper,
        _ => {}
    }

    match mode {
        ThemeMode::Light => match family {
            ThemeFamily::Ayu => Theme::AyuLight,
            ThemeFamily::Catppuccin => Theme::CatppuccinLatte,
            ThemeFamily::Dracula => Theme::Dracula, // no light variant — return dark canonical
            ThemeFamily::Everforest => Theme::EverforestLight,
            ThemeFamily::Github => match variant {
                Some("high-contrast-light") => Theme::GithubLightHighContrast,
                _ => Theme::GithubLight,
            },
            ThemeFamily::Gruvbox => Theme::GruvboxLight,
            ThemeFamily::Kanagawa => Theme::KanagawaLotus,
            ThemeFamily::Material => Theme::MaterialLighter,
            ThemeFamily::Min => Theme::MinLight,
            ThemeFamily::NightOwl => Theme::NightOwlLight,
            ThemeFamily::OneDark => Theme::OneLight,
            ThemeFamily::RosePine => Theme::RosePineDawn,
            ThemeFamily::Slack => Theme::SlackOchin,
            ThemeFamily::Solarized => Theme::SolarizedLight,
            ThemeFamily::TokyoNight => Theme::TokyoNightDay,
            ThemeFamily::Vitesse => Theme::VitesseLight,
            ThemeFamily::VsCode => Theme::VsCodeLightPlus,
            // Solo families already returned above.
            ThemeFamily::Andromeeda
            | ThemeFamily::AuroraX
            | ThemeFamily::Houston
            | ThemeFamily::Laserwave
            | ThemeFamily::Monokai
            | ThemeFamily::Moonlight
            | ThemeFamily::Nord
            | ThemeFamily::Plastic
            | ThemeFamily::Poimandres
            | ThemeFamily::Projectious
            | ThemeFamily::Red
            | ThemeFamily::Snazzy
            | ThemeFamily::Synthwave84
            | ThemeFamily::Vesper => {
                unreachable!("solo families handled above")
            }
        },
        ThemeMode::Dark | ThemeMode::Auto => match family {
            ThemeFamily::Ayu => match variant {
                Some("mirage") => Theme::AyuMirage,
                _ => Theme::AyuDark,
            },
            ThemeFamily::Catppuccin => match variant {
                Some("macchiato") => Theme::CatppuccinMacchiato,
                Some("frappe") => Theme::CatppuccinFrappe,
                _ => Theme::CatppuccinMocha,
            },
            ThemeFamily::Dracula => match variant {
                Some("soft") => Theme::DraculaSoft,
                _ => Theme::Dracula,
            },
            ThemeFamily::Everforest => Theme::EverforestDark,
            ThemeFamily::Github => match variant {
                Some("dimmed") => Theme::GithubDarkDimmed,
                Some("high-contrast-dark") => Theme::GithubDarkHighContrast,
                _ => Theme::GithubDark,
            },
            ThemeFamily::Gruvbox => Theme::GruvboxDark,
            ThemeFamily::Kanagawa => match variant {
                Some("dragon") => Theme::KanagawaDragon,
                _ => Theme::KanagawaWave,
            },
            ThemeFamily::Material => match variant {
                Some("ocean") => Theme::MaterialOcean,
                Some("palenight") => Theme::MaterialPalenight,
                Some("darker") => Theme::MaterialDarker,
                _ => Theme::Material,
            },
            ThemeFamily::Min => Theme::MinDark,
            ThemeFamily::NightOwl => Theme::NightOwl,
            ThemeFamily::OneDark => Theme::OneDarkPro,
            ThemeFamily::RosePine => match variant {
                Some("moon") => Theme::RosePineMoon,
                _ => Theme::RosePine,
            },
            ThemeFamily::Slack => Theme::SlackDark,
            ThemeFamily::Solarized => Theme::SolarizedDark,
            ThemeFamily::TokyoNight => match variant {
                Some("storm") => Theme::TokyoNightStorm,
                _ => Theme::TokyoNight,
            },
            ThemeFamily::Vitesse => match variant {
                Some("black") => Theme::VitesseBlack,
                _ => Theme::VitesseDark,
            },
            ThemeFamily::VsCode => Theme::VsCodeDarkPlus,
            // Solo families already returned above.
            ThemeFamily::Andromeeda
            | ThemeFamily::AuroraX
            | ThemeFamily::Houston
            | ThemeFamily::Laserwave
            | ThemeFamily::Monokai
            | ThemeFamily::Moonlight
            | ThemeFamily::Nord
            | ThemeFamily::Plastic
            | ThemeFamily::Poimandres
            | ThemeFamily::Projectious
            | ThemeFamily::Red
            | ThemeFamily::Snazzy
            | ThemeFamily::Synthwave84
            | ThemeFamily::Vesper => {
                unreachable!("solo families handled above")
            }
        },
    }
}

/// Global light/dark preference applied on top of the selected theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum ThemeMode {
    /// Follow the host OS appearance when detectable; otherwise preserve the
    /// selected concrete theme.
    #[default]
    Auto,
    /// Prefer a light concrete palette when the selected theme family provides
    /// a light variant.
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

static DETECTED_HOST_THEME_MODE: OnceLock<Option<ThemeMode>> = OnceLock::new();

/// Detect the host OS light/dark appearance for `mode = "auto"`.
///
/// This runs in the short-lived host-side `aibox` process during `apply`, `up`,
/// and `set theme.*`. Containers do not receive OS appearance change events;
/// rerun one of those commands to regenerate mounted runtime theme files.
pub(crate) fn detected_host_theme_mode() -> Option<ThemeMode> {
    DETECTED_HOST_THEME_MODE
        .get_or_init(detect_host_theme_mode_uncached)
        .clone()
}

fn detect_host_theme_mode_uncached() -> Option<ThemeMode> {
    if let Some(mode) = host_theme_mode_from_env() {
        return Some(mode);
    }

    #[cfg(test)]
    {
        None
    }

    #[cfg(all(not(test), target_os = "macos"))]
    {
        detect_macos_theme_mode()
    }

    #[cfg(all(not(test), target_os = "linux"))]
    {
        detect_linux_theme_mode()
    }

    #[cfg(all(not(test), target_os = "windows"))]
    {
        detect_windows_theme_mode()
    }

    #[cfg(all(
        not(test),
        not(target_os = "macos"),
        not(target_os = "linux"),
        not(target_os = "windows")
    ))]
    {
        None
    }
}

fn host_theme_mode_from_env() -> Option<ThemeMode> {
    let value = std::env::var("AIBOX_HOST_THEME_MODE").ok()?;
    match value.trim().to_ascii_lowercase().as_str() {
        "light" => Some(ThemeMode::Light),
        "dark" => Some(ThemeMode::Dark),
        _ => None,
    }
}

#[cfg(all(not(test), target_os = "macos"))]
fn detect_macos_theme_mode() -> Option<ThemeMode> {
    let output = Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
        .ok()?;
    if output.status.success() {
        let value = String::from_utf8_lossy(&output.stdout);
        if value.trim().eq_ignore_ascii_case("dark") {
            return Some(ThemeMode::Dark);
        }
    }
    // On macOS, AppleInterfaceStyle is absent in light mode.
    Some(ThemeMode::Light)
}

#[cfg(all(not(test), target_os = "linux"))]
fn detect_linux_theme_mode() -> Option<ThemeMode> {
    if let Some(mode) = command_stdout_theme_mode(
        "gsettings",
        &["get", "org.gnome.desktop.interface", "color-scheme"],
    ) {
        return Some(mode);
    }
    if let Some(mode) = command_stdout_theme_mode(
        "gsettings",
        &["get", "org.gnome.desktop.interface", "gtk-theme"],
    ) {
        return Some(mode);
    }
    command_stdout_theme_mode(
        "kreadconfig5",
        &["--group", "General", "--key", "ColorScheme"],
    )
    .or_else(|| {
        command_stdout_theme_mode(
            "kreadconfig6",
            &["--group", "General", "--key", "ColorScheme"],
        )
    })
}

#[cfg(all(not(test), target_os = "windows"))]
fn detect_windows_theme_mode() -> Option<ThemeMode> {
    command_stdout_theme_mode(
        "powershell.exe",
        &[
            "-NoProfile",
            "-Command",
            "(Get-ItemProperty HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize).AppsUseLightTheme",
        ],
    )
}

#[cfg(any(
    all(not(test), target_os = "linux"),
    all(not(test), target_os = "windows")
))]
fn command_stdout_theme_mode(command: &str, args: &[&str]) -> Option<ThemeMode> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_host_theme_mode_text(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(any(
    test,
    all(not(test), target_os = "linux"),
    all(not(test), target_os = "windows")
))]
fn parse_host_theme_mode_text(value: &str) -> Option<ThemeMode> {
    let normalized = value.trim().trim_matches('\'').to_ascii_lowercase();
    match normalized.as_str() {
        "0" => Some(ThemeMode::Dark),
        "1" => Some(ThemeMode::Light),
        item if item.contains("prefer-dark") || item.contains("dark") => Some(ThemeMode::Dark),
        item if item.contains("prefer-light")
            || item.contains("light")
            || item == "default"
            || item == "standard" =>
        {
            Some(ThemeMode::Light)
        }
        _ => None,
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::GruvboxDark => write!(f, "gruvbox-dark"),
            Theme::GruvboxLight => write!(f, "gruvbox-light"),
            Theme::CatppuccinMocha => write!(f, "catppuccin-mocha"),
            Theme::CatppuccinMacchiato => write!(f, "catppuccin-macchiato"),
            Theme::CatppuccinFrappe => write!(f, "catppuccin-frappe"),
            Theme::CatppuccinLatte => write!(f, "catppuccin-latte"),
            Theme::Dracula => write!(f, "dracula"),
            Theme::DraculaSoft => write!(f, "dracula-soft"),
            Theme::TokyoNight => write!(f, "tokyo-night"),
            Theme::TokyoNightStorm => write!(f, "tokyo-night-storm"),
            Theme::TokyoNightDay => write!(f, "tokyo-night-day"),
            Theme::Nord => write!(f, "nord"),
            Theme::RosePine => write!(f, "rose-pine"),
            Theme::RosePineMoon => write!(f, "rose-pine-moon"),
            Theme::RosePineDawn => write!(f, "rose-pine-dawn"),
            Theme::Material => write!(f, "material"),
            Theme::MaterialOcean => write!(f, "material-ocean"),
            Theme::MaterialPalenight => write!(f, "material-palenight"),
            Theme::MaterialLighter => write!(f, "material-lighter"),
            Theme::MaterialDarker => write!(f, "material-darker"),
            Theme::SolarizedDark => write!(f, "solarized-dark"),
            Theme::SolarizedLight => write!(f, "solarized-light"),
            Theme::GithubDark => write!(f, "github-dark"),
            Theme::GithubLight => write!(f, "github-light"),
            Theme::GithubDarkDimmed => write!(f, "github-dark-dimmed"),
            Theme::GithubDarkHighContrast => write!(f, "github-dark-high-contrast"),
            Theme::GithubLightHighContrast => write!(f, "github-light-high-contrast"),
            Theme::AyuDark => write!(f, "ayu-dark"),
            Theme::AyuMirage => write!(f, "ayu-mirage"),
            Theme::AyuLight => write!(f, "ayu-light"),
            Theme::NightOwl => write!(f, "night-owl"),
            Theme::NightOwlLight => write!(f, "night-owl-light"),
            Theme::Moonlight => write!(f, "moonlight"),
            Theme::Projectious => write!(f, "projectious"),
            // New themes
            Theme::Andromeeda => write!(f, "andromeeda"),
            Theme::AuroraX => write!(f, "aurora-x"),
            Theme::EverforestDark => write!(f, "everforest-dark"),
            Theme::EverforestLight => write!(f, "everforest-light"),
            Theme::Houston => write!(f, "houston"),
            Theme::KanagawaWave => write!(f, "kanagawa-wave"),
            Theme::KanagawaDragon => write!(f, "kanagawa-dragon"),
            Theme::KanagawaLotus => write!(f, "kanagawa-lotus"),
            Theme::Laserwave => write!(f, "laserwave"),
            Theme::MinDark => write!(f, "min-dark"),
            Theme::MinLight => write!(f, "min-light"),
            Theme::Monokai => write!(f, "monokai"),
            Theme::OneDarkPro => write!(f, "one-dark-pro"),
            Theme::OneLight => write!(f, "one-light"),
            Theme::Plastic => write!(f, "plastic"),
            Theme::Poimandres => write!(f, "poimandres"),
            Theme::Red => write!(f, "red"),
            Theme::SlackDark => write!(f, "slack-dark"),
            Theme::SlackOchin => write!(f, "slack-ochin"),
            Theme::SnazzyLight => write!(f, "snazzy-light"),
            Theme::Synthwave84 => write!(f, "synthwave-84"),
            Theme::Vesper => write!(f, "vesper"),
            Theme::VitesseDark => write!(f, "vitesse-dark"),
            Theme::VitesseLight => write!(f, "vitesse-light"),
            Theme::VitesseBlack => write!(f, "vitesse-black"),
            Theme::VsCodeDarkPlus => write!(f, "vscode-dark-plus"),
            Theme::VsCodeLightPlus => write!(f, "vscode-light-plus"),
        }
    }
}

/// Starship prompt presets.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum StarshipPreset {
    #[default]
    Default, // Clean, informative — dir, git, language, duration
    Plain,    // ASCII only — no Nerd Font needed
    Minimal,  // Just directory + git branch
    NerdFont, // Full Nerd Font symbols
    Pastel,   // Soft powerline segments
    #[serde(rename = "powerline-pastel", alias = "pastel-powerline")]
    #[value(name = "powerline-pastel", alias = "pastel-powerline")]
    PastelPowerline, // One-line pastel powerline preset
    Bracketed, // [segments] in brackets
    Arrow,    // Powerline-style chevron/arrow segments (airline-style)
}

impl std::fmt::Display for StarshipPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StarshipPreset::Default => write!(f, "default"),
            StarshipPreset::Plain => write!(f, "plain"),
            StarshipPreset::Minimal => write!(f, "minimal"),
            StarshipPreset::NerdFont => write!(f, "nerd-font"),
            StarshipPreset::Pastel => write!(f, "pastel"),
            StarshipPreset::PastelPowerline => write!(f, "powerline-pastel"),
            StarshipPreset::Bracketed => write!(f, "bracketed"),
            StarshipPreset::Arrow => write!(f, "arrow"),
        }
    }
}

fn default_prompt() -> StarshipPreset {
    StarshipPreset::default()
}

/// Default tmux workspace layout for `aibox up`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum ConfigLayout {
    /// Work window with yazi, first harness, shell; optional AI/lazygit/shell windows
    #[default]
    Dev,
    /// One fullscreen window for files and each harness
    Focus,
    /// Work window with yazi and shell; AI harnesses in a separate window
    Cowork,
    /// Work window with yazi and first harness; secondary harnesses in AI window
    Ai,
}

impl std::fmt::Display for ConfigLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLayout::Dev => write!(f, "dev"),
            ConfigLayout::Focus => write!(f, "focus"),
            ConfigLayout::Cowork => write!(f, "cowork"),
            ConfigLayout::Ai => write!(f, "ai"),
        }
    }
}

fn default_layout() -> ConfigLayout {
    ConfigLayout::default()
}

/// tmux status-line presentation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum TmuxStatusMode {
    /// aibox themed status line with tmux plugin hooks.
    #[default]
    #[serde(
        alias = "powerline",
        alias = "enabled",
        alias = "shell",
        alias = "sidecar",
        alias = "native"
    )]
    #[value(
        alias = "powerline",
        alias = "enabled",
        alias = "shell",
        alias = "sidecar",
        alias = "native"
    )]
    Extended,
    /// Minimal tmux-native status text.
    Plain,
    /// Disable the tmux status line.
    #[serde(alias = "hidden")]
    #[value(alias = "hidden")]
    Disabled,
}

impl std::fmt::Display for TmuxStatusMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TmuxStatusMode::Extended => write!(f, "extended"),
            TmuxStatusMode::Plain => write!(f, "plain"),
            TmuxStatusMode::Disabled => write!(f, "disabled"),
        }
    }
}

fn default_tmux_status_mode() -> TmuxStatusMode {
    TmuxStatusMode::default()
}

fn bool_true() -> bool {
    true
}

fn bool_false() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TmuxStatusAiboxMetricsSection {
    #[serde(default = "bool_true")]
    pub log: bool,
    #[serde(default = "bool_true")]
    pub oom: bool,
    #[serde(default = "bool_true")]
    pub proc: bool,
    #[serde(default = "bool_true")]
    pub ai: bool,
    #[serde(default = "bool_true")]
    pub mcp: bool,
    #[serde(default = "bool_true")]
    pub mig: bool,
}

impl Default for TmuxStatusAiboxMetricsSection {
    fn default() -> Self {
        Self {
            log: true,
            oom: true,
            proc: true,
            ai: true,
            mcp: true,
            mig: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TmuxStatusElementsSection {
    #[serde(default = "bool_true")]
    pub hostname: bool,
    #[serde(default = "bool_true")]
    pub external_ip: bool,
    #[serde(default = "bool_true")]
    pub ssh: bool,
    #[serde(default = "bool_true")]
    pub uptime: bool,
    #[serde(default = "bool_true")]
    pub weather: bool,
    #[serde(default = "bool_true")]
    pub datetime: bool,
    // forge is the default git-aware segment; it auto-detects provider and
    // supersedes the upstream `git` + `github` plugins for default aibox
    // installs.  Users who want to pin to the upstream behaviour can disable
    // forge and enable git/github explicitly in their aibox.toml.
    #[serde(default = "bool_true")]
    pub forge: bool,
    // git and github are kept in the schema for opt-in / legacy configs.
    // They default to false because forge covers their combined functionality.
    // Migration note: if your aibox.toml has git=true or github=true alongside
    // forge=true you will see doubled branch info; set forge=false to opt out.
    #[serde(default = "bool_false")]
    pub git: bool,
    #[serde(default = "bool_false")]
    pub github: bool,
    #[serde(default = "bool_true")]
    pub kubernetes: bool,
    #[serde(default = "bool_true")]
    pub terraform: bool,
    #[serde(default = "bool_true")]
    pub cloud: bool,
    #[serde(default = "bool_false")]
    pub cloudstatus: bool,
    #[serde(default = "bool_true")]
    pub cpu: bool,
    #[serde(default = "bool_true")]
    pub loadavg: bool,
    #[serde(default = "bool_true")]
    pub mem: bool,
    #[serde(default = "bool_true")]
    pub swap: bool,
    #[serde(default = "bool_true")]
    pub disk: bool,
    #[serde(default = "bool_true")]
    pub gpu: bool,
    #[serde(default = "bool_true")]
    pub netspeed: bool,
    #[serde(default = "bool_true")]
    pub ping: bool,
    #[serde(default = "bool_true")]
    pub aibox: bool,
    #[serde(default)]
    pub aibox_metrics: TmuxStatusAiboxMetricsSection,
}

impl Default for TmuxStatusElementsSection {
    fn default() -> Self {
        Self {
            hostname: true,
            external_ip: true,
            ssh: true,
            uptime: true,
            weather: true,
            datetime: true,
            forge: true,
            git: false,
            github: false,
            kubernetes: true,
            terraform: true,
            cloud: true,
            cloudstatus: false,
            cpu: true,
            loadavg: true,
            mem: true,
            swap: true,
            disk: true,
            gpu: true,
            netspeed: true,
            ping: true,
            aibox: true,
            aibox_metrics: TmuxStatusAiboxMetricsSection::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TmuxStatusLayoutSection {
    #[serde(default)]
    pub line1_left: Option<Vec<String>>,
    #[serde(default)]
    pub line1_right: Option<Vec<String>>,
    #[serde(default)]
    pub line2_left: Option<Vec<String>>,
    #[serde(default)]
    pub line2_right: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TmuxStatusLabelsSection {
    #[serde(default = "default_status_label_aibox_log")]
    pub aibox_log: String,
    #[serde(default = "default_status_label_aibox_oom")]
    pub aibox_oom: String,
    #[serde(default = "default_status_label_aibox_proc")]
    pub aibox_proc: String,
    #[serde(default = "default_status_label_aibox_ai")]
    pub aibox_ai: String,
    #[serde(default = "default_status_label_aibox_mcp")]
    pub aibox_mcp: String,
    #[serde(default = "default_status_label_aibox_mig")]
    pub aibox_mig: String,
    #[serde(default = "default_status_label_kubernetes")]
    pub kubernetes: String,
    #[serde(default = "default_status_label_cloud")]
    pub cloud: String,
    #[serde(default = "default_status_label_cloud_aws")]
    pub cloud_aws: String,
    #[serde(default = "default_status_label_cloud_gcp")]
    pub cloud_gcp: String,
    #[serde(default = "default_status_label_cloud_azure")]
    pub cloud_azure: String,
    #[serde(default = "default_status_label_cloud_multi")]
    pub cloud_multi: String,
    #[serde(default = "default_status_label_uptime")]
    pub uptime: String,
    #[serde(default = "default_status_label_netspeed")]
    pub netspeed: String,
    #[serde(default = "default_status_label_netspeed_download")]
    pub netspeed_download: String,
    #[serde(default = "default_status_label_netspeed_upload")]
    pub netspeed_upload: String,
}

fn default_status_label_aibox_log() -> String {
    "\u{f15ab}".to_string()
}

fn default_status_label_aibox_oom() -> String {
    "\u{f035b}\u{f068c}".to_string()
}

fn default_status_label_aibox_proc() -> String {
    "\u{f029a}".to_string()
}

fn default_status_label_aibox_ai() -> String {
    "\u{f167a}".to_string()
}

fn default_status_label_aibox_mcp() -> String {
    "\u{f0339}".to_string()
}

fn default_status_label_aibox_mig() -> String {
    "\u{f06b0}".to_string()
}

fn default_status_label_kubernetes() -> String {
    "\u{f10fe}".to_string()
}

fn default_status_label_cloud() -> String {
    "\u{f0163}".to_string()
}

fn default_status_label_cloud_aws() -> String {
    "\u{f0e0f}".to_string()
}

fn default_status_label_cloud_gcp() -> String {
    "\u{f0b20}".to_string()
}

fn default_status_label_cloud_azure() -> String {
    "\u{f0805}".to_string()
}

fn default_status_label_cloud_multi() -> String {
    "\u{f0164}".to_string()
}

fn default_status_label_uptime() -> String {
    "\u{f254}".to_string()
}

fn default_status_label_netspeed() -> String {
    "\u{f0b5}".to_string()
}

fn default_status_label_netspeed_download() -> String {
    "\u{f01da}".to_string()
}

fn default_status_label_netspeed_upload() -> String {
    "\u{f0552}".to_string()
}

impl Default for TmuxStatusLabelsSection {
    fn default() -> Self {
        Self {
            aibox_log: default_status_label_aibox_log(),
            aibox_oom: default_status_label_aibox_oom(),
            aibox_proc: default_status_label_aibox_proc(),
            aibox_ai: default_status_label_aibox_ai(),
            aibox_mcp: default_status_label_aibox_mcp(),
            aibox_mig: default_status_label_aibox_mig(),
            kubernetes: default_status_label_kubernetes(),
            cloud: default_status_label_cloud(),
            cloud_aws: default_status_label_cloud_aws(),
            cloud_gcp: default_status_label_cloud_gcp(),
            cloud_azure: default_status_label_cloud_azure(),
            cloud_multi: default_status_label_cloud_multi(),
            uptime: default_status_label_uptime(),
            netspeed: default_status_label_netspeed(),
            netspeed_download: default_status_label_netspeed_download(),
            netspeed_upload: default_status_label_netspeed_upload(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TmuxStatusSeparatorStyle {
    Normal,
    Rounded,
    Slant,
    Slantup,
    Trapezoid,
    Flame,
    Pixel,
    Honeycomb,
    None,
}

impl std::fmt::Display for TmuxStatusSeparatorStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TmuxStatusSeparatorStyle::Normal => write!(f, "normal"),
            TmuxStatusSeparatorStyle::Rounded => write!(f, "rounded"),
            TmuxStatusSeparatorStyle::Slant => write!(f, "slant"),
            TmuxStatusSeparatorStyle::Slantup => write!(f, "slantup"),
            TmuxStatusSeparatorStyle::Trapezoid => write!(f, "trapezoid"),
            TmuxStatusSeparatorStyle::Flame => write!(f, "flame"),
            TmuxStatusSeparatorStyle::Pixel => write!(f, "pixel"),
            TmuxStatusSeparatorStyle::Honeycomb => write!(f, "honeycomb"),
            TmuxStatusSeparatorStyle::None => write!(f, "none"),
        }
    }
}

fn default_tmux_status_separator_style() -> TmuxStatusSeparatorStyle {
    TmuxStatusSeparatorStyle::Rounded
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TmuxStatusElementsSpacing {
    False,
    True,
    Both,
    Windows,
    Plugins,
}

impl std::fmt::Display for TmuxStatusElementsSpacing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TmuxStatusElementsSpacing::False => write!(f, "false"),
            TmuxStatusElementsSpacing::True => write!(f, "true"),
            TmuxStatusElementsSpacing::Both => write!(f, "both"),
            TmuxStatusElementsSpacing::Windows => write!(f, "windows"),
            TmuxStatusElementsSpacing::Plugins => write!(f, "plugins"),
        }
    }
}

fn default_tmux_status_elements_spacing() -> TmuxStatusElementsSpacing {
    TmuxStatusElementsSpacing::Both
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TmuxStatusSeparatorsSection {
    #[serde(default = "default_tmux_status_separator_style")]
    pub style: TmuxStatusSeparatorStyle,
    #[serde(default = "default_tmux_status_separator_style")]
    pub edge_style: TmuxStatusSeparatorStyle,
    #[serde(default = "default_tmux_status_elements_spacing")]
    pub elements_spacing: TmuxStatusElementsSpacing,
}

impl Default for TmuxStatusSeparatorsSection {
    fn default() -> Self {
        Self {
            style: default_tmux_status_separator_style(),
            edge_style: default_tmux_status_separator_style(),
            elements_spacing: default_tmux_status_elements_spacing(),
        }
    }
}

fn default_tmux_status_refresh_interval_seconds() -> u32 {
    15
}

fn default_tmux_status_aibox_metrics_cache_ttl_seconds() -> u32 {
    30
}

fn default_tmux_status_netspeed_cache_ttl_seconds() -> u32 {
    10
}

fn default_tmux_status_kubernetes_cache_ttl_seconds() -> u32 {
    120
}

fn default_tmux_status_cloud_cache_ttl_seconds() -> u32 {
    120
}

fn default_tmux_status_github_cache_ttl_seconds() -> u32 {
    120
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TmuxStatusRefreshSection {
    #[serde(default = "default_tmux_status_refresh_interval_seconds")]
    pub interval_seconds: u32,
    #[serde(default = "default_tmux_status_aibox_metrics_cache_ttl_seconds")]
    pub aibox_metrics_cache_ttl_seconds: u32,
    #[serde(default = "default_tmux_status_netspeed_cache_ttl_seconds")]
    pub netspeed_cache_ttl_seconds: u32,
    #[serde(default = "default_tmux_status_kubernetes_cache_ttl_seconds")]
    pub kubernetes_cache_ttl_seconds: u32,
    #[serde(default = "default_tmux_status_cloud_cache_ttl_seconds")]
    pub cloud_cache_ttl_seconds: u32,
    #[serde(default = "default_tmux_status_github_cache_ttl_seconds")]
    pub github_cache_ttl_seconds: u32,
}

impl Default for TmuxStatusRefreshSection {
    fn default() -> Self {
        Self {
            interval_seconds: default_tmux_status_refresh_interval_seconds(),
            aibox_metrics_cache_ttl_seconds: default_tmux_status_aibox_metrics_cache_ttl_seconds(),
            netspeed_cache_ttl_seconds: default_tmux_status_netspeed_cache_ttl_seconds(),
            kubernetes_cache_ttl_seconds: default_tmux_status_kubernetes_cache_ttl_seconds(),
            cloud_cache_ttl_seconds: default_tmux_status_cloud_cache_ttl_seconds(),
            github_cache_ttl_seconds: default_tmux_status_github_cache_ttl_seconds(),
        }
    }
}

fn default_tmux_status_forge_github_hosts() -> Vec<String> {
    vec!["github.com".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TmuxStatusForgeSection {
    #[serde(default = "default_tmux_status_forge_github_hosts")]
    pub github_hosts: Vec<String>,
}

impl Default for TmuxStatusForgeSection {
    fn default() -> Self {
        Self {
            github_hosts: default_tmux_status_forge_github_hosts(),
        }
    }
}

fn default_model_provider_status_cache_ttl_seconds() -> u32 {
    300
}

fn default_model_provider_status_timeout_seconds() -> u32 {
    3
}

fn default_model_provider_status_checks() -> Vec<String> {
    vec![
        "overall".to_string(),
        "models".to_string(),
        "harness".to_string(),
    ]
}

fn default_quota_window() -> String {
    "tokens".to_string()
}

/// CLI binary names whose running processes count as agents for a given
/// provider. Used by Phase 1 of the model-provider status segment to render
/// per-provider agent counts (`ANT ×2` etc.).
pub fn default_agent_binaries_for(provider: &str) -> Vec<String> {
    let s = |v: &[&str]| v.iter().map(|x| (*x).to_string()).collect();
    match provider {
        "anthropic" => s(&["claude"]),
        "openai" => s(&["codex"]),
        "google" => s(&["gemini"]),
        "mistral" => s(&["mistral"]),
        "deepseek" => s(&["deepseek"]),
        "cohere" => s(&["cohere"]),
        "xai" => s(&["grok", "xai"]),
        "microsoft" => s(&["copilot"]),
        _ => Vec::new(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TmuxStatusModelProviderStatusProvider {
    pub provider: String,
    pub label: String,
    #[serde(default = "default_model_provider_status_checks")]
    pub checks: Vec<String>,
    #[serde(default)]
    pub status_url: Option<String>,
    #[serde(default)]
    pub overall_components: Vec<String>,
    #[serde(default)]
    pub model_components: Vec<String>,
    #[serde(default)]
    pub harness_components: Vec<String>,

    // ── Phase 1: local agent count ────────────────────────────────────────
    /// Show local agent count (`× N`) read from
    /// `aibox-status --plugin-json`'s `ai_agents_breakdown`. Free + always-on
    /// by default; set false to suppress.
    #[serde(default = "bool_true")]
    pub show_agent_count: bool,

    /// CLI binary names whose running processes count as an agent for this
    /// provider. If left empty, defaults are populated by
    /// `default_agent_binaries_for(provider)`.
    #[serde(default)]
    pub agent_binaries: Vec<String>,

    // ── Phase 2: rate-limit quota polling (opt-in) ────────────────────────
    /// Poll the provider's API to show rate-limit % remaining. Opt-in
    /// because it sends a billable 1-token request every cache_ttl seconds
    /// (≈ $0.03/day per provider at default 300 s polling).
    #[serde(default = "bool_false")]
    pub show_quota: bool,

    /// Which rate-limit dimension to show: "tokens" or "requests". Tokens
    /// is the more useful signal for most providers.
    #[serde(default = "default_quota_window")]
    pub quota_window: String,

    /// Env var holding the API key for quota polling. When unset, falls
    /// back to the provider's standard key env (`ANTHROPIC_API_KEY` etc.).
    #[serde(default)]
    pub quota_api_key_env: Option<String>,

    // ── Phase 3: admin usage rollup (deep opt-in) ─────────────────────────
    /// Show this month's usage / soft cap drawn from the provider's admin
    /// API (Anthropic `/v1/organizations/usage_report/messages`, OpenAI
    /// `/v1/organization/usage/completions`). Requires an admin key with
    /// usage-read scope and the section-level
    /// `admin_usage_enabled = true` ack.
    #[serde(default = "bool_false")]
    pub show_admin_usage: bool,

    /// Env var holding the admin API key. Required when `show_admin_usage`
    /// is true. Defaults to `<PROVIDER>_ADMIN_KEY` if unset.
    #[serde(default)]
    pub admin_api_key_env: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TmuxStatusModelProviderStatusSection {
    #[serde(default = "bool_false")]
    pub enabled: bool,
    #[serde(default = "default_model_provider_status_cache_ttl_seconds")]
    pub cache_ttl_seconds: u32,
    #[serde(default = "default_model_provider_status_timeout_seconds")]
    pub timeout_seconds: u32,
    /// Legacy: emit a ✓ glyph next to healthy providers. Defaults to false
    /// now that the custom PowerKit theme encodes ok/warning/error via the
    /// chevron color. Flip on if you prefer the redundant glyph.
    #[serde(default = "bool_false")]
    pub show_glyph_when_ok: bool,
    /// Preserved for backward compatibility — older configs read `show_ok`.
    /// Treated as equivalent to `show_glyph_when_ok` when present.
    #[serde(default = "bool_false", alias = "show-ok")]
    pub show_ok: bool,
    /// Section-level ack required before any provider's
    /// `show_admin_usage = true` takes effect. Acts as a guard against
    /// accidentally enabling admin-key polling for a billing surface.
    #[serde(default = "bool_false")]
    pub admin_usage_enabled: bool,
    #[serde(default = "default_model_provider_status_providers")]
    pub providers: Vec<TmuxStatusModelProviderStatusProvider>,
}

pub fn default_model_provider_status_providers() -> Vec<TmuxStatusModelProviderStatusProvider> {
    fn provider(
        provider: &str,
        label: &str,
        status_url: Option<&str>,
        checks: &[&str],
        model_components: &[&str],
        harness_components: &[&str],
    ) -> TmuxStatusModelProviderStatusProvider {
        TmuxStatusModelProviderStatusProvider {
            provider: provider.to_string(),
            label: label.to_string(),
            checks: checks.iter().map(|s| (*s).to_string()).collect(),
            status_url: status_url.map(str::to_string),
            overall_components: Vec::new(),
            model_components: model_components.iter().map(|s| (*s).to_string()).collect(),
            harness_components: harness_components
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            show_agent_count: true,
            agent_binaries: default_agent_binaries_for(provider),
            show_quota: false,
            quota_window: default_quota_window(),
            quota_api_key_env: None,
            show_admin_usage: false,
            admin_api_key_env: None,
        }
    }

    vec![
        provider(
            "openai",
            "OAI",
            Some("https://status.openai.com/api/v2/summary.json"),
            &["overall", "models", "harness"],
            &[
                "Responses",
                "Chat Completions",
                "Embeddings",
                "Realtime",
                "Images",
            ],
            &["CLI", "Codex API", "Codex Web"],
        ),
        provider(
            "anthropic",
            "ANT",
            Some("https://status.claude.com/api/v2/summary.json"),
            &["overall", "models", "harness"],
            &["Claude API"],
            &["Claude Code"],
        ),
        provider(
            "google",
            "GOOG",
            Some("https://status.cloud.google.com/incidents.json"),
            &["overall", "models"],
            &[],
            &[],
        ),
        provider(
            "mistral",
            "MST",
            Some("https://status.mistral.ai/api/v2/summary.json"),
            &["overall", "models"],
            &[],
            &[],
        ),
        provider(
            "deepseek",
            "DS",
            Some("https://status.deepseek.com/api/v2/summary.json"),
            &["overall", "models"],
            &[],
            &[],
        ),
        provider(
            "cohere",
            "COH",
            Some("https://status.cohere.com/api/v2/summary.json"),
            &["overall", "models"],
            &[],
            &[],
        ),
        provider("xai", "XAI", None, &["overall", "models"], &[], &[]),
        provider("alibaba", "QWN", None, &["overall", "models"], &[], &[]),
        provider("aws", "AWS", None, &["overall", "models"], &[], &[]),
        provider("meta", "META", None, &["overall", "models"], &[], &[]),
        provider("microsoft", "MS", None, &["overall", "models"], &[], &[]),
        provider("minimax", "MM", None, &["overall", "models"], &[], &[]),
        provider("moonshot", "KIMI", None, &["overall", "models"], &[], &[]),
        provider("nvidia", "NV", None, &["overall", "models"], &[], &[]),
        provider("xiaomi", "MI", None, &["overall", "models"], &[], &[]),
        provider("zai", "ZAI", None, &["overall", "models"], &[], &[]),
    ]
}

impl Default for TmuxStatusModelProviderStatusSection {
    fn default() -> Self {
        Self {
            enabled: false,
            cache_ttl_seconds: default_model_provider_status_cache_ttl_seconds(),
            timeout_seconds: default_model_provider_status_timeout_seconds(),
            show_glyph_when_ok: false,
            show_ok: false,
            admin_usage_enabled: false,
            providers: default_model_provider_status_providers(),
        }
    }
}

/// [customization.tmux.status] section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TmuxStatusSection {
    #[serde(default = "default_tmux_status_mode")]
    pub mode: TmuxStatusMode,
    #[serde(default)]
    pub elements: TmuxStatusElementsSection,
    #[serde(default)]
    pub layout: TmuxStatusLayoutSection,
    #[serde(default)]
    pub labels: TmuxStatusLabelsSection,
    #[serde(default)]
    pub separators: TmuxStatusSeparatorsSection,
    #[serde(default)]
    pub refresh: TmuxStatusRefreshSection,
    #[serde(default)]
    pub forge: TmuxStatusForgeSection,
    #[serde(default)]
    pub model_providers: TmuxStatusModelProviderStatusSection,
}

impl Default for TmuxStatusSection {
    fn default() -> Self {
        Self {
            mode: default_tmux_status_mode(),
            elements: TmuxStatusElementsSection::default(),
            layout: TmuxStatusLayoutSection::default(),
            labels: TmuxStatusLabelsSection::default(),
            separators: TmuxStatusSeparatorsSection::default(),
            refresh: TmuxStatusRefreshSection::default(),
            forge: TmuxStatusForgeSection::default(),
            model_providers: TmuxStatusModelProviderStatusSection::default(),
        }
    }
}

fn default_tmux_prefix() -> String {
    "C-g".to_string()
}

/// Resolve the tmux session name from a project name and optional working directory.
///
/// Resolution order:
/// 1. `project_name` if non-empty after sanitization.
/// 2. Basename of `cwd` if provided and non-empty after sanitization.
/// 3. Literal `"aibox"` fallback.
///
/// Sanitization: strips or replaces characters illegal in tmux session names
/// (`:` and `.` are replaced with `-`; leading/trailing `-` are trimmed;
/// any run of consecutive `-` is collapsed to one).
pub fn resolve_tmux_session_name(project_name: &str, cwd: Option<&std::path::Path>) -> String {
    sanitize_tmux_session_name(project_name)
        .or_else(|| {
            cwd.and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .and_then(sanitize_tmux_session_name)
        })
        .unwrap_or_else(|| "aibox".to_string())
}

/// Sanitize a candidate tmux session name.
///
/// Returns `Some(name)` if the result is non-empty, `None` otherwise.
/// Rules (matching tmux restrictions):
/// - Replace `:` and `.` with `-`.
/// - Strip any character that is not ASCII alphanumeric, `-`, or `_`.
/// - Collapse consecutive `-` into one.
/// - Trim leading and trailing `-`.
fn sanitize_tmux_session_name(s: &str) -> Option<String> {
    let replaced: String = s
        .chars()
        .map(|c| match c {
            ':' | '.' => '-',
            _ => c,
        })
        .collect();
    let filtered: String = replaced
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    // Collapse consecutive dashes.
    let mut collapsed = String::with_capacity(filtered.len());
    let mut prev_dash = false;
    for c in filtered.chars() {
        if c == '-' {
            if !prev_dash {
                collapsed.push(c);
            }
            prev_dash = true;
        } else {
            collapsed.push(c);
            prev_dash = false;
        }
    }
    let trimmed = collapsed.trim_matches('-').to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Serde sentinel: empty string signals "not explicitly set; resolve at load time".
fn default_tmux_session_name() -> String {
    String::new()
}

/// [customization.tmux] section — tmux runtime presentation and startup.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TmuxSection {
    /// Optional tmux-specific layout override. When absent, `[customization].layout`
    /// remains the source of truth for the default workspace layout.
    #[serde(default)]
    pub layout: Option<ConfigLayout>,
    #[serde(default = "default_tmux_prefix")]
    pub prefix: String,
    /// Tmux session name. Empty string means "derive from project name at load time".
    /// After `AiboxConfig::migrate_legacy_sections()` this is always non-empty.
    #[serde(default = "default_tmux_session_name")]
    pub session_name: String,
    #[serde(default)]
    pub status: TmuxStatusSection,
    #[serde(default)]
    pub layout_switch: TmuxLayoutSwitchSection,
    #[serde(default)]
    pub theme_switch: TmuxThemeSwitchSection,
}

impl Default for TmuxSection {
    fn default() -> Self {
        Self {
            layout: None,
            prefix: default_tmux_prefix(),
            session_name: default_tmux_session_name(),
            status: TmuxStatusSection::default(),
            layout_switch: TmuxLayoutSwitchSection::default(),
            theme_switch: TmuxThemeSwitchSection::default(),
        }
    }
}

fn default_layout_switch_prefix_key() -> String {
    "L".to_string()
}

fn default_layout_switch_style() -> String {
    "menu".to_string()
}

fn default_theme_switch_prefix_key() -> String {
    "T".to_string()
}

fn default_theme_switch_themes() -> Vec<String> {
    vec![
        "gruvbox-dark".to_string(),
        "catppuccin-mocha".to_string(),
        "tokyo-night".to_string(),
        "dracula".to_string(),
        "projectious".to_string(),
    ]
}

/// `customization.tmux.layout_switch` — live layout chooser keybinding.
///
/// Always destructive (kills panes when rebuilding windows). The `confirm`
/// dialog is the default safety net. Set `enabled = false` to omit the
/// binding entirely.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TmuxLayoutSwitchSection {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    #[serde(default = "default_layout_switch_prefix_key")]
    pub prefix_key: String,
    /// `menu` (display-menu, more discoverable) or `table`
    /// (switch-client -T layouts; two-key chord, less screen real estate).
    #[serde(default = "default_layout_switch_style")]
    pub style: String,
    /// Show a "this will kill <apps>" confirmation menu before rebuilding.
    /// Default on; layout swap is always destructive.
    #[serde(default = "bool_true")]
    pub confirm: bool,
}

impl Default for TmuxLayoutSwitchSection {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix_key: default_layout_switch_prefix_key(),
            style: default_layout_switch_style(),
            confirm: true,
        }
    }
}

/// `customization.tmux.theme_switch` — live theme chooser keybinding.
///
/// Tier 1 (light) is non-destructive; tier 2 (`Heavy: restart TUIs`) is
/// gated by `confirm_restart_tuis`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TmuxThemeSwitchSection {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    #[serde(default = "default_theme_switch_prefix_key")]
    pub prefix_key: String,
    /// Themes presented in the chooser menu. Defaults to a curated short
    /// list; users can override with the full 29-theme roster if desired.
    #[serde(default = "default_theme_switch_themes")]
    pub themes: Vec<String>,
    /// Include a "Toggle light/dark" entry that flips `customization.mode`
    /// without changing the theme family.
    #[serde(default = "bool_true")]
    pub include_mode_toggle: bool,
    /// Show a "restart these TUIs" confirmation menu before tier 2 swap.
    #[serde(default = "bool_true")]
    pub confirm_restart_tuis: bool,
}

impl Default for TmuxThemeSwitchSection {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix_key: default_theme_switch_prefix_key(),
            themes: default_theme_switch_themes(),
            include_mode_toggle: true,
            confirm_restart_tuis: true,
        }
    }
}

/// [customization] section — color theme, shell prompt, and tmux layout.
///
/// Deserialization is implemented manually (see `impl<'de> Deserialize`) to
/// support both the new family-form (`theme = "ayu"`) and legacy concrete names
/// (`theme = "ayu-dark"`). When a legacy concrete name is detected the
/// `legacy_theme` sidecar is populated and `resolved_theme()` returns it
/// directly (locked, no auto-flipping).
#[derive(Debug, Clone)]
pub struct CustomizationSection {
    /// Theme family. Set via custom deserializer (see below).
    pub theme: ThemeFamily,
    pub mode: ThemeMode,
    /// Optional alternate variant override (per-family). Validated at resolve
    /// time; unknown values fall through to the family default.
    pub variant: Option<String>,
    pub prompt: StarshipPreset,
    pub layout: ConfigLayout,
    pub tmux: TmuxSection,
    /// Populated by the deserializer when the user supplied a legacy concrete
    /// theme name (e.g. `"ayu-dark"`). Forces `resolved_theme()` to return this
    /// locked concrete theme rather than running family/mode/variant resolution.
    /// Skipped during serialization — callers that emit TOML (container.rs)
    /// always use the family form.
    pub legacy_theme: Option<Theme>,
}

impl Serialize for CustomizationSection {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        // Always serialize as the family form; legacy_theme is deliberately omitted.
        let field_count = 5 + self.variant.is_some() as usize;
        let mut s = serializer.serialize_struct("CustomizationSection", field_count)?;
        s.serialize_field("theme", &self.theme)?;
        s.serialize_field("mode", &self.mode)?;
        if let Some(ref v) = self.variant {
            s.serialize_field("variant", v)?;
        }
        s.serialize_field("prompt", &self.prompt)?;
        s.serialize_field("layout", &self.layout)?;
        s.serialize_field("tmux", &self.tmux)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for CustomizationSection {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{MapAccess, Visitor};

        struct CustomizationVisitor;

        impl<'de> Visitor<'de> for CustomizationVisitor {
            type Value = CustomizationSection;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "a customization/appearance table")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                use serde::de::Error as DeError;

                let mut raw_theme: Option<String> = None;
                let mut mode: Option<ThemeMode> = None;
                let mut variant: Option<String> = None;
                let mut prompt: Option<StarshipPreset> = None;
                let mut layout: Option<ConfigLayout> = None;
                let mut tmux: Option<TmuxSection> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "theme" => {
                            raw_theme = Some(map.next_value()?);
                        }
                        "mode" => {
                            mode = Some(map.next_value()?);
                        }
                        "variant" => {
                            variant = Some(map.next_value()?);
                        }
                        "prompt" => {
                            prompt = Some(map.next_value()?);
                        }
                        "layout" => {
                            layout = Some(map.next_value()?);
                        }
                        "tmux" => {
                            tmux = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore unknown keys (forward compat).
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                // Resolve the raw theme string into (family, legacy_theme,
                // overridden mode, overridden variant). When a legacy concrete
                // name is supplied, mode and variant are derived from it so
                // that re-serialisation (e.g. via `--standardize-config`)
                // captures the user's full intent — otherwise alternate
                // variants like "ayu-mirage" or light themes like "ayu-light"
                // would silently degrade to the family default on rewrite.
                let (theme, legacy_theme, derived_mode, derived_variant) = match raw_theme {
                    None => (ThemeFamily::default(), None, None, None),
                    Some(ref s) => {
                        // Try new family form first.
                        use clap::ValueEnum as _;
                        if let Ok(family) = ThemeFamily::from_str(s, true) {
                            (family, None, None, None)
                        } else if let Ok(concrete) = Theme::from_str(s, true) {
                            // Legacy concrete name — lock it and derive the
                            // family + mode + variant so a standardize-config
                            // round-trip preserves the user's exact choice.
                            let family = family_of(&concrete);
                            let mode_for_concrete = match &concrete {
                                Theme::GruvboxLight
                                | Theme::CatppuccinLatte
                                | Theme::TokyoNightDay
                                | Theme::RosePineDawn
                                | Theme::MaterialLighter
                                | Theme::SolarizedLight
                                | Theme::GithubLight
                                | Theme::GithubLightHighContrast
                                | Theme::AyuLight
                                | Theme::NightOwlLight
                                | Theme::OneLight
                                | Theme::VitesseLight
                                | Theme::MinLight
                                | Theme::KanagawaLotus
                                | Theme::EverforestLight
                                | Theme::VsCodeLightPlus
                                | Theme::SlackOchin
                                | Theme::SnazzyLight => ThemeMode::Light,
                                _ => ThemeMode::Dark,
                            };
                            let variant_for_concrete = variant_name_of(&concrete);
                            // Emit the deprecation warning once on first parse.
                            let fam_str = family.to_string();
                            let mode_str = mode_for_concrete.to_string();
                            let mut hint = format!(
                                "theme = \"{s}\" is the legacy concrete form; \
                                 run `aibox apply --standardize-config` to rewrite as \
                                 theme = \"{fam_str}\", mode = \"{mode_str}\""
                            );
                            if let Some(v) = variant_for_concrete {
                                hint.push_str(&format!(", variant = \"{v}\""));
                            }
                            crate::output::warn(&hint);
                            (
                                family,
                                Some(concrete),
                                Some(mode_for_concrete),
                                variant_for_concrete.map(|v| v.to_string()),
                            )
                        } else {
                            return Err(DeError::custom(format!(
                                "unknown theme or theme family: \"{s}\""
                            )));
                        }
                    }
                };

                Ok(CustomizationSection {
                    theme,
                    // Legacy concrete names override the user's `mode` field
                    // so that a re-serialised file (after standardize-config)
                    // resolves to the same concrete theme. Without this
                    // override, a `theme = "ayu-light", mode = "auto"` config
                    // would round-trip to `theme = "ayu", mode = "auto"` and
                    // resolve to AyuDark via the auto fallback.
                    mode: derived_mode.unwrap_or_else(|| mode.unwrap_or_default()),
                    variant: derived_variant.or(variant),
                    prompt: prompt.unwrap_or_default(),
                    layout: layout.unwrap_or_else(default_layout),
                    tmux: tmux.unwrap_or_default(),
                    legacy_theme,
                })
            }
        }

        deserializer.deserialize_map(CustomizationVisitor)
    }
}

impl CustomizationSection {
    /// Resolve the concrete palette rendered into tool config files.
    ///
    /// When a legacy concrete theme name was supplied in aibox.toml, the
    /// `legacy_theme` lock is returned directly (no auto-flipping). Otherwise,
    /// family + mode + variant are resolved via the resolution matrix.
    pub fn resolved_theme(&self) -> Theme {
        self.resolved_theme_for_host_mode(detected_host_theme_mode())
    }

    pub(crate) fn resolved_theme_for_host_mode(&self, host_mode: Option<ThemeMode>) -> Theme {
        // Legacy concrete name: locked, never auto-flipped.
        if let Some(locked) = &self.legacy_theme {
            return locked.clone();
        }
        let effective_mode = match self.mode {
            ThemeMode::Auto => host_mode.unwrap_or(ThemeMode::Dark),
            ThemeMode::Light => ThemeMode::Light,
            ThemeMode::Dark => ThemeMode::Dark,
        };
        resolve_theme_from_family(&self.theme, effective_mode, self.variant.as_deref())
    }

    pub fn tmux_layout(&self) -> ConfigLayout {
        self.tmux
            .layout
            .clone()
            .unwrap_or_else(|| self.layout.clone())
    }
}

impl Default for CustomizationSection {
    fn default() -> Self {
        Self {
            theme: ThemeFamily::default(),
            mode: default_theme_mode(),
            variant: None,
            prompt: default_prompt(),
            layout: default_layout(),
            tmux: TmuxSection::default(),
            legacy_theme: None,
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

    /// processkit MCP serving selection. `auto` uses the gateway daemon when
    /// the installed processkit release ships it and falls back to separate
    /// per-skill MCP servers otherwise.
    #[serde(default)]
    pub gateway: McpGatewaySection,

    /// MCP permissions configuration: global allow/deny patterns and per-harness overrides.
    /// Controls which MCP tools are available to each harness via allow/deny lists.
    #[serde(default)]
    pub permissions: crate::mcp_registration::McpConfig,
}

/// Serving mode for processkit-managed MCP servers.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum McpGatewayMode {
    /// Prefer the processkit-gateway daemon when available, otherwise use the
    /// internal single-process fallback when installed, otherwise use separate
    /// servers.
    #[default]
    Auto,
    /// Always write one server per processkit skill.
    #[serde(rename = "separate", alias = "granular")]
    Granular,
    /// Spawn processkit-gateway directly as a stdio MCP server per harness.
    Stdio,
    /// Use a managed local HTTP daemon plus one stdio proxy per harness.
    #[serde(rename = "daemon", alias = "daemon-proxy")]
    DaemonProxy,
    /// Use the processkit-aggregate-mcp server — a single stdio process that imports
    /// all per-skill MCP servers in-process.  Eliminates the N-process startup cost
    /// that Codex CLI incurs when it eagerly spawns every configured stdio MCP server.
    /// Requires the `aggregate-mcp` processkit skill to be enabled.
    Aggregate,
    /// Like `Aggregate`, but defers per-skill module imports until the first tool call
    /// for each skill, reducing cold-start latency by ~1.58× on typical installations.
    /// Sets `PROCESSKIT_MCP_MODE=lazy_catalog` on the aggregate-mcp server.
    ///
    /// Requires processkit ≥ v0.26.0 and the `aggregate-mcp` processkit skill.
    /// `Auto` does **not** promote to this variant; opt in explicitly via
    /// `[mcp.gateway] mode = "lazy-aggregate"` in `aibox.toml`.
    LazyAggregate,
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
            lazy_catalog: true,
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

// ---------------------------------------------------------------------------
// [latex] section
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LatexEngine {
    #[default]
    Lualatex,
    Pdflatex,
    Xelatex,
    Tectonic,
}

impl std::fmt::Display for LatexEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Lualatex => "lualatex",
            Self::Pdflatex => "pdflatex",
            Self::Xelatex => "xelatex",
            Self::Tectonic => "tectonic",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LatexDocument {
    pub name: String,
    pub source: String,
    #[serde(default = "default_latex_output_dir")]
    pub output_dir: String,
}

fn default_latex_output_dir() -> String {
    ".latex-cache/output".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LatexPreviewSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_latex_preview_engine")]
    pub engine: String,
    #[serde(default = "default_latex_preview_bind")]
    pub bind: String,
    #[serde(default = "default_latex_preview_port")]
    pub port: u16,
    #[serde(default)]
    pub document: Option<String>,
    #[serde(default)]
    pub allow_public: bool,
}

fn default_latex_preview_engine() -> String {
    "embedpdf".to_string()
}

fn default_latex_preview_bind() -> String {
    "127.0.0.1".to_string()
}

fn default_latex_preview_port() -> u16 {
    8765
}

impl Default for LatexPreviewSection {
    fn default() -> Self {
        Self {
            enabled: false,
            engine: default_latex_preview_engine(),
            bind: default_latex_preview_bind(),
            port: default_latex_preview_port(),
            document: None,
            allow_public: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LatexSection {
    #[serde(default)]
    pub engine: LatexEngine,
    #[serde(default = "default_latex_cache_dir")]
    pub cache_dir: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub documents: Vec<LatexDocument>,
    #[serde(default)]
    pub preview: LatexPreviewSection,
}

fn default_latex_cache_dir() -> String {
    ".latex-cache".to_string()
}

impl Default for LatexSection {
    fn default() -> Self {
        Self {
            engine: LatexEngine::default(),
            cache_dir: default_latex_cache_dir(),
            options: Vec::new(),
            documents: Vec::new(),
            preview: LatexPreviewSection::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// [integrations] section
// ---------------------------------------------------------------------------

/// GitHub HTTPS credential helper mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum GithubCredentialHelper {
    /// Configure the helper when the GitHub CLI is part of the generated toolset.
    #[default]
    Auto,
    /// Always configure Git to ask `gh auth git-credential` for GitHub HTTPS remotes.
    Gh,
    /// Do not configure the managed GitHub credential helper.
    None,
}

impl std::fmt::Display for GithubCredentialHelper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Gh => write!(f, "gh"),
            Self::None => write!(f, "none"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GithubIntegrationSection {
    #[serde(default)]
    pub credential_helper: GithubCredentialHelper,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct IntegrationsSection {
    #[serde(default)]
    pub github: GithubIntegrationSection,
}

// ---------------------------------------------------------------------------
// [apply] section — knobs that govern `aibox apply` cleanup behaviour
// ---------------------------------------------------------------------------

/// `[apply]` section. Controls how aggressive `aibox apply` is when it
/// detects state on the host that no longer matches the config (e.g. an AI
/// harness that was previously enabled and is now disabled).
///
/// Defaults are conservative — when a harness is removed, aibox retains its
/// state and reports the available preserve-or-purge disposition. Set
/// `preserve_disabled_harness_state = true` to record the preserve decision,
/// or `purge_disabled_harness_state = true` to remove the state explicitly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ApplySection {
    /// When true, `aibox apply` hard-deletes per-harness state directories
    /// and MCP registration files for any harness that is no longer listed
    /// in `[ai].harnesses`. Defaults to false.
    #[serde(default)]
    pub purge_disabled_harness_state: bool,
    /// Persist the owner's decision to retain state for disabled AI harnesses.
    /// Defaults to false, which keeps the state and emits a one-time
    /// disposition reminder without creating a Migration entity.
    #[serde(default)]
    pub preserve_disabled_harness_state: bool,
}

// ---------------------------------------------------------------------------
// [security] section
// ---------------------------------------------------------------------------

/// `[security]` section of `aibox.toml`.
///
/// Controls explicit consent for security-sensitive runtime options.
///
/// ```toml
/// [security]
/// acknowledge_seccomp_unconfined = true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SecuritySection {
    /// Set to `true` to acknowledge that `seccomp=unconfined` is intentionally
    /// in use for Codex bubblewrap sandboxing.  Defaults to `false`.
    ///
    /// When `false` and the generated `docker-compose.yml` would emit
    /// `seccomp=unconfined`, `aibox apply` errors with a remediation pointer.
    #[serde(default)]
    pub acknowledge_seccomp_unconfined: bool,
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
    #[serde(default)]
    pub latex: LatexSection,
    #[serde(default)]
    pub integrations: IntegrationsSection,
    #[serde(default)]
    pub apply: ApplySection,

    #[serde(default)]
    pub security: SecuritySection,

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
    /// Whether processkit-backed project context should be installed and wired.
    pub fn processkit_enabled(&self) -> bool {
        self.context.mode == ContextMode::Processkit
    }

    /// Return the tmux session name used by generated runtime files.
    ///
    /// The configured `[customization.tmux].session_name` wins. When it is not
    /// set yet, use the project identity so freshly scaffolded configs get a
    /// stable, project-specific default.
    pub fn tmux_session_name(&self) -> String {
        if !self.customization.tmux.session_name.trim().is_empty() {
            return resolve_tmux_session_name(&self.customization.tmux.session_name, None);
        }

        let fallback = if !self.aibox.project_name.trim().is_empty() {
            &self.aibox.project_name
        } else if !self.metadata.name.trim().is_empty() {
            &self.metadata.name
        } else {
            &self.container.name
        };
        resolve_tmux_session_name(fallback, None)
    }

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
    pub(crate) fn migrate_legacy_sections(&mut self) {
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
        // BR-LEGACY-MUX-EXCISE (DEC-20260508_1515-SilentAsh, v0.25.6) hard-cut
        // the legacy multiplexer-status alias. Old configs that still carry
        // it now fail schema validation with a clear error pointing at the
        // [customization.tmux.status] replacement.
        self.sync_grouped_sections();
        self.sync_legacy_aibox_image_fields();
        self.normalize_legacy_cloudstatus_layout_default();
    }

    fn normalize_legacy_cloudstatus_layout_default(&mut self) {
        let layout = &mut self.customization.tmux.status.layout;
        let Some(line2_left) = layout.line2_left.as_mut() else {
            return;
        };
        if self.customization.tmux.status.elements.cloudstatus {
            return;
        }
        let legacy_default = [
            "git",
            "github",
            "kubernetes",
            "terraform",
            "cloud",
            "cloudstatus",
        ];
        if line2_left.iter().map(String::as_str).eq(legacy_default) {
            line2_left.retain(|entry| entry != "cloudstatus");
        }
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

        // Resolve tmux session name from project name when not explicitly set.
        if self.customization.tmux.session_name.is_empty() {
            self.customization.tmux.session_name =
                resolve_tmux_session_name(&self.aibox.project_name, None);
        }

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
            if let Some(managed) = crate::runtime_home::extra_volume_conflict(self, &vol.target) {
                bail!(
                    "container.extra_volumes target '{}' overlaps aibox-managed runtime home path '{}'. \
                     Use the generated .aibox-home tree for Yazi, tmux, shell, cache, and AI harness config instead of shadowing it with an extra volume.",
                    vol.target,
                    managed
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
                "latex",
                "integrations",
                "apply",
                "mcp",
                "security",
            ],
            &mut mismatches,
        );

        check_child_table(
            root,
            "apply",
            &[
                "purge_disabled_harness_state",
                "preserve_disabled_harness_state",
            ],
            &mut mismatches,
        );

        check_child_table(root, "integrations", &["github"], &mut mismatches);
        if let Some(integrations) = table_child(root, "integrations") {
            check_child_table(
                integrations,
                "github",
                &["credential_helper"],
                &mut mismatches,
            );
        }

        check_child_table(
            root,
            "security",
            &["acknowledge_seccomp_unconfined"],
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
            &["schema_version", "mode", "packages"],
            &mut mismatches,
        );
        check_child_table(root, "process", &["packages"], &mut mismatches);
        check_child_table(
            root,
            "ai",
            &[
                "harnesses",
                "harness_order",
                "model_providers",
                "execution",
                "harness",
                "providers",
                "agents",
                "mcp",
            ],
            &mut mismatches,
        );
        if let Some(ai) = table_child(root, "ai") {
            check_child_table(
                ai,
                "execution",
                &[
                    "filesystem",
                    "approval",
                    "network",
                    "claude",
                    "codex",
                    "gemini",
                    "aider",
                    "continue",
                    "cursor",
                    "copilot",
                    "opencode",
                    "hermes",
                    "mistral",
                ],
                &mut mismatches,
            );
            if let Some(execution) = table_child(ai, "execution") {
                for harness in AiHarness::all() {
                    check_child_table(
                        execution,
                        &harness.to_string(),
                        &["filesystem", "approval", "network"],
                        &mut mismatches,
                    );
                }
            }
            if let Some(harnesses) = table_child(ai, "harness") {
                for (harness, value) in harnesses {
                    if let Some(table) = value.as_table() {
                        check_unknown_keys(
                            &format!("[ai.harness.{harness}]"),
                            table,
                            &["enabled", "install", "version", "execution"],
                            &mut mismatches,
                        );
                        check_child_table(
                            table,
                            "execution",
                            &["filesystem", "approval", "network"],
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
        check_child_table(
            root,
            "latex",
            &["engine", "cache_dir", "options", "documents", "preview"],
            &mut mismatches,
        );
        if let Some(latex) = table_child(root, "latex") {
            check_child_table(
                latex,
                "preview",
                &[
                    "enabled",
                    "engine",
                    "bind",
                    "port",
                    "document",
                    "allow_public",
                ],
                &mut mismatches,
            );
            if let Some(documents) = latex.get("documents").and_then(toml::Value::as_array) {
                for (index, document) in documents.iter().enumerate() {
                    if let Some(table) = document.as_table() {
                        check_unknown_keys(
                            &format!("[[latex.documents]][{index}]"),
                            table,
                            &["name", "source", "output_dir"],
                            &mut mismatches,
                        );
                    }
                }
            }
        }
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

        // Validate published image version is valid semver (allow "latest" sentinel)
        if self.container.image.version != "latest" {
            semver::Version::parse(&self.container.image.version).with_context(|| {
                format!(
                    "Invalid container.image.release_version '{}': must be valid semver",
                    self.container.image.version
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
        if self.processkit_enabled() && self.context.packages.is_empty() {
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
        self.validate_latex()?;
        self.validate_tmux_status_layout()?;
        self.validate_tmux_status_labels()?;
        self.validate_tmux_status_refresh()?;
        self.validate_tmux_status_forge()?;
        self.validate_tmux_model_provider_status()?;

        Ok(())
    }

    fn validate_latex(&self) -> Result<()> {
        use std::net::IpAddr;

        fn safe_relative(field: &str, value: &str) -> Result<()> {
            let path = Path::new(value);
            if value.trim().is_empty() || path.is_absolute() {
                bail!("{field} must be a non-empty project-relative path");
            }
            if path
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
            {
                bail!("{field} must not contain '..'");
            }
            Ok(())
        }

        safe_relative("latex.cache_dir", &self.latex.cache_dir)?;
        let mut names = BTreeSet::new();
        for document in &self.latex.documents {
            if !is_safe_name(&document.name) {
                bail!(
                    "latex document name '{}' must contain only [a-zA-Z0-9_-]",
                    document.name
                );
            }
            if !names.insert(document.name.as_str()) {
                bail!("latex document name '{}' is duplicated", document.name);
            }
            safe_relative("latex.documents.source", &document.source)?;
            safe_relative("latex.documents.output_dir", &document.output_dir)?;
        }
        for option in &self.latex.options {
            if option.contains('\0') || option.contains('\n') || option.contains('\r') {
                bail!("latex.options entries must be single-line arguments");
            }
        }
        if self.latex.preview.engine != "embedpdf" {
            bail!(
                "latex.preview.engine '{}' is unsupported; expected 'embedpdf'",
                self.latex.preview.engine
            );
        }
        if self.latex.preview.port == 0 {
            bail!("latex.preview.port must be between 1 and 65535");
        }
        if let Some(name) = &self.latex.preview.document
            && !self
                .latex
                .documents
                .iter()
                .any(|document| document.name == *name)
        {
            bail!("latex.preview.document '{}' is not configured", name);
        }
        let bind: IpAddr = self.latex.preview.bind.parse().with_context(|| {
            format!(
                "latex.preview.bind '{}' must be an IP address",
                self.latex.preview.bind
            )
        })?;
        if !bind.is_loopback() && !self.latex.preview.allow_public {
            bail!(
                "latex.preview.bind must publish on host loopback unless latex.preview.allow_public = true"
            );
        }
        Ok(())
    }

    fn validate_tmux_status_layout(&self) -> Result<()> {
        const LINE1_LEFT: &[&str] = &["session", "windows"];
        const POWERKIT_PLUGINS: &[&str] = &[
            "aibox_log",
            "aibox_oom",
            "aibox_proc",
            "aibox_ai",
            "aibox_mcp",
            "aibox_mig",
            "weather",
            "uptime",
            "datetime",
            "forge",
            "git",
            "github",
            "kubernetes",
            "terraform",
            "cloud",
            "cloudstatus",
            "hostname",
            "externalip",
            "ssh",
            "netspeed",
            "ping",
            "cpu",
            "loadavg",
            "memory",
            "swap",
            "disk",
            "gpu",
        ];
        const MODEL_PROVIDER_PLUGINS: &[&str] = &[
            "modelstatus_openai",
            "modelstatus_anthropic",
            "modelstatus_google",
            "modelstatus_mistral",
            "modelstatus_deepseek",
            "modelstatus_cohere",
            "modelstatus_xai",
            "modelstatus_alibaba",
            "modelstatus_aws",
            "modelstatus_meta",
            "modelstatus_microsoft",
            "modelstatus_minimax",
            "modelstatus_moonshot",
            "modelstatus_nvidia",
            "modelstatus_xiaomi",
            "modelstatus_zai",
        ];

        fn validate_list(
            field: &str,
            configured: &Option<Vec<String>>,
            allowed: &[&str],
        ) -> Result<()> {
            let Some(entries) = configured else {
                return Ok(());
            };
            let allowed: BTreeSet<&str> = allowed.iter().copied().collect();
            let mut seen = BTreeSet::new();
            for entry in entries {
                if !allowed.contains(entry.as_str()) {
                    bail!(
                        "customization.tmux.status.layout.{field} contains unknown entry '{}'; supported entries: {}",
                        entry,
                        allowed.iter().copied().collect::<Vec<_>>().join(", ")
                    );
                }
                if !seen.insert(entry) {
                    bail!(
                        "customization.tmux.status.layout.{field} contains duplicate entry '{}'",
                        entry
                    );
                }
            }
            Ok(())
        }

        let layout = &self.customization.tmux.status.layout;
        let mut powerkit_plugins = POWERKIT_PLUGINS.to_vec();
        powerkit_plugins.extend(MODEL_PROVIDER_PLUGINS);
        validate_list("line1-left", &layout.line1_left, LINE1_LEFT)?;
        validate_list("line1-right", &layout.line1_right, &powerkit_plugins)?;
        validate_list("line2-left", &layout.line2_left, &powerkit_plugins)?;
        validate_list("line2-right", &layout.line2_right, &powerkit_plugins)?;
        Ok(())
    }

    fn validate_tmux_status_labels(&self) -> Result<()> {
        let labels = &self.customization.tmux.status.labels;
        for (field, value) in [
            ("aibox-log", &labels.aibox_log),
            ("aibox-oom", &labels.aibox_oom),
            ("aibox-proc", &labels.aibox_proc),
            ("aibox-ai", &labels.aibox_ai),
            ("aibox-mcp", &labels.aibox_mcp),
            ("aibox-mig", &labels.aibox_mig),
            ("kubernetes", &labels.kubernetes),
            ("cloud", &labels.cloud),
            ("cloud-aws", &labels.cloud_aws),
            ("cloud-gcp", &labels.cloud_gcp),
            ("cloud-azure", &labels.cloud_azure),
            ("cloud-multi", &labels.cloud_multi),
            ("uptime", &labels.uptime),
            ("netspeed", &labels.netspeed),
            ("netspeed-download", &labels.netspeed_download),
            ("netspeed-upload", &labels.netspeed_upload),
        ] {
            if value.trim().is_empty() {
                bail!("customization.tmux.status.labels.{field} cannot be empty");
            }
        }
        Ok(())
    }

    fn validate_tmux_status_refresh(&self) -> Result<()> {
        let refresh = &self.customization.tmux.status.refresh;
        for (field, value, min, max) in [
            ("interval-seconds", refresh.interval_seconds, 1, 3600),
            (
                "aibox-metrics-cache-ttl-seconds",
                refresh.aibox_metrics_cache_ttl_seconds,
                1,
                3600,
            ),
            (
                "netspeed-cache-ttl-seconds",
                refresh.netspeed_cache_ttl_seconds,
                1,
                3600,
            ),
            (
                "kubernetes-cache-ttl-seconds",
                refresh.kubernetes_cache_ttl_seconds,
                1,
                3600,
            ),
            (
                "cloud-cache-ttl-seconds",
                refresh.cloud_cache_ttl_seconds,
                1,
                3600,
            ),
            (
                "github-cache-ttl-seconds",
                refresh.github_cache_ttl_seconds,
                1,
                3600,
            ),
        ] {
            if value < min || value > max {
                bail!(
                    "customization.tmux.status.refresh.{field} must be between {min} and {max} seconds"
                );
            }
        }
        Ok(())
    }

    fn validate_tmux_status_forge(&self) -> Result<()> {
        let mut seen = BTreeSet::new();
        for host in &self.customization.tmux.status.forge.github_hosts {
            if host.is_empty()
                || !host
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
            {
                bail!(
                    "customization.tmux.status.forge.github-hosts contains invalid SSH host alias '{host}'; use only ASCII letters, digits, dots, underscores, and hyphens"
                );
            }
            if !seen.insert(host) {
                bail!(
                    "customization.tmux.status.forge.github-hosts contains duplicate entry '{host}'"
                );
            }
        }
        Ok(())
    }

    fn validate_tmux_model_provider_status(&self) -> Result<()> {
        const PROVIDERS: &[&str] = &[
            "openai",
            "anthropic",
            "google",
            "mistral",
            "deepseek",
            "cohere",
            "xai",
            "alibaba",
            "aws",
            "meta",
            "microsoft",
            "minimax",
            "moonshot",
            "nvidia",
            "xiaomi",
            "zai",
        ];
        const CHECKS: &[&str] = &["overall", "models", "harness"];

        let providers: BTreeSet<&str> = PROVIDERS.iter().copied().collect();
        let checks: BTreeSet<&str> = CHECKS.iter().copied().collect();
        let mut seen = BTreeSet::new();
        for provider in &self.customization.tmux.status.model_providers.providers {
            if !providers.contains(provider.provider.as_str()) {
                bail!(
                    "customization.tmux.status.model-providers provider '{}' is unknown; supported providers: {}",
                    provider.provider,
                    providers.iter().copied().collect::<Vec<_>>().join(", ")
                );
            }
            if !seen.insert(provider.provider.as_str()) {
                bail!(
                    "customization.tmux.status.model-providers contains duplicate provider '{}'",
                    provider.provider
                );
            }
            if provider.label.trim().is_empty() {
                bail!(
                    "customization.tmux.status.model-providers provider '{}' has an empty label",
                    provider.provider
                );
            }
            for check in &provider.checks {
                if !checks.contains(check.as_str()) {
                    bail!(
                        "customization.tmux.status.model-providers provider '{}' contains unknown check '{}'; supported checks: {}",
                        provider.provider,
                        check,
                        checks.iter().copied().collect::<Vec<_>>().join(", ")
                    );
                }
            }
        }
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
        self.ai.apply_harness_order();

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
        #[cfg(test)]
        if let Some(path) = TEST_HOST_ROOT.with(|cell| cell.borrow().clone()) {
            return path;
        }

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
        &["theme", "mode", "variant", "prompt", "layout", "tmux"],
        mismatches,
    );
    if let Some(customization) = table_child(root, key) {
        check_child_table(
            customization,
            "tmux",
            &["layout", "prefix", "session_name", "status"],
            mismatches,
        );
        if let Some(tmux) = table_child(customization, "tmux") {
            check_child_table(
                tmux,
                "status",
                &[
                    "mode",
                    "elements",
                    "layout",
                    "labels",
                    "separators",
                    "refresh",
                    "forge",
                    "model-providers",
                ],
                mismatches,
            );
            if let Some(status) = table_child(tmux, "status") {
                check_child_table(
                    status,
                    "labels",
                    &[
                        "aibox-log",
                        "aibox-oom",
                        "aibox-proc",
                        "aibox-ai",
                        "aibox-mcp",
                        "aibox-mig",
                        "kubernetes",
                        "cloud",
                        "cloud-aws",
                        "cloud-gcp",
                        "cloud-azure",
                        "cloud-multi",
                        "uptime",
                        "netspeed",
                        "netspeed-download",
                        "netspeed-upload",
                    ],
                    mismatches,
                );
                check_child_table(
                    status,
                    "separators",
                    &["style", "edge-style", "elements-spacing"],
                    mismatches,
                );
                check_child_table(
                    status,
                    "refresh",
                    &[
                        "interval-seconds",
                        "aibox-metrics-cache-ttl-seconds",
                        "netspeed-cache-ttl-seconds",
                        "kubernetes-cache-ttl-seconds",
                        "cloud-cache-ttl-seconds",
                        "github-cache-ttl-seconds",
                    ],
                    mismatches,
                );
                check_child_table(status, "forge", &["github-hosts"], mismatches);
                check_child_table(
                    status,
                    "model-providers",
                    &[
                        "enabled",
                        "cache-ttl-seconds",
                        "timeout-seconds",
                        "show-ok",
                        "providers",
                    ],
                    mismatches,
                );
                if let Some(model_providers) = table_child(status, "model-providers")
                    && let Some(entries) = model_providers
                        .get("providers")
                        .and_then(toml::Value::as_array)
                {
                    for (index, entry) in entries.iter().enumerate() {
                        if let Some(table) = entry.as_table() {
                            check_unknown_keys(
                                &format!(
                                    "[[customization.tmux.status.model-providers.providers]][{index}]"
                                ),
                                table,
                                &[
                                    "provider",
                                    "label",
                                    "checks",
                                    "status-url",
                                    "overall-components",
                                    "model-components",
                                    "harness-components",
                                ],
                                mismatches,
                            );
                        }
                    }
                }
                check_child_table(
                    status,
                    "layout",
                    &["line1-left", "line1-right", "line2-left", "line2-right"],
                    mismatches,
                );
                check_child_table(
                    status,
                    "elements",
                    &[
                        "hostname",
                        "external-ip",
                        "ssh",
                        "uptime",
                        "weather",
                        "datetime",
                        "git",
                        "github",
                        "kubernetes",
                        "terraform",
                        "cloud",
                        "cloudstatus",
                        "cpu",
                        "loadavg",
                        "mem",
                        "swap",
                        "disk",
                        "gpu",
                        "netspeed",
                        "ping",
                        "aibox",
                        "aibox-metrics",
                    ],
                    mismatches,
                );
                if let Some(elements) = table_child(status, "elements") {
                    check_child_table(
                        elements,
                        "aibox-metrics",
                        &["log", "oom", "proc", "ai", "mcp", "mig"],
                        mismatches,
                    );
                }
            }
        }
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
                    &[
                        "enabled",
                        "default_mode",
                        "mode",
                        "allow_patterns",
                        "extra_patterns",
                        "deny_patterns",
                    ],
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
        latex: LatexSection::default(),
        integrations: IntegrationsSection::default(),
        apply: ApplySection::default(),
        process: None,
        mcp: McpSection::default(),
        security: SecuritySection::default(),
        local_env: HashMap::new(),
        local_mcp_servers: vec![],
    };
    config.resolve_ai_provider_addons();
    // Resolve session name from project_name (mirrors what migrate_legacy_sections does).
    config.sync_grouped_sections();
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
python = { version = "3.14" }
uv = { version = "0.7" }

[addons.node.tools]
node = { version = "26" }
pnpm = { version = "11.18.0" }

[addons.rust.tools]
rustc = { version = "1.97.1" }
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

[audio]
enabled = false

[mcp.gateway]
mode = "daemon"
lazy_catalog = true
host = "127.0.0.1"
port = 8765
path = "/mcp"

[security]
acknowledge_seccomp_unconfined = true
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
        assert_eq!(config.addons.tool_version("python", "python"), Some("3.14"));
        assert_eq!(config.addons.tool_version("python", "uv"), Some("0.7"));
        assert_eq!(config.addons.tool_version("rust", "rustc"), Some("1.97.1"));
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
        // full_toml has `theme = "gruvbox-dark"` (legacy concrete form) → parsed
        // as ThemeFamily::Gruvbox with legacy_theme lock. The deserializer also
        // overrides mode to Dark so that a standardize-config round-trip
        // preserves the user's intent (otherwise re-serialised theme="gruvbox"
        // + mode="auto" would resolve via auto-fallback to GruvboxDark anyway,
        // but the explicit lock keeps light themes and alternate variants
        // round-trippable too).
        assert_eq!(config.customization.theme, ThemeFamily::Gruvbox);
        assert_eq!(config.customization.legacy_theme, Some(Theme::GruvboxDark));
        assert_eq!(config.customization.mode, ThemeMode::Dark);
        assert_eq!(config.customization.prompt, StarshipPreset::Default);
        assert_eq!(
            config.customization.tmux.status.mode,
            TmuxStatusMode::Extended
        );

        // [audio]
        assert!(!config.audio.enabled);
    }

    #[test]
    fn mcp_gateway_mode_accepts_public_names_and_legacy_aliases() {
        for (mode, expected) in [
            ("auto", McpGatewayMode::Auto),
            ("daemon", McpGatewayMode::DaemonProxy),
            ("daemon-proxy", McpGatewayMode::DaemonProxy),
            ("stdio", McpGatewayMode::Stdio),
            ("separate", McpGatewayMode::Granular),
            ("granular", McpGatewayMode::Granular),
        ] {
            let toml = format!(
                r#"
[container]
name = "my-project"

[mcp.gateway]
mode = "{mode}"
"#
            );
            let config = parse_toml(&toml).unwrap();
            assert_eq!(config.mcp.gateway.mode, expected, "mode {mode}");
        }
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
        assert_eq!(
            config.ai.execution.filesystem,
            AiExecutionFilesystem::WorkspaceWrite
        );
        assert_eq!(config.ai.execution.approval, AiExecutionApproval::OnRequest);
        assert_eq!(config.ai.execution.network, AiExecutionNetwork::Ask);
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
        assert_eq!(config.container.image.version, "0.23.8");
        assert_eq!(config.aibox.version, "latest");
        assert_eq!(config.aibox.base, BaseImage::Debian);
    }

    #[test]
    fn ai_execution_policy_parses_global_and_harness_overrides() {
        let toml = r#"
[aibox]
version = "0.26.5"

[container]
name = "test"

[ai.execution]
filesystem = "container-full"
approval = "never"
network = "allow"

[ai.harness.codex]
enabled = true
install = true

[ai.execution.codex]
filesystem = "read-only"
network = "deny"

[ai.harness.claude]
enabled = true

[ai.harness.claude.execution]
approval = "ask"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.ai.execution.filesystem,
            AiExecutionFilesystem::ContainerFull
        );
        assert_eq!(config.ai.execution.approval, AiExecutionApproval::Never);
        assert_eq!(config.ai.execution.network, AiExecutionNetwork::Allow);

        let codex = config.ai.harness.get(&AiProvider::Codex).unwrap();
        let execution = codex.execution.unwrap();
        assert_eq!(execution.filesystem, Some(AiExecutionFilesystem::ReadOnly));
        assert_eq!(execution.approval, None);
        assert_eq!(execution.network, Some(AiExecutionNetwork::Deny));

        let claude = config.ai.harness.get(&AiProvider::Claude).unwrap();
        let execution = claude.execution.unwrap();
        assert_eq!(execution.filesystem, None);
        assert_eq!(execution.approval, Some(AiExecutionApproval::Ask));
        assert_eq!(execution.network, None);
    }

    #[test]
    fn ai_execution_axes_display_as_canonical_config_values() {
        assert_eq!(AiExecutionFilesystem::ReadOnly.to_string(), "read-only");
        assert_eq!(
            AiExecutionFilesystem::WorkspaceWrite.to_string(),
            "workspace-write"
        );
        assert_eq!(
            AiExecutionFilesystem::ContainerFull.to_string(),
            "container-full"
        );
        assert_eq!(AiExecutionApproval::Ask.to_string(), "ask");
        assert_eq!(AiExecutionApproval::OnRequest.to_string(), "on-request");
        assert_eq!(AiExecutionApproval::Never.to_string(), "never");
        assert_eq!(AiExecutionNetwork::Deny.to_string(), "deny");
        assert_eq!(AiExecutionNetwork::Ask.to_string(), "ask");
        assert_eq!(AiExecutionNetwork::Allow.to_string(), "allow");
    }

    #[test]
    fn ai_execution_policy_rejects_unknown_axis_values() {
        let toml = r#"
[aibox]
version = "0.26.5"

[container]
name = "test"

[ai.execution]
filesystem = "host-full"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "unknown filesystem axis value must fail");
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
    fn schema_mismatches_accepts_ai_execution_policy_tables() {
        let toml = r#"
[aibox]
version = "0.26.5"

[container]
name = "test"

[ai.execution]
filesystem = "workspace-write"
approval = "on-request"
network = "ask"

[ai.harness.claude]
enabled = true

[ai.execution.claude]
approval = "ask"
network = "deny"
"#;
        let mismatches = AiboxConfig::schema_mismatches(toml).unwrap();
        assert!(
            mismatches.is_empty(),
            "ai execution policy tables should be schema-valid: {mismatches:?}"
        );
    }

    #[test]
    fn schema_mismatches_accepts_customization_variant_key() {
        let toml = r#"
[aibox]
version = "0.26.5"

[container]
name = "test"

[customization]
theme = "ayu"
mode = "dark"
variant = "mirage"
"#;
        let mismatches = AiboxConfig::schema_mismatches(toml).unwrap();
        assert!(
            mismatches.is_empty(),
            "customization.variant should be schema-valid: {mismatches:?}"
        );
    }

    #[test]
    fn schema_mismatches_rejects_legacy_multiplexer_status_table() {
        // BR-LEGACY-MUX-EXCISE (DEC-20260508_1515-SilentAsh, v0.25.6):
        // the legacy multiplexer status alias was hard-cut. Old configs
        // that still carry it must surface a schema error pointing at
        // [customization.tmux.status].
        let toml = r#"
[aibox]
version = "0.25.6"

[container]
name = "my-project"

[customization.legacy_mux_status]
mode = "hidden"
"#;
        let mismatches = AiboxConfig::schema_mismatches(toml).unwrap();
        assert!(
            !mismatches.is_empty(),
            "legacy multiplexer status table must be rejected after the v0.25.6 hard-cut"
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

[customization.tmux.status]
mod = "typo"

[customization.tmux]
prefx = "typo"

[addons.git-ui.tools.lazygit]
enabled = false
enabld = true
"#;
        let mismatches = AiboxConfig::schema_mismatches(toml).unwrap();

        assert!(mismatches.contains(&"[container]: unknown key `nmae`".to_string()));
        assert!(mismatches.contains(&"[status]: unknown key `mod`".to_string()));
        assert!(mismatches.contains(&"[tmux]: unknown key `prefx`".to_string()));
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
        // "dracula" is both a family and a concrete name; family takes precedence.
        assert_eq!(config.customization.theme, ThemeFamily::Dracula);
        assert_eq!(config.customization.legacy_theme, None);
        assert_eq!(config.customization.mode, ThemeMode::Dark);
        assert_eq!(config.customization.prompt, StarshipPreset::Minimal);
    }

    #[test]
    fn customization_tmux_fields_parse() {
        let toml = r#"
[aibox]
version = "0.25.0"

[container]
name = "my-project"

[customization]
layout = "focus"

[customization.tmux]
layout = "ai"
prefix = "C-a"
session_name = "work"

[customization.tmux.status]
mode = "plain"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.customization.layout, ConfigLayout::Focus);
        assert_eq!(config.customization.tmux_layout(), ConfigLayout::Ai);
        assert_eq!(config.customization.tmux.prefix, "C-a");
        assert_eq!(config.customization.tmux.session_name, "work");
        assert_eq!(config.customization.tmux.status.mode, TmuxStatusMode::Plain);
    }

    // -- resolve_tmux_session_name --------------------------------------------

    #[test]
    fn session_name_uses_explicit_project_name() {
        assert_eq!(resolve_tmux_session_name("foo-bar", None), "foo-bar");
    }

    #[test]
    fn session_name_falls_back_to_cwd_basename() {
        let cwd = std::path::Path::new("/workspace/cool-project");
        assert_eq!(resolve_tmux_session_name("", Some(cwd)), "cool-project");
    }

    #[test]
    fn session_name_falls_back_to_aibox_when_both_empty() {
        assert_eq!(resolve_tmux_session_name("", None), "aibox");
    }

    #[test]
    fn session_name_sanitizes_dots_and_colons() {
        // dots and colons → dashes; consecutive dashes collapsed
        assert_eq!(resolve_tmux_session_name("my.project", None), "my-project");
        assert_eq!(resolve_tmux_session_name("host:port", None), "host-port");
        assert_eq!(resolve_tmux_session_name("a..b", None), "a-b");
    }

    #[test]
    fn session_name_strips_illegal_chars() {
        // spaces and other non-ASCII are stripped; fallback applies if empty
        assert_eq!(resolve_tmux_session_name("hello world", None), "helloworld");
        assert_eq!(resolve_tmux_session_name("日本語", None), "aibox");
    }

    #[test]
    fn session_name_trims_leading_trailing_dashes() {
        assert_eq!(resolve_tmux_session_name(".leading", None), "leading");
        assert_eq!(resolve_tmux_session_name("trailing.", None), "trailing");
    }

    #[test]
    fn session_name_derived_from_config_project_name() {
        let toml = r#"
[aibox]
version = "0.25.0"
project_name = "my-app"

[container]
name = "my-app"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.customization.tmux.session_name, "my-app");
    }

    #[test]
    fn session_name_explicit_override_wins_over_project_name() {
        let toml = r#"
[aibox]
version = "0.25.0"
project_name = "my-app"

[container]
name = "my-app"

[customization.tmux]
session_name = "custom"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.customization.tmux.session_name, "custom");
        assert_eq!(config.tmux_session_name(), "custom");
    }

    #[test]
    fn session_name_falls_back_to_container_name_when_project_name_absent() {
        let toml = r#"
[aibox]
version = "0.25.0"

[container]
name = "derived-proj"
"#;
        let config = parse_toml(toml).unwrap();
        // project_name syncs from container.name → session derives from that
        assert_eq!(config.customization.tmux.session_name, "derived-proj");
    }

    #[test]
    fn tmux_status_powerline_alias_maps_to_extended() {
        let toml = r#"
[aibox]
version = "0.25.1"

[container]
name = "my-project"

[customization.tmux.status]
mode = "powerline"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.customization.tmux.status.mode,
            TmuxStatusMode::Extended
        );
    }

    #[test]
    fn schema_mismatches_accepts_tmux_status_element_toggles() {
        let toml = r#"
[aibox]
version = "0.25.2"

[container]
name = "my-project"

[customization.tmux.status]
mode = "extended"

[customization.tmux.status.elements]
external-ip = false
mem = true
aibox = true

[customization.tmux.status.elements.aibox-metrics]
log = true
mig = false
"#;
        let mismatches = AiboxConfig::schema_mismatches(toml).unwrap();
        assert!(
            mismatches.is_empty(),
            "tmux status element toggles should be schema-clean: {mismatches:?}"
        );
    }

    #[test]
    fn schema_mismatches_accepts_tmux_status_separators() {
        let toml = r#"
[aibox]
version = "0.25.14"

[container]
name = "my-project"

[customization.tmux.status.separators]
style = "flame"
edge-style = "honeycomb"
elements-spacing = "plugins"
"#;
        let mismatches = AiboxConfig::schema_mismatches(toml).unwrap();
        assert!(
            mismatches.is_empty(),
            "tmux status separator options should be schema-clean: {mismatches:?}"
        );
    }

    #[test]
    fn schema_and_validation_accept_tmux_forge_github_hosts() {
        let toml = r#"
apiVersion = "aibox.projectious.work/v1"
kind = "Workspace"

[customization.tmux.status.forge]
        github-hosts = ["github.com", "github-bnaard", "github_work"]
"#;
        assert!(AiboxConfig::schema_mismatches(toml).unwrap().is_empty());
        let mut config = test_config();
        config.customization.tmux.status.forge.github_hosts = vec![
            "github.com".to_string(),
            "github-bnaard".to_string(),
            "github_work".to_string(),
        ];
        config.validate().unwrap();
    }

    #[test]
    fn validation_rejects_unsafe_tmux_forge_github_host_alias() {
        let mut config = test_config();
        config.customization.tmux.status.forge.github_hosts =
            vec!["github.com;run-shell".to_string()];
        let error = config.validate().unwrap_err().to_string();
        assert!(error.contains("contains invalid SSH host alias"), "{error}");
    }

    #[test]
    fn parse_tmux_status_layout_lists() {
        let toml = r#"
[aibox]
version = "0.25.8"

[container]
name = "my-project"

[customization.tmux.status.layout]
line1-left = ["session"]
line1-right = ["datetime", "weather"]
line2-left = ["git", "cloudstatus"]
line2-right = ["cpu", "memory"]
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.customization.tmux.status.layout.line1_left,
            Some(vec!["session".to_string()])
        );
        assert_eq!(
            config.customization.tmux.status.layout.line1_right,
            Some(vec!["datetime".to_string(), "weather".to_string()])
        );
        assert_eq!(
            config.customization.tmux.status.layout.line2_left,
            Some(vec!["git".to_string(), "cloudstatus".to_string()])
        );
        assert_eq!(
            config.customization.tmux.status.layout.line2_right,
            Some(vec!["cpu".to_string(), "memory".to_string()])
        );
    }

    #[test]
    fn parse_tmux_status_separators() {
        let toml = r#"
[aibox]
version = "0.25.14"

[container]
name = "my-project"

[customization.tmux.status.separators]
style = "flame"
edge-style = "honeycomb"
elements-spacing = "plugins"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.customization.tmux.status.separators.style,
            TmuxStatusSeparatorStyle::Flame
        );
        assert_eq!(
            config.customization.tmux.status.separators.edge_style,
            TmuxStatusSeparatorStyle::Honeycomb
        );
        assert_eq!(
            config.customization.tmux.status.separators.elements_spacing,
            TmuxStatusElementsSpacing::Plugins
        );
    }

    #[test]
    fn tmux_status_layout_rejects_unknown_entries() {
        let toml = r#"
[aibox]
version = "0.25.8"

[container]
name = "my-project"

[customization.tmux.status.layout]
line2-left = ["git", "not-a-plugin"]
"#;
        let err = parse_toml(toml).expect_err("unknown status plugin must be rejected");
        assert!(
            err.to_string().contains("not-a-plugin"),
            "error should name the bad plugin: {err:?}"
        );
    }

    #[test]
    fn tmux_status_separators_reject_unknown_styles() {
        let toml = r#"
[aibox]
version = "0.25.14"

[container]
name = "my-project"

[customization.tmux.status.separators]
style = "not-a-style"
"#;
        let err = parse_toml(toml).expect_err("unknown separator style must be rejected");
        assert!(
            format!("{err:?}").contains("not-a-style"),
            "error should name the bad separator style: {err:?}"
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
    fn ai_model_provider_env_metadata() {
        assert_eq!(
            AiModelProvider::Anthropic.api_key_env(),
            "ANTHROPIC_API_KEY"
        );
        assert_eq!(
            AiModelProvider::Anthropic.endpoint_env(),
            "ANTHROPIC_BASE_URL"
        );

        assert_eq!(AiModelProvider::OpenAI.api_key_env(), "OPENAI_API_KEY");
        assert_eq!(AiModelProvider::OpenAI.endpoint_env(), "OPENAI_BASE_URL");

        assert_eq!(AiModelProvider::Google.api_key_env(), "GEMINI_API_KEY");
        assert_eq!(AiModelProvider::Google.endpoint_env(), "GEMINI_BASE_URL");

        assert_eq!(AiModelProvider::Mistral.api_key_env(), "MISTRAL_API_KEY");
        assert_eq!(AiModelProvider::Mistral.endpoint_env(), "MISTRAL_BASE_URL");
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
        assert_eq!(config.addons.tool_version("node", "node"), Some("26"));
        assert_eq!(config.addons.tool_version("node", "pnpm"), Some("11.18.0"));
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
        assert_eq!(config.context.mode, ContextMode::Processkit);
        assert_eq!(config.context.packages, vec!["product"]);
    }

    #[test]
    fn context_mode_harness_only_allows_empty_packages() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[context]
mode = "harness-only"
packages = []
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.context.mode, ContextMode::HarnessOnly);
        assert!(config.context.packages.is_empty());
        assert!(!config.processkit_enabled());
    }

    #[test]
    fn schema_mismatches_accepts_context_mode() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[context]
mode = "harness-only"
packages = []
"#;
        let mismatches = AiboxConfig::schema_mismatches(toml).unwrap();
        assert!(
            mismatches.is_empty(),
            "context.mode should be schema-valid: {mismatches:?}"
        );
    }

    #[test]
    fn context_mode_rejects_unknown_value() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[context]
mode = "unknown"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject unknown context mode");
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

    /// Legacy concrete theme names must parse successfully: the family is
    /// inferred from the concrete name and `legacy_theme` is populated.
    #[test]
    fn appearance_legacy_concrete_names_parse_into_family_and_lock() {
        for (input, expected_family, expected_legacy) in [
            ("gruvbox-dark", ThemeFamily::Gruvbox, Theme::GruvboxDark),
            ("gruvbox-light", ThemeFamily::Gruvbox, Theme::GruvboxLight),
            (
                "catppuccin-mocha",
                ThemeFamily::Catppuccin,
                Theme::CatppuccinMocha,
            ),
            (
                "catppuccin-macchiato",
                ThemeFamily::Catppuccin,
                Theme::CatppuccinMacchiato,
            ),
            (
                "catppuccin-frappe",
                ThemeFamily::Catppuccin,
                Theme::CatppuccinFrappe,
            ),
            (
                "catppuccin-latte",
                ThemeFamily::Catppuccin,
                Theme::CatppuccinLatte,
            ),
            (
                "tokyo-night-storm",
                ThemeFamily::TokyoNight,
                Theme::TokyoNightStorm,
            ),
            (
                "tokyo-night-day",
                ThemeFamily::TokyoNight,
                Theme::TokyoNightDay,
            ),
            ("rose-pine-moon", ThemeFamily::RosePine, Theme::RosePineMoon),
            ("rose-pine-dawn", ThemeFamily::RosePine, Theme::RosePineDawn),
            (
                "material-ocean",
                ThemeFamily::Material,
                Theme::MaterialOcean,
            ),
            (
                "material-palenight",
                ThemeFamily::Material,
                Theme::MaterialPalenight,
            ),
            (
                "material-lighter",
                ThemeFamily::Material,
                Theme::MaterialLighter,
            ),
            (
                "solarized-dark",
                ThemeFamily::Solarized,
                Theme::SolarizedDark,
            ),
            (
                "solarized-light",
                ThemeFamily::Solarized,
                Theme::SolarizedLight,
            ),
            ("github-dark", ThemeFamily::Github, Theme::GithubDark),
            ("github-light", ThemeFamily::Github, Theme::GithubLight),
            ("ayu-dark", ThemeFamily::Ayu, Theme::AyuDark),
            ("ayu-mirage", ThemeFamily::Ayu, Theme::AyuMirage),
            ("ayu-light", ThemeFamily::Ayu, Theme::AyuLight),
            (
                "night-owl-light",
                ThemeFamily::NightOwl,
                Theme::NightOwlLight,
            ),
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
            assert_eq!(
                config.customization.theme, expected_family,
                "family for {input}"
            );
            assert_eq!(
                config.customization.legacy_theme,
                Some(expected_legacy),
                "legacy_theme for {input}"
            );
        }
    }

    /// Family theme names (and solo themes that double as family names) parse
    /// into the family form with no legacy_theme lock.
    #[test]
    fn appearance_family_names_parse_without_legacy_lock() {
        for (input, expected_family) in [
            ("ayu", ThemeFamily::Ayu),
            ("catppuccin", ThemeFamily::Catppuccin),
            ("dracula", ThemeFamily::Dracula),
            ("github", ThemeFamily::Github),
            ("gruvbox", ThemeFamily::Gruvbox),
            // "material" is BOTH a family name and a concrete name
            // (Theme::Material). Family check runs first → no legacy lock.
            ("material", ThemeFamily::Material),
            ("moonlight", ThemeFamily::Moonlight),
            ("night-owl", ThemeFamily::NightOwl),
            ("nord", ThemeFamily::Nord),
            ("vscode", ThemeFamily::VsCode),
            ("projectious", ThemeFamily::Projectious),
            ("rose-pine", ThemeFamily::RosePine),
            ("solarized", ThemeFamily::Solarized),
            ("tokyo-night", ThemeFamily::TokyoNight),
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
            assert_eq!(
                config.customization.theme, expected_family,
                "family for {input}"
            );
            assert_eq!(
                config.customization.legacy_theme, None,
                "no legacy lock for family name {input}"
            );
        }
    }

    #[test]
    fn appearance_mode_resolves_concrete_theme() {
        // Solo family (dracula): ignores mode, always returns Dracula.
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
        assert_eq!(config.customization.theme, ThemeFamily::Dracula);
        assert_eq!(config.customization.mode, ThemeMode::Light);
        assert_eq!(config.customization.resolved_theme(), Theme::Dracula);

        // Legacy concrete name (catppuccin-latte) is locked — mode override does NOT flip it.
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
        // Legacy lock: "catppuccin-latte" is preserved, NOT flipped to mocha.
        assert_eq!(
            config.customization.resolved_theme(),
            Theme::CatppuccinLatte
        );

        // New family form with mode = "dark" → canonical dark variant.
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[customization]
theme = "catppuccin"
mode = "dark"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.customization.resolved_theme(),
            Theme::CatppuccinMocha
        );
    }

    #[test]
    fn appearance_auto_resolves_from_host_mode_when_theme_has_partner() {
        let mut config = test_config();
        config.customization.theme = ThemeFamily::Gruvbox;
        config.customization.mode = ThemeMode::Auto;
        assert_eq!(
            config
                .customization
                .resolved_theme_for_host_mode(Some(ThemeMode::Light)),
            Theme::GruvboxLight
        );
        assert_eq!(
            config
                .customization
                .resolved_theme_for_host_mode(Some(ThemeMode::Dark)),
            Theme::GruvboxDark
        );
    }

    #[test]
    fn appearance_auto_preserves_dark_only_themes_in_light_host_mode() {
        let mut config = test_config();
        // Nord is a solo family — always Nord regardless of mode.
        config.customization.theme = ThemeFamily::Nord;
        config.customization.mode = ThemeMode::Auto;
        assert_eq!(
            config
                .customization
                .resolved_theme_for_host_mode(Some(ThemeMode::Light)),
            Theme::Nord
        );
    }

    // -- New family-based resolution tests -----------------------------------

    #[test]
    fn resolved_theme_for_ayu_family_with_dark_mode_returns_ayu_dark() {
        let mut config = test_config();
        config.customization.theme = ThemeFamily::Ayu;
        config.customization.mode = ThemeMode::Dark;
        assert_eq!(config.customization.resolved_theme(), Theme::AyuDark);
    }

    #[test]
    fn resolved_theme_for_ayu_family_with_light_mode_returns_ayu_light() {
        let mut config = test_config();
        config.customization.theme = ThemeFamily::Ayu;
        config.customization.mode = ThemeMode::Light;
        assert_eq!(config.customization.resolved_theme(), Theme::AyuLight);
    }

    #[test]
    fn resolved_theme_for_ayu_with_variant_mirage_returns_ayu_mirage() {
        let mut config = test_config();
        config.customization.theme = ThemeFamily::Ayu;
        config.customization.mode = ThemeMode::Dark;
        config.customization.variant = Some("mirage".to_string());
        assert_eq!(config.customization.resolved_theme(), Theme::AyuMirage);
    }

    #[test]
    fn resolved_theme_for_solo_family_ignores_mode_and_variant() {
        // Nord stays Nord regardless of mode and variant.
        let mut config = test_config();
        config.customization.theme = ThemeFamily::Nord;
        config.customization.mode = ThemeMode::Light;
        config.customization.variant = Some("anything".to_string());
        assert_eq!(config.customization.resolved_theme(), Theme::Nord);

        config.customization.theme = ThemeFamily::Dracula;
        config.customization.mode = ThemeMode::Auto;
        assert_eq!(config.customization.resolved_theme(), Theme::Dracula);
    }

    #[test]
    fn legacy_concrete_theme_locks_resolved_output_against_mode_flip() {
        // theme = "ayu-dark", mode = auto, host = light → MUST resolve AyuDark (locked).
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[customization]
theme = "ayu-dark"
mode = "auto"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config
                .customization
                .resolved_theme_for_host_mode(Some(ThemeMode::Light)),
            Theme::AyuDark,
            "legacy lock must prevent auto-flip to AyuLight"
        );
    }

    #[test]
    fn legacy_concrete_theme_populates_legacy_theme_field_after_deserialize() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[customization]
theme = "ayu-dark"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.customization.theme, ThemeFamily::Ayu);
        assert_eq!(config.customization.legacy_theme, Some(Theme::AyuDark));
    }

    #[test]
    fn new_family_form_does_not_populate_legacy_theme_field() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[customization]
theme = "ayu"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.customization.theme, ThemeFamily::Ayu);
        assert_eq!(config.customization.legacy_theme, None);
    }

    #[test]
    fn auto_with_none_host_falls_back_to_dark() {
        let mut config = test_config();
        config.customization.theme = ThemeFamily::Gruvbox;
        config.customization.mode = ThemeMode::Auto;
        // None host mode → dark fallback (new behaviour; old code preserved selected theme as-is)
        assert_eq!(
            config.customization.resolved_theme_for_host_mode(None),
            Theme::GruvboxDark
        );
    }

    #[test]
    fn host_theme_mode_parser_handles_common_platform_outputs() {
        assert_eq!(
            parse_host_theme_mode_text("'prefer-dark'"),
            Some(ThemeMode::Dark)
        );
        assert_eq!(
            parse_host_theme_mode_text("'prefer-light'"),
            Some(ThemeMode::Light)
        );
        assert_eq!(
            parse_host_theme_mode_text("default"),
            Some(ThemeMode::Light)
        );
        assert_eq!(parse_host_theme_mode_text("0"), Some(ThemeMode::Dark));
        assert_eq!(parse_host_theme_mode_text("1"), Some(ThemeMode::Light));
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
    fn blank_ai_harness_table_does_not_enable_harness() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            [ai.harness.claude]
            [ai.harness.codex]
            enabled = true
            install = true
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert!(!config.ai.harnesses.contains(&AiProvider::Claude));
        assert!(!config.addons.has_addon("ai-claude"));
        assert!(config.ai.harnesses.contains(&AiProvider::Codex));
        assert!(config.addons.has_addon("ai-codex"));
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
    fn ai_harness_order_controls_effective_layout_order() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            harness_order = ["codex", "claude"]
            [ai.harness.claude]
            enabled = true
            install = true
            [ai.harness.codex]
            enabled = true
            install = true
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert_eq!(
            config.ai.harnesses,
            vec![AiProvider::Codex, AiProvider::Claude]
        );
    }

    #[test]
    fn ai_harness_order_appends_enabled_harnesses_not_listed() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            harness_order = ["codex"]
            [ai.harness.claude]
            enabled = true
            install = true
            [ai.harness.codex]
            enabled = true
            install = true
            [ai.harness.gemini]
            enabled = true
            install = true
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert_eq!(
            config.ai.harnesses,
            vec![AiProvider::Codex, AiProvider::Claude, AiProvider::Gemini]
        );
    }

    #[test]
    fn ai_harness_entries_are_ordered_and_split_enable_from_install() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            [[ai.harnesses]]
            harness = "codex"
            enable = true
            install = false
            [[ai.harnesses]]
            harness = "claude"
            enable = false
            install = true
            [[ai.harnesses]]
            harness = "gemini"
            enable = true
            install = true
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert_eq!(
            config.ai.harnesses,
            vec![AiProvider::Codex, AiProvider::Gemini]
        );
        assert!(!config.addons.has_addon("ai-codex"));
        assert!(!config.addons.has_addon("ai-claude"));
        assert!(config.addons.has_addon("ai-gemini"));
    }

    #[test]
    fn ai_harness_entries_default_enable_and_install_to_false() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            [[ai.harnesses]]
            harness = "codex"
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert!(config.ai.harnesses.is_empty());
        assert!(!config.addons.has_addon("ai-codex"));
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
    fn extra_volumes_reject_managed_runtime_home_shadowing() {
        let toml = r#"
[aibox]
version = "0.9.0"
[container]
name = "test"
[[container.extra_volumes]]
source = "~/.config/yazi"
target = "/home/aibox/.config/yazi"
"#;
        let err = AiboxConfig::from_str(toml).unwrap_err().to_string();
        assert!(
            err.contains("overlaps aibox-managed runtime home path"),
            "should reject extra volume shadowing managed runtime config: {err}"
        );
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

    #[test]
    fn resolve_theme_from_family_handles_all_new_families() {
        use Theme as T;
        use ThemeFamily as F;
        // OneDark
        assert_eq!(
            resolve_theme_from_family(&F::OneDark, ThemeMode::Dark, None),
            T::OneDarkPro
        );
        assert_eq!(
            resolve_theme_from_family(&F::OneDark, ThemeMode::Light, None),
            T::OneLight
        );
        // Vitesse
        assert_eq!(
            resolve_theme_from_family(&F::Vitesse, ThemeMode::Dark, None),
            T::VitesseDark
        );
        assert_eq!(
            resolve_theme_from_family(&F::Vitesse, ThemeMode::Light, None),
            T::VitesseLight
        );
        assert_eq!(
            resolve_theme_from_family(&F::Vitesse, ThemeMode::Dark, Some("black")),
            T::VitesseBlack
        );
        // Kanagawa
        assert_eq!(
            resolve_theme_from_family(&F::Kanagawa, ThemeMode::Dark, None),
            T::KanagawaWave
        );
        assert_eq!(
            resolve_theme_from_family(&F::Kanagawa, ThemeMode::Dark, Some("dragon")),
            T::KanagawaDragon
        );
        assert_eq!(
            resolve_theme_from_family(&F::Kanagawa, ThemeMode::Light, None),
            T::KanagawaLotus
        );
        // Min
        assert_eq!(
            resolve_theme_from_family(&F::Min, ThemeMode::Dark, None),
            T::MinDark
        );
        assert_eq!(
            resolve_theme_from_family(&F::Min, ThemeMode::Light, None),
            T::MinLight
        );
        // Slack
        assert_eq!(
            resolve_theme_from_family(&F::Slack, ThemeMode::Dark, None),
            T::SlackDark
        );
        assert_eq!(
            resolve_theme_from_family(&F::Slack, ThemeMode::Light, None),
            T::SlackOchin
        );
        // Everforest
        assert_eq!(
            resolve_theme_from_family(&F::Everforest, ThemeMode::Dark, None),
            T::EverforestDark
        );
        assert_eq!(
            resolve_theme_from_family(&F::Everforest, ThemeMode::Light, None),
            T::EverforestLight
        );
        // VsCode
        assert_eq!(
            resolve_theme_from_family(&F::VsCode, ThemeMode::Dark, None),
            T::VsCodeDarkPlus
        );
        assert_eq!(
            resolve_theme_from_family(&F::VsCode, ThemeMode::Light, None),
            T::VsCodeLightPlus
        );
        // Dracula (multi-variant)
        assert_eq!(
            resolve_theme_from_family(&F::Dracula, ThemeMode::Dark, None),
            T::Dracula
        );
        assert_eq!(
            resolve_theme_from_family(&F::Dracula, ThemeMode::Dark, Some("soft")),
            T::DraculaSoft
        );
        // Github new variants
        assert_eq!(
            resolve_theme_from_family(&F::Github, ThemeMode::Dark, Some("dimmed")),
            T::GithubDarkDimmed
        );
        assert_eq!(
            resolve_theme_from_family(&F::Github, ThemeMode::Dark, Some("high-contrast-dark")),
            T::GithubDarkHighContrast
        );
        assert_eq!(
            resolve_theme_from_family(&F::Github, ThemeMode::Light, Some("high-contrast-light")),
            T::GithubLightHighContrast
        );
        // Material new variant
        assert_eq!(
            resolve_theme_from_family(&F::Material, ThemeMode::Dark, Some("darker")),
            T::MaterialDarker
        );
        // Solo new families
        assert_eq!(
            resolve_theme_from_family(&F::Snazzy, ThemeMode::Dark, None),
            T::SnazzyLight
        );
        assert_eq!(
            resolve_theme_from_family(&F::Monokai, ThemeMode::Dark, None),
            T::Monokai
        );
        assert_eq!(
            resolve_theme_from_family(&F::Poimandres, ThemeMode::Dark, None),
            T::Poimandres
        );
        assert_eq!(
            resolve_theme_from_family(&F::Synthwave84, ThemeMode::Dark, None),
            T::Synthwave84
        );
        assert_eq!(
            resolve_theme_from_family(&F::Andromeeda, ThemeMode::Dark, None),
            T::Andromeeda
        );
        assert_eq!(
            resolve_theme_from_family(&F::AuroraX, ThemeMode::Dark, None),
            T::AuroraX
        );
        assert_eq!(
            resolve_theme_from_family(&F::Vesper, ThemeMode::Dark, None),
            T::Vesper
        );
        assert_eq!(
            resolve_theme_from_family(&F::Laserwave, ThemeMode::Dark, None),
            T::Laserwave
        );
        assert_eq!(
            resolve_theme_from_family(&F::Plastic, ThemeMode::Dark, None),
            T::Plastic
        );
        assert_eq!(
            resolve_theme_from_family(&F::Houston, ThemeMode::Dark, None),
            T::Houston
        );
        assert_eq!(
            resolve_theme_from_family(&F::Red, ThemeMode::Dark, None),
            T::Red
        );
    }

    #[test]
    fn family_of_round_trips_for_new_themes() {
        use Theme as T;
        use ThemeFamily as F;
        assert_eq!(family_of(&T::OneDarkPro), F::OneDark);
        assert_eq!(family_of(&T::OneLight), F::OneDark);
        assert_eq!(family_of(&T::VitesseDark), F::Vitesse);
        assert_eq!(family_of(&T::VitesseLight), F::Vitesse);
        assert_eq!(family_of(&T::VitesseBlack), F::Vitesse);
        assert_eq!(family_of(&T::KanagawaWave), F::Kanagawa);
        assert_eq!(family_of(&T::KanagawaDragon), F::Kanagawa);
        assert_eq!(family_of(&T::KanagawaLotus), F::Kanagawa);
        assert_eq!(family_of(&T::MinDark), F::Min);
        assert_eq!(family_of(&T::MinLight), F::Min);
        assert_eq!(family_of(&T::SlackDark), F::Slack);
        assert_eq!(family_of(&T::SlackOchin), F::Slack);
        assert_eq!(family_of(&T::EverforestDark), F::Everforest);
        assert_eq!(family_of(&T::EverforestLight), F::Everforest);
        assert_eq!(family_of(&T::VsCodeDarkPlus), F::VsCode);
        assert_eq!(family_of(&T::VsCodeLightPlus), F::VsCode);
        assert_eq!(family_of(&T::DraculaSoft), F::Dracula);
        assert_eq!(family_of(&T::GithubDarkDimmed), F::Github);
        assert_eq!(family_of(&T::GithubDarkHighContrast), F::Github);
        assert_eq!(family_of(&T::GithubLightHighContrast), F::Github);
        assert_eq!(family_of(&T::MaterialDarker), F::Material);
        assert_eq!(family_of(&T::SnazzyLight), F::Snazzy);
        assert_eq!(family_of(&T::Monokai), F::Monokai);
        assert_eq!(family_of(&T::Poimandres), F::Poimandres);
        assert_eq!(family_of(&T::Synthwave84), F::Synthwave84);
        assert_eq!(family_of(&T::Andromeeda), F::Andromeeda);
        assert_eq!(family_of(&T::AuroraX), F::AuroraX);
        assert_eq!(family_of(&T::Vesper), F::Vesper);
        assert_eq!(family_of(&T::Laserwave), F::Laserwave);
        assert_eq!(family_of(&T::Plastic), F::Plastic);
        assert_eq!(family_of(&T::Houston), F::Houston);
        assert_eq!(family_of(&T::Red), F::Red);
    }

    #[test]
    fn latex_config_parses_and_validates() {
        let toml = r#"
[container]
name = "latex-project"

[latex]
engine = "lualatex"
cache_dir = ".latex-cache"
options = ["-shell-escape"]

[[latex.documents]]
name = "overview"
source = "docs/overview.tex"
output_dir = ".latex-cache/overview"

[latex.preview]
enabled = true
engine = "embedpdf"
bind = "127.0.0.1"
port = 8765
"#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert_eq!(config.latex.documents[0].name, "overview");
        assert!(AiboxConfig::schema_mismatches(toml).unwrap().is_empty());
    }

    #[test]
    fn latex_preview_requires_explicit_public_consent() {
        let mut config = test_config();
        config.latex.preview.bind = "0.0.0.0".to_string();
        let error = config.validate().unwrap_err().to_string();
        assert!(error.contains("allow_public"));
    }
}
