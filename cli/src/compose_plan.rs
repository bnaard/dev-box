//! Pure Compose/devcontainer renderer for canonical v1 deployment plans.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::deployment_backend::{
    Backend, BackendError, PlanRequest, PlanResponse, ValidateRequest, ValidateResponse,
};
use crate::deployment_compiler::{DesiredDeploymentAction, DesiredDeploymentPlan};
use crate::deployment_contract::{
    BackendCapability, BackendKind, ContractErrorCode, CredentialReferenceKind, PortProtocol,
    WorkspaceService,
};

const LABEL_DEPLOYMENT_ID: &str = "aibox.projectious.work/deployment-id";
const LABEL_SPEC_DIGEST: &str = "aibox.projectious.work/desired-spec-digest";
const LABEL_IMAGE_DIGEST: &str = "aibox.projectious.work/image-digest";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderedDeploymentPlan {
    pub backend: BackendKind,
    pub deployment_id: String,
    pub desired_spec_digest: String,
    pub image_digest: String,
    pub ownership_labels: BTreeMap<String, String>,
    pub compose_yaml: String,
    pub devcontainer_json: String,
}

pub struct ComposeBackend;
impl Backend for ComposeBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Compose
    }
    fn capabilities(&self) -> Vec<BackendCapability> {
        vec![BackendCapability::Validate, BackendCapability::Plan]
    }
    fn validate(&self, request: ValidateRequest) -> Result<ValidateResponse, BackendError> {
        render(&request.plan).map(|_| ValidateResponse { valid: true })
    }
    fn plan(&self, request: PlanRequest) -> Result<PlanResponse, BackendError> {
        render(&request.plan).map(|rendered| PlanResponse { rendered })
    }
}

pub fn render(plan: &DesiredDeploymentPlan) -> Result<RenderedDeploymentPlan, BackendError> {
    if plan.target.backend != BackendKind::Compose {
        return Err(BackendError {
            code: ContractErrorCode::Planning,
            message: "Compose renderer received a non-Compose plan".to_string(),
        });
    }
    let Some(DesiredDeploymentAction::DeployFleet { fleet, .. }) = plan
        .actions
        .iter()
        .find(|action| matches!(action, DesiredDeploymentAction::DeployFleet { .. }))
    else {
        return Err(BackendError {
            code: ContractErrorCode::Planning,
            message: "plan has no deploy-fleet action".to_string(),
        });
    };
    let deployment_id = deployment_id(&fleet.metadata.name, &plan.desired_spec_digest);
    let mut labels = fleet.metadata.labels.clone();
    labels.insert(LABEL_DEPLOYMENT_ID.to_string(), deployment_id.clone());
    labels.insert(
        LABEL_SPEC_DIGEST.to_string(),
        plan.desired_spec_digest.clone(),
    );
    labels.insert(
        LABEL_IMAGE_DIGEST.to_string(),
        fleet.spec.image.digest.clone(),
    );
    let services = fleet
        .spec
        .services
        .iter()
        .chain(&fleet.spec.sidecars)
        .map(|service| {
            (
                service.name.clone(),
                compose_service(service, &fleet.spec.image.reference, &labels),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let compose = ComposeDocument {
        version: "3.9",
        services,
    };
    let primary = fleet.spec.services.first().ok_or_else(|| BackendError {
        code: ContractErrorCode::Planning,
        message: "fleet has no primary service".to_string(),
    })?;
    let devcontainer = Devcontainer {
        name: fleet.metadata.name.clone(),
        docker_compose_file: "docker-compose.yml",
        service: primary.name.clone(),
        workspace_folder: "/workspaces/workspace",
    };
    Ok(RenderedDeploymentPlan {
        backend: BackendKind::Compose,
        deployment_id,
        desired_spec_digest: plan.desired_spec_digest.clone(),
        image_digest: fleet.spec.image.digest.clone(),
        ownership_labels: labels,
        compose_yaml: serde_yaml::to_string(&compose).map_err(serialization_error)?,
        devcontainer_json: serde_json::to_string_pretty(&devcontainer)
            .map_err(serialization_error)?,
    })
}

fn deployment_id(name: &str, digest: &str) -> String {
    let hash = Sha256::digest(format!("{name}\0{digest}"));
    format!(
        "{}-{}",
        name,
        hash.iter()
            .take(8)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}
fn serialization_error(error: impl std::fmt::Display) -> BackendError {
    BackendError {
        code: ContractErrorCode::Planning,
        message: format!("could not render Compose plan: {error}"),
    }
}

#[derive(Serialize)]
struct ComposeDocument {
    version: &'static str,
    services: BTreeMap<String, ComposeService>,
}
#[derive(Serialize)]
struct ComposeService {
    image: String,
    labels: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    environment: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ports: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    volumes: Vec<String>,
}
fn compose_service(
    service: &WorkspaceService,
    image: &str,
    labels: &BTreeMap<String, String>,
) -> ComposeService {
    let environment = service
        .environment
        .iter()
        .map(|item| {
            (
                item.name.clone(),
                match item.value_from.kind {
                    CredentialReferenceKind::EnvironmentVariable => {
                        format!("${{{}}}", item.value_from.reference)
                    }
                    CredentialReferenceKind::File | CredentialReferenceKind::SecretManager => {
                        format!("aibox-ref:{}", item.value_from.reference)
                    }
                },
            )
        })
        .collect();
    let ports = service
        .ports
        .iter()
        .map(|port| {
            let protocol = match port.protocol {
                PortProtocol::Tcp => "tcp",
                PortProtocol::Udp => "udp",
            };
            match port.host_port {
                Some(host) => format!("{host}:{}:{protocol}", port.container_port),
                None => format!("{}:{protocol}", port.container_port),
            }
        })
        .collect();
    let volumes = service
        .mounts
        .iter()
        .map(|mount| {
            format!(
                "{}:{}{}",
                mount.source,
                mount.target,
                if mount.read_only { ":ro" } else { "" }
            )
        })
        .collect();
    ComposeService {
        image: image.to_string(),
        labels: labels.clone(),
        environment,
        ports,
        volumes,
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Devcontainer {
    name: String,
    docker_compose_file: &'static str,
    service: String,
    workspace_folder: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deployment_compiler::{CompileRequest, ImageBuildIntent, compile};
    use crate::deployment_contract::{DeploymentTarget, WorkspaceFleetSpec};

    #[test]
    fn renderer_is_deterministic_and_redacts_credential_values() {
        let fleet: WorkspaceFleetSpec = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/workspace-fleet-spec.json"
        ))
        .unwrap();
        let target: DeploymentTarget = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/deployment-target.json"
        ))
        .unwrap();
        let plan = compile(CompileRequest {
            image: None,
            fleet,
            target,
            image_build: ImageBuildIntent::Disabled,
        })
        .unwrap();
        let first = render(&plan).unwrap();
        assert_eq!(first, render(&plan).unwrap());
        assert_eq!(
            first.compose_yaml,
            include_str!("../contracts/v1alpha1/fixtures/valid/compose-plan.yaml")
        );
        assert!(first.compose_yaml.contains(LABEL_DEPLOYMENT_ID));
        assert!(first.compose_yaml.contains("${AIBOX_REGISTRY_TOKEN}"));
        assert!(!first.compose_yaml.contains("secret-token"));
    }
}
