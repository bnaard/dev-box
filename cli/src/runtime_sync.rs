//! Three-way migration flow for managed `.aibox-home/` runtime files.
//!
//! The runtime config under `.aibox-home/` is user-editable. Sync/start may
//! scaffold missing directories, but they must not overwrite file edits. This
//! module mirrors the processkit content diff model:
//!
//! - reference: the last generated runtime baseline snapshot for the previous
//!   aibox CLI version
//! - generated: the files this CLI version would generate now
//! - live: the user's current `.aibox-home/` files
//!
//! User-relevant upstream changes are surfaced as Migration documents under
//! `context/migrations/pending/`; live files are never overwritten here.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::content_diff::{DiffSummary, FileClassification, classify};
use crate::lock::RuntimeHomeLockSection;

const RUNTIME_SOURCE: &str = "aibox-runtime-home";
const RUNTIME_SOURCE_URL: &str = "aibox://runtime-home";
const RUNTIME_TEMPLATES_DIR: &str = "context/templates/aibox-home";

#[derive(Debug, Clone)]
pub struct RuntimeFileDiff {
    pub rel_path: String,
    pub project_path: PathBuf,
    pub classification: FileClassification,
}

#[derive(Debug, Clone)]
pub struct RuntimeSyncReport {
    pub summary: DiffSummary,
    pub migration_document_path: Option<PathBuf>,
}

pub fn templates_dir_for_version(project_root: &Path, version: &str) -> PathBuf {
    project_root.join(RUNTIME_TEMPLATES_DIR).join(version)
}

pub fn copy_runtime_templates(
    project_root: &Path,
    version: &str,
    config: &crate::config::AiboxConfig,
) -> Result<()> {
    let dest = templates_dir_for_version(project_root, version);
    if dest.exists() {
        fs::remove_dir_all(&dest).with_context(|| {
            format!(
                "failed to clear stale runtime templates dir {}",
                dest.display()
            )
        })?;
    }
    fs::create_dir_all(&dest)
        .with_context(|| format!("failed to create runtime templates dir {}", dest.display()))?;

    for (rel_path, content) in crate::seed::managed_runtime_files(config) {
        let target = dest.join(&rel_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&target, content)
            .with_context(|| format!("failed to write runtime template {}", target.display()))?;
    }

    Ok(())
}

pub fn refresh_runtime_home_template_lock(
    project_root: &Path,
    config: &crate::config::AiboxConfig,
) -> Result<()> {
    let generated_hashes: BTreeMap<String, String> = crate::seed::managed_runtime_files(config)
        .into_iter()
        .map(|(path, content)| {
            (
                path.to_string_lossy().replace('\\', "/"),
                sha256_of_bytes(content.as_bytes()),
            )
        })
        .collect();
    refresh_runtime_home_lock(project_root, generated_hashes)
}

pub fn run_runtime_sync(
    project_root: &Path,
    from_version: Option<&str>,
    to_version: &str,
    config: &crate::config::AiboxConfig,
) -> Result<RuntimeSyncReport> {
    let mut diffs = if let Some(from_version) = from_version {
        three_way_diff(project_root, from_version, config)?
    } else {
        Vec::new()
    };

    // Auto-apply ChangedUpstreamOnly files: the user hasn't touched them,
    // so it's safe to overwrite with the new generated content (e.g. theme
    // changed in aibox.toml). Also auto-apply NewUpstream files.
    let generated = crate::seed::managed_runtime_files(config);
    let generated_map: BTreeMap<String, String> = generated
        .into_iter()
        .map(|(p, c)| (p.to_string_lossy().replace('\\', "/"), c))
        .collect();
    let generated_hashes: BTreeMap<String, String> = generated_map
        .iter()
        .map(|(path, content)| (path.clone(), sha256_of_bytes(content.as_bytes())))
        .collect();
    let host_root = config.host_root_dir();
    let mut auto_applied = 0usize;
    let same_version_sync = from_version.is_some_and(|version| version == to_version);
    let previous_hashes = crate::lock::read_lock(project_root)
        .ok()
        .flatten()
        .and_then(|lock| lock.runtime_home)
        .map(|section| section.files)
        .unwrap_or_default();

    for diff in &mut diffs {
        let live_matches_previous_generated =
            previous_hashes
                .get(&diff.rel_path)
                .is_some_and(|previous_sha| {
                    live_file_matches(&host_root, &diff.rel_path, previous_sha)
                });
        let should_update_same_version_generated_file = same_version_sync
            && (live_matches_previous_generated
                || zellij_layout_contains_unselected_harness(&host_root, &diff.rel_path, config));
        let should_update_stale_managed_zellij_file =
            live_matches_historical_managed_zellij_file(project_root, &host_root, &diff.rel_path);
        let should_update_stale_managed_runtime_helper =
            live_matches_historical_managed_runtime_helper(
                project_root,
                &host_root,
                &diff.rel_path,
            );
        let should_update_stale_managed_yazi_file =
            live_matches_historical_managed_yazi_file(project_root, &host_root, &diff.rel_path);
        let should_update_legacy_managed_yazi_init =
            live_matches_legacy_managed_yazi_init(&host_root, &diff.rel_path);
        let should_restore_missing_zellij_status_toggle_layout = same_version_sync
            && zellij_status_toggle_layout_relpath(&diff.rel_path)
            && !host_root.join(&diff.rel_path).is_file();
        let should_restore_missing_managed_runtime_helper = same_version_sync
            && managed_runtime_helper_relpath(&diff.rel_path)
            && !host_root.join(&diff.rel_path).is_file();

        if ((same_version_sync
            && matches!(
                diff.classification,
                FileClassification::ChangedUpstreamOnly | FileClassification::NewUpstream
            ))
            || should_update_stale_managed_zellij_file
            || should_update_stale_managed_runtime_helper
            || should_update_stale_managed_yazi_file
            || should_update_legacy_managed_yazi_init
            || should_restore_missing_zellij_status_toggle_layout
            || should_restore_missing_managed_runtime_helper)
            && let Some(content) = generated_map.get(&diff.rel_path)
        {
            let target = host_root.join(&diff.rel_path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&target, content).with_context(|| {
                format!("failed to auto-apply runtime file {}", target.display())
            })?;
            ensure_live_runtime_file_permissions(&diff.rel_path, &target)?;
            auto_applied += 1;
            diff.classification = FileClassification::Unchanged;
        } else if matches!(
            diff.classification,
            FileClassification::ChangedLocallyOnly | FileClassification::Conflict
        ) && should_update_same_version_generated_file
            && let Some(content) = generated_map.get(&diff.rel_path)
        {
            let target = host_root.join(&diff.rel_path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&target, content).with_context(|| {
                format!(
                    "failed to provenance-update runtime file {}",
                    target.display()
                )
            })?;
            ensure_live_runtime_file_permissions(&diff.rel_path, &target)?;
            auto_applied += 1;
            diff.classification = FileClassification::Unchanged;
        }
    }
    if auto_applied > 0 {
        crate::output::ok(&format!(
            "Auto-applied {} unchanged runtime file(s) with upstream updates",
            auto_applied,
        ));
    }

    let summary = summarize(&diffs);

    // For cross-version jumps, enumerate every intermediate snapshot on disk
    // and compute hop-by-hop deltas against the NEXT step (so the reviewer
    // can see what each release introduced, even if later releases reverted
    // it). Empty when no intermediates are on disk or versions don't parse.
    let intermediate_hops: Vec<IntermediateHop> = if let Some(from_version) = from_version {
        build_intermediate_hops(project_root, from_version, to_version)
    } else {
        Vec::new()
    };

    let migration_document_path =
        if summary.has_user_relevant_changes() || !intermediate_hops.is_empty() {
            write_migration_document(
                project_root,
                from_version.unwrap_or("unknown"),
                to_version,
                &summary,
                &diffs,
                &intermediate_hops,
            )?
        } else {
            None
        };

    copy_runtime_templates(project_root, to_version, config)?;
    refresh_runtime_home_lock(project_root, generated_hashes)?;

    Ok(RuntimeSyncReport {
        summary,
        migration_document_path,
    })
}

