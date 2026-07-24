//! Adapter from validated `aibox.toml` orchestration intent to canonical v1 contracts.
//!
//! This module contains no backend discovery or mutation. It makes the config/compiler
//! boundary explicit and keeps apply-time image builds disabled.

use anyhow::{Context, Result};

use crate::config::{
    AiboxConfig, CredentialReferenceKind as ConfigCredentialKind, CredentialReferenceSection,
    OrchestrationBackend, OrchestrationPortProtocol,
};
use crate::deployment_compiler::{
    CompileRequest, DesiredDeploymentPlan, ImageBuildIntent, compile,
};
use crate::deployment_contract::{
    ApiVersion, BackendKind, CredentialReference, CredentialReferenceKind, DeploymentTarget,
    DeploymentTargetKind, DeploymentTargetSpec, EnvironmentReference, ImmutableImageReference,
    ObjectMeta, OwnershipReference, PortProtocol, PortSpec, WorkspaceFleetSpec,
    WorkspaceFleetSpecBody, WorkspaceFleetSpecKind, WorkspaceService,
};

/// Compile validated orchestration intent into a deterministic, mutation-free plan.
pub fn compile_config(config: &AiboxConfig) -> Result<DesiredDeploymentPlan> {
    config.validate()?;
    let orchestration = &config.orchestration;
    if !orchestration.enabled {
        anyhow::bail!("orchestration is not enabled");
    }

    let image = orchestration
        .image
        .as_ref()
        .context("orchestration.image is required when orchestration.enabled = true")?;
    let fleet = orchestration
        .fleet
        .as_ref()
        .context("orchestration.fleet is required when orchestration.enabled = true")?;
    let target = orchestration
        .target
        .as_ref()
        .context("orchestration.target is required when orchestration.enabled = true")?;
    let deployment = orchestration
        .deployment
        .as_ref()
        .context("orchestration.deployment is required when orchestration.enabled = true")?;

    let metadata = |name: String| ObjectMeta {
        name,
        owner: OwnershipReference {
            owner_id: deployment.owner_id.clone(),
        },
        labels: deployment.labels.clone(),
    };

    let fleet_contract = WorkspaceFleetSpec {
        api_version: ApiVersion::V1Alpha1,
        kind: WorkspaceFleetSpecKind::V1Alpha1,
        metadata: metadata(fleet.name.clone()),
        spec: WorkspaceFleetSpecBody {
            image: ImmutableImageReference {
                reference: image.reference.clone(),
                digest: image.digest.clone(),
            },
            services: fleet
                .services
                .iter()
                .map(|service| WorkspaceService {
                    name: service.name.clone(),
                    environment: service
                        .environment
                        .iter()
                        .map(|environment| EnvironmentReference {
                            name: environment.name.clone(),
                            value_from: credential_reference(&environment.value_from),
                        })
                        .collect(),
                    mounts: Vec::new(),
                    ports: service
                        .ports
                        .iter()
                        .map(|port| PortSpec {
                            container_port: port.container_port,
                            host_port: port.host_port,
                            protocol: match port.protocol {
                                OrchestrationPortProtocol::Tcp => PortProtocol::Tcp,
                                OrchestrationPortProtocol::Udp => PortProtocol::Udp,
                            },
                        })
                        .collect(),
                    resources: None,
                })
                .collect(),
            sidecars: Vec::new(),
        },
    };

    let target_contract = DeploymentTarget {
        api_version: ApiVersion::V1Alpha1,
        kind: DeploymentTargetKind::V1Alpha1,
        metadata: metadata(format!("{}-target", deployment.name)),
        spec: DeploymentTargetSpec {
            backend: match target.backend {
                OrchestrationBackend::Compose => BackendKind::Compose,
                OrchestrationBackend::Kubernetes => BackendKind::Kubernetes,
            },
            target_ref: target.reference.clone(),
            scope: target.scope.clone(),
            credentials: target
                .credentials
                .iter()
                .map(credential_reference)
                .collect(),
        },
    };

    compile(CompileRequest {
        image: None,
        fleet: fleet_contract,
        target: target_contract,
        image_build: ImageBuildIntent::Disabled,
    })
    .map_err(Into::into)
}

fn credential_reference(value: &CredentialReferenceSection) -> CredentialReference {
    CredentialReference {
        kind: match value.kind {
            ConfigCredentialKind::EnvironmentVariable => {
                CredentialReferenceKind::EnvironmentVariable
            }
            ConfigCredentialKind::File => CredentialReferenceKind::File,
            ConfigCredentialKind::SecretManager => CredentialReferenceKind::SecretManager,
        },
        reference: value.reference.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deployment_compiler::DesiredDeploymentAction;

    const VALID_CONFIG: &str = r#"
[container]
name = "v1-project"

[orchestration]
enabled = true

[orchestration.image]
reference = "ghcr.io/acme/workspace"
digest = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
platform = "linux-amd64"

[orchestration.fleet]
name = "workspace"
services = [
  { name = "web", ports = [{ container_port = 8080, host_port = 18080 }] },
  { name = "worker", environment = [{ name = "API_TOKEN", value_from = { kind = "environment-variable", reference = "AIBOX_API_TOKEN" } }] }
]

[orchestration.target]
backend = "compose"
reference = "docker-context:default"
scope = "workspace"

[orchestration.deployment]
name = "workspace-dev"
owner_id = "team-a"
labels = { environment = "development" }
"#;

    #[test]
    fn valid_config_compiles_to_a_deterministic_apply_only_plan() {
        let config = AiboxConfig::from_str(VALID_CONFIG).unwrap();
        let first = compile_config(&config).unwrap();
        let second = compile_config(&config).unwrap();

        assert_eq!(first, second);
        assert!(first.desired_spec_digest.starts_with("sha256:"));
        assert_eq!(first.actions.len(), 1);
        assert!(matches!(
            first.actions[0],
            DesiredDeploymentAction::DeployFleet { .. }
        ));
    }

    #[test]
    fn reordering_semantically_unordered_services_keeps_the_digest() {
        let first = AiboxConfig::from_str(VALID_CONFIG).unwrap();
        let reordered = VALID_CONFIG.replace(
            "{ name = \"web\", ports = [{ container_port = 8080, host_port = 18080 }] },\n  { name = \"worker\", environment = [{ name = \"API_TOKEN\", value_from = { kind = \"environment-variable\", reference = \"AIBOX_API_TOKEN\" } }] }",
            "{ name = \"worker\", environment = [{ name = \"API_TOKEN\", value_from = { kind = \"environment-variable\", reference = \"AIBOX_API_TOKEN\" } }] },\n  { name = \"web\", ports = [{ container_port = 8080, host_port = 18080 }] }",
        );
        let second = AiboxConfig::from_str(&reordered).unwrap();

        assert_eq!(
            compile_config(&first).unwrap().desired_spec_digest,
            compile_config(&second).unwrap().desired_spec_digest
        );
    }

    #[test]
    fn changed_desired_input_changes_the_digest() {
        let first = AiboxConfig::from_str(VALID_CONFIG).unwrap();
        let changed =
            AiboxConfig::from_str(&VALID_CONFIG.replace("host_port = 18080", "host_port = 18081"))
                .unwrap();

        assert_ne!(
            compile_config(&first).unwrap().desired_spec_digest,
            compile_config(&changed).unwrap().desired_spec_digest
        );
    }
}
