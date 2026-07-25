//! Pure Kubernetes renderer and capability validation for v1 deployment plans.
//!
//! This module deliberately has no Kubernetes client dependency.  Runtime discovery is
//! represented by a narrow trait and tested with an in-memory fake; rendering and the
//! `aibox deploy plan` path remain filesystem- and network-free.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::compose_plan::RenderedDeploymentPlan;
use crate::deployment_backend::{
    Backend, BackendError, PlanRequest, PlanResponse, ValidateRequest, ValidateResponse,
};
use crate::deployment_compiler::{DesiredDeploymentAction, DesiredDeploymentPlan};
use crate::deployment_contract::{
    BackendCapability, BackendKind, ContractErrorCode, CredentialReferenceKind,
    KubernetesReconciliationIntent, PortProtocol, WorkspaceFleetSpec, WorkspaceService,
};

const LABEL_DEPLOYMENT_ID: &str = "aibox.projectious.work/deployment-id";
const LABEL_SPEC_DIGEST: &str = "aibox.projectious.work/desired-spec-digest";
const LABEL_IMAGE_DIGEST: &str = "aibox.projectious.work/image-digest";
const LABEL_NAMESPACE: &str = "aibox.projectious.work/namespace";

/// Read-only result of discovering already-provisioned Kubernetes facilities.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KubernetesCapabilityDescriptor {
    pub context: String,
    pub authorized_namespaces: BTreeSet<String>,
    pub ingress_classes: BTreeSet<String>,
    pub gateway_classes: BTreeSet<String>,
    pub dns_zones: BTreeSet<String>,
}

/// Discovery is intentionally read-only.  Implementations must not create clusters,
/// controllers, networks, namespaces, ingress classes, gateways, or DNS zones.
pub trait KubernetesDiscovery: Send + Sync {
    fn discover(&self, context: &str) -> Result<KubernetesCapabilityDescriptor, BackendError>;
}

/// In-memory test implementation.  It is the only client used by this milestone.
#[cfg(test)]
#[derive(Clone, Debug, Default)]
pub struct FakeKubernetesDiscovery {
    pub descriptors: BTreeMap<String, KubernetesCapabilityDescriptor>,
}
#[cfg(test)]
impl KubernetesDiscovery for FakeKubernetesDiscovery {
    fn discover(&self, context: &str) -> Result<KubernetesCapabilityDescriptor, BackendError> {
        self.descriptors
            .get(context)
            .cloned()
            .ok_or_else(|| BackendError {
                code: ContractErrorCode::Validation,
                message: format!("Kubernetes context '{context}' was not discovered"),
            })
    }
}

struct PlanOnlyDiscovery;
impl KubernetesDiscovery for PlanOnlyDiscovery {
    fn discover(&self, _context: &str) -> Result<KubernetesCapabilityDescriptor, BackendError> {
        Err(BackendError {
            code: ContractErrorCode::Validation,
            message: "Kubernetes discovery is not available during plan-only execution".to_string(),
        })
    }
}

pub struct KubernetesBackend {
    discovery: Arc<dyn KubernetesDiscovery>,
}
impl KubernetesBackend {
    pub fn new(discovery: Arc<dyn KubernetesDiscovery>) -> Self {
        Self { discovery }
    }
    pub fn plan_only() -> Self {
        Self::new(Arc::new(PlanOnlyDiscovery))
    }
}
impl Backend for KubernetesBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Kubernetes
    }
    fn capabilities(&self) -> Vec<BackendCapability> {
        vec![BackendCapability::Validate, BackendCapability::Plan]
    }
    fn validate(&self, request: ValidateRequest) -> Result<ValidateResponse, BackendError> {
        let (context, namespace, intent) = target(&request.plan)?;
        let descriptor = self.discovery.discover(context)?;
        validate_discovery(&descriptor, namespace, intent)?;
        Ok(ValidateResponse { valid: true })
    }
    fn plan(&self, request: PlanRequest) -> Result<PlanResponse, BackendError> {
        render(&request.plan).map(|rendered| PlanResponse { rendered })
    }
}

