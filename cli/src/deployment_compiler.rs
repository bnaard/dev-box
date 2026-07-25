//! Pure compilation of v1 deployment contracts into a deterministic desired plan.
//!
//! This module intentionally has no configuration, filesystem, process, or backend
//! dependencies. Command and backend adapters will translate their input into a
//! [`CompileRequest`] and are responsible for performing any later mutation.

use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::deployment_contract::{
    AddonProvenance, BuildInputProvenance, CredentialReference, CredentialReferenceKind,
    DeploymentTarget, DeploymentTargetIdentity, EnvironmentReference, MountSpec, PortProtocol,
    PortSpec, WorkspaceFleetSpec, WorkspaceImageSpec, WorkspaceService,
};

/// An explicit opt-in for adding an image-build action to a plan.
///
/// `Disabled` is the default so compiling a deployment can never implicitly build an image.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImageBuildIntent {
    #[default]
    Disabled,
    Enabled,
}

/// Narrow compiler input that future configuration adapters can construct.
///
/// The compiler does not parse `aibox.toml`; keeping this type independent makes the
/// compilation boundary testable and avoids coupling v1 contracts to a legacy config shape.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<WorkspaceImageSpec>,
    pub fleet: WorkspaceFleetSpec,
    pub target: DeploymentTarget,
    #[serde(default)]
    pub image_build: ImageBuildIntent,
}

/// A normalized, mutation-free description of what a backend adapter may do.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesiredDeploymentPlan {
    pub target: DeploymentTargetIdentity,
    pub desired_spec_digest: String,
    pub actions: Vec<DesiredDeploymentAction>,
}

/// Actions are ordered deliberately: an explicitly requested image build always precedes
/// deploying the fleet that refers to its immutable image.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum DesiredDeploymentAction {
    BuildImage {
        image: WorkspaceImageSpec,
    },
    DeployFleet {
        fleet: WorkspaceFleetSpec,
        target: DeploymentTargetIdentity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerError {
    InvalidInput(String),
    Serialization(String),
}

impl Display for CompilerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid deployment input: {message}"),
            Self::Serialization(message) => {
                write!(
                    formatter,
                    "could not canonicalize deployment input: {message}"
                )
            }
        }
    }
}

impl std::error::Error for CompilerError {}

/// Compile a deterministic plan and its SHA-256 desired-spec digest.
///
/// This function validates and normalizes only in memory. It performs no backend discovery,
/// image build, deployment, or other external mutation.
pub fn compile(request: CompileRequest) -> Result<DesiredDeploymentPlan, CompilerError> {
    validate_request(&request)?;
    let normalized = normalize_request(request);
    let desired_spec_digest = digest_normalized_request(&normalized)?;
    let target = target_identity(&normalized.target);
    let mut actions = Vec::with_capacity(2);

    if let Some(image) = normalized.image {
        actions.push(DesiredDeploymentAction::BuildImage { image });
    }
    actions.push(DesiredDeploymentAction::DeployFleet {
        fleet: normalized.fleet,
        target: target.clone(),
    });

    Ok(DesiredDeploymentPlan {
        target,
        desired_spec_digest,
        actions,
    })
}

/// Calculate the SHA-256 digest of canonical deployment-contract input.
///
/// This public helper applies the same validation and normalization as [`compile`].
pub fn desired_spec_digest(request: &CompileRequest) -> Result<String, CompilerError> {
    validate_request(request)?;
    digest_normalized_request(&normalize_request(request.clone()))
}

fn digest_normalized_request(request: &CompileRequest) -> Result<String, CompilerError> {
    let canonical = canonical_json(request)?;
    let digest = Sha256::digest(canonical);
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!("sha256:{hex}"))
}

/// Serialize a value as canonical JSON with recursively sorted object keys.
///
/// Arrays retain their order. `compile` normalizes every contract array whose order has no
/// desired-state meaning before calling this helper.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, CompilerError> {
    let value = serde_json::to_value(value)
        .map_err(|error| CompilerError::Serialization(error.to_string()))?;
    let mut output = String::new();
    write_canonical_json(&value, &mut output)?;
    Ok(output.into_bytes())
}