fn ensure_live_runtime_file_permissions(rel_path: &str, target: &Path) -> Result<()> {
    if rel_path == ".local/bin/pdf-watch"
        || rel_path == ".local/bin/open-in-editor"
        || rel_path == ".local/bin/aibox-status-toggle"
    {
        ensure_executable(target)?;
    }
    Ok(())
}

#[cfg(unix)]
fn ensure_executable(path: &Path) -> Result<()> {
    let mut permissions = fs::metadata(path)
        .with_context(|| format!("failed to read permissions for {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("failed to chmod +x {}", path.display()))
}

#[cfg(not(unix))]
fn ensure_executable(_path: &Path) -> Result<()> {
    Ok(())
}

fn three_way_diff(
    project_root: &Path,
    from_version: &str,
    config: &crate::config::AiboxConfig,
) -> Result<Vec<RuntimeFileDiff>> {
    let reference_dir = templates_dir_for_version(project_root, from_version);
    let host_root = config.host_root_dir();
    let generated = crate::seed::managed_runtime_files(config);
    let mut diffs = Vec::new();

    for (rel_path, content) in generated {
        let rel_str = rel_path.to_string_lossy().replace('\\', "/");
        let project_path = PathBuf::from(".aibox-home").join(&rel_path);
        let live_abs = host_root.join(&rel_path);
        let reference_abs = reference_dir.join(&rel_path);
        let generated_sha = sha256_of_bytes(content.as_bytes());
        let live_sha = if live_abs.is_file() {
            Some(crate::lock::sha256_of_file(&live_abs).with_context(|| {
                format!("failed to hash live runtime file {}", live_abs.display())
            })?)
        } else {
            None
        };
        let reference_sha = if reference_abs.is_file() {
            Some(
                crate::lock::sha256_of_file(&reference_abs).with_context(|| {
                    format!(
                        "failed to hash runtime reference file {}",
                        reference_abs.display()
                    )
                })?,
            )
        } else {
            None
        };
        let classification = classify(
            reference_sha.as_deref(),
            Some(&generated_sha),
            live_sha.as_deref(),
        );
        diffs.push(RuntimeFileDiff {
            rel_path: rel_str,
            project_path,
            classification,
        });
    }

    Ok(diffs)
}

fn parse_semver_triple(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim_start_matches('v');
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

/// List snapshot dirs under `context/templates/aibox-home/` whose directory
/// name is strictly greater than `from` and strictly less than `to` (both
/// exclusive). Returns them in ascending semver order.
///
/// Used by [`run_runtime_sync`] to compute per-intermediate deltas when a
/// project jumps across multiple aibox CLI versions in one sync.
fn intermediate_snapshots(project_root: &Path, from: &str, to: &str) -> Vec<(String, PathBuf)> {
    let base = project_root.join(RUNTIME_TEMPLATES_DIR);
    let (Some(from_v), Some(to_v)) = (parse_semver_triple(from), parse_semver_triple(to)) else {
        return Vec::new();
    };
    if from_v >= to_v {
        return Vec::new();
    }
    let Ok(read) = fs::read_dir(&base) else {
        return Vec::new();
    };
    let mut found: Vec<((u32, u32, u32), String, PathBuf)> = Vec::new();
    for entry in read.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(v) = parse_semver_triple(name) else {
            continue;
        };
        if v > from_v && v < to_v {
            found.push((v, name.to_string(), path));
        }
    }
    found.sort_by_key(|(v, _, _)| *v);
    found.into_iter().map(|(_, n, p)| (n, p)).collect()
}

/// Count how many files differ by content hash between two template
/// snapshot directories. Missing-in-either-side also counts as a change.
/// Returns (changed_count, total_files_considered).
fn snapshot_hop_delta(a_dir: &Path, b_dir: &Path) -> (usize, usize) {
    fn walk(root: &Path) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(read) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in read.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file()
                    && let Ok(rel) = path.strip_prefix(root)
                    && let Ok(sha) = crate::lock::sha256_of_file(&path)
                {
                    let key = rel.to_string_lossy().replace('\\', "/");
                    out.insert(key, sha);
                }
            }
        }
        out
    }

    let a = walk(a_dir);
    let b = walk(b_dir);
    let mut keys: BTreeSet<&String> = a.keys().collect();
    keys.extend(b.keys());
    let total = keys.len();
    let mut changed = 0usize;
    for k in keys {
        if a.get(k) != b.get(k) {
            changed += 1;
        }
    }
    (changed, total)
}