/// Render deterministic YAML and JSON resources from the same canonical fleet used by Compose.
pub fn render(plan: &DesiredDeploymentPlan) -> Result<RenderedDeploymentPlan, BackendError> {
    let (context, namespace, _intent) = target(plan)?;
    let fleet = fleet(plan)?;
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
    labels.insert(LABEL_NAMESPACE.to_string(), namespace.to_string());

    let mut resources = Vec::new();
    for service in fleet.spec.services.iter().chain(&fleet.spec.sidecars) {
        resources.push(deployment_resource(
            service,
            &format!("{}@{}", fleet.spec.image.reference, fleet.spec.image.digest),
            namespace,
            &labels,
        ));
        if !service.ports.is_empty() {
            resources.push(service_resource(service, namespace, &labels));
        }
    }
    let yaml = resources
        .iter()
        .map(|resource| serde_yaml::to_string(resource).map_err(serialization_error))
        .collect::<Result<Vec<_>, _>>()?
        .join("---\n");
    let json = serde_json::to_string_pretty(&resources).map_err(serialization_error)?;
    let _ = context; // Context is deliberately validated structurally but never contacted by plan.
    Ok(RenderedDeploymentPlan {
        backend: BackendKind::Kubernetes,
        deployment_id,
        desired_spec_digest: plan.desired_spec_digest.clone(),
        image_digest: fleet.spec.image.digest.clone(),
        ownership_labels: labels,
        compose_yaml: String::new(),
        devcontainer_json: String::new(),
        kubernetes_yaml: Some(yaml),
        kubernetes_json: Some(json),
    })
}

fn target(
    plan: &DesiredDeploymentPlan,
) -> Result<(&str, &str, Option<&KubernetesReconciliationIntent>), BackendError> {
    if plan.target.backend != BackendKind::Kubernetes {
        return Err(error("Kubernetes renderer received a non-Kubernetes plan"));
    }
    let context = plan
        .target
        .target_ref
        .strip_prefix("kube-context:")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error("Kubernetes target must name an explicit kube-context:<context>"))?;
    if !valid_namespace(&plan.target.scope) {
        return Err(error(
            "Kubernetes target scope must be an authorized DNS-label namespace",
        ));
    }
    let intent = plan.target.kubernetes.as_ref();
    Ok((context, &plan.target.scope, intent))
}

fn fleet(plan: &DesiredDeploymentPlan) -> Result<&WorkspaceFleetSpec, BackendError> {
    Ok(deploy_action(plan)?.0)
}

fn deploy_action(
    plan: &DesiredDeploymentPlan,
) -> Result<
    (
        &WorkspaceFleetSpec,
        &crate::deployment_contract::DeploymentTargetIdentity,
    ),
    BackendError,
> {
    plan.actions
        .iter()
        .find_map(|action| match action {
            DesiredDeploymentAction::DeployFleet { fleet, target } => Some((fleet, target)),
            DesiredDeploymentAction::BuildImage { .. } => None,
        })
        .ok_or_else(|| error("plan has no deploy-fleet action"))
}

fn validate_discovery(
    descriptor: &KubernetesCapabilityDescriptor,
    namespace: &str,
    intent: Option<&KubernetesReconciliationIntent>,
) -> Result<(), BackendError> {
    if !descriptor.authorized_namespaces.contains(namespace) {
        return Err(BackendError {
            code: ContractErrorCode::Ownership,
            message: format!(
                "Kubernetes namespace '{namespace}' is not authorized for this target"
            ),
        });
    }
    if let Some(intent) = intent {
        optional_member(
            "IngressClass",
            intent.ingress_class.as_deref(),
            &descriptor.ingress_classes,
        )?;
        optional_member(
            "GatewayClass",
            intent.gateway_class.as_deref(),
            &descriptor.gateway_classes,
        )?;
        optional_member(
            "DNS zone",
            intent.dns_zone.as_deref(),
            &descriptor.dns_zones,
        )?;
    }
    Ok(())
}

fn optional_member(
    kind: &str,
    requested: Option<&str>,
    available: &BTreeSet<String>,
) -> Result<(), BackendError> {
    if let Some(requested) = requested
        && !available.contains(requested)
    {
        return Err(BackendError {
            code: ContractErrorCode::CapabilityUnsupported,
            message: format!("requested existing {kind} '{requested}' is unavailable"),
        });
    }
    Ok(())
}

