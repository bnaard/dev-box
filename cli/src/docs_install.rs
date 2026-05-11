//! Project-local docs-site dependency install.
//!
//! Background — EagerDew (v0.25.7)
//! --------------------------------
//! The docs addons (`docs-docusaurus`, `docs-mkdocs`, …) install upstream
//! tooling globally inside the container image (e.g.
//! `npm install -g @docusaurus/core`). They do **not** install a docs
//! site's project-local dependencies declared in `<docs-dir>/package.json`
//! — for example `prism-react-renderer`, which Docusaurus dynamically
//! requires from the project's own `node_modules` rather than from its
//! global install.
//!
//! During the v0.25.6 release a `docusaurus build` failed with
//! `Cannot find module 'prism-react-renderer'` because nobody had ever run
//! `npm install` inside `docs-site/`. This module closes that gap by
//! detecting any docs-site-shaped subdirectory at the project root and
//! running the appropriate package-manager install when:
//!
//! 1. A docs addon is enabled in `aibox.toml`, AND
//! 2. The docs directory contains a `package.json`, AND
//! 3. `node_modules/` is missing or older than `package.json` /
//!    `package-lock.json` / `bun.lock`.
//!
//! Best-effort: failures are warned-and-continued so a flaky network or
//! a missing `npm` binary on the host doesn't abort `aibox apply`.
//!
//! The install runs on the host (not in the container) because docs
//! deploy/build is itself a host-side workflow in this project (see
//! `scripts/maintain.sh::cmd_docs_deploy`).

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::AiboxConfig;
use crate::output;

/// Conventional locations a docs site might live in, relative to the
/// project root. Order matters: the first one that exists with a
/// `package.json` wins.
const DOCS_DIR_CANDIDATES: &[&str] = &["docs-site", "website", "docs", "site"];

/// Heuristic: which docs addons declare an npm-tooling dependency that
/// implies a project-local `package.json` install will be useful?
///
/// We intentionally do *not* gate on every docs addon — `mdbook` and
/// `mkdocs` are not Node ecosystems and don't need `npm install` even
/// when the project happens to ship a `docs-site/package.json` for
/// other reasons. If the project has a `package.json` AND any
/// node-flavoured docs addon, install.
const NODE_DOCS_ADDONS: &[&str] = &["docs-docusaurus", "docs-starlight", "docs-zensical"];

/// Public entry point used by `cmd_sync`. Best-effort — never returns
/// an error to the caller; logs warnings if anything goes wrong.
pub fn maybe_install_project_docs_deps(config: &AiboxConfig, project_root: &Path) {
    if !any_node_docs_addon_enabled(config) {
        return;
    }
    let Some(docs_dir) = detect_docs_dir(project_root) else {
        return;
    };
    let pkg_json = docs_dir.join("package.json");
    if !pkg_json.is_file() {
        return;
    }
    if node_modules_is_fresh(&docs_dir) {
        output::ok(&format!(
            "docs-site dependencies up-to-date in {}/node_modules",
            relative_display(project_root, &docs_dir),
        ));
        return;
    }
    let pm = detect_package_manager(&docs_dir);
    if !package_manager_available(pm) {
        output::warn(&format!(
            "Skipped project-local docs dependency install in {}: `{}` is not available on \
             this host. Container generation continues; install docs dependencies only when \
             building or deploying docs.",
            relative_display(project_root, &docs_dir),
            pm.as_str(),
        ));
        return;
    }
    output::info(&format!(
        "Installing project-local docs dependencies in {} via {}...",
        relative_display(project_root, &docs_dir),
        pm.as_str(),
    ));
    if let Err(e) = run_install(&docs_dir, pm) {
        output::warn(&format!(
            "Project-local docs install failed in {}: {}. \
             Run `cd {} && {} install` manually to unblock docs build/deploy.",
            relative_display(project_root, &docs_dir),
            e,
            relative_display(project_root, &docs_dir),
            pm.as_str(),
        ));
    } else {
        output::ok(&format!(
            "docs-site dependencies installed in {}/node_modules",
            relative_display(project_root, &docs_dir),
        ));
    }
}