#[derive(Debug, Clone)]
pub struct IntermediateHop {
    pub from: String,
    pub to: String,
    pub changed: usize,
    pub total: usize,
}

fn build_intermediate_hops(project_root: &Path, from: &str, to: &str) -> Vec<IntermediateHop> {
    if from == to {
        return Vec::new();
    }

    let mut hops: Vec<IntermediateHop> = Vec::new();
    let base = project_root.join(RUNTIME_TEMPLATES_DIR);
    let from_dir = base.join(from);
    let to_dir = base.join(to);

    // Collect every version dir on disk between from and to (exclusive both).
    let mids = intermediate_snapshots(project_root, from, to);

    // Build ordered waypoints: from -> m1 -> m2 -> ... -> to. Skip hops whose
    // endpoints are missing on disk so we never compare nonexistent dirs.
    let mut waypoints: Vec<(String, PathBuf)> = Vec::new();
    if from_dir.is_dir() {
        waypoints.push((from.to_string(), from_dir));
    }
    waypoints.extend(mids);
    if to_dir.is_dir() {
        waypoints.push((to.to_string(), to_dir));
    }

    for pair in waypoints.windows(2) {
        let (a_name, a_dir) = &pair[0];
        let (b_name, b_dir) = &pair[1];
        let (changed, total) = snapshot_hop_delta(a_dir, b_dir);
        hops.push(IntermediateHop {
            from: a_name.clone(),
            to: b_name.clone(),
            changed,
            total,
        });
    }
    hops
}

fn summarize(diffs: &[RuntimeFileDiff]) -> DiffSummary {
    let mut summary = DiffSummary::default();
    for diff in diffs {
        match &diff.classification {
            FileClassification::Unchanged => summary.unchanged += 1,
            FileClassification::ChangedLocallyOnly => summary.changed_locally_only += 1,
            FileClassification::ChangedUpstreamOnly => summary.changed_upstream_only += 1,
            FileClassification::Conflict => summary.conflict += 1,
            FileClassification::NewUpstream => summary.new_upstream += 1,
            FileClassification::RemovedUpstream => summary.removed_upstream += 1,
            // Runtime sync (managed runtime files like
            // .config/zellij/...) does not consult older mirrors, so
            // it never produces RemovedUpstreamStale itself. Count it
            // toward the same bucket for parity if it ever appears
            // (e.g. a future caller passes a content_diff result here).
            FileClassification::RemovedUpstreamStale { .. } => summary.removed_upstream_stale += 1,
        }
    }
    summary
}

fn runtime_group_for(rel_path: &str) -> String {
    if rel_path.starts_with(".config/zellij/") {
        return "runtime-zellij".to_string();
    }
    if rel_path.starts_with(".config/yazi/") {
        return "runtime-yazi".to_string();
    }
    if rel_path.starts_with(".config/lazygit/") {
        return "runtime-lazygit".to_string();
    }
    if rel_path.starts_with(".config/git/") {
        return "runtime-git".to_string();
    }
    if rel_path.starts_with(".claude/") {
        return "runtime-claude".to_string();
    }
    if rel_path == ".config/starship.toml" {
        return "runtime-starship".to_string();
    }
    if rel_path == ".vim/vimrc" {
        return "runtime-vim".to_string();
    }
    if rel_path == ".asoundrc" {
        return "runtime-audio".to_string();
    }
    "runtime-misc".to_string()
}

fn sha256_of_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn live_file_matches(host_root: &Path, rel_path: &str, expected_sha: &str) -> bool {
    let live_abs = host_root.join(rel_path);
    live_abs
        .is_file()
        .then(|| crate::lock::sha256_of_file(&live_abs).ok())
        .flatten()
        .is_some_and(|live_sha| live_sha == expected_sha)
}

fn zellij_layout_contains_unselected_harness(
    host_root: &Path,
    rel_path: &str,
    config: &crate::config::AiboxConfig,
) -> bool {
    if !rel_path.starts_with(".config/zellij/layouts/") || !rel_path.ends_with(".kdl") {
        return false;
    }
    let live_abs = host_root.join(rel_path);
    let Ok(content) = fs::read_to_string(live_abs) else {
        return false;
    };
    crate::config::AiProvider::all()
        .iter()
        .filter(|harness| harness.is_active())
        .filter(|harness| !config.ai.harnesses.contains(harness))
        .any(|harness| {
            let command = harness.binary_name();
            !command.is_empty() && content.contains(&format!("command \"{}\"", command))
        })
}

