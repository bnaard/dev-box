mod addon_cmd;
mod addon_loader;
#[allow(dead_code)]
mod addon_registry;
pub mod compat;
mod compose_plan;
#[allow(dead_code)]
mod deployment_backend;
#[allow(dead_code)]
mod deployment_compiler;
#[allow(dead_code)]
mod deployment_contract;
mod dirs;
mod kit;
pub mod kubernetes_connection;
mod kubernetes_ownership;
mod kubernetes_plan;
mod latex;

mod addons;
mod audio;
mod audit;
mod cli;
mod compliance;
mod config;
mod container;
#[allow(dead_code)]
mod content_diff;
mod content_init;
#[allow(dead_code)]
mod content_install;
mod content_migration;
#[allow(dead_code)]
mod content_source;
mod context;
mod docs_install;
mod doctor;
mod env;
mod generate;
mod harness_commands;
mod hook_registration;
mod image_provenance;
mod integrity;
#[allow(dead_code)]
mod lock;
mod log;
mod mcp_registration;
mod migration;
mod model_migration;
#[allow(dead_code)]
mod orchestration_compile;
mod output;
mod preauth;
#[allow(dead_code)]
mod processkit_protocol;
mod processkit_vocab;
mod provider_backend;
mod prune;
mod release_evidence;
mod reset;
mod runtime;
mod runtime_home;
mod runtime_resources;
mod runtime_sync;
mod seed;
mod sync_perimeter;
mod theme_cmd;
mod themes;
mod tmux;
mod update;
mod v1_release_readiness;
mod v1_v2_migration;
mod version_resolve;
mod workspace_manifest;

use clap::{CommandFactory, Parser, ValueEnum};
use std::path::Path;
use tracing_subscriber::EnvFilter;

fn main() {
    let cli = cli::Cli::parse();

    let filter = EnvFilter::try_new(&cli.log_level).unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let result = dispatch(cli);

    if let Err(e) = result {
        output::error(&format!("{:#}", e));
        std::process::exit(1);
    }
}

