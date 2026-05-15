use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::cli::PruneScope;
use crate::output;

pub struct PruneOptions {
    pub positional_scope: Option<PruneScope>,
    pub scopes: Vec<PruneScope>,
    pub dry_run: bool,
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct PruneReport {
    dry_run: bool,
    applied: bool,
    scopes: Vec<String>,
    total_reclaimable_bytes: u64,
    total_removed_bytes: u64,
    actions: Vec<PruneAction>,
}

#[derive(Debug, Serialize)]
struct PruneAction {
    scope: String,
    label: String,
    status: PruneActionStatus,
    destructive: bool,
    path: Option<String>,
    size_bytes: u64,
    command: Option<Vec<String>>,
    stdout: Option<String>,
    stderr: Option<String>,
    notes: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum PruneActionStatus {
    WouldRemove,
    Removed,
    Missing,
    Skipped,
    Inspected,
}

pub fn cmd_prune(options: PruneOptions) -> Result<()> {
    let scopes = selected_scopes(options.positional_scope, options.scopes);
    let report = run_prune(&scopes, options.dry_run)?;

    if options.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        emit_human_report(&report);
    }

    Ok(())
}

fn selected_scopes(
    positional_scope: Option<PruneScope>,
    scopes: Vec<PruneScope>,
) -> Vec<PruneScope> {
    let mut requested = Vec::new();
    if let Some(scope) = positional_scope {
        requested.push(scope);
    }
    requested.extend(scopes);
    if requested.is_empty() {
        requested.push(PruneScope::Safe);
    }

    let mut seen = BTreeSet::new();
    let mut selected = Vec::new();
    for scope in requested {
        let expanded = match scope {
            PruneScope::Safe => vec![PruneScope::BuildCache, PruneScope::RuntimeHome],
            PruneScope::All => vec![
                PruneScope::BuildCache,
                PruneScope::RuntimeHome,
                PruneScope::AgentWorktrees,
                PruneScope::Containers,
                PruneScope::E2eCompanion,
            ],
            other => vec![other],
        };
        for item in expanded {
            let slug = scope_slug(&item).to_string();
            if seen.insert(slug) {
                selected.push(item);
            }
        }
    }
    selected
}

fn run_prune(scopes: &[PruneScope], dry_run: bool) -> Result<PruneReport> {
    let mut actions = Vec::new();
    for scope in scopes {
        match scope {
            PruneScope::Safe | PruneScope::All => unreachable!("composite scopes are expanded"),
            PruneScope::BuildCache => {
                actions.push(remove_dir_scope(
                    PruneScope::BuildCache,
                    "Rust debug incremental cache",
                    Path::new("cli/target/debug/incremental"),
                    dry_run,
                )?);
            }
            PruneScope::RuntimeHome => {
                actions.push(remove_dir_scope(
                    PruneScope::RuntimeHome,
                    "PowerKit render cache",
                    Path::new(".aibox-home/.cache/tmux-powerkit"),
                    dry_run,
                )?);
                actions.push(remove_dir_scope(
                    PruneScope::RuntimeHome,
                    "runtime diagnostics ring",
                    Path::new(".aibox/diagnostics"),
                    dry_run,
                )?);
            }
            PruneScope::AgentWorktrees => actions.extend(prune_agent_worktrees(dry_run)?),
            PruneScope::Containers => actions.push(prune_containers()),
            PruneScope::E2eCompanion => actions.push(prune_e2e_companion(dry_run)?),
        }
    }

    let total_reclaimable_bytes = actions
        .iter()
        .filter(|action| matches!(action.status, PruneActionStatus::WouldRemove))
        .map(|action| action.size_bytes)
        .sum();
    let total_removed_bytes = actions
        .iter()
        .filter(|action| matches!(action.status, PruneActionStatus::Removed))
        .map(|action| action.size_bytes)
        .sum();

    Ok(PruneReport {
        dry_run,
        applied: !dry_run,
        scopes: scopes
            .iter()
            .map(|scope| scope_slug(scope).to_string())
            .collect(),
        total_reclaimable_bytes,
        total_removed_bytes,
        actions,
    })
}

fn remove_dir_scope(
    scope: PruneScope,
    label: &str,
    path: &Path,
    dry_run: bool,
) -> Result<PruneAction> {
    if !path.exists() {
        return Ok(PruneAction {
            scope: scope_slug(&scope).to_string(),
            label: label.to_string(),
            status: PruneActionStatus::Missing,
            destructive: false,
            path: Some(path.display().to_string()),
            size_bytes: 0,
            command: None,
            stdout: None,
            stderr: None,
            notes: vec!["nothing to prune".to_string()],
        });
    }

    let size = dir_size(path).unwrap_or(0);
    if dry_run {
        return Ok(PruneAction {
            scope: scope_slug(&scope).to_string(),
            label: label.to_string(),
            status: PruneActionStatus::WouldRemove,
            destructive: true,
            path: Some(path.display().to_string()),
            size_bytes: size,
            command: None,
            stdout: None,
            stderr: None,
            notes: Vec::new(),
        });
    }

    fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
    Ok(PruneAction {
        scope: scope_slug(&scope).to_string(),
        label: label.to_string(),
        status: PruneActionStatus::Removed,
        destructive: true,
        path: Some(path.display().to_string()),
        size_bytes: size,
        command: None,
        stdout: None,
        stderr: None,
        notes: Vec::new(),
    })
}

fn prune_agent_worktrees(dry_run: bool) -> Result<Vec<PruneAction>> {
    let worktrees = git_worktrees()?;

    if worktrees.is_empty() {
        return Ok(vec![PruneAction {
            scope: scope_slug(&PruneScope::AgentWorktrees).to_string(),
            label: "agent worktrees".to_string(),
            status: PruneActionStatus::Skipped,
            destructive: false,
            path: None,
            size_bytes: 0,
            command: None,
            stdout: None,
            stderr: None,
            notes: vec!["nothing to prune".to_string()],
        }]);
    }

    let mut actions = Vec::new();
    for path in &worktrees {
        let size = dir_size(path).unwrap_or(0);
        if dry_run {
            actions.push(PruneAction {
                scope: scope_slug(&PruneScope::AgentWorktrees).to_string(),
                label: "agent worktree".to_string(),
                status: PruneActionStatus::WouldRemove,
                destructive: true,
                path: Some(path.display().to_string()),
                size_bytes: size,
                command: Some(vec![
                    "git".to_string(),
                    "worktree".to_string(),
                    "remove".to_string(),
                    "--force".to_string(),
                    "--force".to_string(),
                    path.to_string_lossy().to_string(),
                ]),
                stdout: None,
                stderr: None,
                notes: Vec::new(),
            });
        } else {
            let command = vec![
                "git".to_string(),
                "worktree".to_string(),
                "remove".to_string(),
                "--force".to_string(),
                "--force".to_string(),
                path.to_string_lossy().to_string(),
            ];
            let status = Command::new("git")
                .args(&command[1..])
                .status()
                .with_context(|| format!("failed to remove worktree {}", path.display()))?;
            if !status.success() {
                bail!("git worktree remove failed for {}", path.display());
            }
            actions.push(PruneAction {
                scope: scope_slug(&PruneScope::AgentWorktrees).to_string(),
                label: "agent worktree".to_string(),
                status: PruneActionStatus::Removed,
                destructive: true,
                path: Some(path.display().to_string()),
                size_bytes: size,
                command: Some(command),
                stdout: None,
                stderr: None,
                notes: Vec::new(),
            });
        }
    }

    if !dry_run {
        let _ = Command::new("git")
            .args(["worktree", "prune", "--verbose"])
            .status();
    }

    Ok(actions)
}

fn prune_containers() -> PruneAction {
    PruneAction {
        scope: scope_slug(&PruneScope::Containers).to_string(),
        label: "local containers".to_string(),
        status: PruneActionStatus::Skipped,
        destructive: false,
        path: None,
        size_bytes: 0,
        command: None,
        stdout: None,
        stderr: None,
        notes: vec![
            "container cleanup is host/runtime dependent; use aibox down or host container tooling"
                .to_string(),
        ],
    }
}

fn git_worktrees() -> Result<Vec<PathBuf>> {
    let current_dir = std::env::current_dir()
        .context("failed to resolve current directory")?
        .to_string_lossy()
        .to_string();
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()
        .context("failed to list git worktrees")?;
    if !output.status.success() {
        bail!("git worktree list failed");
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .filter(|path| *path != current_dir.as_str())
        .map(PathBuf::from)
        .filter(|path| path.to_string_lossy().contains(".claude/worktrees"))
        .collect())
}

fn prune_e2e_companion(dry_run: bool) -> Result<PruneAction> {
    let key = Path::new(".aibox-e2e-runner-home/.ssh/id_ed25519");
    if !key.exists() {
        return Ok(PruneAction {
            scope: scope_slug(&PruneScope::E2eCompanion).to_string(),
            label: "E2E companion".to_string(),
            status: PruneActionStatus::Skipped,
            destructive: false,
            path: None,
            size_bytes: 0,
            command: None,
            stdout: None,
            stderr: None,
            notes: vec!["E2E companion SSH key missing; skipping companion prune".to_string()],
        });
    }

    let host = std::env::var("E2E_HOST").unwrap_or_else(|_| "aibox-e2e-testrunner".to_string());
    let port = std::env::var("E2E_PORT").unwrap_or_else(|_| "22".to_string());
    let remote = if dry_run {
        "echo 'podman system df'; podman system df; echo; echo 'container storage'; du -sh /home/testuser/.local/share/containers/storage 2>/dev/null || true; echo; echo 'workspaces'; du -sh /workspaces/* 2>/dev/null | sort -h | tail -20 || true"
    } else {
        "ids=$(podman ps -aq); if [ -n \"$ids\" ]; then podman rm -f $ids; fi; podman system prune -af --volumes; find /workspaces -mindepth 1 -maxdepth 1 -exec rm -rf {} + 2>/dev/null || sudo find /workspaces -mindepth 1 -maxdepth 1 -exec rm -rf {} +; podman system df; du -sh /home/testuser/.local/share/containers/storage 2>/dev/null || true"
    };

    let command = vec![
        "ssh".to_string(),
        "-i".to_string(),
        key.to_string_lossy().to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=no".to_string(),
        "-o".to_string(),
        "UserKnownHostsFile=/dev/null".to_string(),
        "-o".to_string(),
        "ConnectTimeout=5".to_string(),
        "-o".to_string(),
        "LogLevel=ERROR".to_string(),
        "-p".to_string(),
        port,
        format!("testuser@{host}"),
        remote.to_string(),
    ];
    let completed = Command::new("ssh")
        .args(&command[1..])
        .stdin(Stdio::null())
        .output()
        .context("failed to run E2E companion prune over SSH")?;
    if !completed.status.success() {
        bail!("E2E companion prune failed");
    }

    Ok(PruneAction {
        scope: scope_slug(&PruneScope::E2eCompanion).to_string(),
        label: "E2E companion".to_string(),
        status: if dry_run {
            PruneActionStatus::Inspected
        } else {
            PruneActionStatus::Removed
        },
        destructive: !dry_run,
        path: None,
        size_bytes: 0,
        command: Some(command),
        stdout: Some(String::from_utf8_lossy(&completed.stdout).to_string()),
        stderr: Some(String::from_utf8_lossy(&completed.stderr).to_string()),
        notes: Vec::new(),
    })
}

fn emit_human_report(report: &PruneReport) {
    if report.dry_run {
        output::info("Prune dry run; pass --yes to delete the selected scope");
    } else {
        output::info("Pruning selected aibox-managed state");
    }

    for action in &report.actions {
        match action.status {
            PruneActionStatus::WouldRemove => output::info(&format!(
                "{}: would remove {} ({})",
                action.label,
                action.path.as_deref().unwrap_or("(no path)"),
                format_bytes(action.size_bytes)
            )),
            PruneActionStatus::Removed => output::ok(&format!(
                "{}: removed {} ({})",
                action.label,
                action.path.as_deref().unwrap_or("(no path)"),
                format_bytes(action.size_bytes)
            )),
            PruneActionStatus::Missing | PruneActionStatus::Skipped => {
                let note = action
                    .notes
                    .first()
                    .map(String::as_str)
                    .unwrap_or("nothing to prune");
                output::ok(&format!("{}: {note}", action.label));
            }
            PruneActionStatus::Inspected => {
                output::info(&format!("{}: inspecting remote storage", action.label));
                if let Some(stdout) = &action.stdout {
                    print!("{stdout}");
                }
                if let Some(stderr) = &action.stderr {
                    eprint!("{stderr}");
                }
            }
        }
    }
}

fn scope_slug(scope: &PruneScope) -> &'static str {
    match scope {
        PruneScope::Safe => "safe",
        PruneScope::BuildCache => "build-cache",
        PruneScope::RuntimeHome => "runtime-home",
        PruneScope::AgentWorktrees => "agent-worktrees",
        PruneScope::Containers => "containers",
        PruneScope::E2eCompanion => "e2e-companion",
        PruneScope::All => "all",
    }
}