fn live_matches_historical_managed_zellij_file(
    project_root: &Path,
    host_root: &Path,
    rel_path: &str,
) -> bool {
    if !managed_zellij_relpath(rel_path) {
        return false;
    }

    let live_abs = host_root.join(rel_path);
    let Ok(live_content) = fs::read_to_string(&live_abs) else {
        return false;
    };
    if !is_managed_zellij_file(rel_path, &live_content) {
        return false;
    }
    let live_normalized = normalize_historical_zellij_file(&live_content);
    let live_sha = sha256_of_bytes(live_normalized.as_bytes());
    let snapshots_root = project_root.join(RUNTIME_TEMPLATES_DIR);
    let Ok(entries) = fs::read_dir(snapshots_root) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        let snapshot_file = entry.path().join(rel_path);
        let Ok(snapshot_content) = fs::read_to_string(snapshot_file) else {
            return false;
        };
        if !is_managed_zellij_file(rel_path, &snapshot_content) {
            return false;
        }
        let snapshot_normalized = normalize_historical_zellij_file(&snapshot_content);
        sha256_of_bytes(snapshot_normalized.as_bytes()) == live_sha
    })
}

fn managed_zellij_relpath(rel_path: &str) -> bool {
    rel_path == ".config/zellij/config.kdl"
        || (rel_path.starts_with(".config/zellij/layouts/") && rel_path.ends_with(".kdl"))
}

fn zellij_status_toggle_layout_relpath(rel_path: &str) -> bool {
    rel_path == ".config/zellij/layouts/aibox-status-visible.kdl"
        || rel_path == ".config/zellij/layouts/aibox-status-hidden.kdl"
}

fn managed_runtime_helper_relpath(rel_path: &str) -> bool {
    rel_path == ".local/bin/pdf-watch"
        || rel_path == ".local/bin/open-in-editor"
        || rel_path == ".local/bin/aibox-status"
        || rel_path == ".local/bin/aibox-status-toggle"
}

fn managed_yazi_relpath(rel_path: &str) -> bool {
    matches!(
        rel_path,
        ".config/yazi/init.lua"
            | ".config/yazi/yazi.toml"
            | ".config/yazi/keymap.toml"
            | ".config/yazi/theme.toml"
    )
}

fn live_matches_legacy_managed_yazi_init(host_root: &Path, rel_path: &str) -> bool {
    if rel_path != ".config/yazi/init.lua" {
        return false;
    }

    let live_abs = host_root.join(rel_path);
    let Ok(content) = fs::read_to_string(live_abs) else {
        return false;
    };

    let legacy_managed = r#"-- =============================================================================
-- Yazi init.lua — aibox defaults
-- Runs on every Yazi startup. Register plugins that need setup here.
-- =============================================================================

-- git.yazi: show git status in the file list with explicit, visible signs.
-- Fetcher registration is in yazi.toml [plugin.prepend_fetchers].
th.git = th.git or {}
th.git.modified_sign = "M"
th.git.added_sign = "A"
th.git.deleted_sign = "D"
th.git.updated_sign = "U"
th.git.untracked_sign = "?"
th.git.ignored_sign = "I"

require("git"):setup {}

-- status-git.yazi: git branch + summary (left) and disk free (right) in status bar.
-- Data refresh is triggered via the fetcher registered in yazi.toml.
require("status-git"):setup()
"#;

    content == legacy_managed
}

fn live_matches_historical_managed_yazi_file(
    project_root: &Path,
    host_root: &Path,
    rel_path: &str,
) -> bool {
    if !managed_yazi_relpath(rel_path) {
        return false;
    }

    let live_abs = host_root.join(rel_path);
    let Ok(live_content) = fs::read(&live_abs) else {
        return false;
    };
    let live_sha = sha256_of_bytes(&live_content);
    let snapshots_root = project_root.join(RUNTIME_TEMPLATES_DIR);
    let Ok(entries) = fs::read_dir(snapshots_root) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        let snapshot_file = entry.path().join(rel_path);
        let Ok(snapshot_content) = fs::read(snapshot_file) else {
            return false;
        };
        sha256_of_bytes(&snapshot_content) == live_sha
    })
}

fn live_matches_historical_managed_runtime_helper(
    project_root: &Path,
    host_root: &Path,
    rel_path: &str,
) -> bool {
    if !managed_runtime_helper_relpath(rel_path) {
        return false;
    }

    let live_abs = host_root.join(rel_path);
    let Ok(live_content) = fs::read(&live_abs) else {
        return false;
    };
    let live_sha = sha256_of_bytes(&live_content);
    let snapshots_root = project_root.join(RUNTIME_TEMPLATES_DIR);
    let Ok(entries) = fs::read_dir(snapshots_root) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        let snapshot_file = entry.path().join(rel_path);
        let Ok(snapshot_content) = fs::read(snapshot_file) else {
            return false;
        };
        sha256_of_bytes(&snapshot_content) == live_sha
    })
}

fn is_managed_zellij_file(rel_path: &str, content: &str) -> bool {
    if rel_path == ".config/zellij/config.kdl" {
        return content.contains("// aibox zellij configuration");
    }

    content.contains("aibox-status.wasm")
        || content.contains("zellij:status-bar")
        || content.contains("tab_template name=\"aibox-tab\"")
        || content.contains("default_tab_template")
}

fn normalize_historical_zellij_file(content: &str) -> String {
    content.replace(
        "file:/workspace/.aibox-home/.local/share/aibox/zellij/aibox-status.wasm",
        "file:/usr/local/share/aibox/zellij/aibox-status.wasm",
    )
}

