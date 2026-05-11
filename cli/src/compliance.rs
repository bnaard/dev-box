//! Compliance-contract helpers — surface the processkit skill-gate compliance
//! contract to each AI harness at sync time.
//!
//! Three things happen on every `aibox apply`:
//!
//! 1. **Drift detection** (Issue #46): compare the
//!    `<!-- pk-compliance-contract v1 BEGIN -->…END` block embedded in
//!    `AGENTS.md` against the canonical source file
//!    `context/skills/processkit/skill-gate/assets/compliance-contract.md`.
//!    If they differ, emit a warning.
//!
//! 2. **Cursor rules** (Issue #47): when `cursor` is enabled as an AI harness,
//!    write `.cursor/rules/processkit-compliance.md` from the canonical
//!    source.
//!
//! 3. **Aider conf** (Issue #48): when `aider` is enabled as an AI harness,
//!    ensure `.aider.conf.yml` contains `read:` entries for `AGENTS.md`
//!    and `context/skills/processkit/skill-gate/assets/compliance-contract.md`.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::{AiHarness, AiboxConfig};
use crate::output;

// ---------------------------------------------------------------------------
// Canonical paths
// ---------------------------------------------------------------------------

/// Project-root-relative path to the canonical compliance contract.
const COMPLIANCE_CONTRACT_REL: &str =
    "context/skills/processkit/skill-gate/assets/compliance-contract.md";

/// BEGIN/END markers for the embedded block in AGENTS.md.
///
/// processkit ships two contract revisions in the wild as of v0.18.1:
///   v1 — the original contract (what skill-gate's hook still emits).
///   v2 — adds the `skip_decision_record` clause (template ships this,
///        but the matching MCP tool isn't released upstream yet).
///
/// We accept either version so the drift checker doesn't false-alarm on
/// projects whose AGENTS.md template was synced from a v2-flavored
/// processkit release while the canonical contract source is still v1.
/// When upstream reconciles (and ships `skip_decision_record`), drop the
/// v1 entries and re-pin to v2 only.
const AGENTS_BLOCK_BEGIN_V1: &str = "<!-- pk-compliance-contract v1 BEGIN -->";
const AGENTS_BLOCK_END_V1: &str = "<!-- pk-compliance-contract v1 END -->";
const AGENTS_BLOCK_BEGIN_V2: &str = "<!-- pk-compliance-contract v2 BEGIN -->";
const AGENTS_BLOCK_END_V2: &str = "<!-- pk-compliance-contract v2 END -->";

// ---------------------------------------------------------------------------
// Issue #46 — drift detection
// ---------------------------------------------------------------------------