fn write_canonical_json(
    value: &serde_json::Value,
    output: &mut String,
) -> Result<(), CompilerError> {
    match value {
        serde_json::Value::Null => output.push_str("null"),
        serde_json::Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        serde_json::Value::Number(value) => output.push_str(&value.to_string()),
        serde_json::Value::String(value) => output.push_str(
            &serde_json::to_string(value)
                .map_err(|error| CompilerError::Serialization(error.to_string()))?,
        ),
        serde_json::Value::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(']');
        }
        serde_json::Value::Object(values) => {
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_unstable_by_key(|(key, _)| *key);
            output.push('{');
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(
                    &serde_json::to_string(key)
                        .map_err(|error| CompilerError::Serialization(error.to_string()))?,
                );
                output.push(':');
                write_canonical_json(value, output)?;
            }
            output.push('}');
        }
    }
    Ok(())
}

fn validate_request(request: &CompileRequest) -> Result<(), CompilerError> {
    match (&request.image_build, &request.image) {
        (ImageBuildIntent::Enabled, None) => {
            return invalid("imageBuild is enabled but no image specification was provided");
        }
        (ImageBuildIntent::Disabled, Some(_)) => {
            return invalid("an image specification requires explicit imageBuild: enabled");
        }
        _ => {}
    }

    validate_metadata(
        "fleet",
        &request.fleet.metadata.name,
        &request.fleet.metadata.owner.owner_id,
    )?;
    validate_metadata(
        "target",
        &request.target.metadata.name,
        &request.target.metadata.owner.owner_id,
    )?;
    if request.fleet.metadata.owner.owner_id != request.target.metadata.owner.owner_id {
        return invalid("fleet and target must have the same ownerId");
    }
    if let Some(image) = &request.image {
        validate_metadata(
            "image",
            &image.metadata.name,
            &image.metadata.owner.owner_id,
        )?;
        if image.metadata.owner.owner_id != request.fleet.metadata.owner.owner_id {
            return invalid("image and fleet must have the same ownerId");
        }
        validate_image(image)?;
    }

    validate_non_empty("fleet image reference", &request.fleet.spec.image.reference)?;
    validate_non_empty("fleet image digest", &request.fleet.spec.image.digest)?;
    validate_non_empty("target reference", &request.target.spec.target_ref)?;
    validate_non_empty("target scope", &request.target.spec.scope)?;
    validate_credentials("target credentials", &request.target.spec.credentials)?;
    validate_services(&request.fleet)?;
    Ok(())
}

fn validate_image(image: &WorkspaceImageSpec) -> Result<(), CompilerError> {
    validate_non_empty("base image reference", &image.spec.base_image.reference)?;
    validate_non_empty("base image digest", &image.spec.base_image.digest)?;
    for addon in &image.spec.addons {
        validate_non_empty("addon name", &addon.name)?;
        validate_non_empty("addon version", &addon.version)?;
        validate_non_empty("addon checksum", &addon.checksum)?;
    }
    for input in &image.spec.build_inputs {
        validate_non_empty("build input source", &input.source)?;
        validate_non_empty("build input digest", &input.digest)?;
    }
    Ok(())
}

fn validate_services(fleet: &WorkspaceFleetSpec) -> Result<(), CompilerError> {
    if fleet.spec.services.is_empty() {
        return invalid("fleet must contain at least one service");
    }
    let mut service_names = BTreeSet::new();
    for service in fleet.spec.services.iter().chain(&fleet.spec.sidecars) {
        validate_non_empty("service name", &service.name)?;
        if !service_names.insert(service.name.as_str()) {
            return invalid(format!("service name '{}' is duplicated", service.name));
        }

        let mut environment_names = BTreeSet::new();
        for environment in &service.environment {
            validate_non_empty("environment name", &environment.name)?;
            if !environment_names.insert(environment.name.as_str()) {
                return invalid(format!(
                    "service '{}' has duplicate environment '{}'",
                    service.name, environment.name
                ));
            }
            validate_credential("environment valueFrom", &environment.value_from)?;
        }

        let mut mount_targets = BTreeSet::new();
        for mount in &service.mounts {
            validate_non_empty("mount source", &mount.source)?;
            validate_non_empty("mount target", &mount.target)?;
            if !mount_targets.insert(mount.target.as_str()) {
                return invalid(format!(
                    "service '{}' has duplicate mount target '{}'",
                    service.name, mount.target
                ));
            }
        }

        let mut ports = BTreeSet::new();
        for port in &service.ports {
            if !ports.insert((port.container_port, port_protocol_key(&port.protocol))) {
                return invalid(format!(
                    "service '{}' has duplicate container port",
                    service.name
                ));
            }
        }
    }
    Ok(())
}