fn dispatch(cli: cli::Cli) -> anyhow::Result<()> {
    // Initialize addon definitions from YAML files.
    // Commands that don't need addons (completions, help) still work if this fails.
    if let Err(e) = addon_loader::init() {
        // Only fail for commands that actually need addons
        match &cli.command {
            cli::Commands::SelfCmd {
                action: cli::SelfAction::Completion { .. },
            } => {} // doesn't need addons
            cli::Commands::Config { .. }
            | cli::Commands::Deploy { .. }
            | cli::Commands::Image { .. }
            | cli::Commands::Connect(_) => {} // v1 orchestration does not need addons
            _ => {
                output::error(&format!("Failed to load addon definitions: {:#}", e));
                std::process::exit(1);
            }
        }
    }

    let config_path = &cli.config;
    let global_yes = cli.yes;

    match cli.command {
        cli::Commands::Init {
            name,
            base,
            profile,
            process,
            context_mode,
            ai,
            user,
            theme,
            prompt,
            tmux_status,
            addons,
            addon_tool,
            processkit_source,
            processkit_version,
            include_prerelease,
            processkit_branch,
            no_container,
        } => {
            let timer = crate::log::LogTimer::start("init");
            let result = container::cmd_init(
                config_path,
                container::InitParams {
                    name,
                    base,
                    profile,
                    process,
                    context_mode,
                    ai,
                    user,
                    theme,
                    prompt,
                    tmux_status,
                    addons,
                    addon_tool,
                    processkit_source,
                    processkit_version,
                    include_prerelease,
                    processkit_branch,
                    no_container,
                },
            );
            timer.finish(
                Path::new("."),
                if result.is_ok() { 0 } else { 1 },
                if result.is_ok() {
                    "init completed"
                } else {
                    "init failed"
                },
            );
            result
        }
        cli::Commands::Apply {
            resource,
            name,
            no_cache,
            config_only,
            standardize_config,
            port,
            fix_compliance_contract,
            no_container,
        } => {
            let timer = crate::log::LogTimer::start("apply");
            let result = match resource {
                None => container::cmd_sync(
                    config_path,
                    no_cache,
                    config_only,
                    standardize_config,
                    fix_compliance_contract,
                    no_container,
                ),
                Some(cli::ApplyResource::Audio) => audio::cmd_audio_setup(port),
                Some(cli::ApplyResource::GeneratedRuntime) => {
                    container::cmd_apply_generated_runtime(config_path)
                }
                Some(cli::ApplyResource::Migration) => {
                    let cwd = std::env::current_dir()?;
                    let id = required_name(name, "migration id")?;
                    content_migration::cmd_migrate_apply(&cwd, &id)
                }
                Some(cli::ApplyResource::Env) => {
                    let env_name = required_name(name, "environment name")?;
                    env::cmd_env_switch(config_path, &env_name, global_yes)
                }
            };
            timer.finish(
                Path::new("."),
                if result.is_ok() { 0 } else { 1 },
                if result.is_ok() {
                    "apply completed"
                } else {
                    "apply failed"
                },
            );
            result
        }
        cli::Commands::Config { action } => match action {
            cli::ConfigAction::Compile { format } => {
                orchestration_compile::cmd_config_compile(config_path, format)
            }
            cli::ConfigAction::MigrateV1 {
                apply,
                intent_file,
                restore,
                format,
            } => v1_release_readiness::cmd_config_migrate_v1(
                config_path,
                apply,
                intent_file.as_deref(),
                restore.as_deref(),
                format,
            ),
            cli::ConfigAction::ReleaseReadiness { format } => {
                v1_release_readiness::cmd_release_readiness(config_path, format)
            }
        },
        cli::Commands::Deploy { action } => match action {
            cli::DeployAction::Plan { format } => {
                orchestration_compile::cmd_deploy_plan(config_path, format)
            }
            cli::DeployAction::Apply { format } => {
                orchestration_compile::cmd_deploy_apply(config_path, format)
            }
            cli::DeployAction::Status { format } => {
                orchestration_compile::cmd_deploy_status(config_path, format)
            }
            cli::DeployAction::Destroy { format } => {
                orchestration_compile::cmd_deploy_destroy(config_path, format)
            }
            cli::DeployAction::Logs { service, format } => {
                orchestration_compile::cmd_deploy_logs(config_path, service, format)
            }
        },
        cli::Commands::Image { action } => match action {
            cli::ImageAction::Build { format, push } => {
                orchestration_compile::cmd_image_build(config_path, format, push)
            }
            cli::ImageAction::Inspect { format } => {
                orchestration_compile::cmd_image_inspect(config_path, format)
            }
        },
        cli::Commands::Connect(args) => {
            orchestration_compile::cmd_connect(config_path, &args.name, args.command)
        }
        cli::Commands::Up {
            legacy_runtime,
            layout,
            apply,
            forget_tmux_state,
        } => {
            if !legacy_runtime {
                if layout.is_some() || apply || forget_tmux_state {
                    anyhow::bail!(
                        "`aibox up` applies the v1 deployment and does not attach. Use `aibox connect <name>` after it completes. The legacy attach flags require `aibox up --legacy-runtime` (removed 2026-12-31)."
                    );
                }
                output::info(
                    "Applying v1 deployment; use `aibox connect <name>` to open a session after it completes.",
                );
                return orchestration_compile::cmd_deploy_apply(
                    config_path,
                    cli::DeployOutputFormat::Human,
                );
            }
            output::warn(
                "`aibox up --legacy-runtime` is deprecated and will be removed on 2026-12-31; migrate to `aibox up` plus `aibox connect`.",
            );
            if apply {
                container::cmd_sync(config_path, false, false, false, false, false)?;
            }
            let config = crate::config::AiboxConfig::from_cli_option(config_path)?;
            let resolved_layout = layout
                .map(|l| l.to_string())
                .unwrap_or_else(|| config.customization.tmux_layout().to_string());
            let timer = crate::log::LogTimer::start("up");
            let result =
                container::cmd_start(config_path, &resolved_layout, apply || forget_tmux_state);
            timer.finish(
                Path::new("."),
                if result.is_ok() { 0 } else { 1 },
                if result.is_ok() {
                    "up completed"
                } else {
                    "up failed"
                },
            );
            result
        }
        cli::Commands::Emergency { harness } => {
            let timer = crate::log::LogTimer::start("emergency");
            let result = container::cmd_emergency(config_path, harness);
            timer.finish(
                Path::new("."),
                if result.is_ok() { 0 } else { 1 },
                if result.is_ok() {
                    "emergency completed"
                } else {
                    "emergency failed"
                },
            );
            result
        }
        cli::Commands::Prune {
            scope,
            scopes,
            dry_run,
            json,
            yes,
        } => {
            let timer = crate::log::LogTimer::start("prune");
            let result = prune::cmd_prune(prune::PruneOptions {
                positional_scope: scope,
                scopes,
                dry_run: dry_run || !yes,
                json,
            });
            timer.finish(
                Path::new("."),
                if result.is_ok() { 0 } else { 1 },
                if result.is_ok() {
                    "prune completed"
                } else {
                    "prune failed"
                },
            );
            result
        }
        cli::Commands::Down { legacy_runtime } => {
            if !legacy_runtime {
                output::info("Destroying v1 deployment with ownership safeguards...");
                return orchestration_compile::cmd_deploy_destroy(
                    config_path,
                    cli::DeployOutputFormat::Human,
                );
            }
            output::warn(
                "`aibox down --legacy-runtime` is deprecated and will be removed on 2026-12-31; migrate to `aibox down`.",
            );
            let config = crate::config::AiboxConfig::from_cli_option(config_path)?;
            latex::cleanup_legacy_host_previews(config_path, &config)?;
            container::cmd_stop(config_path)
        }
        cli::Commands::Get {
            resource,
            resources,
            all,
            category,
            format,
        } => match resource {
            cli::GetResource::Runtime if resources => {
                runtime_resources::cmd_runtime_resources(format)
            }
            cli::GetResource::Runtime => container::cmd_status(config_path, format),
            cli::GetResource::Addon => addon_cmd::cmd_addon_list(config_path, format),
            cli::GetResource::Env => env::cmd_env_list(format),
            cli::GetResource::Kit => kit::cmd_kit_list(config_path, format),
            cli::GetResource::Skill => {
                kit::cmd_kit_skill_list(config_path, category.as_deref(), all, format)
            }
            cli::GetResource::SkillCategory => kit::cmd_kit_skill_categories(config_path, format),
            cli::GetResource::Process => kit::cmd_kit_process_list(config_path, all, format),
            cli::GetResource::Migration => {
                let cwd = std::env::current_dir()?;
                content_migration::cmd_migrate_continue(&cwd)
            }
        },
        cli::Commands::Describe {
            resource,
            name,
            format,
        } => match resource {
            cli::DescribeResource::Runtime => container::cmd_status(config_path, format),
            cli::DescribeResource::Addon => {
                let addon = required_name(name, "add-on name")?;
                addon_cmd::cmd_addon_info(&addon, format)
            }
            cli::DescribeResource::AddonCatalog => addon_cmd::cmd_addon_catalog(format),
            cli::DescribeResource::ImageProvenancePolicy => {
                image_provenance::cmd_image_provenance_policy(config_path, format)
            }
            cli::DescribeResource::ProviderBackends => {
                provider_backend::cmd_provider_backends(config_path, format)
            }
            cli::DescribeResource::WorkspaceManifest => {
                workspace_manifest::cmd_workspace_manifest(config_path, format)
            }
            cli::DescribeResource::Env => env::cmd_env_status(config_path),
            cli::DescribeResource::Kit => kit::cmd_kit_list(config_path, format),
            cli::DescribeResource::Skill => {
                let skill = required_name(name, "skill name")?;
                kit::cmd_kit_skill_info(config_path, &skill, format)
            }
            cli::DescribeResource::Process => {
                let process = required_name(name, "process name")?;
                kit::cmd_kit_process_info(config_path, &process, format)
            }
        },
        cli::Commands::Set {
            target,
            value,
            extra,
            apply,
            restart_session,
        } => cmd_set(config_path, &target, value, extra, apply, restart_session),
        cli::Commands::Edit { resource } => match resource {
            cli::EditResource::Config => edit_config(config_path),
        },
        cli::Commands::Reset {
            resource,
            from_processkit,
            no_backup,
            dry_run,
            yes,
        } => match resource {
            cli::ResetResource::Project => {
                let timer = crate::log::LogTimer::start("reset-project");
                let result = reset::cmd_reset(config_path, no_backup, dry_run, yes || global_yes);
                timer.finish(
                    Path::new("."),
                    if result.is_ok() { 0 } else { 1 },
                    if result.is_ok() {
                        "reset project completed"
                    } else {
                        "reset project failed"
                    },
                );
                result
            }
            cli::ResetResource::Context => {
                let timer = crate::log::LogTimer::start("reset-context");
                let result = reset::cmd_reset_context_plan(
                    config_path,
                    from_processkit.as_deref(),
                    no_backup,
                    dry_run,
                    yes || global_yes,
                );
                timer.finish(
                    Path::new("."),
                    if result.is_ok() { 0 } else { 1 },
                    if result.is_ok() {
                        "reset context plan completed"
                    } else {
                        "reset context plan failed"
                    },
                );
                result
            }
        },
        cli::Commands::Delete {
            resource,
            name,
            reason,
            yes,
            apply,
        } => match resource {
            cli::DeleteResource::Runtime => container::cmd_remove(config_path),
            cli::DeleteResource::Addon => {
                let addon = required_name(name, "add-on name")?;
                addon_cmd::cmd_addon_remove(config_path, &addon, apply, false)
            }
            cli::DeleteResource::Skill => {
                let skill = required_name(name, "skill name")?;
                kit::cmd_kit_skill_uninstall(config_path, &skill)?;
                if apply {
                    container::cmd_sync(config_path, false, false, false, false, false)?;
                }
                Ok(())
            }
            cli::DeleteResource::Env => {
                let env_name = required_name(name, "environment name")?;
                env::cmd_env_delete(&env_name, yes || global_yes)
            }
            cli::DeleteResource::Migration => {
                let cwd = std::env::current_dir()?;
                let id = required_name(name, "migration id")?;
                let reason = required_name(reason, "rejection reason (--reason)")?;
                content_migration::cmd_migrate_reject(&cwd, &id, &reason)
            }
        },
        cli::Commands::Doctor {
            target,
            integrity,
            format,
        } => {
            if matches!(target, Some(cli::DoctorTarget::Audio)) {
                audio::cmd_audio_check(Some(4714))
            } else if matches!(target, Some(cli::DoctorTarget::Security)) {
                audit::cmd_audit(config_path)
            } else if integrity {
                let cwd = std::env::current_dir()?;
                integrity::cmd_doctor_integrity(&cwd, format.clone())
            } else {
                doctor::cmd_doctor(config_path)
            }
        }
        cli::Commands::Create { action } => match action {
            cli::CreateAction::Env { name } => env::cmd_env_create(config_path, &name),
            cli::CreateAction::Backup {
                output_dir,
                dry_run,
            } => reset::cmd_backup(config_path, output_dir, dry_run),
        },
        cli::Commands::SelfCmd { action } => match action {
            cli::SelfAction::Update { check, dry_run } => {
                update::cmd_update(config_path, check, dry_run, global_yes)
            }
            cli::SelfAction::Completion { shell } => {
                let mut cmd = cli::Cli::command();
                let bin_name = cmd.get_name().to_string();
                clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
                Ok(())
            }
            cli::SelfAction::Uninstall { dry_run, purge } => {
                reset::cmd_uninstall(dry_run, purge, global_yes)
            }
        },
    }
}