fn refresh_runtime_home_lock(
    project_root: &Path,
    generated_hashes: BTreeMap<String, String>,
) -> Result<()> {
    let Some(mut lock) = crate::lock::read_lock(project_root)? else {
        return Ok(());
    };

    if lock
        .runtime_home
        .as_ref()
        .is_some_and(|section| section.files == generated_hashes)
    {
        return Ok(());
    }

    lock.runtime_home = Some(RuntimeHomeLockSection {
        generated_at: chrono::Utc::now().to_rfc3339(),
        files: generated_hashes,
    });
    crate::lock::write_lock(project_root, &lock)
}

fn write_migration_document(
    project_root: &Path,
    from_version: &str,
    to_version: &str,
    summary: &DiffSummary,
    diffs: &[RuntimeFileDiff],
    intermediate_hops: &[IntermediateHop],
) -> Result<Option<PathBuf>> {
    // No-op guard: at `from == to` with no intermediate hops, every
    // "conflict" is a local-only edit that upstream never touched, so no
    // migration is actually needed. Only write when upstream itself moved
    // something or there is a cross-version hop to record.
    if from_version == to_version
        && intermediate_hops.is_empty()
        && !summary.has_upstream_side_changes()
    {
        return Ok(None);
    }

    let pending_dir = project_root.join("context/migrations/pending");
    let in_progress_dir = project_root.join("context/migrations/in-progress");
    if existing_migration_matches(&pending_dir, from_version, to_version)?
        || existing_migration_matches(&in_progress_dir, from_version, to_version)?
    {
        return Ok(None);
    }

    fs::create_dir_all(&pending_dir)
        .with_context(|| format!("failed to create {}", pending_dir.display()))?;

    let now = chrono::Utc::now();
    let now_iso = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let id = format!("MIG-RUNTIME-{}", now.format("%Y%m%dT%H%M%S"));
    let out_path = pending_dir.join(format!("{}.md", id));

    let mut affected_groups = BTreeSet::new();
    for diff in diffs {
        if diff.classification != FileClassification::Unchanged {
            affected_groups.insert(runtime_group_for(&diff.rel_path));
        }
    }

    let summary_line = format!(
        "{} changed upstream, {} conflicts, {} new, {} removed ({} groups affected)",
        summary.changed_upstream_only,
        summary.conflict,
        summary.new_upstream,
        summary.removed_upstream,
        affected_groups.len(),
    );

    let mut body = String::new();
    body.push_str("---\n");
    body.push_str("apiVersion: processkit.projectious.work/v1\n");
    body.push_str("kind: Migration\n");
    body.push_str("metadata:\n");
    body.push_str(&format!("  id: {}\n", id));
    body.push_str(&format!("  created: {}\n", now_iso));
    body.push_str("spec:\n");
    body.push_str(&format!("  source: {}\n", yaml_scalar(RUNTIME_SOURCE)));
    body.push_str(&format!(
        "  source_url: {}\n",
        yaml_scalar(RUNTIME_SOURCE_URL)
    ));
    body.push_str(&format!("  from_version: {}\n", yaml_scalar(from_version)));
    body.push_str(&format!("  to_version: {}\n", yaml_scalar(to_version)));
    body.push_str("  state: pending\n");
    body.push_str("  generated_by: aibox apply\n");
    body.push_str(&format!("  generated_at: {}\n", now_iso));
    body.push_str(&format!("  summary: {}\n", yaml_scalar(&summary_line)));
    body.push_str("  affected_groups:\n");
    if affected_groups.is_empty() {
        body.push_str("    []\n");
    } else {
        for group in &affected_groups {
            body.push_str(&format!("    - {}\n", yaml_scalar(group)));
        }
    }
    body.push_str("---\n\n");
    body.push_str(&format!("# Migration {}\n\n", id));
    body.push_str(&format!(
        "Managed `.aibox-home/` runtime changes from `{}` to `{}`.\n\n",
        from_version, to_version
    ));
    body.push_str(&format!("{}\n\n", summary_line));
    body.push_str("## Counts\n\n");
    body.push_str(&format!("- unchanged: {}\n", summary.unchanged));
    body.push_str(&format!(
        "- changed-locally-only: {}\n",
        summary.changed_locally_only
    ));
    body.push_str(&format!(
        "- changed-upstream-only: {}\n",
        summary.changed_upstream_only
    ));
    body.push_str(&format!("- conflict: {}\n", summary.conflict));
    body.push_str(&format!("- new-upstream: {}\n", summary.new_upstream));
    body.push_str(&format!(
        "- removed-upstream: {}\n\n",
        summary.removed_upstream
    ));

    let mut by_group: BTreeMap<String, BTreeMap<&'static str, Vec<&RuntimeFileDiff>>> =
        BTreeMap::new();
    for diff in diffs {
        if diff.classification == FileClassification::Unchanged {
            continue;
        }
        by_group
            .entry(runtime_group_for(&diff.rel_path))
            .or_default()
            .entry(diff.classification.label())
            .or_default()
            .push(diff);
    }

    if !intermediate_hops.is_empty() {
        body.push_str("## Per-intermediate review\n\n");
        body.push_str(
            "Every released version between `from_version` and `to_version` whose              template snapshot is present on disk is listed below, with the number              of files that changed between consecutive snapshots. Useful for              catching scaffolding changes that were introduced and later reverted              across the span of a multi-version upgrade.\n\n",
        );
        for hop in intermediate_hops {
            body.push_str(&format!(
                "- `{}` → `{}`: {} file(s) changed of {}\n",
                hop.from, hop.to, hop.changed, hop.total
            ));
        }
        body.push('\n');
    }

    if by_group.is_empty() {
        body.push_str("_No user-relevant changes._\n");
    } else {
        body.push_str("## Changes by group\n\n");
        for (group, by_class) in &by_group {
            body.push_str(&format!("### {}\n\n", group));
            for (cls, entries) in by_class {
                body.push_str(&format!("**{}**\n\n", cls));
                for diff in entries {
                    body.push_str(&format!(
                        "- `.aibox-home/{}` -> `{}`\n",
                        diff.rel_path,
                        diff.project_path.display()
                    ));
                }
                body.push('\n');
            }
        }
    }

    fs::write(&out_path, body)
        .with_context(|| format!("failed to write {}", out_path.display()))?;
    Ok(Some(out_path))
}

