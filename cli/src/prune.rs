use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::cli::PruneScope;
use crate::output;

pub fn cmd_prune(scope: PruneScope, dry_run: bool) -> Result<()> {
    if dry_run {
        output::info("Prune dry run; pass --yes to delete the selected scope");
    } else {
        output::info("Pruning selected aibox-managed state");
    }

    match scope {
        PruneScope::Safe => {
            prune_build_cache(dry_run)?;
            prune_runtime_home(dry_run)?;
        }
        PruneScope::BuildCache => prune_build_cache(dry_run)?,
        PruneScope::RuntimeHome => prune_runtime_home(dry_run)?,
        PruneScope::AgentWorktrees => prune_agent_worktrees(dry_run)?,
        PruneScope::E2eCompanion => prune_e2e_companion(dry_run)?,
        PruneScope::All => {
            prune_build_cache(dry_run)?;
            prune_runtime_home(dry_run)?;
            prune_agent_worktrees(dry_run)?;
            prune_e2e_companion(dry_run)?;
        }
    }

    Ok(())
}

fn prune_build_cache(dry_run: bool) -> Result<()> {
    remove_dir_scope(
        "Rust debug incremental cache",
        Path::new("cli/target/debug/incremental"),
        dry_run,
    )
}

fn prune_runtime_home(dry_run: bool) -> Result<()> {
    remove_dir_scope(
        "PowerKit render cache",
        Path::new(".aibox-home/.cache/tmux-powerkit"),
        dry_run,
    )?;
    remove_dir_scope(
        "runtime diagnostics ring",
        Path::new(".aibox/diagnostics"),
        dry_run,
    )
}

fn remove_dir_scope(label: &str, path: &Path, dry_run: bool) -> Result<()> {
    if !path.exists() {
        output::ok(&format!("{label}: nothing to prune"));
        return Ok(());
    }

    let size = dir_size(path).unwrap_or(0);
    if dry_run {
        output::info(&format!(
            "{label}: would remove {} ({})",
            path.display(),
            format_bytes(size)
        ));
        return Ok(());
    }

    fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
    output::ok(&format!(
        "{label}: removed {} ({})",
        path.display(),
        format_bytes(size)
    ));
    Ok(())
}

fn prune_agent_worktrees(dry_run: bool) -> Result<()> {
    let worktrees = git_worktrees()?
        .into_iter()
        .filter(|path| path.to_string_lossy().contains(".claude/worktrees"))
        .collect::<Vec<_>>();

    if worktrees.is_empty() {
        output::ok("agent worktrees: nothing to prune");
        return Ok(());
    }

    for path in &worktrees {
        let size = dir_size(path).unwrap_or(0);
        if dry_run {
            output::info(&format!(
                "agent worktree: would remove {} ({})",
                path.display(),
                format_bytes(size)
            ));
        } else {
            let status = Command::new("git")
                .args([
                    "worktree".to_string(),
                    "remove".to_string(),
                    "--force".to_string(),
                    "--force".to_string(),
                    path.to_string_lossy().to_string(),
                ])
                .status()
                .with_context(|| format!("failed to remove worktree {}", path.display()))?;
            if !status.success() {
                bail!("git worktree remove failed for {}", path.display());
            }
            output::ok(&format!("agent worktree: removed {}", path.display()));
        }
    }

    if !dry_run {
        let _ = Command::new("git")
            .args(["worktree", "prune", "--verbose"])
            .status();
    }

    Ok(())
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

fn prune_e2e_companion(dry_run: bool) -> Result<()> {
    let key = Path::new(".aibox-e2e-runner-home/.ssh/id_ed25519");
    if !key.exists() {
        output::warn("E2E companion SSH key missing; skipping companion prune");
        return Ok(());
    }

    let host = std::env::var("E2E_HOST").unwrap_or_else(|_| "aibox-e2e-testrunner".to_string());
    let port = std::env::var("E2E_PORT").unwrap_or_else(|_| "22".to_string());
    let remote = if dry_run {
        "echo 'podman system df'; podman system df; echo; echo 'container storage'; du -sh /home/testuser/.local/share/containers/storage 2>/dev/null || true; echo; echo 'workspaces'; du -sh /workspaces/* 2>/dev/null | sort -h | tail -20 || true"
    } else {
        "ids=$(podman ps -aq); if [ -n \"$ids\" ]; then podman rm -f $ids; fi; podman system prune -af --volumes; find /workspaces -mindepth 1 -maxdepth 1 -exec rm -rf {} + 2>/dev/null || sudo find /workspaces -mindepth 1 -maxdepth 1 -exec rm -rf {} +; podman system df; du -sh /home/testuser/.local/share/containers/storage 2>/dev/null || true"
    };

    output::info(if dry_run {
        "E2E companion: inspecting remote storage"
    } else {
        "E2E companion: pruning nested runtime storage"
    });

    let args = vec![
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
    let status = Command::new("ssh")
        .args(args)
        .stdin(Stdio::null())
        .status()
        .context("failed to run E2E companion prune over SSH")?;
    if !status.success() {
        bail!("E2E companion prune failed");
    }
    Ok(())
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