fn deployment_resource(
    service: &WorkspaceService,
    image: &str,
    namespace: &str,
    labels: &BTreeMap<String, String>,
) -> serde_json::Value {
    let mut container = serde_json::Map::new();
    container.insert("name".to_string(), service.name.clone().into());
    container.insert("image".to_string(), image.into());
    if !service.environment.is_empty() {
        container.insert("env".to_string(), service.environment.iter().map(|environment| {
            serde_json::json!({"name": environment.name, "value": credential_placeholder(&environment.value_from.kind, &environment.value_from.reference)})
        }).collect());
    }
    if !service.ports.is_empty() {
        container.insert("ports".to_string(), service.ports.iter().map(|port| serde_json::json!({
            "containerPort": port.container_port, "protocol": protocol(port.protocol.clone())
        })).collect());
    }
    if let Some(resources) = &service.resources {
        let mut requests = serde_json::Map::new();
        if let Some(cpu) = &resources.cpu {
            requests.insert("cpu".to_string(), cpu.clone().into());
        }
        if let Some(memory) = &resources.memory {
            requests.insert("memory".to_string(), memory.clone().into());
        }
        if !requests.is_empty() {
            container.insert(
                "resources".to_string(),
                serde_json::json!({"requests": requests}),
            );
        }
    }
    serde_json::json!({
        "apiVersion": "apps/v1", "kind": "Deployment",
        "metadata": {"name": service.name, "namespace": namespace, "labels": labels},
        "spec": {"replicas": 1, "selector": {"matchLabels": {"aibox.projectious.work/service": service.name}},
        "template": {"metadata": {"labels": {"aibox.projectious.work/service": service.name}}, "spec": {"containers": [container]}}}
    })
}

fn service_resource(
    service: &WorkspaceService,
    namespace: &str,
    labels: &BTreeMap<String, String>,
) -> serde_json::Value {
    serde_json::json!({
        "apiVersion": "v1", "kind": "Service",
        "metadata": {"name": service.name, "namespace": namespace, "labels": labels},
        "spec": {"type": "ClusterIP", "selector": {"aibox.projectious.work/service": service.name},
        "ports": service.ports.iter().map(|port| serde_json::json!({"port": port.container_port, "targetPort": port.container_port, "protocol": protocol(port.protocol.clone())})).collect::<Vec<_>>()}
    })
}

