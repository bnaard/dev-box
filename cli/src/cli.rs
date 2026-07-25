use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::config::{
    AiHarness, AiProvider, AiboxProfile, BaseImage, ContextMode, StarshipPreset, ThemeFamily,
    TmuxStatusMode,
};

/// Parse a truthy/falsy string for env-var-driven boolean flags.
/// Accepts 1/0, true/false, yes/no, on/off (case-insensitive). Empty string is
/// treated as false so an unset `AIBOX_NO_CONTAINER=` doesn't trip the parser.
fn parse_truthy_flag(s: &str) -> Result<bool, String> {
    match s.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" | "" => Ok(false),
        other => Err(format!(
            "expected one of 1/0/true/false/yes/no/on/off, got '{}'",
            other
        )),
    }
}

/// Output format for list commands.
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table (default)
    #[default]
    Table,
    /// JSON array
    Json,
    /// YAML sequence
    Yaml,
}

/// Output format for deterministic configuration compilation.
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum CompileOutputFormat {
    /// Concise human-readable plan
    #[default]
    Human,
    /// Canonical plan as JSON
    Json,
}

/// Output format for v1 deployment and image operations.
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum DeployOutputFormat {
    /// Concise human-readable operation result
    #[default]
    Human,
    /// Complete machine-readable operation result as JSON
    Json,
}

/// Available tmux IDE layouts.
#[derive(Clone, Debug, ValueEnum)]
pub enum Layout {
    /// Work window with yazi, first harness, shell; optional AI/lazygit/shell windows
    Dev,
    /// One fullscreen window for files and each harness
    Focus,
    /// Work window with yazi and shell; AI harnesses in a separate window
    Cowork,
    /// Work window with yazi and first harness; secondary harnesses in AI window
    Ai,
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Dev => write!(f, "dev"),
            Layout::Focus => write!(f, "focus"),
            Layout::Cowork => write!(f, "cowork"),
            Layout::Ai => write!(f, "ai"),
        }
    }
}

const LONG_ABOUT: &str = concat!(
    "aibox v",
    env!("CARGO_PKG_VERSION"),
    " — apply and run AI-ready development workspaces

aibox treats aibox.toml as desired state. Use `apply` to reconcile
generated files and images, then `up` to enter the workspace.

Examples:
  aibox init my-app --addon python --harness claude
  aibox apply                                Reconcile config + build image
  aibox apply --no-cache                     Force full rebuild without cache
  aibox apply --standardize-config           Rewrite aibox.toml into current canonical shape
  aibox up                                   Start and attach
  aibox up --layout focus                    Start with a specific layout
  aibox get runtime                          Show container state
  aibox set theme.mode dark --apply          Switch runtime UI theme mode
  aibox get addon                            List available add-ons
  aibox doctor                               Validate project structure
  aibox self update --check                  Check for newer versions"
);