/// Compare the compliance-contract block in `AGENTS.md` against the canonical
/// source. Prints a warning if they differ; returns `Ok(())` in all cases
/// (drift is non-fatal).
///
/// If `fix` is `true` and drift is detected, the embedded block is
/// rewritten from the canonical source. When the embedded block uses
/// `v1` markers but the canonical source is `v2`, markers are migrated
/// to `v2` as part of the fix.
pub fn check_compliance_contract_drift(project_root: &Path, fix: bool) -> Result<()> {
    let canonical_path = project_root.join(COMPLIANCE_CONTRACT_REL);
    let agents_path = project_root.join("AGENTS.md");

    // If either file is absent we can't compare — skip silently.
    if !canonical_path.is_file() || !agents_path.is_file() {
        if fix {
            output::warn(
                "Compliance contract auto-fix skipped: AGENTS.md or the canonical contract is \
                 missing; nothing to rewrite.",
            );
        }
        return Ok(());
    }

    let canonical = fs::read_to_string(&canonical_path).with_context(|| {
        format!(
            "reading canonical compliance contract: {}",
            canonical_path.display()
        )
    })?;

    let agents_content = fs::read_to_string(&agents_path)
        .with_context(|| format!("reading AGENTS.md: {}", agents_path.display()))?;

    let embedded_v1 = extract_block(&agents_content, AGENTS_BLOCK_BEGIN_V1, AGENTS_BLOCK_END_V1);
    let embedded_v2 = extract_block(&agents_content, AGENTS_BLOCK_BEGIN_V2, AGENTS_BLOCK_END_V2);
    let canonical_version = detect_contract_version(&canonical);

    match (embedded_v1, embedded_v2) {
        (None, None) => {
            // Block markers not present — nothing to compare or fix.
            if fix {
                output::warn(
                    "Compliance contract auto-fix skipped: no pk-compliance-contract markers \
                     found in AGENTS.md; nothing to rewrite.",
                );
            }
        }
        (Some(block_v1), None) => {
            if block_v1.trim() == canonical.trim() {
                return Ok(());
            }
            if fix {
                // Rewrite the v1 block. If the canonical source is already v2,
                // migrate the surrounding markers to v2 at the same time so
                // future comparisons pin cleanly to the new version.
                let (begin, end) = match canonical_version {
                    ContractVersion::V2 => (AGENTS_BLOCK_BEGIN_V2, AGENTS_BLOCK_END_V2),
                    _ => (AGENTS_BLOCK_BEGIN_V1, AGENTS_BLOCK_END_V1),
                };
                rewrite_block(
                    &agents_path,
                    &agents_content,
                    AGENTS_BLOCK_BEGIN_V1,
                    AGENTS_BLOCK_END_V1,
                    begin,
                    end,
                    canonical.trim(),
                )?;
                output::ok(&format!(
                    "Rewrote compliance contract in AGENTS.md from canonical source ({}).",
                    if matches!(canonical_version, ContractVersion::V2) {
                        "markers migrated v1 → v2"
                    } else {
                        "v1 markers preserved"
                    }
                ));
            } else {
                output::warn(
                    "Compliance contract in AGENTS.md differs from the canonical source at \
                     context/skills/processkit/skill-gate/assets/compliance-contract.md.",
                );
                output::warn("Run `aibox apply --fix-compliance-contract` to update AGENTS.md.");
            }
        }
        (_, Some(block_v2)) => {
            if block_v2.trim() == canonical.trim() {
                return Ok(());
            }
            if fix {
                rewrite_block(
                    &agents_path,
                    &agents_content,
                    AGENTS_BLOCK_BEGIN_V2,
                    AGENTS_BLOCK_END_V2,
                    AGENTS_BLOCK_BEGIN_V2,
                    AGENTS_BLOCK_END_V2,
                    canonical.trim(),
                )?;
                output::ok("Rewrote compliance contract in AGENTS.md from canonical source (v2).");
            } else {
                output::warn(
                    "Compliance contract in AGENTS.md differs from the canonical source at \
                     context/skills/processkit/skill-gate/assets/compliance-contract.md.",
                );
                output::warn("Run `aibox apply --fix-compliance-contract` to update AGENTS.md.");
            }
        }
    }

    Ok(())
}

/// Which contract revision a piece of text claims to be. Derived from
/// the `<!-- pk-compliance vN -->` marker at the top of the canonical
/// file (or an embedded block).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContractVersion {
    V1,
    V2,
    Unknown,
}

fn detect_contract_version(text: &str) -> ContractVersion {
    if text.contains("<!-- pk-compliance v2 -->") {
        ContractVersion::V2
    } else if text.contains("<!-- pk-compliance v1 -->") {
        ContractVersion::V1
    } else {
        ContractVersion::Unknown
    }
}

/// Replace the region between `old_begin`..`old_end` in `original` with
/// `new_begin\n{new_content}\n{new_end}` and write it back to `agents_path`.
///
/// `old_begin`/`old_end` locate the existing block to replace;
/// `new_begin`/`new_end` are the markers to emit (these differ only
/// when migrating v1 → v2 markers during a fix).
#[allow(clippy::too_many_arguments)]
fn rewrite_block(
    agents_path: &Path,
    original: &str,
    old_begin: &str,
    old_end: &str,
    new_begin: &str,
    new_end: &str,
    new_content: &str,
) -> Result<()> {
    let begin_idx = original
        .find(old_begin)
        .with_context(|| format!("begin marker '{}' not found in AGENTS.md", old_begin))?;
    let after_begin = begin_idx + old_begin.len();
    let end_rel = original[after_begin..]
        .find(old_end)
        .with_context(|| format!("end marker '{}' not found in AGENTS.md", old_end))?;
    let end_idx = after_begin + end_rel + old_end.len();

    let mut rewritten = String::with_capacity(original.len() + new_content.len());
    rewritten.push_str(&original[..begin_idx]);
    rewritten.push_str(new_begin);
    rewritten.push('\n');
    rewritten.push_str(new_content);
    rewritten.push('\n');
    rewritten.push_str(new_end);
    rewritten.push_str(&original[end_idx..]);

    fs::write(agents_path, &rewritten)
        .with_context(|| format!("writing {}", agents_path.display()))?;
    Ok(())
}

/// Extract the content between `begin_marker` and `end_marker` (exclusive).
/// Returns `None` if either marker is absent.
fn extract_block<'a>(text: &'a str, begin_marker: &str, end_marker: &str) -> Option<&'a str> {
    let start = text.find(begin_marker).map(|i| i + begin_marker.len())?;
    let end = text[start..].find(end_marker).map(|i| start + i)?;
    Some(&text[start..end])
}

