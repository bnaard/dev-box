//! Emission of v1→v2 Migration documents on processkit cutover releases.
//!
//! When upstream processkit supersedes a v1 entity type with a v2 primitive,
//! the CLI emits a Migration entity into `context/migrations/pending/` so
//! legacy v1 content gets explicitly addressed (transformed, archived, or
//! accepted-as-historical) instead of silently lingering.
//!
//! # Catalog
//!
//! [`V1_TO_V2_CUTOVERS`] is a compile-time catalogue of every known v1→v2
//! cutover.  Each entry maps an upstream processkit release to the v1 entity
//! kind it supersedes and the v2 primitive that replaces it.  The catalogue
//! is intentionally minimal — backfilling historical cutovers (Actor→
//! TeamMember, Process→Scope+Gate, StateMachine→lifecycle) is deferred to
//! a follow-up WorkItem.
//!
//! # Idempotency
//!
//! A marker string unique to each cutover (derived from `v1_kind` and
//! `v2_kind`) is embedded in the frontmatter. Before writing, the function
//! scans `context/migrations/{pending,in-progress,applied}/` for any
//! existing file that already contains the marker; if found, it is skipped.
//! Running `aibox apply` twice therefore never double-emits.
//!
//! # Wiring
//!
//! [`emit_v1_v2_migrations`] is called from `container.rs::cmd_sync` after
//! the lock comparison step, before any host-state actions are taken.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Catalogue
// ---------------------------------------------------------------------------

/// The transformation hint describes what the project owner should do with
/// legacy v1 files after the v2 primitive ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // variants are used in V1_TO_V2_CUTOVERS and tests
pub enum TransformationHint {
    /// Convert each v1 file into the equivalent v2 entity.
    Migrate,
    /// Move v1 files into an archive location; they are historical artefacts.
    Archive,
    /// v1 files can remain as-is; the v2 primitive is additive.
    NoOp,
}

impl TransformationHint {
    fn as_str(self) -> &'static str {
        match self {
            TransformationHint::Migrate => "migrate",
            TransformationHint::Archive => "archive",
            TransformationHint::NoOp => "no-op",
        }
    }
}

/// One entry in the compile-time v1→v2 cutover catalogue.
///
/// `upstream_release` is the first processkit release that ships the v2
/// primitive.  The CLI emits a Migration for a cutover when the project's
/// lock crosses from a version *before* `upstream_release` to a version
/// *at or after* it — AND the `v1_dir` directory exists and is non-empty.
pub struct CutoverDescriptor {
    /// First processkit release that ships the v2 primitive (e.g. `v0.18.0`).
    pub upstream_release: &'static str,
    /// v1 entity kind being superseded (e.g. `Actor`).
    pub v1_kind: &'static str,
    /// v2 entity kind / primitive that replaces it (e.g. `TeamMember`).
    pub v2_kind: &'static str,
    /// Project-relative path to the directory holding v1 entities.
    pub v1_dir: &'static str,
    /// Project-relative path to the directory holding v2 entities.
    pub v2_dir: &'static str,
    /// Decision record ID that authorised this cutover.
    pub dec_ref: &'static str,
    /// One-line human summary written into the Migration frontmatter.
    pub summary: &'static str,
    /// Recommended action for v1 files.
    pub transformation_hint: TransformationHint,
}

/// Compile-time catalogue of every known v1→v2 cutover.
///
/// **Backfilling** of the three historical cutovers (Actor→TeamMember,
/// Process→Scope+Gate, StateMachine→lifecycle) that shipped without explicit
/// Migrations is tracked in a separate follow-up WorkItem.  Only future
/// cutovers (those that land *after* this mechanism ships) belong here for
/// now.  The entries below are therefore intentionally empty — they serve as
/// the template for whoever adds the first real entry.
///
/// When adding an entry:
/// 1. Set `upstream_release` to the processkit tag that first ships the v2
///    primitive.
/// 2. Fill in `v1_kind`, `v2_kind`, `v1_dir`, `v2_dir`, `dec_ref`, and
///    `summary`.
/// 3. Choose `transformation_hint` (`Migrate`, `Archive`, or `NoOp`).
pub const V1_TO_V2_CUTOVERS: &[CutoverDescriptor] = &[
    // Example (un-comment and edit when the next real cutover ships):
    //
    // CutoverDescriptor {
    //     upstream_release: "v0.XX.0",
    //     v1_kind:          "OldEntity",
    //     v2_kind:          "NewPrimitive",
    //     v1_dir:           "context/old-entities/",
    //     v2_dir:           "context/new-primitives/",
    //     dec_ref:          "DEC-YYYYMMDD_HHMM-SlugSlug",
    //     summary:          "v1 OldEntity superseded by v2 NewPrimitive.",
    //     transformation_hint: TransformationHint::Migrate,
    // },
];

