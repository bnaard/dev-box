//! Typed, candidate-bound release evidence contracts.
//!
//! Release evidence is an attestation from a real producer, not a configuration
//! input.  The parser is deliberately closed to unknown fields and scenarios
//! so a later producer change cannot silently weaken a release gate.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub const DISPOSABLE_CLUSTER_EVIDENCE_API_VERSION: &str = "aibox.projectious.work/v1alpha1";
pub const DISPOSABLE_CLUSTER_EVIDENCE_KIND: &str = "DisposableClusterEvidence";

pub const REQUIRED_M7C_SCENARIOS: [M7cScenarioId; 8] = [
    M7cScenarioId::FirstApply,
    M7cScenarioId::UnchangedApply,
    M7cScenarioId::ChangedApply,
    M7cScenarioId::DriftRecovery,
    M7cScenarioId::StatusLogs,
    M7cScenarioId::ExecPortForward,
    M7cScenarioId::Ingress,
    M7cScenarioId::ForeignDestroyRefusal,
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DisposableClusterEvidence {
    pub api_version: String,
    pub kind: String,
    pub status: EvidenceStatus,
    pub candidate_commit: String,
    pub binary_sha256: String,
    pub cluster: String,
    pub command: String,
    pub scenarios: Vec<M7cScenarioEvidence>,
    pub recorded_at: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceStatus {
    Passed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct M7cScenarioEvidence {
    pub id: M7cScenarioId,
    pub status: ScenarioStatus,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M7cScenarioId {
    FirstApply,
    UnchangedApply,
    ChangedApply,
    DriftRecovery,
    StatusLogs,
    ExecPortForward,
    Ingress,
    ForeignDestroyRefusal,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScenarioStatus {
    Passed,
}

impl DisposableClusterEvidence {
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        let evidence: Self = serde_json::from_slice(bytes)
            .map_err(|error| format!("invalid DisposableClusterEvidence: {error}"))?;
        evidence.validate()?;
        Ok(evidence)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.api_version != DISPOSABLE_CLUSTER_EVIDENCE_API_VERSION
            || self.kind != DISPOSABLE_CLUSTER_EVIDENCE_KIND
            || self.candidate_commit.len() != 40
            || !self
                .candidate_commit
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || !is_sha256_digest(&self.binary_sha256)
            || self.cluster.trim().is_empty()
            || !self.command.contains("kubernetes")
            || self.recorded_at.trim().is_empty()
        {
            return Err("incomplete disposable-cluster evidence envelope".to_string());
        }

        let actual: HashSet<_> = self.scenarios.iter().map(|scenario| scenario.id).collect();
        if actual.len() != self.scenarios.len()
            || self.scenarios.len() != REQUIRED_M7C_SCENARIOS.len()
            || REQUIRED_M7C_SCENARIOS
                .iter()
                .any(|required| !actual.contains(required))
        {
            return Err(
                "disposable-cluster evidence scenarios must be complete and unique".to_string(),
            );
        }
        Ok(())
    }
}

fn is_sha256_digest(value: &str) -> bool {
    value.len() == "sha256:".len() + 64
        && value.starts_with("sha256:")
        && value["sha256:".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str =
        include_str!("../contracts/v1alpha1/fixtures/valid/disposable-cluster-evidence.json");

    #[test]
    fn accepts_the_published_evidence_fixture() {
        let evidence = DisposableClusterEvidence::from_json(VALID.as_bytes()).unwrap();
        assert_eq!(evidence.scenarios.len(), REQUIRED_M7C_SCENARIOS.len());
    }

    #[test]
    fn rejects_invalid_scenario_fixtures() {
        for fixture in [
            include_str!(
                "../contracts/v1alpha1/fixtures/invalid/disposable-cluster-evidence-missing-scenario.json"
            ),
            include_str!(
                "../contracts/v1alpha1/fixtures/invalid/disposable-cluster-evidence-duplicate-scenario.json"
            ),
            include_str!(
                "../contracts/v1alpha1/fixtures/invalid/disposable-cluster-evidence-unexecuted-scenario.json"
            ),
            include_str!(
                "../contracts/v1alpha1/fixtures/invalid/disposable-cluster-evidence-unknown-scenario.json"
            ),
        ] {
            assert!(DisposableClusterEvidence::from_json(fixture.as_bytes()).is_err());
        }
    }
}