// ---------------------------------------------------------------------------
// Issue #47 — Cursor rules
// ---------------------------------------------------------------------------

/// Write `.cursor/rules/processkit-compliance.md` from the canonical
/// compliance contract. Only runs when `cursor` is enabled as an AI harness.
pub fn write_cursor_compliance_rules(config: &AiboxConfig, project_root: &Path) -> Result<()> {
    if !config.ai.harnesses.contains(&AiHarness::Cursor) {
        return Ok(());
    }

    let canonical_path = project_root.join(COMPLIANCE_CONTRACT_REL);
    if !canonical_path.is_file() {
        // Canonical file not yet installed — skip silently.
        return Ok(());
    }

    let content = fs::read_to_string(&canonical_path).with_context(|| {
        format!(
            "reading canonical compliance contract: {}",
            canonical_path.display()
        )
    })?;

    let rules_dir = project_root.join(".cursor").join("rules");
    fs::create_dir_all(&rules_dir).with_context(|| format!("creating {}", rules_dir.display()))?;

    let dest = rules_dir.join("processkit-compliance.md");
    fs::write(&dest, &content).with_context(|| format!("writing {}", dest.display()))?;

    output::ok(&format!("Wrote compliance contract to {}", dest.display()));

    Ok(())
}

// ---------------------------------------------------------------------------
// Issue #48 — Aider conf
// ---------------------------------------------------------------------------

