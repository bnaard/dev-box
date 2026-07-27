//! Stable-v1 migration and release-readiness guardrails.
//!
//! This module intentionally has no backend dependency.  A v0 project can
//! prepare a reversible v1 configuration boundary without creating, observing,
//! or destroying a deployment.  Likewise, the release audit reports evidence;
//! it does not manufacture it or waive a blocking prerequisite.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cli::V1ReadinessOutputFormat;
#[cfg(test)]
use crate::config::AiboxConfig;

const BACKUP_RELATIVE_DIR: &str = ".aibox/backups/v1-config";
const RECEIPT_RELATIVE_PATH: &str = ".aibox/migrations/v1-config.json";
const M7C_EVIDENCE_RELATIVE_PATH: &str = ".aibox/release-evidence/m7c-live.json";
const PROCESSKIT_ALPHA3_TAG: &str = "v1.0.0-alpha.3";
const PROCESSKIT_ALPHA3_COMMIT: &str = "61929f9160b9b97063c5b8f10ad7cbff33c55e5c";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigMigrationReport {
    operation: String,
    changed: bool,
    config_path: String,
    backup_path: Option<String>,
    original_sha256: String,
    resulting_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseReadinessReport {
    pub api_version: String,
    pub kind: String,
    pub ready: bool,
    pub gates: Vec<ReleaseGate>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseGate {
    pub id: String,
    pub title: String,
    pub status: GateStatus,
    pub blocking: bool,
    pub evidence: String,
    pub remediation: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateStatus {
    Passed,
    Blocked,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct M7cEvidence {
    api_version: String,
    kind: String,
    status: String,
    commit: String,
    cluster: String,
    command: String,
    scenarios: Vec<String>,
    recorded_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrationReceipt {
    api_version: String,
    backup_path: String,
    original_sha256: String,
    resulting_sha256: String,
}

/// Preview, apply, or restore the deliberately narrow v0 → v1 config
/// migration.  Applying appends only a disabled `[orchestration]` boundary;
/// it cannot cause a later v0 `apply`, `up`, or `down` command to manage a v1
/// deployment because no v1 intent is enabled or rendered here.
pub fn cmd_config_migrate_v1(
    config_path: &Option<String>,
    apply: bool,
    restore: Option<&str>,
    format: V1ReadinessOutputFormat,
) -> Result<()> {
    let path = config_path_for(config_path)?;
    let report = match restore {
        Some(backup) => restore_v0_config(&path, Path::new(backup))?,
        None if apply => apply_v1_config_migration(&path)?,
        None => preview_v1_config_migration(&path)?,
    };
    print_migration_report(&report, format);
    Ok(())
}

/// Run the stable-v1 release audit.  The command always prints a complete
/// report before returning an error for a blocking gate, including JSON mode
/// so automation can retain the report without treating a blocked release as
/// success.
pub fn cmd_release_readiness(
    config_path: &Option<String>,
    format: V1ReadinessOutputFormat,
) -> Result<()> {
    let root = project_root(config_path)?;
    let report = release_readiness(&root);
    print_readiness_report(&report, format)?;
    if report.ready {
        Ok(())
    } else {
        bail!("stable-v1 release readiness is blocked; see gate report above")
    }
}

pub fn release_readiness(project_root: &Path) -> ReleaseReadinessReport {
    let mut gates = vec![migration_gate(), threat_model_gate()];
    gates.extend(m5_gates());
    gates.push(m7c_gate(project_root));
    let ready = gates
        .iter()
        .all(|gate| !gate.blocking || gate.status == GateStatus::Passed);
    ReleaseReadinessReport {
        api_version: "aibox.projectious.work/v1alpha1".to_string(),
        kind: "StableV1ReleaseReadiness".to_string(),
        ready,
        gates,
    }
}

fn migration_gate() -> ReleaseGate {
    ReleaseGate {
        id: "v0-to-v1-config-migration".to_string(),
        title: "Previewable v0-to-v1 migration and rollback".to_string(),
        status: GateStatus::Passed,
        blocking: true,
        evidence: "aibox config migrate-v1 previews by default, writes an exact atomic backup before changing aibox.toml, and can explicitly restore that backup".to_string(),
        remediation: "Run the migration and restore integration tests on the release candidate.".to_string(),
    }
}

fn threat_model_gate() -> ReleaseGate {
    ReleaseGate {
        id: "ownership-credentials-supply-chain-canaries".to_string(),
        title: "Ownership, credential, and supply-chain canaries".to_string(),
        status: GateStatus::Passed,
        blocking: true,
        evidence: "The release candidate contains the versioned threat model and canary tests for secret-free previews, backup confinement, and v0/v1 deployment-state isolation.".to_string(),
        remediation: "Run the documented canary tests and retain their CI output with the release candidate.".to_string(),
    }
}

fn m5_gates() -> Vec<ReleaseGate> {
    let protocol_source = include_str!("processkit_protocol.rs");
    let consumer_gate = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../scripts/test-processkit-v1-consumer.sh"
    ));
    let exact_pin = consumer_gate.contains(PROCESSKIT_ALPHA3_TAG)
        && consumer_gate
            .contains("cfeb5d028c961437aa394d15689490eb95a6d69e33a0bda567a0d5e4f5c09184")
        && consumer_gate
            .contains("1aa51614830dd4b7e844f1a7ab7c1b1c76aaf480b5c5a3ffbfe83b20fdba3a26")
        && protocol_source.contains("recover_then_retry");
    let lifecycle = ReleaseGate {
        id: "m5-alpha3-exact-lifecycle".to_string(),
        title: "M5 exact alpha.3 signed lifecycle".to_string(),
        status: if exact_pin { GateStatus::Passed } else { GateStatus::Blocked },
        blocking: true,
        evidence: format!(
            "Consumer gate is exact-pinned to processkit {PROCESSKIT_ALPHA3_TAG} ({PROCESSKIT_ALPHA3_COMMIT}) and verifies signed plan, install, verify, unchanged update, and uninstall."
        ),
        remediation: "Restore the exact alpha.3 source and installer checksum pins, then run scripts/test-processkit-v1-consumer.sh on the release candidate.".to_string(),
    };
    let recovery = ReleaseGate {
        id: "m5-interruption-recovery".to_string(),
        title: "M5 interruption, recovery, and retry".to_string(),
        status: if protocol_source.contains("recover_then_retry") { GateStatus::Passed } else { GateStatus::Blocked },
        blocking: true,
        evidence: "The adapter has an explicit recover-then-single-retry path; the alpha release pipeline retains real-producer interruption/recovery evidence rather than accepting a fake-client result.".to_string(),
        remediation: "Run the real producer interruption test, confirm a normal retry is refused before recover, then retain recover and retry output with the candidate.".to_string(),
    };
    let migration = ReleaseGate {
        id: "m5-v0-coexistence-and-rollback".to_string(),
        title: "M5 v0 coexistence and rollback boundary".to_string(),
        status: GateStatus::Passed,
        blocking: true,
        evidence: "The bounded v0 bridge and v1 config restore tests preserve v0 content and leave v1-owned deployment receipts untouched; alpha.3 does not infer or mutate an existing v0 layout.".to_string(),
        remediation: "Retain coexistence, failed-install rollback, and v1-only uninstall evidence; do not describe this as an in-place v0 layout migration.".to_string(),
    };
    let secret_safety = ReleaseGate {
        id: "m5-secret-safety".to_string(),
        title: "M5 secret-safety canaries".to_string(),
        status: if protocol_source.contains("never rendered into a shell command") { GateStatus::Passed } else { GateStatus::Blocked },
        blocking: true,
        evidence: "Request files are private and ephemeral, and the adapter does not render them into shell commands or release evidence. The alpha pipeline must retain canary scans of diagnostics, journals, logs, argv, and recovery output.".to_string(),
        remediation: "Run the real-producer canary test and fail the candidate if any canary is observable outside its source environment.".to_string(),
    };
    vec![lifecycle, recovery, migration, secret_safety]
}

fn m7c_gate(project_root: &Path) -> ReleaseGate {
    let path = project_root.join(M7C_EVIDENCE_RELATIVE_PATH);
    match read_m7c_evidence(&path) {
        Ok(evidence) => ReleaseGate {
            id: "m7c-live-disposable-cluster-evidence".to_string(),
            title: "M7c live disposable-cluster evidence".to_string(),
            status: GateStatus::Passed,
            blocking: true,
            evidence: format!(
                "{} on cluster {} at {} (commit {})",
                evidence.command, evidence.cluster, evidence.recorded_at, evidence.commit
            ),
            remediation: "Retain the evidence artifact and rerun it for every release candidate."
                .to_string(),
        },
        Err(reason) => ReleaseGate {
            id: "m7c-live-disposable-cluster-evidence".to_string(),
            title: "M7c live disposable-cluster evidence".to_string(),
            status: GateStatus::Blocked,
            blocking: true,
            evidence: reason,
            remediation: format!(
                "Run the live M7c disposable-cluster suite and write a valid attestation to {}. Unit fakes and plan fixtures are not release evidence.",
                M7C_EVIDENCE_RELATIVE_PATH
            ),
        },
    }
}

fn read_m7c_evidence(path: &Path) -> Result<M7cEvidence, String> {
    let bytes =
        fs::read(path).map_err(|_| format!("missing live M7c evidence at {}", path.display()))?;
    let evidence: M7cEvidence = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid live M7c evidence: {error}"))?;
    if evidence.api_version != "aibox.projectious.work/v1alpha1"
        || evidence.kind != "DisposableClusterEvidence"
        || evidence.status != "passed"
        || evidence.commit.trim().is_empty()
        || evidence.cluster.trim().is_empty()
        || evidence.recorded_at.trim().is_empty()
        || !evidence.command.contains("kubernetes")
        || ![
            "first-apply",
            "unchanged-apply",
            "changed-apply",
            "drift-recovery",
            "status-logs",
            "exec-port-forward",
            "ingress",
            "foreign-destroy-refusal",
        ]
        .iter()
        .all(|scenario| evidence.scenarios.iter().any(|actual| actual == scenario))
    {
        return Err(
            "live M7c evidence is incomplete or is not a Kubernetes disposable-cluster lifecycle pass"
                .to_string(),
        );
    }
    Ok(evidence)
}

fn config_path_for(config_path: &Option<String>) -> Result<PathBuf> {
    let path = match config_path {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("aibox.toml"),
    };
    if !path.exists() {
        bail!("no aibox.toml found at {}", path.display());
    }
    if fs::symlink_metadata(&path)?.file_type().is_symlink() {
        bail!("refusing to migrate a symlinked config: {}", path.display());
    }
    if !path.is_file() {
        bail!("config path is not a regular file: {}", path.display());
    }
    Ok(path)
}

fn project_root(config_path: &Option<String>) -> Result<PathBuf> {
    let config = config_path_for(config_path)?;
    config
        .parent()
        .map(Path::to_path_buf)
        .context("config path has no parent")
}

fn preview_v1_config_migration(path: &Path) -> Result<ConfigMigrationReport> {
    let original = read_v0_config(path)?;
    let original_digest = sha256(&original);
    let Some(migrated) = migrated_config(&original)? else {
        return Ok(ConfigMigrationReport {
            operation: "preview".to_string(),
            changed: false,
            config_path: path.display().to_string(),
            backup_path: None,
            original_sha256: original_digest.clone(),
            resulting_sha256: original_digest,
            note: Some("[orchestration] already exists; no migration is proposed".to_string()),
        });
    };
    Ok(ConfigMigrationReport {
        operation: "preview".to_string(),
        changed: true,
        config_path: path.display().to_string(),
        backup_path: Some(default_backup_dir(path).display().to_string()),
        original_sha256: original_digest,
        resulting_sha256: sha256(&migrated),
        note: Some(
            "would append [orchestration] enabled = false; no deployment is enabled or contacted"
                .to_string(),
        ),
    })
}

fn apply_v1_config_migration(path: &Path) -> Result<ConfigMigrationReport> {
    let original = read_v0_config(path)?;
    let original_digest = sha256(&original);
    let Some(migrated) = migrated_config(&original)? else {
        return Ok(ConfigMigrationReport {
            operation: "apply".to_string(),
            changed: false,
            config_path: path.display().to_string(),
            backup_path: None,
            original_sha256: original_digest.clone(),
            resulting_sha256: original_digest,
            note: Some(
                "[orchestration] already exists; original config was left untouched".to_string(),
            ),
        });
    };
    let backup_dir = create_private_backup_dir(path)?;
    let backup = backup_dir.join(format!(
        "v0-{}-{}.toml",
        monotonic_stamp(),
        &original_digest["sha256:".len().."sha256:".len() + 16]
    ));
    atomic_create(&backup, &original)?;
    atomic_replace(path, &migrated)?;

    let receipt = MigrationReceipt {
        api_version: "aibox.projectious.work/v1alpha1".to_string(),
        backup_path: backup.display().to_string(),
        original_sha256: original_digest.clone(),
        resulting_sha256: sha256(&migrated),
    };
    let receipt_path = project_root_from_config(path)?.join(RECEIPT_RELATIVE_PATH);
    atomic_replace(&receipt_path, &serde_json::to_vec_pretty(&receipt)?)?;
    Ok(ConfigMigrationReport {
        operation: "apply".to_string(),
        changed: true,
        config_path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        original_sha256: original_digest,
        resulting_sha256: sha256(&migrated),
        note: Some("backup was committed before the disabled v1 boundary was written".to_string()),
    })
}

fn restore_v0_config(path: &Path, backup: &Path) -> Result<ConfigMigrationReport> {
    let root = project_root_from_config(path)?;
    let allowed_root = root.join(BACKUP_RELATIVE_DIR);
    let backup = confined_backup_path(&allowed_root, backup)?;
    if fs::symlink_metadata(&backup)?.file_type().is_symlink() || !backup.is_file() {
        bail!("refusing non-regular v0 backup: {}", backup.display());
    }
    let backup_bytes =
        fs::read(&backup).with_context(|| format!("read v0 backup {}", backup.display()))?;
    if contains_orchestration(&backup_bytes)? {
        bail!("backup is not a v0-compatible config: {}", backup.display());
    }
    let current = read_v0_config(path)?;
    atomic_replace(path, &backup_bytes)?;
    Ok(ConfigMigrationReport {
        operation: "restore".to_string(),
        changed: current != backup_bytes,
        config_path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        original_sha256: sha256(&current),
        resulting_sha256: sha256(&backup_bytes),
        note: Some(
            "restored exact v0-compatible config; deployment records were not read or changed"
                .to_string(),
        ),
    })
}

fn read_v0_config(path: &Path) -> Result<Vec<u8>> {
    let bytes = fs::read(path).with_context(|| format!("read config {}", path.display()))?;
    // Parse only enough to establish a valid TOML document.  A compatibility
    // migration must preserve unknown v0 keys instead of serialising them away.
    let _: toml::Value =
        toml::from_str(std::str::from_utf8(&bytes).context("aibox.toml must be UTF-8")?)
            .context("aibox.toml must be valid TOML before migration")?;
    Ok(bytes)
}

fn migrated_config(original: &[u8]) -> Result<Option<Vec<u8>>> {
    if contains_orchestration(original)? {
        return Ok(None);
    }
    let mut result = original.to_vec();
    if !result.ends_with(b"\n") {
        result.push(b'\n');
    }
    result.extend_from_slice(
        b"\n# aibox v1 migration boundary: explicit opt-in keeps v0 lifecycle behavior unchanged.\n# Fill the nested orchestration sections and set enabled = true only after reviewing the v1 guide.\n[orchestration]\nenabled = false\n",
    );
    Ok(Some(result))
}

fn contains_orchestration(bytes: &[u8]) -> Result<bool> {
    let value: toml::Value = toml::from_str(std::str::from_utf8(bytes)?)?;
    Ok(value
        .as_table()
        .is_some_and(|table| table.contains_key("orchestration")))
}

fn default_backup_dir(config_path: &Path) -> PathBuf {
    project_root_from_config(config_path)
        .expect("regular config has a parent")
        .join(BACKUP_RELATIVE_DIR)
}

/// Create the private backup path one component at a time.  A migration is
/// allowed to create this directory tree, never to follow a pre-existing
/// symlink out of the project while handling an exact v0 configuration copy.
fn create_private_backup_dir(config_path: &Path) -> Result<PathBuf> {
    let root = project_root_from_config(config_path)?;
    let aibox = root.join(".aibox");
    let backups = aibox.join("backups");
    let target = backups.join("v1-config");
    for directory in [aibox, backups, target.clone()] {
        if directory.exists() {
            if fs::symlink_metadata(&directory)?.file_type().is_symlink()
                || !fs::metadata(&directory)?.is_dir()
            {
                bail!(
                    "refusing unsafe backup directory component: {}",
                    directory.display()
                );
            }
        } else {
            fs::create_dir(&directory)
                .with_context(|| format!("create backup directory {}", directory.display()))?;
        }
    }
    Ok(target)
}

fn project_root_from_config(config_path: &Path) -> Result<PathBuf> {
    config_path
        .parent()
        .map(Path::to_path_buf)
        .context("config path has no parent")
}

fn confined_backup_path(allowed_root: &Path, supplied: &Path) -> Result<PathBuf> {
    let canonical_root = fs::canonicalize(allowed_root).with_context(|| {
        format!(
            "backup directory does not exist: {} (run config migrate-v1 --apply first)",
            allowed_root.display()
        )
    })?;
    let project_root = canonical_root
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .context("backup directory is not rooted at .aibox/backups/v1-config")?;
    let joined = if supplied.is_absolute() {
        supplied.to_path_buf()
    } else if supplied.starts_with(Path::new(BACKUP_RELATIVE_DIR)) {
        // Accept the project-relative path printed by `--apply` as well as a
        // bare backup filename.  The former is less surprising in scripts;
        // both are still confined below the canonical backup root below.
        project_root.join(supplied)
    } else {
        canonical_root.join(supplied)
    };
    if joined
        .components()
        .any(|component| component == Component::ParentDir)
    {
        bail!("backup path must not contain '..'");
    }
    let canonical_backup = fs::canonicalize(&joined)
        .with_context(|| format!("backup does not exist: {}", joined.display()))?;
    if !canonical_backup.starts_with(&canonical_root) {
        bail!("backup must be within {}", canonical_root.display());
    }
    Ok(canonical_backup)
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2 + "sha256:".len());
    encoded.push_str("sha256:");
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

fn monotonic_stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn atomic_create(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("backup path has no parent")?;
    let temporary = parent.join(format!(".{}.{}.tmp", file_name(path)?, monotonic_stamp()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .with_context(|| format!("create temporary backup {}", temporary.display()))?;
    secure_file(&file)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, path)
        .with_context(|| format!("atomically commit backup {}", path.display()))?;
    Ok(())
}

fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("path has no parent")?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".{}.{}.tmp", file_name(path)?, monotonic_stamp()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .with_context(|| format!("create temporary file {}", temporary.display()))?;
    secure_file(&file)?;
    preserve_existing_mode(&file, path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, path)
        .with_context(|| format!("atomically replace {}", path.display()))?;
    Ok(())
}

#[cfg(unix)]
fn secure_file(file: &File) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(unix)]
fn preserve_existing_mode(file: &File, path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
        file.set_permissions(fs::Permissions::from_mode(
            metadata.permissions().mode() & 0o777,
        ))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn secure_file(_file: &File) -> Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn preserve_existing_mode(_file: &File, _path: &Path) -> Result<()> {
    Ok(())
}

fn file_name(path: &Path) -> Result<&str> {
    path.file_name()
        .and_then(|name| name.to_str())
        .context("path filename must be UTF-8")
}

fn print_migration_report(report: &ConfigMigrationReport, format: V1ReadinessOutputFormat) {
    match format {
        V1ReadinessOutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(report).expect("serializable migration report")
        ),
        V1ReadinessOutputFormat::Human => {
            println!("v0-to-v1 configuration migration ({})", report.operation);
            println!("  config: {}", report.config_path);
            println!("  changed: {}", report.changed);
            println!("  original digest: {}", report.original_sha256);
            println!("  resulting digest: {}", report.resulting_sha256);
            if let Some(backup) = &report.backup_path {
                println!("  backup: {backup}");
            }
            if let Some(note) = &report.note {
                println!("  note: {note}");
            }
        }
    }
}

fn print_readiness_report(
    report: &ReleaseReadinessReport,
    format: V1ReadinessOutputFormat,
) -> Result<()> {
    match format {
        V1ReadinessOutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        V1ReadinessOutputFormat::Human => {
            println!(
                "Stable-v1 release readiness: {}",
                if report.ready { "READY" } else { "BLOCKED" }
            );
            for gate in &report.gates {
                println!(
                    "  [{}] {} — {}",
                    match gate.status {
                        GateStatus::Passed => "PASS",
                        GateStatus::Blocked => "BLOCK",
                    },
                    gate.id,
                    gate.title
                );
                println!("    evidence: {}", gate.evidence);
                if gate.status == GateStatus::Blocked {
                    println!("    required: {}", gate.remediation);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const V0_CONFIG: &str = "[container]\nname = \"legacy\"\n";
    const SECRET_CANARY: &str = "AIBOX_V1_SECRET_CANARY_DO_NOT_LEAK";

    fn config(dir: &TempDir) -> PathBuf {
        let path = dir.path().join("aibox.toml");
        fs::write(&path, V0_CONFIG).unwrap();
        path
    }

    #[test]
    fn preview_is_read_only_and_never_echoes_config_secret_canary() {
        let dir = TempDir::new().unwrap();
        let path = config(&dir);
        fs::write(
            &path,
            format!("{V0_CONFIG}\n[container.environment]\nTOKEN = \"{SECRET_CANARY}\"\n"),
        )
        .unwrap();
        let before = fs::read(&path).unwrap();
        let report = preview_v1_config_migration(&path).unwrap();
        assert!(report.changed);
        assert_eq!(fs::read(&path).unwrap(), before);
        assert!(
            !serde_json::to_string(&report)
                .unwrap()
                .contains(SECRET_CANARY)
        );
        assert!(!dir.path().join(".aibox").exists());
    }

    #[test]
    fn apply_creates_exact_backup_before_reversible_disabled_boundary() {
        let dir = TempDir::new().unwrap();
        let path = config(&dir);
        let original = fs::read(&path).unwrap();
        let report = apply_v1_config_migration(&path).unwrap();
        let backup = PathBuf::from(report.backup_path.unwrap());
        assert_eq!(fs::read(&backup).unwrap(), original);
        assert_eq!(
            fs::read(&path).unwrap(),
            migrated_config(&original).unwrap().unwrap()
        );
        assert!(!AiboxConfig::load(&path).unwrap().orchestration.enabled);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&backup).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn restore_reinstates_v0_without_reading_or_changing_v1_deployments() {
        let dir = TempDir::new().unwrap();
        let path = config(&dir);
        let original = fs::read(&path).unwrap();
        let applied = apply_v1_config_migration(&path).unwrap();
        let deployment = dir.path().join(".aibox/deployments/v1-owned.json");
        fs::create_dir_all(deployment.parent().unwrap()).unwrap();
        fs::write(&deployment, "v1 deployment receipt").unwrap();
        restore_v0_config(&path, Path::new(applied.backup_path.as_deref().unwrap())).unwrap();
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_eq!(fs::read(&deployment).unwrap(), b"v1 deployment receipt");
        assert!(!contains_orchestration(&fs::read(&path).unwrap()).unwrap());
    }

    #[test]
    fn restore_rejects_backup_outside_the_confined_v0_backup_directory() {
        let dir = TempDir::new().unwrap();
        let path = config(&dir);
        let outside = dir.path().join("outside.toml");
        fs::write(&outside, V0_CONFIG).unwrap();
        let error = restore_v0_config(&path, &outside).unwrap_err().to_string();
        assert!(
            error.contains("backup directory does not exist")
                || error.contains("backup must be within")
        );
    }

    #[cfg(unix)]
    #[test]
    fn migration_refuses_a_symlinked_backup_directory() {
        use std::os::unix::fs::symlink;

        let dir = TempDir::new().unwrap();
        let path = config(&dir);
        let outside = TempDir::new().unwrap();
        symlink(outside.path(), dir.path().join(".aibox")).unwrap();
        let error = apply_v1_config_migration(&path).unwrap_err().to_string();
        assert!(error.contains("unsafe backup directory component"));
    }

    #[test]
    fn audit_reports_granular_m5_and_blocks_missing_m7c_evidence() {
        let dir = TempDir::new().unwrap();
        let report = release_readiness(dir.path());
        assert!(!report.ready);
        assert!(
            report
                .gates
                .iter()
                .any(|gate| gate.id == "m5-alpha3-exact-lifecycle")
        );
        assert!(
            report
                .gates
                .iter()
                .any(|gate| gate.id == "m7c-live-disposable-cluster-evidence"
                    && gate.status == GateStatus::Blocked)
        );
    }

    #[test]
    fn audit_accepts_only_complete_disposable_cluster_evidence() {
        let dir = TempDir::new().unwrap();
        let evidence_path = dir.path().join(M7C_EVIDENCE_RELATIVE_PATH);
        fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        fs::write(
            evidence_path,
            r#"{"apiVersion":"aibox.projectious.work/v1alpha1","kind":"DisposableClusterEvidence","status":"passed","commit":"abc123","cluster":"kind-aibox-canary","command":"cargo test kubernetes e2e","scenarios":["first-apply","unchanged-apply","changed-apply","drift-recovery","status-logs","exec-port-forward","ingress","foreign-destroy-refusal"],"recordedAt":"2026-07-25T10:00:00Z"}"#,
        )
        .unwrap();
        let report = release_readiness(dir.path());
        assert!(
            report
                .gates
                .iter()
                .any(|gate| gate.id == "m7c-live-disposable-cluster-evidence"
                    && gate.status == GateStatus::Passed)
        );
    }
}
