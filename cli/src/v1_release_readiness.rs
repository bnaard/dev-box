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
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::cli::V1ReadinessOutputFormat;
#[cfg(test)]
use crate::config::AiboxConfig;
use crate::release_evidence::{DisposableClusterEvidence, ReleaseGateEvidence};

const BACKUP_RELATIVE_DIR: &str = ".aibox/backups/v1-config";
const RECEIPT_RELATIVE_PATH: &str = ".aibox/migrations/v1-config.json";
const M7C_EVIDENCE_RELATIVE_PATH: &str = ".aibox/release-evidence/m7c-live.json";
const GATE_EVIDENCE_RELATIVE_DIR: &str = ".aibox/release-evidence/v1-readiness";
const PROCESSKIT_ALPHA3_TAG: &str = "v1.0.0-alpha.3";

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
    mapped_fields: Vec<ConfigMigrationMapping>,
    unresolved_decisions: Vec<ConfigMigrationDecision>,
    ready_to_enable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigMigrationMapping {
    source: String,
    targets: Vec<String>,
    value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigMigrationDecision {
    id: String,
    target: String,
    reason: String,
    allowed_values: Vec<String>,
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
    intent_file: Option<&str>,
    restore: Option<&str>,
    format: V1ReadinessOutputFormat,
) -> Result<()> {
    let path = config_path_for(config_path)?;
    let report = match restore {
        Some(backup) => restore_v0_config(&path, Path::new(backup))?,
        None if apply => apply_v1_config_migration(&path, intent_file.map(Path::new))?,
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
    let mut gates = vec![
        migration_gate(project_root),
        threat_model_gate(project_root),
    ];
    gates.extend(m5_gates(project_root));
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

fn migration_gate(project_root: &Path) -> ReleaseGate {
    candidate_evidence_gate(
        project_root,
        "v0-to-v1-config-migration",
        "Previewable v0-to-v1 migration and rollback",
        true,
        "Run the migration and restore integration tests on the release candidate and retain their candidate-bound record.",
    )
}

fn threat_model_gate(project_root: &Path) -> ReleaseGate {
    candidate_evidence_gate(
        project_root,
        "ownership-credentials-supply-chain-canaries",
        "Ownership, credential, and supply-chain canaries",
        true,
        "Run the documented canary tests and retain their candidate-bound evidence record.",
    )
}

fn m5_gates(project_root: &Path) -> Vec<ReleaseGate> {
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
    let lifecycle = candidate_evidence_gate(
        project_root,
        "m5-alpha3-exact-lifecycle",
        "M5 exact alpha.3 signed lifecycle",
        exact_pin,
        "Restore the exact alpha.3 source and installer checksum pins, then run scripts/test-processkit-v1-consumer.sh and retain its candidate-bound evidence.",
    );
    let recovery = candidate_evidence_gate(
        project_root,
        "m5-interruption-recovery",
        "M5 interruption, recovery, and retry",
        protocol_source.contains("recover_then_retry"),
        "Run the real producer interruption test, confirm a normal retry is refused before recover, then retain recover and retry evidence bound to the candidate.",
    );
    let migration = candidate_evidence_gate(
        project_root,
        "m5-v0-coexistence-and-rollback",
        "M5 v0 coexistence and rollback boundary",
        true,
        "Retain coexistence, failed-install rollback, and v1-only uninstall evidence bound to the candidate; do not describe this as an in-place v0 layout migration.",
    );
    let secret_safety = candidate_evidence_gate(
        project_root,
        "m5-secret-safety",
        "M5 secret-safety canaries",
        protocol_source.contains("never rendered into a shell command"),
        "Run the real-producer canary test and retain a candidate-bound canary record that fails if a canary is observable outside its source environment.",
    );
    vec![lifecycle, recovery, migration, secret_safety]
}

fn candidate_evidence_gate(
    project_root: &Path,
    id: &str,
    title: &str,
    implementation_present: bool,
    remediation: &str,
) -> ReleaseGate {
    if !implementation_present {
        return ReleaseGate {
            id: id.to_string(),
            title: title.to_string(),
            status: GateStatus::Blocked,
            blocking: true,
            evidence: "required implementation surface is missing; source inspection is not release evidence".to_string(),
            remediation: remediation.to_string(),
        };
    }

    let path = project_root
        .join(GATE_EVIDENCE_RELATIVE_DIR)
        .join(format!("{id}.json"));
    match read_release_gate_evidence(&path) {
        Ok(evidence)
            if evidence.gate == id
                && release_gate_evidence_matches_runtime_candidate(&evidence)
                && release_gate_artifacts_match(project_root, &evidence) =>
        {
            ReleaseGate {
                id: id.to_string(),
                title: title.to_string(),
                status: GateStatus::Passed,
                blocking: true,
                evidence: format!(
                    "candidate-bound record from {} completed at {}",
                    evidence.command, evidence.completed_at
                ),
                remediation: remediation.to_string(),
            }
        }
        Ok(evidence) if evidence.gate != id => ReleaseGate {
            id: id.to_string(),
            title: title.to_string(),
            status: GateStatus::Blocked,
            blocking: true,
            evidence: format!(
                "candidate evidence names gate {}, expected {id}",
                evidence.gate
            ),
            remediation: remediation.to_string(),
        },
        Ok(_) => ReleaseGate {
            id: id.to_string(),
            title: title.to_string(),
            status: GateStatus::Blocked,
            blocking: true,
            evidence: "candidate evidence is not bound to the runtime release candidate"
                .to_string(),
            remediation: remediation.to_string(),
        },
        Err(reason) => ReleaseGate {
            id: id.to_string(),
            title: title.to_string(),
            status: GateStatus::Blocked,
            blocking: true,
            evidence: reason,
            remediation: remediation.to_string(),
        },
    }
}

fn m7c_gate(project_root: &Path) -> ReleaseGate {
    let path = project_root.join(M7C_EVIDENCE_RELATIVE_PATH);
    match read_m7c_evidence(&path) {
        Ok(evidence) if evidence_matches_runtime_candidate(&evidence) => ReleaseGate {
            id: "m7c-live-disposable-cluster-evidence".to_string(),
            title: "M7c live disposable-cluster evidence".to_string(),
            status: GateStatus::Passed,
            blocking: true,
            evidence: format!(
                "{} on cluster {} at {} (commit {})",
                evidence.command, evidence.cluster, evidence.recorded_at, evidence.candidate_commit
            ),
            remediation: "Retain the evidence artifact and rerun it for every release candidate."
                .to_string(),
        },
        Ok(_) => ReleaseGate {
            id: "m7c-live-disposable-cluster-evidence".to_string(),
            title: "M7c live disposable-cluster evidence".to_string(),
            status: GateStatus::Blocked,
            blocking: true,
            evidence: "live M7c evidence is not bound to the runtime release candidate".to_string(),
            remediation:
                "Rerun the live disposable-cluster suite with the candidate commit and binary."
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

fn evidence_matches_runtime_candidate(evidence: &DisposableClusterEvidence) -> bool {
    let expected_commit = std::env::var("RELEASE_CANDIDATE_SHA").ok();
    let expected_binary = std::env::var("AIBOX_RELEASE_BINARY_SHA256").ok();
    evidence_matches_candidate(
        evidence,
        expected_commit.as_deref(),
        expected_binary.as_deref(),
    )
}

fn release_gate_evidence_matches_runtime_candidate(evidence: &ReleaseGateEvidence) -> bool {
    let expected_commit = std::env::var("RELEASE_CANDIDATE_SHA").ok();
    let expected_binary = std::env::var("AIBOX_RELEASE_BINARY_SHA256").ok();
    evidence_matches_candidate_values(
        &evidence.candidate_commit,
        &evidence.binary_sha256,
        expected_commit.as_deref(),
        expected_binary.as_deref(),
    )
}

fn release_gate_artifacts_match(project_root: &Path, evidence: &ReleaseGateEvidence) -> bool {
    evidence.artifacts.iter().all(|artifact| {
        let path = project_root.join(&artifact.path);
        fs::read(path)
            .map(|bytes| sha256(&bytes) == artifact.sha256)
            .unwrap_or(false)
    })
}

fn evidence_matches_candidate(
    evidence: &DisposableClusterEvidence,
    expected_commit: Option<&str>,
    expected_binary: Option<&str>,
) -> bool {
    // Evidence without both expectations is only a well-formed attestation,
    // not release evidence.  Accepting it here would let the public readiness
    // command report a stale or hand-written file as passed when it was not
    // bound to the candidate being released.
    evidence_matches_candidate_values(
        &evidence.candidate_commit,
        &evidence.binary_sha256,
        expected_commit,
        expected_binary,
    )
}

fn evidence_matches_candidate_values(
    candidate_commit: &str,
    binary_sha256: &str,
    expected_commit: Option<&str>,
    expected_binary: Option<&str>,
) -> bool {
    let (Some(expected_commit), Some(expected_binary)) = (expected_commit, expected_binary) else {
        return false;
    };
    expected_commit == candidate_commit && expected_binary == binary_sha256
}

fn read_m7c_evidence(path: &Path) -> Result<DisposableClusterEvidence, String> {
    let bytes =
        fs::read(path).map_err(|_| format!("missing live M7c evidence at {}", path.display()))?;
    DisposableClusterEvidence::from_json(&bytes)
        .map_err(|error| format!("invalid live M7c evidence: {error}"))
}

fn read_release_gate_evidence(path: &Path) -> Result<ReleaseGateEvidence, String> {
    let bytes = fs::read(path).map_err(|_| {
        format!(
            "missing candidate-bound gate evidence at {}",
            path.display()
        )
    })?;
    ReleaseGateEvidence::from_json(&bytes)
        .map_err(|error| format!("invalid candidate-bound gate evidence: {error}"))
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
    let (mapped_fields, unresolved_decisions) = migration_analysis(&original)?;
    let Some(migrated) = migrated_config(&original)? else {
        return Ok(ConfigMigrationReport {
            operation: "preview".to_string(),
            changed: false,
            config_path: path.display().to_string(),
            backup_path: None,
            original_sha256: original_digest.clone(),
            resulting_sha256: original_digest,
            note: Some("[orchestration] already exists; no migration is proposed".to_string()),
            mapped_fields: Vec::new(),
            unresolved_decisions: Vec::new(),
            ready_to_enable: false,
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
            "would append a disabled boundary; resolve every reported decision before enabling deployment"
                .to_string(),
        ),
        mapped_fields,
        ready_to_enable: unresolved_decisions.is_empty(),
        unresolved_decisions,
    })
}

fn apply_v1_config_migration(
    path: &Path,
    intent_file: Option<&Path>,
) -> Result<ConfigMigrationReport> {
    let original = read_v0_config(path)?;
    let original_digest = sha256(&original);
    let (mapped_fields, mut unresolved_decisions) = migration_analysis(&original)?;
    let intent = intent_file.map(read_migration_intent).transpose()?;
    if intent.is_some() {
        unresolved_decisions.clear();
    }
    let Some(migrated) = migrated_config_with_intent(&original, intent.as_deref())? else {
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
            mapped_fields: Vec::new(),
            unresolved_decisions: Vec::new(),
            ready_to_enable: false,
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
        note: Some(
            "backup was committed before the disabled boundary; resolve every reported decision before enabling deployment"
                .to_string(),
        ),
        mapped_fields,
        ready_to_enable: intent.is_some() && unresolved_decisions.is_empty(),
        unresolved_decisions,
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
        mapped_fields: Vec::new(),
        unresolved_decisions: Vec::new(),
        ready_to_enable: false,
    })
}

fn migration_analysis(
    original: &[u8],
) -> Result<(Vec<ConfigMigrationMapping>, Vec<ConfigMigrationDecision>)> {
    let value: toml::Value = toml::from_str(std::str::from_utf8(original)?)?;
    let container = value
        .get("container")
        .and_then(toml::Value::as_table)
        .context("[container] is required for v0-to-v1 migration")?;
    let name = container
        .get("name")
        .and_then(toml::Value::as_str)
        .context("[container].name is required for v0-to-v1 migration")?;
    let mapped = vec![ConfigMigrationMapping {
        source: "container.name".to_string(),
        targets: vec![
            "orchestration.fleet.name".to_string(),
            "orchestration.fleet.services[0].name".to_string(),
            "orchestration.deployment.name".to_string(),
        ],
        value: name.to_string(),
    }];
    let mut decisions = vec![
        migration_decision(
            "immutable-image",
            "orchestration.image.reference,digest",
            "a v0 generated image tag is mutable and cannot prove a deployable image digest",
            &["registry reference plus sha256 digest"],
        ),
        migration_decision(
            "platform",
            "orchestration.image.platform",
            "the deployment platform is an operator choice, not a property of the v0 config",
            &["linux-amd64", "linux-arm64"],
        ),
        migration_decision(
            "target",
            "orchestration.target",
            "v0 local-container settings do not identify an authorized deployment target",
            &[
                "compose context and scope",
                "kubernetes context and namespace",
            ],
        ),
        migration_decision(
            "owner-id",
            "orchestration.deployment.owner_id",
            "resource ownership must be explicitly assigned",
            &["stable non-secret owner identifier"],
        ),
        migration_decision(
            "connections",
            "orchestration.connections",
            "v0 attach behavior does not determine the intended v1 connection transports",
            &[
                "compose-exec",
                "kubernetes-exec",
                "kubernetes-port-forward",
                "ssh",
            ],
        ),
    ];
    if container
        .get("environment")
        .and_then(toml::Value::as_table)
        .is_some_and(|environment| !environment.is_empty())
    {
        decisions.push(migration_decision(
            "environment",
            "orchestration.fleet.services[0].environment",
            "v0 environment values may contain secrets and cannot be copied into deployment intent",
            &[
                "environment-variable reference",
                "file reference",
                "secret-manager reference",
            ],
        ));
    }
    if container
        .get("extra_volumes")
        .and_then(toml::Value::as_array)
        .is_some_and(|volumes| !volumes.is_empty())
    {
        decisions.push(migration_decision(
            "volumes",
            "orchestration fleet storage",
            "host bind mounts are local v0 behavior with no portable v1 deployment equivalent",
            &["remove", "backend-owned storage configuration"],
        ));
    }
    Ok((mapped, decisions))
}

fn migration_decision(
    id: &str,
    target: &str,
    reason: &str,
    allowed_values: &[&str],
) -> ConfigMigrationDecision {
    ConfigMigrationDecision {
        id: id.to_string(),
        target: target.to_string(),
        reason: reason.to_string(),
        allowed_values: allowed_values
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    }
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
    migrated_config_with_intent(original, None)
}

fn migrated_config_with_intent(original: &[u8], intent: Option<&[u8]>) -> Result<Option<Vec<u8>>> {
    if contains_orchestration(original)? {
        return Ok(None);
    }
    let mut result = original.to_vec();
    if !result.ends_with(b"\n") {
        result.push(b'\n');
    }
    result.extend_from_slice(
        b"\n# aibox v1 migration boundary: disabled until explicit activation after plan review.\n",
    );
    match intent {
        Some(intent) => result.extend_from_slice(intent),
        None => result.extend_from_slice(
            b"# Fill the nested orchestration sections and set enabled = true only after reviewing the v1 guide.\n[orchestration]\nenabled = false\n",
        ),
    }
    Ok(Some(result))
}

fn read_migration_intent(path: &Path) -> Result<Vec<u8>> {
    if fs::symlink_metadata(path)?.file_type().is_symlink() || !path.is_file() {
        bail!("refusing non-regular migration intent: {}", path.display());
    }
    let bytes =
        fs::read(path).with_context(|| format!("read migration intent {}", path.display()))?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| anyhow::anyhow!("migration intent must be UTF-8 TOML"))?;
    let mut value: toml::Value =
        toml::from_str(text).map_err(|_| anyhow::anyhow!("migration intent must be valid TOML"))?;
    let root = value
        .as_table_mut()
        .context("migration intent must be a TOML document")?;
    if root.len() != 1 || !root.contains_key("orchestration") {
        bail!("migration intent may contain only one [orchestration] document");
    }
    let orchestration = root
        .get_mut("orchestration")
        .and_then(toml::Value::as_table_mut)
        .context("migration intent requires an [orchestration] table")?;
    orchestration.insert("enabled".to_string(), toml::Value::Boolean(false));

    let rendered = toml::to_string(&value)?;
    let mut validation_document = value;
    validation_document
        .get_mut("orchestration")
        .and_then(toml::Value::as_table_mut)
        .expect("validated orchestration table")
        .insert("enabled".to_string(), toml::Value::Boolean(true));
    let validation_toml = toml::to_string(&validation_document)?;
    let validation = format!("[container]\nname = \"migration-validation\"\n\n{validation_toml}");
    crate::config::AiboxConfig::from_str(&validation).map_err(|_| {
        anyhow::anyhow!("migration intent is not a complete valid v1 orchestration configuration")
    })?;
    Ok(rendered.into_bytes())
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
            for mapping in &report.mapped_fields {
                println!(
                    "  mapped: {} -> {} ({})",
                    mapping.source,
                    mapping.targets.join(", "),
                    mapping.value
                );
            }
            println!("  ready to enable: {}", report.ready_to_enable);
            for decision in &report.unresolved_decisions {
                println!(
                    "  unresolved {}: {} — {} (choose: {})",
                    decision.id,
                    decision.target,
                    decision.reason,
                    decision.allowed_values.join(", ")
                );
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
        let report = apply_v1_config_migration(&path, None).unwrap();
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
        let applied = apply_v1_config_migration(&path, None).unwrap();
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
        let error = apply_v1_config_migration(&path, None)
            .unwrap_err()
            .to_string();
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
    fn actual_shell_producer_shape_requires_candidate_binding_inputs() {
        let dir = TempDir::new().unwrap();
        let evidence_path = dir.path().join(M7C_EVIDENCE_RELATIVE_PATH);
        fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        fs::write(
            &evidence_path,
            include_str!("../contracts/v1alpha1/fixtures/valid/disposable-cluster-evidence.json"),
        )
        .unwrap();
        let evidence = read_m7c_evidence(&evidence_path).unwrap();
        assert!(evidence_matches_candidate(
            &evidence,
            Some("0123456789abcdef0123456789abcdef01234567"),
            Some("sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
        ));

        let report = release_readiness(dir.path());
        assert!(
            report
                .gates
                .iter()
                .any(|gate| gate.id == "m7c-live-disposable-cluster-evidence"
                    && gate.status == GateStatus::Blocked)
        );
    }

    #[test]
    fn audit_rejects_evidence_bound_to_another_runtime_candidate() {
        let dir = TempDir::new().unwrap();
        let evidence_path = dir.path().join(M7C_EVIDENCE_RELATIVE_PATH);
        fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        fs::write(
            evidence_path,
            include_str!("../contracts/v1alpha1/fixtures/valid/disposable-cluster-evidence.json"),
        )
        .unwrap();
        let evidence = read_m7c_evidence(&dir.path().join(M7C_EVIDENCE_RELATIVE_PATH)).unwrap();
        assert!(!evidence_matches_candidate(
            &evidence,
            Some("ffffffffffffffffffffffffffffffffffffffff"),
            Some("sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
        ));
    }

    #[test]
    fn evidence_without_both_candidate_binding_inputs_is_rejected() {
        let evidence = DisposableClusterEvidence::from_json(
            include_str!("../contracts/v1alpha1/fixtures/valid/disposable-cluster-evidence.json")
                .as_bytes(),
        )
        .unwrap();
        assert!(!evidence_matches_candidate(&evidence, None, None));
        assert!(!evidence_matches_candidate(
            &evidence,
            Some("0123456789abcdef0123456789abcdef01234567"),
            None,
        ));
    }

    #[test]
    fn generic_gate_evidence_requires_the_same_candidate_binding() {
        let evidence = ReleaseGateEvidence::from_json(include_bytes!(
            "../contracts/v1alpha1/fixtures/valid/release-gate-evidence.json"
        ))
        .unwrap();
        assert!(evidence_matches_candidate_values(
            &evidence.candidate_commit,
            &evidence.binary_sha256,
            Some("0123456789abcdef0123456789abcdef01234567"),
            Some("sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
        ));
        assert!(!evidence_matches_candidate_values(
            &evidence.candidate_commit,
            &evidence.binary_sha256,
            Some("ffffffffffffffffffffffffffffffffffffffff"),
            Some("sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
        ));
    }

    #[test]
    fn generic_gate_evidence_requires_untampered_artifacts() {
        let dir = TempDir::new().unwrap();
        let mut evidence = ReleaseGateEvidence::from_json(include_bytes!(
            "../contracts/v1alpha1/fixtures/valid/release-gate-evidence.json"
        ))
        .unwrap();
        let artifact = dir.path().join("logs/gate.log");
        fs::create_dir_all(artifact.parent().unwrap()).unwrap();
        fs::write(&artifact, b"producer passed\n").unwrap();
        evidence.artifacts[0].path = "logs/gate.log".to_string();
        evidence.artifacts[0].sha256 = sha256(b"producer passed\n");
        assert!(release_gate_artifacts_match(dir.path(), &evidence));

        fs::write(&artifact, b"tampered\n").unwrap();
        assert!(!release_gate_artifacts_match(dir.path(), &evidence));
    }
}
