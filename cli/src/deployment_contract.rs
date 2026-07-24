//! Versioned, backend-neutral deployment contracts.
//!
//! These types are the canonical desired-state and receipt vocabulary for v1.
//! They deliberately do not select, render, or mutate a backend.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const API_VERSION_V1ALPHA1: &str = "aibox.projectious.work/v1alpha1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectMeta {
    pub name: String,
    pub owner: OwnershipReference,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub labels: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnershipReference {
    pub owner_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct CredentialReference {
    pub kind: CredentialReferenceKind,
    pub reference: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CredentialReferenceKind {
    EnvironmentVariable,
    File,
    SecretManager,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Platform {
    LinuxAmd64,
    LinuxArm64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmutableImageReference {
    pub reference: String,
    pub digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonProvenance {
    pub name: String,
    pub version: String,
    pub checksum: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceImageSpec {
    #[serde(rename = "apiVersion")]
    pub api_version: ApiVersion,
    pub kind: WorkspaceImageSpecKind,
    pub metadata: ObjectMeta,
    pub spec: WorkspaceImageSpecBody,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceImageSpecBody {
    pub base_image: ImmutableImageReference,
    pub platform: Platform,
    #[serde(default)]
    pub addons: Vec<AddonProvenance>,
    #[serde(default)]
    pub build_inputs: Vec<BuildInputProvenance>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildInputProvenance {
    pub source: String,
    pub digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFleetSpec {
    #[serde(rename = "apiVersion")]
    pub api_version: ApiVersion,
    pub kind: WorkspaceFleetSpecKind,
    pub metadata: ObjectMeta,
    pub spec: WorkspaceFleetSpecBody,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFleetSpecBody {
    pub image: ImmutableImageReference,
    pub services: Vec<WorkspaceService>,
    #[serde(default)]
    pub sidecars: Vec<WorkspaceService>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceService {
    pub name: String,
    #[serde(default)]
    pub environment: Vec<EnvironmentReference>,
    #[serde(default)]
    pub mounts: Vec<MountSpec>,
    #[serde(default)]
    pub ports: Vec<PortSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceRequirements>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentReference {
    pub name: String,
    pub value_from: CredentialReference,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MountSpec {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortSpec {
    pub container_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_port: Option<u16>,
    pub protocol: PortProtocol,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortProtocol {
    Tcp,
    Udp,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRequirements {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentTarget {
    #[serde(rename = "apiVersion")]
    pub api_version: ApiVersion,
    pub kind: DeploymentTargetKind,
    pub metadata: ObjectMeta,
    pub spec: DeploymentTargetSpec,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentTargetSpec {
    pub backend: BackendKind,
    pub target_ref: String,
    pub scope: String,
    #[serde(default)]
    pub credentials: Vec<CredentialReference>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendKind {
    Compose,
    Kubernetes,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendDescriptor {
    #[serde(rename = "apiVersion")]
    pub api_version: ApiVersion,
    pub kind: BackendDescriptorKind,
    pub metadata: ObjectMeta,
    pub spec: BackendDescriptorSpec,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendDescriptorSpec {
    pub backend: BackendKind,
    pub capabilities: Vec<BackendCapability>,
    pub connection_transports: Vec<ConnectionTransport>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendCapability {
    Validate,
    Plan,
    Apply,
    Status,
    Destroy,
    Logs,
    Exec,
    PortForward,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentRecord {
    #[serde(rename = "apiVersion")]
    pub api_version: ApiVersion,
    pub kind: DeploymentRecordKind,
    pub metadata: ObjectMeta,
    pub spec: DeploymentRecordSpec,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentRecordSpec {
    pub deployment_id: String,
    pub target: DeploymentTargetIdentity,
    pub desired_spec_digest: String,
    pub image: ImmutableImageReference,
    pub ownership: DeploymentOwnership,
    pub status: DeploymentStatus,
    #[serde(default)]
    pub services: Vec<DeployedService>,
    #[serde(default)]
    pub connections: Vec<ConnectionEndpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processkit_result: Option<OpaqueProvenanceReference>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentTargetIdentity {
    pub backend: BackendKind,
    pub target_ref: String,
    pub scope: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentOwnership {
    pub deployment_id_label: String,
    pub desired_spec_digest_label: String,
    pub image_digest_label: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentStatus {
    Desired,
    Observed,
    Degraded,
    Unavailable,
    Destroyed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployedService {
    pub name: String,
    pub resource_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionEndpoint {
    pub service: String,
    pub endpoint: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpaqueProvenanceReference {
    pub reference: String,
    pub digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTarget {
    #[serde(rename = "apiVersion")]
    pub api_version: ApiVersion,
    pub kind: ConnectionTargetKind,
    pub metadata: ObjectMeta,
    pub spec: ConnectionTargetSpec,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTargetSpec {
    pub deployment_id: String,
    pub service: String,
    pub transport: ConnectionTransport,
    pub interactive: bool,
    pub endpoint: String,
    #[serde(default)]
    pub invocation: Vec<String>,
    #[serde(default)]
    pub credentials: Vec<CredentialReference>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionTransport {
    ComposeExec,
    KubernetesExec,
    KubernetesPortForward,
    Ssh,
}

/// Stable contract error vocabulary. Backends will return these in later milestones.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContractErrorCode {
    Validation,
    CapabilityUnsupported,
    Planning,
    Mutation,
    Observation,
    Connection,
    Ownership,
    DelegatedInstaller,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ApiVersion {
    #[serde(rename = "aibox.projectious.work/v1alpha1")]
    V1Alpha1,
}

macro_rules! kinds {
    ($($name:ident => $value:literal),+ $(,)?) => {
        $(
            #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
            pub enum $name {
                #[serde(rename = $value)]
                V1Alpha1,
            }
        )+
    };
}

kinds!(
    WorkspaceImageSpecKind => "WorkspaceImageSpec",
    WorkspaceFleetSpecKind => "WorkspaceFleetSpec",
    DeploymentTargetKind => "DeploymentTarget",
    DeploymentRecordKind => "DeploymentRecord",
    BackendDescriptorKind => "BackendDescriptor",
    ConnectionTargetKind => "ConnectionTarget",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deployment_target_serializes_credential_references_without_secret_values() {
        let target = DeploymentTarget {
            api_version: ApiVersion::V1Alpha1,
            kind: DeploymentTargetKind::V1Alpha1,
            metadata: ObjectMeta {
                name: "local-compose".to_string(),
                owner: OwnershipReference {
                    owner_id: "team-a".to_string(),
                },
                labels: BTreeMap::new(),
            },
            spec: DeploymentTargetSpec {
                backend: BackendKind::Compose,
                target_ref: "docker-context:default".to_string(),
                scope: "demo".to_string(),
                credentials: vec![CredentialReference {
                    kind: CredentialReferenceKind::EnvironmentVariable,
                    reference: "AIBOX_COMPOSE_TOKEN".to_string(),
                }],
            },
        };

        let serialized = serde_json::to_string(&target).unwrap();
        assert!(serialized.contains("AIBOX_COMPOSE_TOKEN"));
        assert!(!serialized.contains("super-secret-token"));
        assert!(!serialized.contains("credentialValue"));
    }

    #[test]
    fn valid_fixtures_deserialize_into_their_canonical_models() {
        let image: WorkspaceImageSpec = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/workspace-image-spec.json"
        ))
        .unwrap();
        let fleet: WorkspaceFleetSpec = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/workspace-fleet-spec.json"
        ))
        .unwrap();
        let target: DeploymentTarget = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/deployment-target.json"
        ))
        .unwrap();
        let record: DeploymentRecord = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/deployment-record.json"
        ))
        .unwrap();
        let backend: BackendDescriptor = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/deployment-backend.json"
        ))
        .unwrap();
        let connection: ConnectionTarget = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/connection-target.json"
        ))
        .unwrap();

        assert_eq!(image.api_version, ApiVersion::V1Alpha1);
        assert_eq!(fleet.spec.services[0].name, "workspace");
        assert_eq!(target.spec.backend, BackendKind::Compose);
        assert_eq!(record.spec.status, DeploymentStatus::Observed);
        assert!(backend.spec.capabilities.contains(&BackendCapability::Plan));
        assert_eq!(connection.spec.transport, ConnectionTransport::ComposeExec);
    }

    #[test]
    fn invalid_fixtures_reject_unknown_versions_and_raw_credentials() {
        let invalid_version = include_str!(
            "../contracts/v1alpha1/fixtures/invalid/workspace-image-spec-unknown-version.json"
        );
        assert!(serde_json::from_str::<WorkspaceImageSpec>(invalid_version).is_err());

        let raw_credential = include_str!(
            "../contracts/v1alpha1/fixtures/invalid/deployment-target-raw-credential.json"
        );
        assert!(serde_json::from_str::<DeploymentTarget>(raw_credential).is_err());
    }

    #[test]
    fn schemas_are_versioned_json_documents() {
        for schema in [
            include_str!("../contracts/v1alpha1/schemas/workspace-image-spec.schema.json"),
            include_str!("../contracts/v1alpha1/schemas/workspace-fleet-spec.schema.json"),
            include_str!("../contracts/v1alpha1/schemas/deployment-target.schema.json"),
            include_str!("../contracts/v1alpha1/schemas/deployment-record.schema.json"),
            include_str!("../contracts/v1alpha1/schemas/deployment-backend.schema.json"),
            include_str!("../contracts/v1alpha1/schemas/connection-target.schema.json"),
        ] {
            let value: serde_json::Value = serde_json::from_str(schema).unwrap();
            assert_eq!(
                value["$schema"],
                "https://json-schema.org/draft/2020-12/schema"
            );
            assert!(value["$id"].as_str().unwrap().contains("/v1alpha1/"));
        }
    }
}
