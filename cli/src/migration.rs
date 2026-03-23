//! Migration system for aibox version changes.
//!
//! On `aibox sync`, compares `.aibox-version` file content against the
//! running CLI version. If they differ, generates a migration document at
//! `context/migrations/{from}-to-{to}.md`.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::output;

/// Check for version mismatch and generate migration document if needed.
/// Called during `aibox sync`. Operates in the current working directory.
pub fn check_and_generate_migration() -> Result<()> {
    check_and_generate_migration_in(Path::new("."))
}

/// Check for version mismatch and generate migration document if needed.
/// Operates relative to the given `root` directory.
fn check_and_generate_migration_in(root: &Path) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    let version_file = root.join(".aibox-version");

    // If no version file, this is a fresh project — no migration needed
    if !version_file.exists() {
        return Ok(());
    }

    let stored_version = fs::read_to_string(&version_file)
        .context("Failed to read .aibox-version")?
        .trim()
        .to_string();

    if stored_version == current_version {
        return Ok(());
    }

    output::info(&format!(
        "Version change detected: {} \u{2192} {}",
        stored_version, current_version
    ));

    // Generate migration document
    generate_migration_doc(root, &stored_version, current_version)?;

    // Update .aibox-version to current
    fs::write(&version_file, current_version).context("Failed to update .aibox-version")?;

    Ok(())
}

/// Generate a migration document at `{root}/context/migrations/{from}-to-{to}.md`.
fn generate_migration_doc(root: &Path, from: &str, to: &str) -> Result<()> {
    let migrations_dir = root.join("context").join("migrations");
    fs::create_dir_all(&migrations_dir).context("Failed to create context/migrations/")?;

    let filename = format!("{}-to-{}.md", from, to);
    let filepath = migrations_dir.join(&filename);

    // Don't overwrite existing migration docs (user may have edited status)
    if filepath.exists() {
        output::info(&format!(
            "Migration document {} already exists, skipping",
            filename
        ));
        return Ok(());
    }

    let date = chrono_free_date();
    let content = format_migration_doc(from, to, &date);

    fs::write(&filepath, content)
        .with_context(|| format!("Failed to write migration document {}", filename))?;

    output::ok(&format!(
        "Generated migration document: context/migrations/{}",
        filename
    ));
    output::warn("Review the migration document with your AI agent before proceeding");

    Ok(())
}

/// Get the current date without requiring the chrono crate.
fn chrono_free_date() -> String {
    std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

// ---------------------------------------------------------------------------
// Migration registry — version-specific migration knowledge
// ---------------------------------------------------------------------------

/// A known migration entry with specific breaking changes and action items.
struct MigrationEntry {
    from: &'static str,
    to: &'static str,
    breaking_changes: &'static [&'static str],
    action_items: &'static [&'static str],
}

/// Registry of known migrations with version-specific details.
static KNOWN_MIGRATIONS: &[MigrationEntry] = &[
    // Will be populated as we release versions
    // Example:
    // MigrationEntry {
    //     from: "0.8.0",
    //     to: "0.9.0",
    //     breaking_changes: &[
    //         "`[aibox] image` renamed to `[aibox] base` — only \"debian\" is valid",
    //         "Process packages replace monolithic process levels",
    //     ],
    //     action_items: &[
    //         "Update aibox.toml: change `image = \"python\"` to `base = \"debian\"` and add `[addons.python.tools]`",
    //     ],
    // },
];

/// Find a known migration entry for the given version pair.
fn find_known_migration(from: &str, to: &str) -> Option<&'static MigrationEntry> {
    KNOWN_MIGRATIONS
        .iter()
        .find(|m| m.from == from && m.to == to)
}

// ---------------------------------------------------------------------------
// Document formatting
// ---------------------------------------------------------------------------