// ---------------------------------------------------------------------------
// Emission result
// ---------------------------------------------------------------------------

/// Describes one Migration document that was written to disk.
#[derive(Debug, Clone)]
pub struct MigrationEmission {
    /// The Migration ID embedded in the document (e.g. `MIG-V1V2-Actor-to-TeamMember-20260510T120000`).
    pub id: String,
    /// Absolute path to the written `.md` file.
    pub path: PathBuf,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Emit v1→v2 Migration documents for every cutover that the lock transition
/// `lock_old_version → lock_new_version` crosses, provided the project's v1
/// directory is non-empty.
///
/// # Arguments
///
/// * `lock_old_version` — processkit version recorded in the *old* lock
///   (the version installed before this apply).  Pass `""` for a fresh
///   project with no prior lock.
/// * `lock_new_version` — processkit version that will be recorded in the
///   *new* lock (the version being installed now).
/// * `project_root`     — absolute path to the project root; all relative
///   paths in [`CutoverDescriptor`] are resolved against this.
///
/// # Returns
///
/// A `Vec` of [`MigrationEmission`] — one per Migration document written.
/// Returns an empty `Vec` when there is nothing to emit (no crossing, no v1
/// files, or all relevant Migrations already exist).
pub fn emit_v1_v2_migrations(
    lock_old_version: &str,
    lock_new_version: &str,
    project_root: &Path,
) -> Result<Vec<MigrationEmission>> {
    let old_sv = crate::content_source::parse_loose_semver(lock_old_version);
    let new_sv = crate::content_source::parse_loose_semver(lock_new_version);

    let mut emitted = Vec::new();

    for descriptor in V1_TO_V2_CUTOVERS {
        let cutover_sv = crate::content_source::parse_loose_semver(descriptor.upstream_release);

        // Determine whether this lock transition crosses the cutover boundary.
        //
        // "Crosses" means: old < cutover_release <= new.
        // When old_version is empty / unparseable we treat it as "below every
        // release", i.e. always crossing.
        let crosses = match (&old_sv, &new_sv, &cutover_sv) {
            (_, _, None) => false, // Malformed catalogue entry — skip.
            (_, None, _) => false, // Unparseable new version — skip.
            (None, Some(new), Some(cutover)) => new >= cutover,
            (Some(old), Some(new), Some(cutover)) => old < cutover && new >= cutover,
        };

        if !crosses {
            continue;
        }

        // Check that the v1 directory exists and has at least one file.
        let v1_path = project_root.join(descriptor.v1_dir);
        if !dir_has_files(&v1_path)? {
            continue;
        }

        // Idempotency check: skip if a Migration with the same marker already
        // exists anywhere under context/migrations/.
        let marker = cutover_marker(descriptor.v1_kind, descriptor.v2_kind);
        if migration_already_exists(project_root, &marker)? {
            continue;
        }

        // Emit the Migration document.
        let emission = write_v1_v2_migration(project_root, descriptor, &v1_path, &marker)?;
        emitted.push(emission);
    }

    Ok(emitted)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Returns the idempotency marker string embedded in each Migration document.
fn cutover_marker(v1_kind: &str, v2_kind: &str) -> String {
    format!("v1v2-cutover-{}-to-{}", v1_kind.to_lowercase(), v2_kind.to_lowercase())
}

/// Returns `true` if `dir` exists and contains at least one regular file
/// (recursively).
fn dir_has_files(dir: &Path) -> Result<bool> {
    if !dir.is_dir() {
        return Ok(false);
    }
    for entry in
        fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            return Ok(true);
        }
        if path.is_dir() && dir_has_files(&path)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Returns `true` when any migration file under `context/migrations/` (in any
/// state sub-directory) already contains `marker`.
fn migration_already_exists(project_root: &Path, marker: &str) -> Result<bool> {
    for state_dir in ["pending", "in-progress", "applied"] {
        let dir = project_root.join("context/migrations").join(state_dir);
        if !dir.is_dir() {
            continue;
        }
        for file in walk_files(&dir)? {
            if file.extension().and_then(|e| e.to_str()) == Some("md")
                && fs::read_to_string(&file)
                    .map(|body| body.contains(marker))
                    .unwrap_or(false)
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Recursively collect all files under `root`.
fn walk_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !root.is_dir() {
        return Ok(out);
    }
    for entry in
        fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_files(&path)?);
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(out)
}

/// Collect all `.md` file paths relative to `v1_path` (for the "Affected
/// files" section of the Migration document).
fn collect_v1_file_rel_paths(v1_path: &Path, project_root: &Path) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    for file in walk_files(v1_path)? {
        if file.extension().and_then(|e| e.to_str()) == Some("md") {
            let rel = file
                .strip_prefix(project_root)
                .unwrap_or(&file)
                .to_string_lossy()
                .replace('\\', "/");
            paths.push(rel);
        }
    }
    paths.sort();
    Ok(paths)
}

/// Write one v1→v2 Migration document and return its emission descriptor.
fn write_v1_v2_migration(
    project_root: &Path,
    desc: &CutoverDescriptor,
    v1_path: &Path,
    marker: &str,
) -> Result<MigrationEmission> {
    let pending_dir = project_root.join("context/migrations/pending");
    fs::create_dir_all(&pending_dir)
        .with_context(|| format!("failed to create {}", pending_dir.display()))?;

    let now = chrono::Utc::now();
    let now_iso = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let timestamp = now.format("%Y%m%dT%H%M%S");
    let id = format!(
        "MIG-V1V2-{}-to-{}-{}",
        desc.v1_kind, desc.v2_kind, timestamp
    );
    let out_path = pending_dir.join(format!("{}.md", id));

    let affected_files = collect_v1_file_rel_paths(v1_path, project_root)?;

    let mut body = String::new();

    // --- YAML frontmatter ---
    body.push_str("---\n");
    body.push_str("apiVersion: processkit.projectious.work/v1\n");
    body.push_str("kind: Migration\n");
    body.push_str("metadata:\n");
    body.push_str(&format!("  id: {}\n", id));
    body.push_str(&format!("  created: {}\n", now_iso));
    body.push_str("spec:\n");
    body.push_str("  source: aibox\n");
    body.push_str(&format!(
        "  from_version: {}\n",
        yaml_scalar(desc.upstream_release)
    ));
    body.push_str(&format!(
        "  to_version: {}\n",
        yaml_scalar(desc.upstream_release)
    ));
    body.push_str("  state: pending\n");
    body.push_str("  generated_by: aibox apply\n");
    body.push_str(&format!("  generated_at: {}\n", now_iso));
    body.push_str(&format!("  summary: {}\n", yaml_scalar(desc.summary)));
    body.push_str("  affected_groups:\n");
    body.push_str(&format!("    - {}\n", desc.v1_kind.to_lowercase()));
    body.push_str(&format!(
        "  transformation_hint: {}\n",
        desc.transformation_hint.as_str()
    ));
    body.push_str(&format!("  dec_ref: {}\n", yaml_scalar(desc.dec_ref)));
    body.push_str(&format!("  marker: {}\n", marker));
    body.push_str("---\n\n");

    // --- Markdown body ---
    body.push_str(&format!("# Migration {}\n\n", id));
    body.push_str(&format!(
        "processkit `{}` supersedes the v1 `{}` entity type with the v2 `{}` primitive.\n\n",
        desc.upstream_release, desc.v1_kind, desc.v2_kind
    ));
    body.push_str(&format!(
        "Legacy `{}` files in `{}` need to be {} into `{}` entities — \
         or explicitly accepted as historical artefacts — so they no longer \
         interfere with processkit tooling.\n\n",
        desc.v1_kind,
        desc.v1_dir,
        desc.transformation_hint.as_str(),
        desc.v2_kind,
    ));

    // Affected files section
    body.push_str("## Affected files\n\n");
    if affected_files.is_empty() {
        body.push_str("_(no `.md` files found — directory may contain non-markdown content)_\n\n");
    } else {
        for f in &affected_files {
            body.push_str(&format!("- `{}`\n", f));
        }
        body.push('\n');
    }

    // Proposed transformation
    body.push_str("## Proposed transformation\n\n");
    match desc.transformation_hint {
        TransformationHint::Migrate => {
            body.push_str(&format!(
                "For each `{}` file above, create a corresponding `{}` entity in `{}`.\n\
                 Copy over relevant spec fields, update any cross-references, \
                 then remove or archive the old `{}` file.\n\n",
                desc.v1_kind, desc.v2_kind, desc.v2_dir, desc.v1_kind
            ));
        }
        TransformationHint::Archive => {
            body.push_str(&format!(
                "Move each `{}` file to an archive location (e.g. `context/archive/`) \
                 with a note recording why it was superseded.  Do not delete without \
                 archiving — these may be historical artefacts worth preserving.\n\n",
                desc.v1_kind
            ));
        }
        TransformationHint::NoOp => {
            body.push_str(&format!(
                "`{}` is additive.  Existing `{}` files can remain as-is; mark this \
                 migration applied once you have reviewed them.\n\n",
                desc.v2_kind, desc.v1_kind
            ));
        }
    }

    // How to apply
    body.push_str("## How to apply\n\n");
    body.push_str(
        "1. Complete the transformation described above for each affected file.\n\
         2. Call the `apply_migration` MCP tool with this Migration's ID:\n\n",
    );
    body.push_str("   ```\n");
    body.push_str(&format!(
        "   apply_migration(id=\"{}\")\n",
        id
    ));
    body.push_str("   ```\n\n");
    body.push_str(
        "3. Alternatively, manually update `spec.state` from `pending` to `applied` \
         in this file and move it to `context/migrations/applied/`.\n\n",
    );

    // References
    body.push_str("## References\n\n");
    body.push_str(&format!(
        "- Decision authorising this cutover: `{}`\n",
        desc.dec_ref
    ));
    body.push_str(&format!(
        "- v1 entity directory: `{}`\n",
        desc.v1_dir
    ));
    body.push_str(&format!(
        "- v2 entity directory: `{}`\n",
        desc.v2_dir
    ));
    body.push_str(&format!(
        "- upstream processkit release: `{}`\n",
        desc.upstream_release
    ));

    fs::write(&out_path, &body)
        .with_context(|| format!("failed to write {}", out_path.display()))?;

    Ok(MigrationEmission { id, path: out_path })
}

/// Minimal YAML scalar quoting — mirrors the implementation in `runtime_sync`.
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Synthetic cutover descriptor used across tests.
    const TEST_CUTOVER: CutoverDescriptor = CutoverDescriptor {
        upstream_release: "v0.50.0",
        v1_kind: "Widget",
        v2_kind: "Gadget",
        v1_dir: "context/widgets/",
        v2_dir: "context/gadgets/",
        dec_ref: "DEC-20260101_0000-TestSlug",
        summary: "v1 Widget superseded by v2 Gadget.",
        transformation_hint: TransformationHint::Migrate,
    };

    fn setup_v1_dir(root: &Path) {
        let widgets = root.join("context/widgets");
        fs::create_dir_all(&widgets).unwrap();
        fs::write(widgets.join("WIDGET-alpha.md"), "---\nkind: Widget\n---\n").unwrap();
    }

    fn run_emit(
        root: &Path,
        old: &str,
        new: &str,
        cutovers: &[CutoverDescriptor],
    ) -> Vec<MigrationEmission> {
        // We can't call emit_v1_v2_migrations with a custom catalogue slice
        // because the catalogue is a compile-time const.  Instead, exercise
        // the lower-level helpers directly.
        let old_sv = crate::content_source::parse_loose_semver(old);
        let new_sv = crate::content_source::parse_loose_semver(new);

        let mut emitted = Vec::new();
        for desc in cutovers {
            let cutover_sv = crate::content_source::parse_loose_semver(desc.upstream_release);
            let crosses = match (&old_sv, &new_sv, &cutover_sv) {
                (_, _, None) => false,
                (_, None, _) => false,
                (None, Some(new), Some(cutover)) => new >= cutover,
                (Some(old), Some(new), Some(cutover)) => old < cutover && new >= cutover,
            };
            if !crosses {
                continue;
            }
            let v1_path = root.join(desc.v1_dir);
            if !dir_has_files(&v1_path).unwrap() {
                continue;
            }
            let marker = cutover_marker(desc.v1_kind, desc.v2_kind);
            if migration_already_exists(root, &marker).unwrap() {
                continue;
            }
            emitted.push(write_v1_v2_migration(root, desc, &v1_path, &marker).unwrap());
        }
        emitted
    }

    #[test]
    fn emits_exactly_one_migration_when_lock_crosses_cutover() {
        let tmp = tempfile::tempdir().unwrap();
        setup_v1_dir(tmp.path());

        let emissions = run_emit(tmp.path(), "v0.49.9", "v0.50.0", &[TEST_CUTOVER]);

        assert_eq!(emissions.len(), 1, "expected exactly one Migration emitted");
        let emission = &emissions[0];
        assert!(emission.id.starts_with("MIG-V1V2-Widget-to-Gadget-"));
        let body = fs::read_to_string(&emission.path).unwrap();
        assert!(body.contains("kind: Migration"));
        assert!(body.contains("state: pending"));
        assert!(body.contains("v1v2-cutover-widget-to-gadget"));
        assert!(body.contains("WIDGET-alpha.md"));
        assert!(body.contains("DEC-20260101_0000-TestSlug"));
    }

    #[test]
    fn emits_nothing_when_v1_dir_absent() {
        let tmp = tempfile::tempdir().unwrap();
        // Do NOT create context/widgets — v1_dir is absent.

        let emissions = run_emit(tmp.path(), "v0.49.9", "v0.50.0", &[TEST_CUTOVER]);
        assert!(emissions.is_empty(), "no emission expected when v1_dir is absent");
    }

    #[test]
    fn emits_nothing_when_lock_does_not_cross_cutover() {
        let tmp = tempfile::tempdir().unwrap();
        setup_v1_dir(tmp.path());

        // Both old and new are after the cutover — no crossing.
        let emissions = run_emit(tmp.path(), "v0.50.0", "v0.51.0", &[TEST_CUTOVER]);
        assert!(
            emissions.is_empty(),
            "no emission expected when lock does not cross the cutover boundary"
        );
    }

    #[test]
    fn is_idempotent_does_not_double_emit() {
        let tmp = tempfile::tempdir().unwrap();
        setup_v1_dir(tmp.path());

        let first = run_emit(tmp.path(), "v0.49.9", "v0.50.0", &[TEST_CUTOVER]);
        assert_eq!(first.len(), 1);

        // Second run should find the existing marker and skip.
        let second = run_emit(tmp.path(), "v0.49.9", "v0.50.0", &[TEST_CUTOVER]);
        assert!(second.is_empty(), "second run must not double-emit");
    }

    #[test]
    fn emits_nothing_when_old_version_already_past_cutover() {
        let tmp = tempfile::tempdir().unwrap();
        setup_v1_dir(tmp.path());

        // old is already at/after the cutover — never crossing.
        let emissions = run_emit(tmp.path(), "v0.50.0", "v0.52.0", &[TEST_CUTOVER]);
        assert!(emissions.is_empty(), "no emission when old >= cutover");
    }

    #[test]
    fn marker_uniquely_identifies_cutover() {
        let m1 = cutover_marker("Actor", "TeamMember");
        let m2 = cutover_marker("Process", "Scope");
        assert_ne!(m1, m2);
        assert_eq!(m1, "v1v2-cutover-actor-to-teammember");
        assert_eq!(m2, "v1v2-cutover-process-to-scope");
    }
}