fn validate_metadata(kind: &str, name: &str, owner_id: &str) -> Result<(), CompilerError> {
    validate_non_empty(&format!("{kind} metadata.name"), name)?;
    validate_non_empty(&format!("{kind} metadata.owner.ownerId"), owner_id)
}

fn validate_credentials(
    context: &str,
    credentials: &[CredentialReference],
) -> Result<(), CompilerError> {
    for credential in credentials {
        validate_credential(context, credential)?;
    }
    Ok(())
}

fn validate_credential(
    context: &str,
    credential: &CredentialReference,
) -> Result<(), CompilerError> {
    validate_non_empty(&format!("{context} reference"), &credential.reference)
}

fn validate_non_empty(field: &str, value: &str) -> Result<(), CompilerError> {
    if value.trim().is_empty() {
        return invalid(format!("{field} must not be empty"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, CompilerError> {
    Err(CompilerError::InvalidInput(message.into()))
}

fn normalize_request(mut request: CompileRequest) -> CompileRequest {
    if let Some(image) = &mut request.image {
        image.spec.addons.sort_unstable_by_key(addon_key);
        image
            .spec
            .build_inputs
            .sort_unstable_by_key(build_input_key);
    }
    request
        .fleet
        .spec
        .services
        .sort_unstable_by_key(|service| service.name.clone());
    request
        .fleet
        .spec
        .sidecars
        .sort_unstable_by_key(|service| service.name.clone());
    for service in request
        .fleet
        .spec
        .services
        .iter_mut()
        .chain(request.fleet.spec.sidecars.iter_mut())
    {
        normalize_service(service);
    }
    request
        .target
        .spec
        .credentials
        .sort_unstable_by_key(credential_key);
    request
}

fn normalize_service(service: &mut WorkspaceService) {
    service.environment.sort_unstable_by_key(environment_key);
    service.mounts.sort_unstable_by_key(mount_key);
    service.ports.sort_unstable_by_key(port_key);
}

fn target_identity(target: &DeploymentTarget) -> DeploymentTargetIdentity {
    DeploymentTargetIdentity {
        backend: target.spec.backend.clone(),
        target_ref: target.spec.target_ref.clone(),
        scope: target.spec.scope.clone(),
        kubernetes: target.spec.kubernetes.clone(),
    }
}

fn addon_key(addon: &AddonProvenance) -> (String, String, String) {
    (
        addon.name.clone(),
        addon.version.clone(),
        addon.checksum.clone(),
    )
}

fn build_input_key(input: &BuildInputProvenance) -> (String, String) {
    (input.source.clone(), input.digest.clone())
}

fn environment_key(environment: &EnvironmentReference) -> (String, String, String) {
    (
        environment.name.clone(),
        credential_kind_key(&environment.value_from.kind).to_string(),
        environment.value_from.reference.clone(),
    )
}

fn credential_key(credential: &CredentialReference) -> (String, String) {
    (
        credential_kind_key(&credential.kind).to_string(),
        credential.reference.clone(),
    )
}

fn mount_key(mount: &MountSpec) -> (String, String, bool) {
    (mount.target.clone(), mount.source.clone(), mount.read_only)
}

fn port_key(port: &PortSpec) -> (u16, String, Option<u16>) {
    (
        port.container_port,
        port_protocol_key(&port.protocol).to_string(),
        port.host_port,
    )
}

fn credential_kind_key(kind: &CredentialReferenceKind) -> &'static str {
    match kind {
        CredentialReferenceKind::EnvironmentVariable => "environment-variable",
        CredentialReferenceKind::File => "file",
        CredentialReferenceKind::SecretManager => "secret-manager",
    }
}

fn port_protocol_key(protocol: &PortProtocol) -> &'static str {
    match protocol {
        PortProtocol::Tcp => "tcp",
        PortProtocol::Udp => "udp",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deployment_contract::{
        ApiVersion, DeploymentTargetKind, ObjectMeta, OwnershipReference, WorkspaceFleetSpecKind,
    };
    use std::collections::BTreeMap;

    fn fixture_request() -> CompileRequest {
        CompileRequest {
            image: None,
            fleet: serde_json::from_str(include_str!(
                "../contracts/v1alpha1/fixtures/valid/workspace-fleet-spec.json"
            ))
            .unwrap(),
            target: serde_json::from_str(include_str!(
                "../contracts/v1alpha1/fixtures/valid/deployment-target.json"
            ))
            .unwrap(),
            image_build: ImageBuildIntent::Disabled,
        }
    }

    #[test]
    fn reordering_contract_arrays_produces_the_same_normalized_plan_and_digest() {
        let mut first = fixture_request();
        first.fleet.spec.services.push(WorkspaceService {
            name: "api".to_string(),
            environment: vec![EnvironmentReference {
                name: "CONFIG".to_string(),
                value_from: CredentialReference {
                    kind: CredentialReferenceKind::File,
                    reference: "config/api.env".to_string(),
                },
            }],
            mounts: vec![],
            ports: vec![],
            resources: None,
        });
        first.target.spec.credentials.push(CredentialReference {
            kind: CredentialReferenceKind::File,
            reference: "credentials/compose".to_string(),
        });
        let mut second = first.clone();
        second.fleet.spec.services.reverse();
        second.target.spec.credentials.reverse();

        let first_plan = compile(first).unwrap();
        let second_plan = compile(second).unwrap();
        assert_eq!(first_plan, second_plan);
    }

    #[test]
    fn changing_desired_contract_content_changes_the_digest() {
        let first = compile(fixture_request()).unwrap();
        let mut changed = fixture_request();
        changed.fleet.spec.services[0].ports[0].container_port = 4000;
        let second = compile(changed).unwrap();

        assert_ne!(first.desired_spec_digest, second.desired_spec_digest);
    }

    #[test]
    fn image_build_is_never_implicit() {
        let plan = compile(fixture_request()).unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert!(matches!(
            plan.actions[0],
            DesiredDeploymentAction::DeployFleet { .. }
        ));

        let mut explicit = fixture_request();
        explicit.image_build = ImageBuildIntent::Enabled;
        explicit.image = Some(
            serde_json::from_str(include_str!(
                "../contracts/v1alpha1/fixtures/valid/workspace-image-spec.json"
            ))
            .unwrap(),
        );
        let explicit_plan = compile(explicit).unwrap();
        assert!(matches!(
            explicit_plan.actions[0],
            DesiredDeploymentAction::BuildImage { .. }
        ));
    }

    #[test]
    fn invalid_input_fails_before_a_plan_is_created() {
        let duplicate_service: WorkspaceFleetSpec = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/invalid/compiler-duplicate-service.json"
        ))
        .unwrap();
        let mut request = fixture_request();
        request.fleet = duplicate_service;
        assert!(matches!(
            compile(request),
            Err(CompilerError::InvalidInput(_))
        ));

        let mut missing_image = fixture_request();
        missing_image.image_build = ImageBuildIntent::Enabled;
        assert!(matches!(
            compile(missing_image),
            Err(CompilerError::InvalidInput(_))
        ));
    }

    #[test]
    fn canonical_json_sorts_object_keys() {
        let first = serde_json::json!({"b": [2, 1], "a": {"z": true, "y": false}});
        let second = serde_json::json!({"a": {"y": false, "z": true}, "b": [2, 1]});
        assert_eq!(
            canonical_json(&first).unwrap(),
            canonical_json(&second).unwrap()
        );
    }

    #[test]
    fn metadata_validation_is_explicit() {
        let request = CompileRequest {
            image: None,
            fleet: WorkspaceFleetSpec {
                api_version: ApiVersion::V1Alpha1,
                kind: WorkspaceFleetSpecKind::V1Alpha1,
                metadata: ObjectMeta {
                    name: "fleet".to_string(),
                    owner: OwnershipReference {
                        owner_id: "team-a".to_string(),
                    },
                    labels: BTreeMap::new(),
                },
                spec: fixture_request().fleet.spec,
            },
            target: DeploymentTarget {
                api_version: ApiVersion::V1Alpha1,
                kind: DeploymentTargetKind::V1Alpha1,
                metadata: ObjectMeta {
                    name: "target".to_string(),
                    owner: OwnershipReference {
                        owner_id: "team-b".to_string(),
                    },
                    labels: BTreeMap::new(),
                },
                spec: fixture_request().target.spec,
            },
            image_build: ImageBuildIntent::Disabled,
        };
        assert!(matches!(
            compile(request),
            Err(CompilerError::InvalidInput(_))
        ));
    }
}