/// Format the full migration document content.
fn format_migration_doc(from: &str, to: &str, date: &str) -> String {
    let known = find_known_migration(from, to);

    let breaking_changes = if let Some(entry) = known {
        entry
            .breaking_changes
            .iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        format!(
            "- Review the [changelog](https://github.com/projectious-work/aibox/releases) \
             for breaking changes between v{} and v{}.",
            from, to
        )
    };

    let action_items = if let Some(entry) = known {
        let mut items: Vec<String> = entry
            .action_items
            .iter()
            .map(|a| format!("- [ ] {}", a))
            .collect();
        // Always include standard items
        items.push("- [ ] Review this migration document with the project owner".to_string());
        items.push("- [ ] Run `aibox sync` to regenerate container files".to_string());
        items.push("- [ ] Rebuild the container: `aibox build`".to_string());
        items.push("- [ ] Verify all context files are intact".to_string());
        items.push(
            "- [ ] Mark this migration as completed (change Status to \"completed\")".to_string(),
        );
        items.join("\n")
    } else {
        "\
- [ ] Review this migration document with the project owner
- [ ] Run `aibox sync` to regenerate container files
- [ ] Rebuild the container: `aibox build`
- [ ] Verify all context files are intact
- [ ] Mark this migration as completed (change Status to \"completed\")"
            .to_string()
    };

    format!(
        "\
# Migration: v{from} \u{2192} v{to}

> **SAFETY: Do not execute any actions in this document automatically.**
> **Discuss each item with the project owner before proceeding.**
> **Do not modify aibox.toml without explicit user confirmation.**

**Generated:** {date}
**Status:** pending
**aibox CLI version:** v{to}

## Summary

aibox has been updated from v{from} to v{to}. Review the changes below
and discuss each action item with the project owner.

## Breaking Changes

{breaking_changes}

## Action Items

{action_items}

## New Skills Available

Check the skills catalog for new skills added in v{to}. Add desired skills
to `[skills] include` in aibox.toml.

## Changed Skills

Skills with updated content will be automatically updated on next `aibox sync`
if the local file has not been modified. Modified skills are left untouched.

## Deprecated Skills

Check the release notes for any removed or renamed skills.

## Context File Changes

Review `context/AIBOX.md` for the updated baseline document.

## Verification Checklist

- [ ] `aibox sync` completes without errors
- [ ] Container builds successfully (`aibox build`)
- [ ] Context files are intact
- [ ] Skills are correctly deployed
- [ ] Agent entry point (CLAUDE.md etc.) points to context/AIBOX.md

## Rollback

To revert this migration:
```
git checkout HEAD -- .aibox-version context/ .devcontainer/
aibox sync
```

## Known Issues

Check https://github.com/projectious-work/aibox/issues for known issues
with v{to}.
"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_no_migration_when_versions_match() {
        let tmp = TempDir::new().unwrap();
        let current = env!("CARGO_PKG_VERSION");
        fs::write(tmp.path().join(".aibox-version"), current).unwrap();
        fs::create_dir_all(tmp.path().join("context/migrations")).unwrap();

        check_and_generate_migration_in(tmp.path()).unwrap();

        // No migration document should be created
        let entries: Vec<_> = fs::read_dir(tmp.path().join("context/migrations"))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(entries.is_empty(), "no migration doc when versions match");
    }

    #[test]
    fn test_no_migration_when_no_version_file() {
        let tmp = TempDir::new().unwrap();

        // No .aibox-version file exists — fresh project
        check_and_generate_migration_in(tmp.path()).unwrap();

        // context/migrations/ should not even be created
        assert!(
            !tmp.path().join("context/migrations").exists(),
            "no migrations dir for fresh project"
        );
    }

    #[test]
    fn test_migration_doc_generated_on_version_change() {
        let tmp = TempDir::new().unwrap();
        let current = env!("CARGO_PKG_VERSION");
        fs::write(tmp.path().join(".aibox-version"), "0.0.1").unwrap();

        check_and_generate_migration_in(tmp.path()).unwrap();

        let expected_path = tmp
            .path()
            .join("context/migrations")
            .join(format!("0.0.1-to-{}.md", current));
        assert!(expected_path.exists(), "migration doc should be created");

        let content = fs::read_to_string(&expected_path).unwrap();
        assert!(content.contains(&format!("v0.0.1 \u{2192} v{}", current)));
        assert!(content.contains("**Status:** pending"));

        // .aibox-version should be updated
        let updated = fs::read_to_string(tmp.path().join(".aibox-version")).unwrap();
        assert_eq!(updated, current);
    }

    #[test]
    fn test_migration_doc_not_overwritten() {
        let tmp = TempDir::new().unwrap();
        let current = env!("CARGO_PKG_VERSION");
        let migrations_dir = tmp.path().join("context/migrations");
        fs::create_dir_all(&migrations_dir).unwrap();

        let filename = format!("0.0.1-to-{}.md", current);
        let filepath = migrations_dir.join(&filename);
        let existing_content = "# User-edited migration doc\nStatus: in-progress\n";
        fs::write(&filepath, existing_content).unwrap();

        fs::write(tmp.path().join(".aibox-version"), "0.0.1").unwrap();

        check_and_generate_migration_in(tmp.path()).unwrap();

        // File should not be overwritten
        let content = fs::read_to_string(&filepath).unwrap();
        assert_eq!(
            content, existing_content,
            "existing migration doc should not be overwritten"
        );
    }

    #[test]
    fn test_migration_doc_contains_required_sections() {
        let doc = format_migration_doc("0.7.0", "0.8.0", "2026-03-23");

        // Safety header
        assert!(doc.contains("SAFETY: Do not execute any actions in this document automatically."));
        assert!(doc.contains("Discuss each item with the project owner before proceeding."));
        assert!(doc.contains("Do not modify aibox.toml without explicit user confirmation."));

        // Status
        assert!(doc.contains("**Status:** pending"));

        // Action items with checkboxes
        assert!(doc.contains("- [ ] Review this migration document with the project owner"));
        assert!(doc.contains("- [ ] Run `aibox sync` to regenerate container files"));
        assert!(doc.contains("- [ ] Rebuild the container: `aibox build`"));

        // Verification checklist
        assert!(doc.contains("## Verification Checklist"));
        assert!(doc.contains("- [ ] `aibox sync` completes without errors"));
        assert!(doc.contains("- [ ] Container builds successfully"));
        assert!(doc
            .contains("- [ ] Agent entry point (CLAUDE.md etc.) points to context/AIBOX.md"));

        // Rollback section
        assert!(doc.contains("## Rollback"));
        assert!(doc.contains("git checkout HEAD -- .aibox-version context/ .devcontainer/"));

        // Other required sections
        assert!(doc.contains("## Breaking Changes"));
        assert!(doc.contains("## New Skills Available"));
        assert!(doc.contains("## Changed Skills"));
        assert!(doc.contains("## Deprecated Skills"));
        assert!(doc.contains("## Context File Changes"));
        assert!(doc.contains("## Known Issues"));
    }

    #[test]
    fn test_format_migration_doc_versions_and_date() {
        let doc = format_migration_doc("1.2.3", "2.0.0", "2026-01-15");

        assert!(doc.contains("# Migration: v1.2.3 \u{2192} v2.0.0"));
        assert!(doc.contains("**Generated:** 2026-01-15"));
        assert!(doc.contains("**aibox CLI version:** v2.0.0"));
        assert!(doc.contains("from v1.2.3 to v2.0.0"));
    }

    #[test]
    fn test_chrono_free_date_returns_valid_format() {
        let date = chrono_free_date();
        // Should be YYYY-MM-DD or "unknown"
        if date != "unknown" {
            assert_eq!(date.len(), 10, "date should be 10 chars: {}", date);
            assert_eq!(&date[4..5], "-", "should have dash at pos 4");
            assert_eq!(&date[7..8], "-", "should have dash at pos 7");
        }
    }
}