fn protocol(protocol: PortProtocol) -> &'static str {
    match protocol {
        PortProtocol::Tcp => "TCP",
        PortProtocol::Udp => "UDP",
    }
}
fn credential_placeholder(kind: &CredentialReferenceKind, reference: &str) -> String {
    let kind = match kind {
        CredentialReferenceKind::EnvironmentVariable => "environment-variable",
        CredentialReferenceKind::File => "file",
        CredentialReferenceKind::SecretManager => "secret-manager",
    };
    format!("aibox-ref:{kind}:{reference}")
}
fn deployment_id(name: &str, digest: &str) -> String {
    let hash = Sha256::digest(format!("{name}\0{digest}"));
    format!(
        "{name}-{}",
        hash.iter()
            .take(8)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}
fn valid_namespace(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        && !value.starts_with('-')
        && !value.ends_with('-')
}
fn error(message: &str) -> BackendError {
    BackendError {
        code: ContractErrorCode::Validation,
        message: message.to_string(),
    }
}
fn serialization_error(error: impl std::fmt::Display) -> BackendError {
    BackendError {
        code: ContractErrorCode::Planning,
        message: format!("could not render Kubernetes plan: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deployment_compiler::{CompileRequest, ImageBuildIntent, compile};
    use crate::deployment_contract::{
        DeploymentTarget, KubernetesReconciliationIntent, WorkspaceFleetSpec,
    };

    fn plan() -> DesiredDeploymentPlan {
        let fleet: WorkspaceFleetSpec = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/workspace-fleet-spec.json"
        ))
        .unwrap();
        let mut target: DeploymentTarget = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/deployment-target.json"
        ))
        .unwrap();
        target.spec.backend = BackendKind::Kubernetes;
        target.spec.target_ref = "kube-context:staging".to_string();
        target.spec.scope = "workspace-dev".to_string();
        target.spec.kubernetes = Some(KubernetesReconciliationIntent {
            ingress_class: Some("nginx".to_string()),
            gateway_class: Some("egress".to_string()),
            dns_zone: Some("example.test".to_string()),
            dns_credentials: vec![],
        });
        compile(CompileRequest {
            image: None,
            fleet,
            target,
            image_build: ImageBuildIntent::Disabled,
        })
        .unwrap()
    }

    #[test]
    fn renderer_is_deterministic_and_uses_namespace_scoped_ownership() {
        let plan = plan();
        let first = render(&plan).unwrap();
        assert_eq!(first, render(&plan).unwrap());
        let yaml = first.kubernetes_yaml.unwrap();
        assert_eq!(
            yaml,
            include_str!("../contracts/v1alpha1/fixtures/valid/kubernetes-plan.yaml")
        );
        assert!(yaml.contains("namespace: workspace-dev"));
        assert!(yaml.contains(LABEL_DEPLOYMENT_ID));
        assert!(yaml.contains("kind: Deployment"));
        assert!(yaml.contains("kind: Service"));
        assert!(yaml.contains("aibox-ref:environment-variable:AIBOX_REGISTRY_TOKEN"));
        assert!(!yaml.contains("secret-token"));
    }

    #[test]
    fn fake_discovery_requires_namespace_and_existing_optional_facilities() {
        let plan = plan();
        let descriptor = KubernetesCapabilityDescriptor {
            context: "staging".to_string(),
            authorized_namespaces: ["workspace-dev".to_string()].into_iter().collect(),
            ingress_classes: ["nginx".to_string()].into_iter().collect(),
            gateway_classes: ["egress".to_string()].into_iter().collect(),
            dns_zones: ["example.test".to_string()].into_iter().collect(),
        };
        let backend = KubernetesBackend::new(Arc::new(FakeKubernetesDiscovery {
            descriptors: [("staging".to_string(), descriptor)].into_iter().collect(),
        }));
        assert_eq!(
            backend.validate(ValidateRequest { plan }).unwrap(),
            ValidateResponse { valid: true }
        );
    }

    #[test]
    fn validation_rejects_implicit_context_and_unauthorized_namespace() {
        let mut implicit_context = plan();
        implicit_context.target.target_ref = "staging".to_string();
        assert_eq!(
            render(&implicit_context).unwrap_err().code,
            ContractErrorCode::Validation
        );

        let descriptor = KubernetesCapabilityDescriptor {
            context: "staging".to_string(),
            ..KubernetesCapabilityDescriptor::default()
        };
        let backend = KubernetesBackend::new(Arc::new(FakeKubernetesDiscovery {
            descriptors: [("staging".to_string(), descriptor)].into_iter().collect(),
        }));
        assert_eq!(
            backend
                .validate(ValidateRequest { plan: plan() })
                .unwrap_err()
                .code,
            ContractErrorCode::Ownership
        );
    }

    #[test]
    fn plan_is_pure_even_when_discovery_is_not_available() {
        let backend = KubernetesBackend::plan_only();
        assert!(backend.plan(PlanRequest { plan: plan() }).is_ok());
        assert!(backend.validate(ValidateRequest { plan: plan() }).is_err());
    }

    #[test]
    fn compose_and_kubernetes_render_the_same_canonical_fleet() {
        let kubernetes_plan = plan();
        let fleet = fleet(&kubernetes_plan).unwrap().clone();
        let mut compose_target: DeploymentTarget = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/deployment-target.json"
        ))
        .unwrap();
        compose_target.spec.kubernetes = None;
        let compose_plan = compile(CompileRequest {
            image: None,
            fleet,
            target: compose_target,
            image_build: ImageBuildIntent::Disabled,
        })
        .unwrap();

        let compose = crate::compose_plan::render(&compose_plan).unwrap();
        let kubernetes = render(&kubernetes_plan).unwrap();
        assert_eq!(compose.image_digest, kubernetes.image_digest);
        assert!(compose.compose_yaml.contains("workspace:"));
        assert!(
            kubernetes
                .kubernetes_yaml
                .unwrap()
                .contains("name: workspace")
        );
    }
}