fn existing_migration_matches(dir: &Path, from_version: &str, to_version: &str) -> Result<bool> {
    if !dir.is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("MIG-") || !name.ends_with(".md") {
            continue;
        }
        let Ok(body) = fs::read_to_string(&path) else {
            continue;
        };
        if let Some((source_url, from, to)) = extract_migration_pair(&body)
            && source_url == RUNTIME_SOURCE_URL
            && from == from_version
            && to == to_version
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn extract_migration_pair(body: &str) -> Option<(String, String, String)> {
    let rest = body.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let frontmatter = &rest[..end];
    let mut source_url = None;
    let mut from_version = None;
    let mut to_version = None;
    for line in frontmatter.lines() {
        let trimmed = line.trim_start();
        if let Some(v) = trimmed.strip_prefix("source_url:") {
            source_url = Some(parse_yaml_scalar_value(v.trim()));
        } else if let Some(v) = trimmed.strip_prefix("from_version:") {
            from_version = Some(parse_yaml_scalar_value(v.trim()));
        } else if let Some(v) = trimmed.strip_prefix("to_version:") {
            to_version = Some(parse_yaml_scalar_value(v.trim()));
        }
    }
    Some((source_url?, from_version?, to_version?))
}

fn parse_yaml_scalar_value(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        let inner = &s[1..s.len() - 1];
        return inner.replace("\\\"", "\"").replace("\\\\", "\\");
    }
    s.to_string()
}