fn required_name(value: Option<String>, label: &str) -> anyhow::Result<String> {
    value.ok_or_else(|| anyhow::anyhow!("missing {label}"))
}

fn cmd_set(
    config_path: &Option<String>,
    target: &str,
    value: Option<String>,
    extra: Vec<String>,
    apply: bool,
    restart_session: bool,
) -> anyhow::Result<()> {
    match target {
        "theme.mode" => {
            let raw = required_name(value, "theme mode")?;
            let mode = crate::config::ThemeMode::from_str(&raw, true)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            theme_cmd::cmd_theme(config_path, mode, None, restart_session)
        }
        "theme.name" | "theme" => {
            let raw = required_name(value, "theme name")?;
            let theme = crate::config::ThemeFamily::from_str(&raw, true)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let config = crate::config::AiboxConfig::from_cli_option(config_path)?;
            theme_cmd::cmd_theme(
                config_path,
                config.customization.mode,
                Some(theme),
                restart_session,
            )
        }
        "addon" => {
            let addon = required_name(value, "add-on name")?;
            let state = extra.first().map(String::as_str).unwrap_or("enabled");
            match state {
                "enabled" | "enable" | "true" => {
                    addon_cmd::cmd_addon_add(config_path, &addon, apply, false)
                }
                "disabled" | "disable" | "false" => {
                    addon_cmd::cmd_addon_remove(config_path, &addon, apply, false)
                }
                _ => anyhow::bail!("expected addon state 'enabled' or 'disabled'"),
            }
        }
        "skill" => {
            let skill = required_name(value, "skill name")?;
            let state = extra.first().map(String::as_str).unwrap_or("enabled");
            match state {
                "enabled" | "enable" | "true" => kit::cmd_kit_skill_install(config_path, &skill)?,
                "disabled" | "disable" | "false" => {
                    kit::cmd_kit_skill_uninstall(config_path, &skill)?
                }
                _ => anyhow::bail!("expected skill state 'enabled' or 'disabled'"),
            }
            if apply {
                container::cmd_sync(config_path, false, false, false, false, false)?;
            }
            Ok(())
        }
        "migration" => {
            let id = required_name(value, "migration id")?;
            let state = extra.first().map(String::as_str).unwrap_or("in-progress");
            let cwd = std::env::current_dir()?;
            match state {
                "in-progress" | "started" | "start" => {
                    content_migration::cmd_migrate_start(&cwd, &id)
                }
                "applied" | "apply" => content_migration::cmd_migrate_apply(&cwd, &id),
                _ => anyhow::bail!("expected migration state 'in-progress' or 'applied'"),
            }
        }
        _ => anyhow::bail!(
            "unsupported setting '{target}'. Try theme.mode, theme.name, addon, skill, or migration"
        ),
    }
}

fn edit_config(config_path: &Option<String>) -> anyhow::Result<()> {
    let path = config_path.as_deref().unwrap_or("aibox.toml");
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = std::process::Command::new(&editor).arg(path).status()?;
    if !status.success() {
        anyhow::bail!("editor exited with status {status}");
    }
    Ok(())
}