#[derive(Parser)]
#[command(
    name = "aibox",
    about = "Manage AI-ready development container environments",
    long_about = LONG_ABOUT,
    version
)]
pub struct Cli {
    /// Path to aibox.toml (default: ./aibox.toml)
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, global = true, env = "AIBOX_LOG_LEVEL", default_value = "info")]
    pub log_level: String,

    /// Skip all confirmation prompts (like apt-get -y)
    #[arg(short = 'y', long, global = true)]
    pub yes: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new project with aibox.toml and generated files
    ///
    /// Creates aibox.toml, generates .devcontainer/ files, scaffolds
    /// context directory, seeds .aibox-home/ with default configs, and
    /// sets up .gitignore.
    ///
    /// Without flags, runs interactively. With all flags, runs non-interactively.
    Init {
        /// Project name (default: current directory name)
        name: Option<String>,

        /// Base image (default: debian)
        #[arg(long, value_enum)]
        base: Option<BaseImage>,

        /// Usage profile (default: human-dev)
        #[arg(long, value_enum)]
        profile: Option<AiboxProfile>,

        /// Deprecated: processkit package tier. New scaffolds use explicit standard skills.
        #[arg(long = "context", visible_alias = "package", num_args = 1.., hide = true)]
        process: Option<Vec<String>>,

        /// Project context mode (default: processkit)
        #[arg(long = "context-mode", value_enum)]
        context_mode: Option<ContextMode>,

        /// AI harnesses to configure (default: claude)
        #[arg(long = "harness", value_enum, num_args = 1..)]
        ai: Option<Vec<AiProvider>>,

        /// Container user (default: aibox)
        #[arg(long)]
        user: Option<String>,

        /// Theme family for all tools (default: gruvbox)
        #[arg(long, value_enum)]
        theme: Option<ThemeFamily>,

        /// Starship prompt preset (default: default)
        #[arg(long, value_enum)]
        prompt: Option<StarshipPreset>,

        /// tmux status presentation (default: extended)
        #[arg(long = "tmux-status", value_enum)]
        tmux_status: Option<TmuxStatusMode>,

        /// Addon names to enable (e.g., python, infrastructure, kubernetes).
        /// Each selected addon's `requires` are auto-added transitively
        /// (e.g. selecting `docs-docusaurus` also pulls in `node`).
        #[arg(long = "addon", num_args = 1..)]
        addons: Option<Vec<String>>,

        /// Pin a specific tool version inside an addon. Repeatable.
        /// Format: `addon:tool=version`. Examples:
        ///
        ///   --addon-tool python:python=3.14 --addon-tool node:pnpm=10
        ///
        /// Overrides the addon's default version and skips the
        /// interactive version picker for that tool.
        #[arg(long = "addon-tool")]
        addon_tool: Vec<String>,

        /// processkit source URL (default: projectious-work/processkit upstream).
        /// Use this to point at a fork or a compatible alternative repo.
        #[arg(long)]
        processkit_source: Option<String>,

        /// processkit version tag to pin. If omitted, aibox lists the
        /// available versions at the source and (interactively) lets you
        /// pick one or (non-interactively) defaults to the latest.
        #[arg(long)]
        processkit_version: Option<String>,

        /// Include prerelease processkit tags in automatic and interactive
        /// version selection. Explicit --processkit-version prerelease pins
        /// always work without this flag.
        #[arg(long)]
        include_prerelease: bool,

        /// processkit branch override. Tracks the moving HEAD of a branch
        /// instead of a pinned tag — discouraged for production use, fine
        /// for testing pre-release work. Mutually informative with
        /// `--processkit-version`: when both are set the branch wins at
        /// fetch time but the version is still recorded in aibox.toml.
        #[arg(long)]
        processkit_branch: Option<String>,

        /// Skip all container-runtime interaction (runtime probe + image
        /// build). Primarily useful for CI and nested devcontainer tests.
        /// Also settable via `AIBOX_NO_CONTAINER=1`.
        #[arg(
            long,
            env = "AIBOX_NO_CONTAINER",
            value_parser = parse_truthy_flag,
            num_args = 0..=1,
            default_missing_value = "true",
            default_value = "false",
        )]
        no_container: bool,
    },
    /// Reconcile project state with aibox.toml desired state
    ///
    /// Seeds config files, regenerates .devcontainer/ files, runs the
    /// processkit content diff, and builds the container image. The
    /// primary command for applying any config change.
    Apply {
        /// Optional resource to apply instead of the whole project
        #[arg(value_enum)]
        resource: Option<ApplyResource>,

        /// Resource name, currently used by `apply migration <id>`
        name: Option<String>,

        /// Force a full image rebuild without using cached layers
        #[arg(long = "no-cache", visible_alias = "rebuild")]
        no_cache: bool,

        /// Skip the container image build step (config-only apply)
        #[arg(long)]
        config_only: bool,

        /// Force a canonical rewrite of aibox.toml after compatibility
        /// migrations. Recognized settings are preserved through the current
        /// schema; unknown keys still block so they are not silently dropped.
        #[arg(long)]
        standardize_config: bool,

        /// PulseAudio TCP port for `apply audio`
        #[arg(long, default_value = "4714")]
        port: Option<u16>,

        /// Rewrite the compliance-contract block in AGENTS.md from the
        /// canonical source at
        /// `context/skills/processkit/skill-gate/assets/compliance-contract.md`.
        /// Used to silence the "compliance contract in AGENTS.md differs
        /// from the canonical source" warning emitted by apply. If the block
        /// uses `pk-compliance-contract v1` markers but the canonical
        /// source is v2, markers are migrated to v2 as part of the fix.
        #[arg(long)]
        fix_compliance_contract: bool,

        /// Skip all container-runtime interaction (runtime probe + image
        /// build). All file scaffolding still runs — useful inside dev
        /// containers where building sub-containers is wasteful (E2E
        /// tests, CI). Distinct from `--config-only`, which still probes the
        /// runtime. Also settable via `AIBOX_NO_CONTAINER=1`.
        #[arg(
            long,
            env = "AIBOX_NO_CONTAINER",
            value_parser = parse_truthy_flag,
            num_args = 0..=1,
            default_missing_value = "true",
            default_value = "false",
        )]
        no_container: bool,
    },
    /// Inspect compiled v1 orchestration intent without changing project state
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Plan a v1 deployment without writing files or contacting a runtime
    Deploy {
        #[command(subcommand)]
        action: DeployAction,
    },
    /// Inspect or explicitly resolve the immutable image consumed by a v1 deployment.
    Image {
        #[command(subcommand)]
        action: ImageAction,
    },
    /// Connect to a named v1 deployment service.
    Connect(ConnectArgs),
    /// Apply the v1 deployment; use `connect` separately to open a session.
    ///
    /// The legacy attach behavior is available only through `--legacy-runtime`
    /// during the v1 prerelease transition and is scheduled for removal on
    /// 2026-12-31.
    Up {
        /// Use the deprecated v0 container runtime path (removed 2026-12-31).
        #[arg(long)]
        legacy_runtime: bool,

        /// tmux layout to use with --legacy-runtime (dev, focus, cowork, ai)
        #[arg(long, value_enum)]
        layout: Option<Layout>,

        /// Reconcile generated v0 state before attaching; requires --legacy-runtime.
        #[arg(long, requires = "legacy_runtime")]
        apply: bool,

        /// Recreate the tmux layout; requires --legacy-runtime.
        #[arg(long, requires = "legacy_runtime")]
        forget_tmux_state: bool,
    },
    /// Recover into the workspace without tmux, Yazi, or status tooling
    Emergency {
        /// AI harness to launch after printing the emergency briefing
        #[arg(value_enum)]
        harness: AiHarness,
    },
    /// Reclaim aibox-managed disk usage
    ///
    /// Reports reclaimable build caches, runtime-home caches, provider
    /// worktrees, and E2E companion storage by default. Pass `--yes` to
    /// apply the selected cleanup scope.
    Prune {
        /// Cleanup scope to inspect or apply
        #[arg(value_enum)]
        scope: Option<PruneScope>,

        /// Additional cleanup scope to inspect or apply; repeat for multiple scopes
        #[arg(long = "scope", value_enum)]
        scopes: Vec<PruneScope>,

        /// Preview only, even when --yes is supplied
        #[arg(long)]
        dry_run: bool,

        /// Emit structured JSON for automation
        #[arg(long)]
        json: bool,

        /// Apply the selected cleanup
        #[arg(long)]
        yes: bool,
    },
    /// Destroy the v1 deployment guarded by its ownership record.
    ///
    /// The deprecated v0 stop behavior requires --legacy-runtime and is
    /// scheduled for removal on 2026-12-31.
    Down {
        /// Use the deprecated v0 container runtime path (removed 2026-12-31).
        #[arg(long)]
        legacy_runtime: bool,
    },
    /// List compact state for a resource
    Get {
        /// Resource to list
        #[arg(value_enum)]
        resource: GetResource,

        /// For `get runtime`: include cgroup/procfs memory and process pressure
        #[arg(long)]
        resources: bool,

        /// Show all available items when supported
        #[arg(long)]
        all: bool,

        /// Filter skill listings by category
        #[arg(long)]
        category: Option<String>,

        /// Output format
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "table"
        )]
        format: OutputFormat,
    },
    /// Show detailed state for a resource
    Describe {
        /// Resource to inspect
        #[arg(value_enum)]
        resource: DescribeResource,

        /// Resource name when required
        name: Option<String>,

        /// Output format
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "table"
        )]
        format: OutputFormat,
    },
    /// Change a project setting or enable a configurable resource
    ///
    /// Examples:
    ///   aibox set theme.mode dark --apply
    ///   aibox set theme.name tokyo-night --apply
    ///   aibox set addon python enabled --apply
    ///   aibox set skill model-recommender-route enabled
    Set {
        /// Setting path or resource name
        target: String,

        /// First value or resource instance name
        value: Option<String>,

        /// Additional values for resource-shaped settings
        extra: Vec<String>,

        /// Reconcile desired state after changing config
        #[arg(long)]
        apply: bool,

        /// Restart and attach only the project tmux session for theme changes
        #[arg(long)]
        restart_session: bool,
    },
    /// Open an editable project resource
    Edit {
        /// Resource to edit
        #[arg(value_enum)]
        resource: EditResource,
    },
    /// Reset an explicit resource to a clean state
    Reset {
        /// Resource to reset
        #[arg(value_enum)]
        resource: ResetResource,

        /// processkit version to use as the target for context reset planning
        #[arg(long)]
        from_processkit: Option<String>,
        /// Skip backup — permanently delete without saving
        #[arg(long)]
        no_backup: bool,
        /// Preview what would happen without modifying anything
        #[arg(long)]
        dry_run: bool,
        /// Skip the confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// Delete an explicit resource
    Delete {
        /// Resource to delete
        #[arg(value_enum)]
        resource: DeleteResource,

        /// Resource name when required
        name: Option<String>,

        /// Reason for rejecting a migration
        #[arg(long)]
        reason: Option<String>,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,

        /// Reconcile desired state after changing config
        #[arg(long)]
        apply: bool,
    },
    /// Validate context structure and produce migration artifacts
    ///
    /// Checks: config validity, container runtime, .aibox-home/ directories,
    /// .devcontainer/ files, context structure, .gitignore entries, and
    /// schema version. Generates migration artifacts when versions differ.
    ///
    /// With `--integrity`: skip the legacy doctor and run only the
    /// install-integrity check (cheap, scriptable). Exits non-zero on
    /// any non-Healthy / non-NotInstalled outcome.
    Doctor {
        /// Optional diagnostic target
        #[arg(value_enum)]
        target: Option<DoctorTarget>,

        /// Run only the install-integrity check. Cheap, scriptable.
        #[arg(long)]
        integrity: bool,
        /// Emit JSON instead of human output. Implies --integrity-only
        /// behaviour for now (the legacy doctor doesn't yet support JSON).
        #[arg(long, short = 'o', visible_alias = "output", value_enum)]
        format: Option<OutputFormat>,
    },
    /// Create a resource snapshot or saved environment
    Create {
        #[command(subcommand)]
        action: CreateAction,
    },
    /// Manage the aibox binary and shell integration
    #[command(name = "self")]
    SelfCmd {
        #[command(subcommand)]
        action: SelfAction,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum ConfigAction {
    /// Validate and compile orchestration intent into a deterministic plan
    Compile {
        /// Output format
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "human"
        )]
        format: CompileOutputFormat,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum DeployAction {
    /// Render backend artifacts from validated v1 orchestration intent
    Plan {
        /// Output format
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "human"
        )]
        format: DeployOutputFormat,
    },
    /// Reconcile the rendered v1 deployment using the selected backend.
    Apply {
        /// Output format
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "human"
        )]
        format: DeployOutputFormat,
    },
    /// Read and classify the current runtime state of the v1 deployment.
    Status {
        /// Output format
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "human"
        )]
        format: DeployOutputFormat,
    },
    /// Remove only resources proven to belong to the recorded deployment.
    Destroy {
        /// Output format
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "human"
        )]
        format: DeployOutputFormat,
    },
    /// Print backend logs, optionally restricted to one service.
    Logs {
        #[arg(long)]
        service: Option<String>,
        /// Output format
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "human"
        )]
        format: DeployOutputFormat,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum ImageAction {
    /// Validate and explicitly resolve the immutable image consumed by a deployment.
    Build {
        /// Output format
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "human"
        )]
        format: DeployOutputFormat,
    },
    /// Show the immutable image selected by orchestration configuration.
    Inspect {
        /// Output format
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "human"
        )]
        format: DeployOutputFormat,
    },
}