fn yaml_scalar(s: &str) -> String {
    let needs_quote = s.is_empty()
        || s.contains(':')
        || s.contains('#')
        || s.contains('\n')
        || s.contains('"')
        || s.starts_with(' ')
        || s.ends_with(' ')
        || s.starts_with('-')
        || s.starts_with('[')
        || s.starts_with('{');
    if !needs_quote {
        return s.to_string();
    }
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_snapshot(base: &Path, version: &str, files: &[(&str, &str)]) {
        let dir = base.join(RUNTIME_TEMPLATES_DIR).join(version);
        fs::create_dir_all(&dir).unwrap();
        for (rel, body) in files {
            let target = dir.join(rel);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&target, body).unwrap();
        }
    }

    #[test]
    fn intermediate_snapshots_orders_ascending_and_excludes_endpoints() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        for v in ["0.17.20", "0.18.0", "0.18.1", "0.18.2", "0.18.3"] {
            write_snapshot(root, v, &[(".vim/vimrc", v)]);
        }
        let got: Vec<String> = intermediate_snapshots(root, "0.17.20", "0.18.3")
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        assert_eq!(got, vec!["0.18.0", "0.18.1", "0.18.2"]);
    }

    #[test]
    fn build_intermediate_hops_counts_changed_files_per_hop() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // from: 1 file. 0.18.0 changes the file. 0.18.1 adds a new file. to: removes the new file again.
        write_snapshot(root, "0.17.20", &[(".vim/vimrc", "A")]);
        write_snapshot(root, "0.18.0", &[(".vim/vimrc", "B")]);
        write_snapshot(root, "0.18.1", &[(".vim/vimrc", "B"), (".asoundrc", "X")]);
        write_snapshot(root, "0.18.2", &[(".vim/vimrc", "B")]);

        let hops = build_intermediate_hops(root, "0.17.20", "0.18.2");
        // Expected hops: 0.17.20 -> 0.18.0 (1 changed), 0.18.0 -> 0.18.1 (1 added), 0.18.1 -> 0.18.2 (1 removed).
        let summary: Vec<(String, String, usize)> = hops
            .iter()
            .map(|h| (h.from.clone(), h.to.clone(), h.changed))
            .collect();
        assert_eq!(
            summary,
            vec![
                ("0.17.20".to_string(), "0.18.0".to_string(), 1),
                ("0.18.0".to_string(), "0.18.1".to_string(), 1),
                ("0.18.1".to_string(), "0.18.2".to_string(), 1),
            ]
        );
    }

    #[test]
    fn build_intermediate_hops_empty_when_no_snapshots_on_disk() {
        let tmp = TempDir::new().unwrap();
        let hops = build_intermediate_hops(tmp.path(), "0.17.20", "0.18.2");
        assert!(hops.is_empty());
    }

    #[test]
    fn build_intermediate_hops_empty_for_same_version() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write_snapshot(root, "0.22.0", &[(".config/git/config", "A")]);
        let hops = build_intermediate_hops(root, "0.22.0", "0.22.0");
        assert!(hops.is_empty());
    }

    #[test]
    fn build_intermediate_hops_empty_for_bad_versions() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write_snapshot(root, "0.18.0", &[(".vim/vimrc", "A")]);
        let hops = build_intermediate_hops(root, "bogus", "0.18.2");
        assert!(hops.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn runtime_status_toggle_helper_stays_executable_after_auto_apply() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join(".local/bin/aibox-status-toggle");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, "#!/bin/sh\n").unwrap();

        ensure_live_runtime_file_permissions(".local/bin/aibox-status-toggle", &target).unwrap();

        assert_ne!(
            fs::metadata(&target).unwrap().permissions().mode() & 0o111,
            0,
            "aibox-status-toggle should remain executable after runtime auto-apply"
        );
    }

    #[test]
    fn write_migration_document_skips_when_same_version_and_only_conflicts() {
        // Regression guard: a same-version sync with only locally-modified
        // files (classified as Conflict against unchanged upstream) must
        // not emit a runtime migration document.
        let tmp = TempDir::new().unwrap();
        let summary = DiffSummary {
            conflict: 2,
            ..Default::default()
        };
        let diffs: Vec<RuntimeFileDiff> = Vec::new();
        let hops: Vec<IntermediateHop> = Vec::new();

        let written =
            write_migration_document(tmp.path(), "0.18.6", "0.18.6", &summary, &diffs, &hops)
                .unwrap();
        assert!(written.is_none());
    }

    #[test]
    fn write_migration_document_still_writes_on_same_version_when_upstream_moved() {
        // A same-version sync that nevertheless has genuine upstream-side
        // deltas (e.g. a new runtime file) still produces a migration doc.
        let tmp = TempDir::new().unwrap();
        let summary = DiffSummary {
            new_upstream: 1,
            ..Default::default()
        };
        let diffs = vec![RuntimeFileDiff {
            rel_path: ".asoundrc".to_string(),
            project_path: PathBuf::from(".asoundrc"),
            classification: FileClassification::NewUpstream,
        }];
        let hops: Vec<IntermediateHop> = Vec::new();

        let written =
            write_migration_document(tmp.path(), "0.18.6", "0.18.6", &summary, &diffs, &hops)
                .unwrap();
        assert!(written.is_some());
    }

    #[test]
    fn zellij_layout_detects_unselected_harness_command() {
        let tmp = TempDir::new().unwrap();
        let rel = ".config/zellij/layouts/ai.kdl";
        let path = tmp.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "pane { command \"claude\" }\n").unwrap();
        let config = crate::config::AiboxConfig::from_str(
            r#"[aibox]
version = "0.22.0"

[container]
name = "demo"

[ai]
harnesses = ["codex"]
"#,
        )
        .unwrap();

        assert!(zellij_layout_contains_unselected_harness(
            tmp.path(),
            rel,
            &config
        ));
    }

    #[test]
    fn zellij_layout_allows_selected_harness_command() {
        let tmp = TempDir::new().unwrap();
        let rel = ".config/zellij/layouts/ai.kdl";
        let path = tmp.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "pane { command \"codex\" }\n").unwrap();
        let config = crate::config::AiboxConfig::from_str(
            r#"[aibox]
version = "0.22.0"

[container]
name = "demo"

[ai]
harnesses = ["codex"]
"#,
        )
        .unwrap();

        assert!(!zellij_layout_contains_unselected_harness(
            tmp.path(),
            rel,
            &config
        ));
    }

    #[test]
    fn detects_historical_managed_zellij_shell_status_layout() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let host_root = root.join(".aibox-home");
        let rel = ".config/zellij/layouts/dev.kdl";
        let old_managed = r#"layout {
    default_tab_template {
        children
        pane size=1 borderless=true {
            plugin location="zellij:status-bar"
        }
        pane size=1 borderless=true {
            command "bash"
            args "-lc" "if [ -x \"$HOME/.local/bin/aibox-status\" ]; then exec \"$HOME/.local/bin/aibox-status\" --watch; else exec aibox-status --watch; fi"
        }
    }
    tab name="dev" {}
}
"#;
        write_snapshot(root, "0.23.1", &[(rel, old_managed)]);
        let live = host_root.join(rel);
        fs::create_dir_all(live.parent().unwrap()).unwrap();
        fs::write(&live, old_managed).unwrap();

        assert!(live_matches_historical_managed_zellij_file(
            root, &host_root, rel
        ));
    }

    #[test]
    fn detects_historical_managed_zellij_native_status_layout() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let host_root = root.join(".aibox-home");
        let rel = ".config/zellij/layouts/ai.kdl";
        let snapshot_managed = r#"layout {
    default_tab_template {
        children
        pane size=1 borderless=true {
            plugin location="file:/usr/local/share/aibox/zellij/aibox-status.wasm" {
                role "keybar"
            }
        }
        pane size=1 borderless=true {
            plugin location="file:/usr/local/share/aibox/zellij/aibox-status.wasm" {
                role "status"
            }
        }
    }
    tab name="ai" {}
}
"#;
        let live_historical_bad_path = snapshot_managed.replace(
            "file:/usr/local/share/aibox/zellij/aibox-status.wasm",
            "file:/workspace/.aibox-home/.local/share/aibox/zellij/aibox-status.wasm",
        );
        write_snapshot(root, "0.23.7", &[(rel, snapshot_managed)]);
        let live = host_root.join(rel);
        fs::create_dir_all(live.parent().unwrap()).unwrap();
        fs::write(&live, live_historical_bad_path).unwrap();

        assert!(live_matches_historical_managed_zellij_file(
            root, &host_root, rel
        ));
    }

    #[test]
    fn detects_historical_managed_zellij_config() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let host_root = root.join(".aibox-home");
        let rel = ".config/zellij/config.kdl";
        let old_managed = r#"// aibox zellij configuration