fn package_manager_available(pm: PackageManager) -> bool {
    std::process::Command::new(pm.as_str())
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// True iff at least one node-flavoured docs addon is listed in
/// `[addons]` of `aibox.toml`. We do not check tool-level
/// `enabled = false` here because the user is unlikely to declare an
/// addon block without wanting it active, and the host install is
/// idempotent.
pub(crate) fn any_node_docs_addon_enabled(config: &AiboxConfig) -> bool {
    config
        .addons
        .addons
        .keys()
        .any(|name| NODE_DOCS_ADDONS.contains(&name.as_str()))
}

/// Walk the conventional candidate list and return the first directory
/// under `project_root` that exists *and* contains a `package.json`.
pub(crate) fn detect_docs_dir(project_root: &Path) -> Option<PathBuf> {
    for candidate in DOCS_DIR_CANDIDATES {
        let p = project_root.join(candidate);
        if p.is_dir() && p.join("package.json").is_file() {
            return Some(p);
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PackageManager {
    Npm,
    Bun,
    Pnpm,
    Yarn,
}

impl PackageManager {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Bun => "bun",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Yarn => "yarn",
        }
    }
}

/// Pick a package manager based on which lockfile is present. `npm`
/// (the most universally installed) is the conservative fallback.
///
/// Lockfile precedence is intentional: if the user has both a
/// `package-lock.json` AND a `bun.lock` (as the aibox docs-site does
/// today), prefer `npm` because it works on every developer's machine
/// even when `bun` isn't on PATH.
pub(crate) fn detect_package_manager(docs_dir: &Path) -> PackageManager {
    if docs_dir.join("package-lock.json").is_file() {
        PackageManager::Npm
    } else if docs_dir.join("pnpm-lock.yaml").is_file() {
        PackageManager::Pnpm
    } else if docs_dir.join("yarn.lock").is_file() {
        PackageManager::Yarn
    } else if docs_dir.join("bun.lock").is_file() || docs_dir.join("bun.lockb").is_file() {
        PackageManager::Bun
    } else {
        PackageManager::Npm
    }
}

/// `node_modules` is considered fresh if it exists AND its mtime is
/// strictly newer than every relevant manifest file (package.json plus
/// any present lockfile). Any failure to stat collapses to "not fresh"
/// so that we err on the side of running the install.
pub(crate) fn node_modules_is_fresh(docs_dir: &Path) -> bool {
    let nm = docs_dir.join("node_modules");
    let Ok(nm_meta) = std::fs::metadata(&nm) else {
        return false;
    };
    if !nm_meta.is_dir() {
        return false;
    }
    let Ok(nm_mtime) = nm_meta.modified() else {
        return false;
    };

    let manifests = [
        docs_dir.join("package.json"),
        docs_dir.join("package-lock.json"),
        docs_dir.join("pnpm-lock.yaml"),
        docs_dir.join("yarn.lock"),
        docs_dir.join("bun.lock"),
        docs_dir.join("bun.lockb"),
    ];
    for m in manifests.iter() {
        if let Ok(meta) = std::fs::metadata(m)
            && let Ok(mtime) = meta.modified()
            && mtime > nm_mtime
        {
            return false;
        }
    }
    true
}

/// Run `<pm> install --prefix <docs_dir>` (or the package-manager
/// idiomatic equivalent). The `--prefix` flag is what makes this
/// install touch the *project* `package.json`, not whatever is at
/// `cwd`. This is the key fix for EagerDew.
fn run_install(docs_dir: &Path, pm: PackageManager) -> Result<()> {
    let mut cmd = std::process::Command::new(pm.as_str());
    match pm {
        PackageManager::Npm | PackageManager::Pnpm | PackageManager::Yarn => {
            cmd.arg("install").arg("--prefix").arg(docs_dir);
        }
        PackageManager::Bun => {
            // bun has no --prefix; cd into the dir.
            cmd.arg("install").current_dir(docs_dir);
        }
    }
    let status = cmd
        .status()
        .map_err(|e| anyhow::anyhow!("failed to spawn `{}`: {}", pm.as_str(), e))?;
    if !status.success() {
        anyhow::bail!("`{} install` exited with {}", pm.as_str(), status);
    }
    Ok(())
}

fn relative_display(project_root: &Path, p: &Path) -> String {
    p.strip_prefix(project_root)
        .map(|r| r.display().to_string())
        .unwrap_or_else(|_| p.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write(p: &Path, s: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, s).unwrap();
    }

    #[test]
    fn detect_docs_dir_prefers_docs_site_over_docs() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("docs-site/package.json"), "{}");
        write(&root.join("docs/package.json"), "{}");
        let found = detect_docs_dir(root).expect("should detect");
        assert!(found.ends_with("docs-site"), "got {:?}", found);
    }

    #[test]
    fn detect_docs_dir_returns_none_when_no_package_json() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        // A `docs/` directory with no package.json should not match —
        // mdbook/mkdocs sites have no package.json and don't need npm.
        fs::create_dir_all(root.join("docs")).unwrap();
        assert!(detect_docs_dir(root).is_none());
    }

    #[test]
    fn detect_package_manager_prefers_npm_over_bun_when_both_present() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), "{}");
        write(&root.join("package-lock.json"), "{}");
        write(&root.join("bun.lock"), "");
        assert_eq!(detect_package_manager(root), PackageManager::Npm);
    }

    #[test]
    fn detect_package_manager_falls_back_to_npm_with_no_lockfile() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), "{}");
        assert_eq!(detect_package_manager(root), PackageManager::Npm);
    }

    #[test]
    fn detect_package_manager_picks_bun_when_only_bun_lock() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), "{}");
        write(&root.join("bun.lock"), "");
        assert_eq!(detect_package_manager(root), PackageManager::Bun);
    }

    #[test]
    fn node_modules_missing_is_not_fresh() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), "{}");
        assert!(!node_modules_is_fresh(root));
    }

    #[test]
    fn node_modules_present_and_newer_is_fresh() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), "{}");
        // Sleep briefly to ensure mtime ordering, then create node_modules.
        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::create_dir_all(root.join("node_modules")).unwrap();
        assert!(node_modules_is_fresh(root));
    }

    #[test]
    fn node_modules_older_than_package_json_is_not_fresh() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("node_modules")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        write(&root.join("package.json"), "{}");
        assert!(!node_modules_is_fresh(root));
    }

    #[test]
    fn any_node_docs_addon_enabled_detects_docusaurus() {
        let mut config = crate::config::test_config();
        config.addons.addons.insert(
            "docs-docusaurus".to_string(),
            crate::config::AddonToolsSection::default(),
        );
        assert!(any_node_docs_addon_enabled(&config));
    }

    #[test]
    fn any_node_docs_addon_enabled_skips_mdbook_only() {
        let mut config = crate::config::test_config();
        config.addons.addons.clear();
        config.addons.addons.insert(
            "docs-mdbook".to_string(),
            crate::config::AddonToolsSection::default(),
        );
        assert!(!any_node_docs_addon_enabled(&config));
    }

    #[test]
    fn maybe_install_is_noop_without_node_addon() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("docs-site/package.json"), "{}");
        let mut config = crate::config::test_config();
        config.addons.addons.clear();
        // Should not run anything (no addon enabled). Success path =
        // function returns without panicking and without creating a
        // node_modules dir.
        maybe_install_project_docs_deps(&config, root);
        assert!(!root.join("docs-site/node_modules").exists());
    }
}