fn dir_size(path: &Path) -> Result<u64> {
    let meta = fs::symlink_metadata(path)?;
    if meta.is_file() {
        return Ok(meta.len());
    }
    if !meta.is_dir() {
        return Ok(0);
    }

    let mut total = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        total += dir_size(&entry.path()).unwrap_or(0);
    }
    Ok(total)
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slugs(scopes: Vec<PruneScope>) -> Vec<String> {
        scopes
            .iter()
            .map(|scope| scope_slug(scope).to_string())
            .collect()
    }

    #[test]
    fn selected_scopes_defaults_to_safe_expansion() {
        assert_eq!(
            slugs(selected_scopes(None, Vec::new())),
            vec!["build-cache", "runtime-home"]
        );
    }

    #[test]
    fn selected_scopes_accepts_repeatable_scope_flags() {
        assert_eq!(
            slugs(selected_scopes(
                None,
                vec![PruneScope::RuntimeHome, PruneScope::BuildCache]
            )),
            vec!["runtime-home", "build-cache"]
        );
    }

    #[test]
    fn selected_scopes_expands_all_without_duplicates() {
        assert_eq!(
            slugs(selected_scopes(
                Some(PruneScope::Safe),
                vec![PruneScope::RuntimeHome, PruneScope::All]
            )),
            vec![
                "build-cache",
                "runtime-home",
                "agent-worktrees",
                "containers",
                "e2e-companion"
            ]
        );
    }
}