/// Connect to a named v1 orchestration connection target.
#[derive(Clone, Debug, Args)]
pub struct ConnectArgs {
    /// Name from [[orchestration.connections]].
    pub name: String,
    /// Override the configured command. Values after `--` are passed as argv.
    #[arg(last = true)]
    pub command: Vec<String>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ApplyResource {
    /// Configure host audio support
    Audio,
    /// Refresh only git-tracked generated runtime surfaces
    GeneratedRuntime,
    /// Apply a specific processkit migration
    Migration,
    /// Switch to a saved environment
    Env,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum GetResource {
    Runtime,
    Addon,
    Env,
    Kit,
    Skill,
    SkillCategory,
    Process,
    Migration,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DescribeResource {
    Runtime,
    Addon,
    AddonCatalog,
    ImageProvenancePolicy,
    ProviderBackends,
    WorkspaceManifest,
    Env,
    Kit,
    Skill,
    Process,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum EditResource {
    Config,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ResetResource {
    Project,
    Context,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DeleteResource {
    Runtime,
    Addon,
    Skill,
    Env,
    Migration,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DoctorTarget {
    Project,
    Audio,
    Security,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum PruneScope {
    /// Conservative local cleanup: Rust incremental cache and managed runtime caches
    Safe,
    /// Rust debug incremental cache
    BuildCache,
    /// Managed runtime-home caches
    RuntimeHome,
    /// Provider-created Git worktrees under .claude/worktrees
    AgentWorktrees,
    /// Local aibox-owned container cleanup when available
    Containers,
    /// Nested E2E companion containers, workspaces, images, and volumes
    E2eCompanion,
    /// All known cleanup scopes, including provider worktrees
    All,
}

impl std::fmt::Display for PruneScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            PruneScope::Safe => "safe",
            PruneScope::BuildCache => "build-cache",
            PruneScope::RuntimeHome => "runtime-home",
            PruneScope::AgentWorktrees => "agent-worktrees",
            PruneScope::Containers => "containers",
            PruneScope::E2eCompanion => "e2e-companion",
            PruneScope::All => "all",
        };
        write!(f, "{value}")
    }
}

#[derive(Subcommand)]
pub enum CreateAction {
    /// Save current project state as a named environment
    Env {
        /// Environment name
        name: String,
    },
    /// Back up aibox files to a timestamped directory
    Backup {
        /// Output directory for backup (default: .aibox/backup/)
        #[arg(long)]
        output_dir: Option<String>,
        /// Preview what would be backed up without copying
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum SelfAction {
    /// Check for or apply version updates
    Update {
        /// Only check versions, don't apply any changes
        #[arg(long)]
        check: bool,
        /// Preview what would change without writing files
        #[arg(long)]
        dry_run: bool,
    },
    /// Generate shell completion script
    Completion {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Uninstall the aibox CLI binary
    Uninstall {
        /// Preview what would be removed without deleting anything
        #[arg(long)]
        dry_run: bool,
        /// Also remove global config and cache (~/.aibox/)
        #[arg(long)]
        purge: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up_accepts_forget_tmux_state_flag() {
        let cli = Cli::parse_from(["aibox", "up", "--legacy-runtime", "--forget-tmux-state"]);
        match cli.command {
            Commands::Up {
                forget_tmux_state, ..
            } => assert!(forget_tmux_state),
            _ => panic!("expected up command"),
        }
    }

    #[test]
    fn v1_up_rejects_legacy_attach_flags_without_explicit_escape_hatch() {
        assert!(Cli::try_parse_from(["aibox", "up", "--forget-tmux-state"]).is_err());
    }

    #[test]
    fn host_latex_commands_are_not_exposed() {
        assert!(Cli::try_parse_from(["aibox", "latex", "build", "overview"]).is_err());
        assert!(Cli::try_parse_from(["aibox", "preview", "latex", "overview"]).is_err());
    }
}