theme "nord"
default_layout "ai"
"#;
        write_snapshot(root, "0.23.0", &[(rel, old_managed)]);
        let live = host_root.join(rel);
        fs::create_dir_all(live.parent().unwrap()).unwrap();
        fs::write(&live, old_managed).unwrap();

        assert!(live_matches_historical_managed_zellij_file(
            root, &host_root, rel
        ));
    }

    #[test]
    fn zellij_status_toggle_layout_relpath_matches_only_toggle_layouts() {
        assert!(zellij_status_toggle_layout_relpath(
            ".config/zellij/layouts/aibox-status-visible.kdl"
        ));
        assert!(zellij_status_toggle_layout_relpath(
            ".config/zellij/layouts/aibox-status-hidden.kdl"
        ));
        assert!(!zellij_status_toggle_layout_relpath(
            ".config/zellij/layouts/ai.kdl"
        ));
        assert!(!zellij_status_toggle_layout_relpath(
            ".config/zellij/config.kdl"
        ));
    }

    #[test]
    fn detects_historical_managed_runtime_helper() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let host_root = root.join(".aibox-home");
        let rel = ".local/bin/aibox-status";
        let old_managed = "#!/usr/bin/env bash\necho old-status\n";
        write_snapshot(root, "0.23.0", &[(rel, old_managed)]);
        let live = host_root.join(rel);
        fs::create_dir_all(live.parent().unwrap()).unwrap();
        fs::write(&live, old_managed).unwrap();

        assert!(live_matches_historical_managed_runtime_helper(
            root, &host_root, rel
        ));
    }

    #[test]
    fn detects_historical_managed_yazi_config() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let host_root = root.join(".aibox-home");
        let rel = ".config/yazi/yazi.toml";
        let old_managed = r#"[open]
rules = [
    { mime = "text/*", use = "edit" },
    { name = "*", use = "edit" },
]
"#;
        write_snapshot(root, "0.23.17", &[(rel, old_managed)]);
        let live = host_root.join(rel);
        fs::create_dir_all(live.parent().unwrap()).unwrap();
        fs::write(&live, old_managed).unwrap();

        assert!(live_matches_historical_managed_yazi_file(
            root, &host_root, rel
        ));
    }

    #[test]
    fn detects_legacy_managed_yazi_init_git_theme_mutation() {
        let tmp = TempDir::new().unwrap();
        let host_root = tmp.path().join(".aibox-home");
        let rel = ".config/yazi/init.lua";
        let live = host_root.join(rel);
        fs::create_dir_all(live.parent().unwrap()).unwrap();
        fs::write(
            &live,
            r#"-- =============================================================================
-- Yazi init.lua — aibox defaults
-- Runs on every Yazi startup. Register plugins that need setup here.
-- =============================================================================

-- git.yazi: show git status in the file list with explicit, visible signs.
-- Fetcher registration is in yazi.toml [plugin.prepend_fetchers].
th.git = th.git or {}
th.git.modified_sign = "M"
th.git.added_sign = "A"
th.git.deleted_sign = "D"
th.git.updated_sign = "U"
th.git.untracked_sign = "?"
th.git.ignored_sign = "I"

require("git"):setup {}

-- status-git.yazi: git branch + summary (left) and disk free (right) in status bar.
-- Data refresh is triggered via the fetcher registered in yazi.toml.
require("status-git"):setup()
"#,
        )
        .unwrap();

        assert!(live_matches_legacy_managed_yazi_init(&host_root, rel));
    }

    #[test]
    fn legacy_yazi_init_detector_rejects_user_edited_file() {
        let tmp = TempDir::new().unwrap();
        let host_root = tmp.path().join(".aibox-home");
        let rel = ".config/yazi/init.lua";
        let live = host_root.join(rel);
        fs::create_dir_all(live.parent().unwrap()).unwrap();
        fs::write(
            &live,
            r#"-- Yazi init.lua — custom user file
th.git = th.git or {}
th.git.modified_sign = "M"
require("git"):setup {}
"#,
        )
        .unwrap();

        assert!(!live_matches_legacy_managed_yazi_init(&host_root, rel));
    }

    #[test]
    fn historical_yazi_detector_rejects_user_edited_file() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let host_root = root.join(".aibox-home");
        let rel = ".config/yazi/theme.toml";
        let old_managed = r##"[filetype]
rules = [
    { name = "*.rs", fg = "#ffffff" },
]
"##;
        write_snapshot(root, "0.23.17", &[(rel, old_managed)]);
        let live = host_root.join(rel);
        fs::create_dir_all(live.parent().unwrap()).unwrap();
        fs::write(&live, format!("{old_managed}\n# user edit\n")).unwrap();

        assert!(!live_matches_historical_managed_yazi_file(
            root, &host_root, rel
        ));
    }

    #[test]
    fn historical_runtime_helper_detector_rejects_user_edited_file() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let host_root = root.join(".aibox-home");
        let rel = ".local/bin/aibox-status";
        let old_managed = "#!/usr/bin/env bash\necho old-status\n";
        write_snapshot(root, "0.23.0", &[(rel, old_managed)]);
        let live = host_root.join(rel);
        fs::create_dir_all(live.parent().unwrap()).unwrap();
        fs::write(&live, format!("{old_managed}\necho user edit\n")).unwrap();

        assert!(!live_matches_historical_managed_runtime_helper(
            root, &host_root, rel
        ));
    }

    #[test]
    fn historical_zellij_status_detector_rejects_user_edited_layout() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let host_root = root.join(".aibox-home");
        let rel = ".config/zellij/layouts/dev.kdl";
        let old_managed = r#"plugin location="zellij:status-bar"
args "-lc" "aibox-status --watch"
"#;
        write_snapshot(root, "0.23.1", &[(rel, old_managed)]);
        let live = host_root.join(rel);
        fs::create_dir_all(live.parent().unwrap()).unwrap();
        fs::write(&live, format!("{old_managed}\n# user edit\n")).unwrap();

        assert!(!live_matches_historical_managed_zellij_file(
            root, &host_root, rel
        ));
    }
}