/// Ensure `.aider.conf.yml` has `read:` entries for `AGENTS.md` and the
/// compliance contract. Only runs when `aider` is enabled as an AI harness.
/// Preserves all existing entries.
pub fn write_aider_compliance_conf(config: &AiboxConfig, project_root: &Path) -> Result<()> {
    if !config.ai.harnesses.contains(&AiHarness::Aider) {
        return Ok(());
    }

    let conf_path = project_root.join(".aider.conf.yml");

    // Read existing YAML (if any).
    let mut doc: serde_yaml::Value = if conf_path.is_file() {
        let raw = fs::read_to_string(&conf_path)
            .with_context(|| format!("reading {}", conf_path.display()))?;
        serde_yaml::from_str(&raw).unwrap_or(serde_yaml::Value::Mapping(Default::default()))
    } else {
        serde_yaml::Value::Mapping(Default::default())
    };

    // Ensure the root is a mapping.
    if !doc.is_mapping() {
        doc = serde_yaml::Value::Mapping(Default::default());
    }

    let required: &[&str] = &["AGENTS.md", COMPLIANCE_CONTRACT_REL];

    let mapping = doc.as_mapping_mut().expect("root is a mapping");
    let read_key = serde_yaml::Value::String("read".to_string());

    // Get or create the `read:` sequence.
    let read_entry = mapping
        .entry(read_key)
        .or_insert_with(|| serde_yaml::Value::Sequence(vec![]));

    if let serde_yaml::Value::Sequence(seq) = read_entry {
        for &entry in required {
            let v = serde_yaml::Value::String(entry.to_string());
            if !seq.contains(&v) {
                seq.push(v);
            }
        }
    }

    let yaml_str = serde_yaml::to_string(&doc).with_context(|| "serialising .aider.conf.yml")?;

    fs::write(&conf_path, &yaml_str).with_context(|| format!("writing {}", conf_path.display()))?;

    output::ok("Updated .aider.conf.yml with compliance contract read: entries");

    Ok(())
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run all three compliance-contract sync steps. Best-effort: failures in
/// individual steps are warned-and-continued so they do not abort the sync.
///
/// `fix_compliance_contract` — kept for CLI compatibility. `apply` now
/// automatically refreshes the managed AGENTS.md compliance block when
/// drift is detected; the flag remains a harmless explicit opt-in.
pub fn regenerate_compliance_configs(
    config: &AiboxConfig,
    project_root: &Path,
    _fix_compliance_contract: bool,
) -> Result<()> {
    if let Err(e) = check_compliance_contract_drift(project_root, true) {
        output::warn(&format!("Compliance drift check failed: {}", e));
    }
    if let Err(e) = write_cursor_compliance_rules(config, project_root) {
        output::warn(&format!("Cursor compliance rules failed: {}", e));
    }
    if let Err(e) = write_aider_compliance_conf(config, project_root) {
        output::warn(&format!("Aider compliance conf failed: {}", e));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper: build a minimal AiboxConfig with the given harnesses.
    fn config_with_harnesses(harnesses: Vec<AiHarness>) -> AiboxConfig {
        let toml_str = format!(
            "[aibox]\nversion = \"0.18.2\"\n\n[container]\nname = \"test\"\n\n[ai]\nharnesses = [{harnesses}]\n",
            harnesses = harnesses
                .iter()
                .map(|h| format!("\"{}\"", h))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let mut config: AiboxConfig = toml::from_str(&toml_str).expect("parse test config");
        config.ai.migrate_legacy();
        config
    }

    fn write_canonical(root: &Path) {
        let dir = root
            .join("context")
            .join("skills")
            .join("processkit")
            .join("skill-gate")
            .join("assets");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("compliance-contract.md"), "canonical content\n").unwrap();
    }

    // -----------------------------------------------------------------------
    // Issue #46 — drift detection
    // -----------------------------------------------------------------------

    #[test]
    fn drift_detected_when_blocks_differ() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_canonical(root);

        // Write AGENTS.md with a stale embedded block.
        let agents = format!(
            "{}\nstale content\n{}",
            AGENTS_BLOCK_BEGIN_V1, AGENTS_BLOCK_END_V1
        );
        fs::write(root.join("AGENTS.md"), agents).unwrap();

        // Should complete without error (drift is non-fatal).
        check_compliance_contract_drift(root, false).expect("should not error");
        // The warning is emitted to stderr — we verify the function runs
        // without returning Err and that the comparison logic executes.
    }

    #[test]
    fn no_drift_when_blocks_match() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_canonical(root);

        let agents = format!(
            "{}\ncanonical content\n{}",
            AGENTS_BLOCK_BEGIN_V1, AGENTS_BLOCK_END_V1
        );
        fs::write(root.join("AGENTS.md"), agents).unwrap();

        check_compliance_contract_drift(root, false).expect("should not error");
    }

    #[test]
    fn drift_skipped_when_canonical_missing() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // No canonical file — should not error.
        fs::write(root.join("AGENTS.md"), "no markers here").unwrap();
        check_compliance_contract_drift(root, false).expect("should not error");
    }

    #[test]
    fn fix_rewrites_v1_block_and_preserves_surroundings() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_canonical(root);

        let prefix = "# AGENTS.md\n\nsome prose before the block.\n\n";
        let suffix = "\n\nsome prose after the block.\n";
        let agents = format!(
            "{prefix}{}\nstale content\n{}{suffix}",
            AGENTS_BLOCK_BEGIN_V1, AGENTS_BLOCK_END_V1
        );
        fs::write(root.join("AGENTS.md"), &agents).unwrap();

        check_compliance_contract_drift(root, true).expect("fix should succeed");

        let rewritten = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert!(rewritten.starts_with(prefix), "prefix preserved");
        assert!(rewritten.ends_with(suffix), "suffix preserved");
        assert!(
            rewritten.contains("canonical content"),
            "canonical content written into block"
        );
        assert!(
            !rewritten.contains("stale content"),
            "stale content removed"
        );
        // Canonical fixture has no version marker → ContractVersion::Unknown →
        // v1 markers preserved.
        assert!(rewritten.contains(AGENTS_BLOCK_BEGIN_V1));
        assert!(rewritten.contains(AGENTS_BLOCK_END_V1));
    }

    #[test]
    fn fix_migrates_v1_markers_to_v2_when_canonical_is_v2() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let canonical_dir = root
            .join("context")
            .join("skills")
            .join("processkit")
            .join("skill-gate")
            .join("assets");
        fs::create_dir_all(&canonical_dir).unwrap();
        fs::write(
            canonical_dir.join("compliance-contract.md"),
            "<!-- pk-compliance v2 -->\n\nnew v2 body\n",
        )
        .unwrap();

        let agents = format!(
            "{}\n<!-- pk-compliance v1 -->\nold v1 body\n{}",
            AGENTS_BLOCK_BEGIN_V1, AGENTS_BLOCK_END_V1
        );
        fs::write(root.join("AGENTS.md"), agents).unwrap();

        check_compliance_contract_drift(root, true).expect("fix should succeed");

        let rewritten = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert!(
            rewritten.contains(AGENTS_BLOCK_BEGIN_V2) && rewritten.contains(AGENTS_BLOCK_END_V2),
            "v2 markers emitted"
        );
        assert!(
            !rewritten.contains(AGENTS_BLOCK_BEGIN_V1) && !rewritten.contains(AGENTS_BLOCK_END_V1),
            "v1 markers replaced"
        );
        assert!(rewritten.contains("new v2 body"));
        assert!(!rewritten.contains("old v1 body"));
    }

    #[test]
    fn fix_is_idempotent_when_no_drift() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_canonical(root);

        let agents = format!(
            "{}\ncanonical content\n{}",
            AGENTS_BLOCK_BEGIN_V1, AGENTS_BLOCK_END_V1
        );
        fs::write(root.join("AGENTS.md"), &agents).unwrap();

        check_compliance_contract_drift(root, true).expect("should succeed");

        let after = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert_eq!(after, agents, "no rewrite when already in sync");
    }

    #[test]
    fn fix_no_op_when_no_markers() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_canonical(root);
        fs::write(root.join("AGENTS.md"), "no markers here\n").unwrap();

        check_compliance_contract_drift(root, true).expect("should not error");

        let after = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert_eq!(after, "no markers here\n", "file untouched when no markers");
    }

    // -----------------------------------------------------------------------
    // Issue #47 — Cursor rules
    // -----------------------------------------------------------------------

    #[test]
    fn cursor_rules_written_when_cursor_in_harnesses() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_canonical(root);

        let config = config_with_harnesses(vec![AiHarness::Cursor]);
        write_cursor_compliance_rules(&config, root).expect("should succeed");

        let dest = root
            .join(".cursor")
            .join("rules")
            .join("processkit-compliance.md");
        assert!(
            dest.is_file(),
            ".cursor/rules/processkit-compliance.md should exist"
        );
        assert_eq!(
            fs::read_to_string(&dest).unwrap().trim(),
            "canonical content"
        );
    }

    #[test]
    fn cursor_rules_skipped_when_cursor_not_in_harnesses() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_canonical(root);

        let config = config_with_harnesses(vec![AiHarness::Claude]);
        write_cursor_compliance_rules(&config, root).expect("should succeed");

        let dest = root
            .join(".cursor")
            .join("rules")
            .join("processkit-compliance.md");
        assert!(
            !dest.exists(),
            "file should not be written for non-cursor harness"
        );
    }

    // -----------------------------------------------------------------------
    // Issue #48 — Aider conf
    // -----------------------------------------------------------------------

    #[test]
    fn aider_conf_created_with_required_entries() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let config = config_with_harnesses(vec![AiHarness::Aider]);
        write_aider_compliance_conf(&config, root).expect("should succeed");

        let conf_path = root.join(".aider.conf.yml");
        assert!(conf_path.is_file());

        let raw = fs::read_to_string(&conf_path).unwrap();
        assert!(raw.contains("AGENTS.md"), "should contain AGENTS.md");
        assert!(
            raw.contains(COMPLIANCE_CONTRACT_REL),
            "should contain compliance contract path"
        );
    }

    #[test]
    fn aider_conf_preserves_existing_entries() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Pre-existing conf with a custom read entry.
        let existing = "read:\n  - some-custom-file.md\n";
        fs::write(root.join(".aider.conf.yml"), existing).unwrap();

        let config = config_with_harnesses(vec![AiHarness::Aider]);
        write_aider_compliance_conf(&config, root).expect("should succeed");

        let raw = fs::read_to_string(root.join(".aider.conf.yml")).unwrap();
        assert!(
            raw.contains("some-custom-file.md"),
            "custom entry should be preserved"
        );
        assert!(raw.contains("AGENTS.md"), "AGENTS.md should be added");
        assert!(
            raw.contains(COMPLIANCE_CONTRACT_REL),
            "compliance path should be added"
        );
    }

    #[test]
    fn aider_conf_idempotent_does_not_duplicate() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let config = config_with_harnesses(vec![AiHarness::Aider]);

        // Run twice.
        write_aider_compliance_conf(&config, root).expect("first run");
        write_aider_compliance_conf(&config, root).expect("second run");

        let raw = fs::read_to_string(root.join(".aider.conf.yml")).unwrap();
        // Each required entry should appear exactly once.
        assert_eq!(raw.matches("AGENTS.md").count(), 1, "AGENTS.md duplicated");
        assert_eq!(
            raw.matches(COMPLIANCE_CONTRACT_REL).count(),
            1,
            "compliance path duplicated"
        );
    }

    #[test]
    fn aider_conf_skipped_when_aider_not_in_harnesses() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let config = config_with_harnesses(vec![AiHarness::Claude]);
        write_aider_compliance_conf(&config, root).expect("should succeed");

        assert!(
            !root.join(".aider.conf.yml").exists(),
            ".aider.conf.yml should not be created for non-aider harness"
        );
    }
}
