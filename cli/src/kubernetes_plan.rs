//! Pure Kubernetes renderer and capability validation for v1 deployment plans.
//!
//! This module deliberately has no Kubernetes client dependency.  Runtime discovery is
//! represented by a narrow trait and tested with an in-memory fake; rendering and the
//! `aibox deploy plan` path remain filesystem- and network-free.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::compose_plan::RenderedDeploymentPlan;
use crate::deployment_backend::{
    ApplyRequest, ApplyResponse, Backend, BackendError, DestroyRequest, DestroyResponse,
    LogsRequest, LogsResponse, PlanRequest, PlanResponse, StatusRequest, StatusResponse,
    ValidateRequest, ValidateResponse,
};
use crate::deployment_compiler::{DesiredDeploymentAction, DesiredDeploymentPlan};
use crate::deployment_contract::{
    ApiVersion, BackendCapability, BackendKind, ContractErrorCode, CredentialReferenceKind,
    DeployedService, DeploymentOwnership, DeploymentRecord, DeploymentRecordKind,
    DeploymentRecordSpec, DeploymentStatus, KubernetesReconciliationIntent, ObjectMeta,
    PortProtocol, WorkspaceFleetSpec, WorkspaceService,
};

const LABEL_DEPLOYMENT_ID: &str = "aibox.projectious.work/deployment-id";
const LABEL_SPEC_DIGEST: &str = "aibox.projectious.work/desired-spec-digest";
const LABEL_IMAGE_DIGEST: &str = "aibox.projectious.work/image-digest";
const LABEL_NAMESPACE: &str = "aibox.projectious.work/namespace";
const ANNOTATION_RECORD: &str = "aibox.projectious.work/deployment-record";
const ANNOTATION_RECORD_DIGEST: &str = "aibox.projectious.work/deployment-record-digest";

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

/// Narrow, typed Kubernetes lifecycle boundary.  Resources carry their full
/// manifest, so the transport never has to reconstruct workload semantics.
pub trait KubernetesClient: Send + Sync {
    fn context(&self) -> &str;
    fn apply(&self, resource: KubernetesResource) -> Result<(), BackendError>;
    fn list_by_deployment(
        &self,
        deployment_id: &str,
    ) -> Result<Vec<KubernetesResource>, BackendError>;
    fn list_namespace(&self, namespace: &str) -> Result<Vec<KubernetesResource>, BackendError>;
    fn delete(&self, key: &KubernetesResourceKey) -> Result<(), BackendError>;
    fn logs(&self, key: &KubernetesResourceKey) -> Result<Vec<String>, BackendError>;
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KubernetesResourceKind {
    Deployment,
    Service,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KubernetesResourceKey {
    pub kind: KubernetesResourceKind,
    pub namespace: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KubernetesResourceHealth {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KubernetesResource {
    pub key: KubernetesResourceKey,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    #[serde(default)]
    pub annotations: BTreeMap<String, String>,
    pub health: KubernetesResourceHealth,
    /// Canonical workload document sent to the Kubernetes API.  This retains
    /// the immutable image reference, credential placeholders, ports, service
    /// shape, and ownership metadata rendered from the deployment contract.
    #[serde(default)]
    pub manifest: serde_json::Value,
}

/// Production `kubectl` discovery and lifecycle adapter.  It deliberately
/// passes argv directly to `Command`, never through a shell.  Kubernetes API
/// objects remain the typed boundary; kubectl is just the authenticated API
/// transport available in every supported aibox image.
#[derive(Clone, Debug)]
pub struct KubectlClient {
    context: String,
}
impl KubectlClient {
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
        }
    }
    fn command(
        &self,
        args: impl IntoIterator<Item = String>,
    ) -> Result<std::process::Output, BackendError> {
        let output = Command::new("kubectl")
            .args(args)
            .output()
            .map_err(|error| BackendError {
                code: ContractErrorCode::Observation,
                message: format!("could not execute kubectl: {error}"),
            })?;
        if output.status.success() {
            Ok(output)
        } else {
            Err(BackendError {
                code: ContractErrorCode::Observation,
                message: format!(
                    "kubectl failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            })
        }
    }
    fn prefix(&self) -> Vec<String> {
        vec!["--context".to_string(), self.context.clone()]
    }
    fn resources_from_json(
        &self,
        output: std::process::Output,
    ) -> Result<Vec<KubernetesResource>, BackendError> {
        let value: serde_json::Value =
            serde_json::from_slice(&output.stdout).map_err(serialization_error)?;
        let items = value
            .get("items")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| error("kubectl response did not contain items"))?;
        items
            .iter()
            .filter_map(resource_from_object)
            .collect::<Result<_, _>>()
    }
}
impl KubernetesDiscovery for KubectlClient {
    fn discover(&self, context: &str) -> Result<KubernetesCapabilityDescriptor, BackendError> {
        if context != self.context {
            return Err(error("Kubernetes discovery context does not match client"));
        }
        let namespaces = self
            .resources_from_json(self.command([
                "--context".into(),
                context.into(),
                "get".into(),
                "namespace".into(),
                "-o".into(),
                "json".into(),
            ])?)?
            .into_iter()
            .map(|resource| resource.key.name)
            .collect();
        Ok(KubernetesCapabilityDescriptor {
            context: context.to_string(),
            authorized_namespaces: namespaces,
            ingress_classes: self.class_names("ingressclass")?,
            gateway_classes: self
                .class_names("gatewayclass.gateway.networking.k8s.io")
                .unwrap_or_default(),
            // DNS zones are provider-specific and intentionally cannot be
            // invented by this adapter; a configured zone needs a provider adapter.
            dns_zones: BTreeSet::new(),
        })
    }
}
impl KubectlClient {
    fn class_names(&self, kind: &str) -> Result<BTreeSet<String>, BackendError> {
        let mut args = self.prefix();
        args.extend(["get".into(), kind.into(), "-o".into(), "json".into()]);
        self.resources_from_json(self.command(args)?)
            .map(|resources| {
                resources
                    .into_iter()
                    .map(|resource| resource.key.name)
                    .collect()
            })
    }
}
impl KubernetesClient for KubectlClient {
    fn context(&self) -> &str {
        &self.context
    }
    fn apply(&self, resource: KubernetesResource) -> Result<(), BackendError> {
        let manifest = serde_json::to_vec(&resource.manifest).map_err(serialization_error)?;
        let mut args = self.prefix();
        args.extend([
            "apply".into(),
            "--server-side".into(),
            "--field-manager=aibox".into(),
            "-f".into(),
            "-".into(),
        ]);
        let mut child = Command::new("kubectl")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| BackendError {
                code: ContractErrorCode::Mutation,
                message: format!("could not execute kubectl: {e}"),
            })?;
        use std::io::Write as _;
        child
            .stdin
            .take()
            .ok_or_else(|| error("could not open kubectl stdin"))?
            .write_all(&manifest)
            .map_err(store_error)?;
        let output = child.wait_with_output().map_err(store_error)?;
        if output.status.success() {
            Ok(())
        } else {
            Err(BackendError {
                code: ContractErrorCode::Mutation,
                message: format!(
                    "kubectl apply failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            })
        }
    }
    fn list_by_deployment(
        &self,
        deployment_id: &str,
    ) -> Result<Vec<KubernetesResource>, BackendError> {
        let mut args = self.prefix();
        args.extend([
            "get".into(),
            "deployment,service".into(),
            "--all-namespaces".into(),
            "-l".into(),
            format!("{LABEL_DEPLOYMENT_ID}={deployment_id}"),
            "-o".into(),
            "json".into(),
        ]);
        self.resources_from_json(self.command(args)?)
    }
    fn list_namespace(&self, namespace: &str) -> Result<Vec<KubernetesResource>, BackendError> {
        let mut args = self.prefix();
        args.extend([
            "--namespace".into(),
            namespace.into(),
            "get".into(),
            "deployment,service".into(),
            "-o".into(),
            "json".into(),
        ]);
        self.resources_from_json(self.command(args)?)
    }
    fn delete(&self, key: &KubernetesResourceKey) -> Result<(), BackendError> {
        let kind = match key.kind {
            KubernetesResourceKind::Deployment => "deployment",
            KubernetesResourceKind::Service => "service",
        };
        let mut args = self.prefix();
        args.extend([
            "--namespace".into(),
            key.namespace.clone(),
            "delete".into(),
            format!("{kind}/{}", key.name),
            "--ignore-not-found=true".into(),
        ]);
        self.command(args).map(|_| ())
    }
    fn logs(&self, key: &KubernetesResourceKey) -> Result<Vec<String>, BackendError> {
        let mut args = self.prefix();
        args.extend([
            "--namespace".into(),
            key.namespace.clone(),
            "logs".into(),
            format!("deployment/{}", key.name),
            "--all-containers=true".into(),
        ]);
        let output = self.command(args)?;
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect())
    }
}

fn resource_from_object(
    value: &serde_json::Value,
) -> Option<Result<KubernetesResource, BackendError>> {
    let kind = match value.get("kind")?.as_str()? {
        "Deployment" => KubernetesResourceKind::Deployment,
        "Service" => KubernetesResourceKind::Service,
        "Namespace" | "IngressClass" | "GatewayClass" => KubernetesResourceKind::Service,
        _ => return None,
    };
    let metadata = value.get("metadata")?;
    let name = metadata.get("name")?.as_str()?.to_string();
    let namespace = metadata
        .get("namespace")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let string_map = |key: &str| {
        metadata
            .get(key)
            .and_then(serde_json::Value::as_object)
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_string())))
                    .collect()
            })
            .unwrap_or_default()
    };
    let health = if kind == KubernetesResourceKind::Deployment {
        let available = value
            .pointer("/status/availableReplicas")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let desired = value
            .pointer("/spec/replicas")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1);
        if available >= desired {
            KubernetesResourceHealth::Ready
        } else {
            KubernetesResourceHealth::Degraded
        }
    } else {
        KubernetesResourceHealth::Ready
    };
    Some(Ok(KubernetesResource {
        key: KubernetesResourceKey {
            kind,
            namespace,
            name,
        },
        labels: string_map("labels"),
        annotations: string_map("annotations"),
        health,
        manifest: value.clone(),
    }))
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
    lifecycle: Option<KubernetesLifecycle>,
}

struct KubernetesLifecycle {
    project_dir: PathBuf,
    client: Option<Arc<dyn KubernetesClient>>,
}
impl KubernetesBackend {
    #[allow(dead_code)] // deterministic fake-client and plan-only tests
    pub fn new(discovery: Arc<dyn KubernetesDiscovery>) -> Self {
        Self {
            discovery,
            lifecycle: None,
        }
    }
    #[allow(dead_code)] // deterministic fake-client and plan-only tests
    pub fn plan_only() -> Self {
        Self::new(Arc::new(PlanOnlyDiscovery))
    }
    /// Build the production backend.  The concrete kubectl client is selected
    /// from each target's explicit `kube-context:` at operation time.
    pub fn for_project(project_dir: PathBuf) -> Self {
        Self {
            discovery: Arc::new(PlanOnlyDiscovery),
            lifecycle: Some(KubernetesLifecycle {
                project_dir,
                client: None,
            }),
        }
    }
    #[allow(dead_code)] // deterministic fake-client tests
    pub fn with_client(
        project_dir: PathBuf,
        discovery: Arc<dyn KubernetesDiscovery>,
        client: Arc<dyn KubernetesClient>,
    ) -> Self {
        Self {
            discovery,
            lifecycle: Some(KubernetesLifecycle {
                project_dir,
                client: Some(client),
            }),
        }
    }

    fn lifecycle(&self) -> Result<&KubernetesLifecycle, BackendError> {
        self.lifecycle
            .as_ref()
            .ok_or_else(|| BackendError::unsupported("Kubernetes lifecycle"))
    }
}
impl KubernetesLifecycle {
    fn client_for_plan(
        &self,
        plan: &DesiredDeploymentPlan,
    ) -> Result<Arc<dyn KubernetesClient>, BackendError> {
        if let Some(client) = &self.client {
            return Ok(client.clone());
        }
        let (context, _, _) = target(plan)?;
        Ok(Arc::new(KubectlClient::new(context)))
    }
    fn client_for_record(
        &self,
        record: &DeploymentRecord,
    ) -> Result<Arc<dyn KubernetesClient>, BackendError> {
        if let Some(client) = &self.client {
            return Ok(client.clone());
        }
        let context = record
            .spec
            .target
            .target_ref
            .strip_prefix("kube-context:")
            .filter(|context| !context.is_empty())
            .ok_or_else(|| error("Kubernetes deployment record has no explicit kube-context"))?;
        Ok(Arc::new(KubectlClient::new(context)))
    }
}
impl Backend for KubernetesBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Kubernetes
    }
    fn capabilities(&self) -> Vec<BackendCapability> {
        let mut capabilities = vec![
            BackendCapability::Validate,
            BackendCapability::Plan,
            BackendCapability::Exec,
            BackendCapability::PortForward,
        ];
        if self.lifecycle.is_some() {
            capabilities.extend([
                BackendCapability::Apply,
                BackendCapability::Status,
                BackendCapability::Destroy,
                BackendCapability::Logs,
            ]);
        }
        capabilities
    }
    fn validate(&self, request: ValidateRequest) -> Result<ValidateResponse, BackendError> {
        let (context, namespace, intent) = target(&request.plan)?;
        let descriptor = if self
            .lifecycle
            .as_ref()
            .is_some_and(|lifecycle| lifecycle.client.is_none())
        {
            KubectlClient::new(context).discover(context)?
        } else {
            self.discovery.discover(context)?
        };
        validate_discovery(&descriptor, namespace, intent)?;
        Ok(ValidateResponse { valid: true })
    }
    fn plan(&self, request: PlanRequest) -> Result<PlanResponse, BackendError> {
        render(&request.plan).map(|rendered| PlanResponse { rendered })
    }
    fn apply(&self, request: ApplyRequest) -> Result<ApplyResponse, BackendError> {
        self.apply_lifecycle(request)
    }
    fn status(&self, request: StatusRequest) -> Result<StatusResponse, BackendError> {
        self.status_lifecycle(request)
    }
    fn destroy(&self, request: DestroyRequest) -> Result<DestroyResponse, BackendError> {
        self.destroy_lifecycle(request)
    }
    fn logs(&self, request: LogsRequest) -> Result<LogsResponse, BackendError> {
        self.logs_lifecycle(request)
    }
    fn connection(
        &self,
        request: crate::deployment_backend::ConnectionRequest,
    ) -> Result<crate::deployment_backend::ConnectionResponse, BackendError> {
        let command = crate::kubernetes_connection::connection_command(&request.target)?;
        Ok(crate::deployment_backend::ConnectionResponse {
            command: command.argv,
        })
    }
}

impl KubernetesBackend {
    fn apply_lifecycle(&self, request: ApplyRequest) -> Result<ApplyResponse, BackendError> {
        let lifecycle = self.lifecycle()?;
        let client = lifecycle.client_for_plan(&request.plan)?;
        let rendered = render(&request.plan)?;
        let (_context, namespace, _intent) = target(&request.plan)?;
        if client.context()
            != request
                .plan
                .target
                .target_ref
                .trim_start_matches("kube-context:")
        {
            return Err(BackendError {
                code: ContractErrorCode::Validation,
                message: "Kubernetes client context does not match deployment target".to_string(),
            });
        }
        let store = KubernetesDeploymentStore::new(lifecycle.project_dir.clone());
        let _lock = store.lock(&rendered.deployment_id)?;
        let record = record_for(&request.plan, &rendered)?;

        if let Some(existing) = store.load(&rendered.deployment_id)? {
            if existing.spec.status == DeploymentStatus::Observed
                && existing.spec.desired_spec_digest == rendered.desired_spec_digest
                && existing.spec.image.digest == rendered.image_digest
            {
                let observed = observe(client.as_ref(), &existing)?;
                store.save(&observed)?;
                if observed.spec.status == DeploymentStatus::Observed {
                    return Ok(ApplyResponse { record: observed });
                }
            }
        } else if let Some(reconstructed) =
            reconstruct(client.as_ref(), &rendered.deployment_id, Some(namespace))?
        {
            let observed = observe(client.as_ref(), &reconstructed)?;
            store.save(&observed)?;
            if observed.spec.status == DeploymentStatus::Observed
                && observed.spec.desired_spec_digest == rendered.desired_spec_digest
                && observed.spec.image.digest == rendered.image_digest
            {
                return Ok(ApplyResponse { record: observed });
            }
        }

        // Persist before the first mutation.  A failed or interrupted apply is
        // therefore resumable, and never leaves an untracked resource set.
        store.save(&record)?;
        let resources = desired_resources(&request.plan, &record)?;
        for resource in resources {
            if let Err(error) = client.apply(resource) {
                let unavailable = with_status(&record, DeploymentStatus::Unavailable);
                store.save(&unavailable)?;
                return Err(error);
            }
        }
        let observed = observe(client.as_ref(), &record)?;
        store.save(&observed)?;
        Ok(ApplyResponse { record: observed })
    }

    fn status_lifecycle(&self, request: StatusRequest) -> Result<StatusResponse, BackendError> {
        let lifecycle = self.lifecycle()?;
        let store = KubernetesDeploymentStore::new(lifecycle.project_dir.clone());
        let record = match store.load(&request.deployment_id)? {
            Some(record) => record,
            None => match &lifecycle.client {
                Some(client) => reconstruct(client.as_ref(), &request.deployment_id, None)?.ok_or_else(|| BackendError { code: ContractErrorCode::Observation, message: "deployment record not found and remote ownership metadata could not reconstruct it".to_string() })?,
                None => return Err(BackendError { code: ContractErrorCode::Observation, message: "deployment record not found; status requires local record for production Kubernetes targets".to_string() }),
            },
        };
        let observed = observe(lifecycle.client_for_record(&record)?.as_ref(), &record)?;
        store.save(&observed)?;
        Ok(StatusResponse { record: observed })
    }

    fn destroy_lifecycle(&self, request: DestroyRequest) -> Result<DestroyResponse, BackendError> {
        let lifecycle = self.lifecycle()?;
        let store = KubernetesDeploymentStore::new(lifecycle.project_dir.clone());
        let _lock = store.lock(&request.deployment_id)?;
        let mut record = match store.load(&request.deployment_id)? {
            Some(record) => record,
            None => match &lifecycle.client {
                Some(client) => reconstruct(client.as_ref(), &request.deployment_id, None)?
                    .ok_or_else(|| BackendError {
                        code: ContractErrorCode::Ownership,
                        message: "deployment record not found; refusing untracked destroy"
                            .to_string(),
                    })?,
                None => {
                    return Err(BackendError {
                        code: ContractErrorCode::Ownership,
                        message: "deployment record not found; refusing untracked destroy"
                            .to_string(),
                    });
                }
            },
        };
        if record.spec.status == DeploymentStatus::Destroyed {
            return Ok(DestroyResponse { record });
        }
        // Select by namespace and expected names rather than solely by the
        // ownership label: a changed or removed label must cause a guarded
        // refusal, never make a resource invisible to destroy.
        let service_names = record
            .spec
            .services
            .iter()
            .map(|service| service.name.as_str())
            .collect::<BTreeSet<_>>();
        let client = lifecycle.client_for_record(&record)?;
        let resources = client
            .list_namespace(&record.spec.target.scope)?
            .into_iter()
            .filter(|resource| {
                matches!(
                    resource.key.kind,
                    KubernetesResourceKind::Deployment | KubernetesResourceKind::Service
                ) && service_names.contains(resource.key.name.as_str())
            })
            .collect::<Vec<_>>();
        assert_owned(&record, &resources)?;
        for resource in resources {
            client.delete(&resource.key)?;
        }
        record.spec.status = DeploymentStatus::Destroyed;
        store.save(&record)?;
        Ok(DestroyResponse { record })
    }

    fn logs_lifecycle(&self, request: LogsRequest) -> Result<LogsResponse, BackendError> {
        let lifecycle = self.lifecycle()?;
        let store = KubernetesDeploymentStore::new(lifecycle.project_dir.clone());
        let record = match store.load(&request.deployment_id)? {
            Some(record) => record,
            None => match &lifecycle.client {
                Some(client) => reconstruct(client.as_ref(), &request.deployment_id, None)?
                    .ok_or_else(|| BackendError {
                        code: ContractErrorCode::Observation,
                        message: "deployment record not found".to_string(),
                    })?,
                None => {
                    return Err(BackendError {
                        code: ContractErrorCode::Observation,
                        message: "deployment record not found".to_string(),
                    });
                }
            },
        };
        if record.spec.status == DeploymentStatus::Destroyed {
            return Err(BackendError {
                code: ContractErrorCode::Observation,
                message: "deployment has been destroyed".to_string(),
            });
        }
        let client = lifecycle.client_for_record(&record)?;
        let resources = client.list_by_deployment(&record.spec.deployment_id)?;
        assert_owned(&record, &resources)?;
        let deployments = resources
            .into_iter()
            .filter(|resource| resource.key.kind == KubernetesResourceKind::Deployment)
            .filter(|resource| {
                request
                    .service
                    .as_ref()
                    .is_none_or(|service| service == &resource.key.name)
            })
            .collect::<Vec<_>>();
        if deployments.is_empty() {
            return Err(BackendError {
                code: ContractErrorCode::Observation,
                message: "no owned Kubernetes deployment matched requested logs".to_string(),
            });
        }
        let mut lines = Vec::new();
        for deployment in deployments {
            lines.extend(client.logs(&deployment.key)?);
        }
        Ok(LogsResponse { lines })
    }
}

fn desired_resources(
    plan: &DesiredDeploymentPlan,
    record: &DeploymentRecord,
) -> Result<Vec<KubernetesResource>, BackendError> {
    let (_context, namespace, _intent) = target(plan)?;
    let fleet = fleet(plan)?;
    let annotations = record_annotations(record)?;
    let mut resources = Vec::new();
    for service in fleet.spec.services.iter().chain(&fleet.spec.sidecars) {
        let deployment_manifest = managed_manifest(
            deployment_resource(
                service,
                &format!("{}@{}", fleet.spec.image.reference, fleet.spec.image.digest),
                namespace,
                &record.metadata.labels,
            ),
            &annotations,
        );
        resources.push(KubernetesResource {
            key: KubernetesResourceKey {
                kind: KubernetesResourceKind::Deployment,
                namespace: namespace.to_string(),
                name: service.name.clone(),
            },
            labels: record.metadata.labels.clone(),
            annotations: annotations.clone(),
            health: KubernetesResourceHealth::Ready,
            manifest: deployment_manifest,
        });
        if !service.ports.is_empty() {
            let service_manifest = managed_manifest(
                service_resource(service, namespace, &record.metadata.labels),
                &annotations,
            );
            resources.push(KubernetesResource {
                key: KubernetesResourceKey {
                    kind: KubernetesResourceKind::Service,
                    namespace: namespace.to_string(),
                    name: service.name.clone(),
                },
                labels: record.metadata.labels.clone(),
                annotations: annotations.clone(),
                health: KubernetesResourceHealth::Ready,
                manifest: service_manifest,
            });
        }
    }
    Ok(resources)
}

fn managed_manifest(
    mut manifest: serde_json::Value,
    annotations: &BTreeMap<String, String>,
) -> serde_json::Value {
    let metadata = manifest
        .get_mut("metadata")
        .and_then(serde_json::Value::as_object_mut)
        .expect("Kubernetes renderer always provides metadata");
    metadata.insert("annotations".to_string(), serde_json::json!(annotations));
    manifest
}

fn record_annotations(record: &DeploymentRecord) -> Result<BTreeMap<String, String>, BackendError> {
    let body = serde_json::to_string(record).map_err(serialization_error)?;
    let mut annotations = BTreeMap::new();
    annotations.insert(ANNOTATION_RECORD.to_string(), body.clone());
    annotations.insert(
        ANNOTATION_RECORD_DIGEST.to_string(),
        sha256_digest(body.as_bytes()),
    );
    Ok(annotations)
}

fn reconstruct(
    client: &dyn KubernetesClient,
    deployment_id: &str,
    expected_namespace: Option<&str>,
) -> Result<Option<DeploymentRecord>, BackendError> {
    let resources = client.list_by_deployment(deployment_id)?;
    if resources.is_empty() {
        return Ok(None);
    }
    let first = resources.first().expect("checked non-empty");
    let body = first.annotations.get(ANNOTATION_RECORD).ok_or_else(|| {
        ownership_error(
            "refusing remote record reconstruction without a deployment record annotation",
        )
    })?;
    let expected_digest = first
        .annotations
        .get(ANNOTATION_RECORD_DIGEST)
        .ok_or_else(|| {
            ownership_error("refusing remote record reconstruction without a record digest")
        })?;
    if sha256_digest(body.as_bytes()) != *expected_digest {
        return Err(ownership_error(
            "refusing remote record reconstruction with a mismatched record digest",
        ));
    }
    let record = serde_json::from_str::<DeploymentRecord>(body).map_err(|error| BackendError {
        code: ContractErrorCode::Ownership,
        message: format!("refusing invalid remote deployment record annotation: {error}"),
    })?;
    if record.spec.deployment_id != deployment_id
        || record.spec.target.backend != BackendKind::Kubernetes
        || expected_namespace.is_some_and(|namespace| record.spec.target.scope != namespace)
    {
        return Err(ownership_error(
            "refusing remote record whose identity does not match this deployment",
        ));
    }
    assert_owned(&record, &resources)?;
    for resource in &resources {
        if resource.annotations.get(ANNOTATION_RECORD) != Some(body)
            || resource.annotations.get(ANNOTATION_RECORD_DIGEST) != Some(expected_digest)
        {
            return Err(ownership_error(
                "refusing remote reconstruction with inconsistent ownership metadata",
            ));
        }
    }
    Ok(Some(record))
}

fn observe(
    client: &dyn KubernetesClient,
    record: &DeploymentRecord,
) -> Result<DeploymentRecord, BackendError> {
    if record.spec.status == DeploymentStatus::Destroyed {
        return Ok(record.clone());
    }
    let resources = client.list_by_deployment(&record.spec.deployment_id)?;
    if resources.is_empty() {
        return Ok(with_status(record, DeploymentStatus::Unavailable));
    }
    if assert_owned(record, &resources).is_err() {
        return Ok(with_status(record, DeploymentStatus::Orphaned));
    }
    let expected = record
        .spec
        .services
        .iter()
        .map(|service| service.name.as_str())
        .collect::<BTreeSet<_>>();
    let deployments = resources
        .iter()
        .filter(|resource| resource.key.kind == KubernetesResourceKind::Deployment)
        .collect::<Vec<_>>();
    let observed = deployments
        .iter()
        .map(|resource| resource.key.name.as_str())
        .collect::<BTreeSet<_>>();
    let status = if observed != expected {
        DeploymentStatus::Degraded
    } else if deployments
        .iter()
        .all(|resource| resource.health == KubernetesResourceHealth::Ready)
    {
        DeploymentStatus::Observed
    } else if deployments
        .iter()
        .any(|resource| resource.health == KubernetesResourceHealth::Unavailable)
    {
        DeploymentStatus::Unavailable
    } else {
        DeploymentStatus::Degraded
    };
    Ok(with_status(record, status))
}

fn assert_owned(
    record: &DeploymentRecord,
    resources: &[KubernetesResource],
) -> Result<(), BackendError> {
    if resources.is_empty() {
        return Err(ownership_error(
            "refusing to manage: no Kubernetes resources were found",
        ));
    }
    for resource in resources {
        let labels_match = resource.labels.get(LABEL_DEPLOYMENT_ID)
            == Some(&record.spec.ownership.deployment_id_label)
            && resource.labels.get(LABEL_SPEC_DIGEST)
                == Some(&record.spec.ownership.desired_spec_digest_label)
            && resource.labels.get(LABEL_IMAGE_DIGEST)
                == Some(&record.spec.ownership.image_digest_label)
            && resource.labels.get(LABEL_NAMESPACE) == Some(&record.spec.target.scope)
            && resource.key.namespace == record.spec.target.scope
            && record
                .metadata
                .labels
                .iter()
                .all(|(key, value)| resource.labels.get(key) == Some(value));
        if !labels_match {
            return Err(ownership_error(
                "refusing resources not owned by this Kubernetes deployment record",
            ));
        }
    }
    Ok(())
}

fn record_for(
    plan: &DesiredDeploymentPlan,
    rendered: &RenderedDeploymentPlan,
) -> Result<DeploymentRecord, BackendError> {
    let fleet = fleet(plan)?;
    let namespace = &plan.target.scope;
    Ok(DeploymentRecord {
        api_version: ApiVersion::V1Alpha1,
        kind: DeploymentRecordKind::V1Alpha1,
        metadata: ObjectMeta {
            name: rendered.deployment_id.clone(),
            owner: fleet.metadata.owner.clone(),
            labels: rendered.ownership_labels.clone(),
        },
        spec: DeploymentRecordSpec {
            deployment_id: rendered.deployment_id.clone(),
            target: plan.target.clone(),
            desired_spec_digest: rendered.desired_spec_digest.clone(),
            image: fleet.spec.image.clone(),
            ownership: DeploymentOwnership {
                deployment_id_label: rendered.deployment_id.clone(),
                desired_spec_digest_label: rendered.desired_spec_digest.clone(),
                image_digest_label: rendered.image_digest.clone(),
            },
            status: DeploymentStatus::Desired,
            services: fleet
                .spec
                .services
                .iter()
                .chain(&fleet.spec.sidecars)
                .map(|service| DeployedService {
                    name: service.name.clone(),
                    resource_id: format!("{namespace}/Deployment/{}", service.name),
                })
                .collect(),
            connections: vec![],
            processkit_result: None,
        },
    })
}

fn with_status(record: &DeploymentRecord, status: DeploymentStatus) -> DeploymentRecord {
    let mut observed = record.clone();
    observed.spec.status = status;
    observed
}

struct KubernetesDeploymentStore {
    root: PathBuf,
}
impl KubernetesDeploymentStore {
    fn new(project_dir: PathBuf) -> Self {
        Self {
            root: project_dir.join(".aibox").join("deployments"),
        }
    }
    fn record_path(&self, deployment_id: &str) -> PathBuf {
        self.root.join(format!("{deployment_id}.json"))
    }
    fn lock(&self, deployment_id: &str) -> Result<KubernetesDeploymentLock, BackendError> {
        fs::create_dir_all(&self.root).map_err(store_error)?;
        let path = self.root.join(format!("{deployment_id}.lock"));
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| BackendError {
                code: ContractErrorCode::Mutation,
                message: if error.kind() == std::io::ErrorKind::AlreadyExists {
                    format!("deployment operation already in progress for {deployment_id}")
                } else {
                    format!("could not acquire deployment lock: {error}")
                },
            })?;
        Ok(KubernetesDeploymentLock { path })
    }
    fn load(&self, deployment_id: &str) -> Result<Option<DeploymentRecord>, BackendError> {
        let path = self.record_path(deployment_id);
        if !path.exists() {
            return Ok(None);
        }
        serde_json::from_str(&fs::read_to_string(&path).map_err(store_error)?)
            .map(Some)
            .map_err(|error| BackendError {
                code: ContractErrorCode::Observation,
                message: format!("invalid deployment record {}: {error}", path.display()),
            })
    }
    fn save(&self, record: &DeploymentRecord) -> Result<(), BackendError> {
        fs::create_dir_all(&self.root).map_err(store_error)?;
        atomic_write(
            &self.record_path(&record.spec.deployment_id),
            &serde_json::to_vec_pretty(record).map_err(serialization_error)?,
        )
    }
}
struct KubernetesDeploymentLock {
    path: PathBuf,
}
impl Drop for KubernetesDeploymentLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), BackendError> {
    let parent = path.parent().ok_or_else(|| BackendError {
        code: ContractErrorCode::Mutation,
        message: "record path has no parent".to_string(),
    })?;
    fs::create_dir_all(parent).map_err(store_error)?;
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temporary)
        .map_err(store_error)?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(store_error)?;
    fs::rename(temporary, path).map_err(store_error)
}
fn store_error(error: std::io::Error) -> BackendError {
    BackendError {
        code: ContractErrorCode::Mutation,
        message: format!("deployment record store error: {error}"),
    }
}
fn ownership_error(message: &str) -> BackendError {
    BackendError {
        code: ContractErrorCode::Ownership,
        message: message.to_string(),
    }
}
fn sha256_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!(
        "sha256:{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
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
    use std::sync::Mutex;
    use tempfile::tempdir;

    struct FakeKubernetesClient {
        context: String,
        resources: Mutex<BTreeMap<KubernetesResourceKey, KubernetesResource>>,
        logs: Mutex<BTreeMap<KubernetesResourceKey, Vec<String>>>,
        applies: Mutex<usize>,
        deletes: Mutex<usize>,
        fail_after: Mutex<Option<usize>>,
    }
    impl FakeKubernetesClient {
        fn new(context: &str) -> Self {
            Self {
                context: context.to_string(),
                resources: Mutex::new(BTreeMap::new()),
                logs: Mutex::new(BTreeMap::new()),
                applies: Mutex::new(0),
                deletes: Mutex::new(0),
                fail_after: Mutex::new(None),
            }
        }
        fn applies(&self) -> usize {
            *self.applies.lock().unwrap()
        }
        fn deletes(&self) -> usize {
            *self.deletes.lock().unwrap()
        }
        fn fail_after(&self, after: Option<usize>) {
            *self.fail_after.lock().unwrap() = after;
        }
        fn mutate(
            &self,
            key: &KubernetesResourceKey,
            change: impl FnOnce(&mut KubernetesResource),
        ) {
            change(self.resources.lock().unwrap().get_mut(key).unwrap());
        }
        fn remove(&self, key: &KubernetesResourceKey) {
            self.resources.lock().unwrap().remove(key);
        }
        fn set_logs(&self, key: KubernetesResourceKey, lines: &[&str]) {
            self.logs
                .lock()
                .unwrap()
                .insert(key, lines.iter().map(ToString::to_string).collect());
        }
    }
    impl KubernetesClient for FakeKubernetesClient {
        fn context(&self) -> &str {
            &self.context
        }
        fn apply(&self, resource: KubernetesResource) -> Result<(), BackendError> {
            let mut remaining = self.fail_after.lock().unwrap();
            if matches!(*remaining, Some(0)) {
                return Err(BackendError {
                    code: ContractErrorCode::Mutation,
                    message: "fake apply interrupted".to_string(),
                });
            }
            if let Some(value) = remaining.as_mut() {
                *value -= 1;
            }
            *self.applies.lock().unwrap() += 1;
            self.resources
                .lock()
                .unwrap()
                .insert(resource.key.clone(), resource);
            Ok(())
        }
        fn list_by_deployment(
            &self,
            deployment_id: &str,
        ) -> Result<Vec<KubernetesResource>, BackendError> {
            Ok(self
                .resources
                .lock()
                .unwrap()
                .values()
                .filter(|resource| {
                    resource
                        .labels
                        .get(LABEL_DEPLOYMENT_ID)
                        .is_some_and(|value| value == deployment_id)
                })
                .cloned()
                .collect())
        }
        fn list_namespace(&self, namespace: &str) -> Result<Vec<KubernetesResource>, BackendError> {
            Ok(self
                .resources
                .lock()
                .unwrap()
                .values()
                .filter(|resource| resource.key.namespace == namespace)
                .cloned()
                .collect())
        }
        fn delete(&self, key: &KubernetesResourceKey) -> Result<(), BackendError> {
            *self.deletes.lock().unwrap() += 1;
            self.resources.lock().unwrap().remove(key);
            Ok(())
        }
        fn logs(&self, key: &KubernetesResourceKey) -> Result<Vec<String>, BackendError> {
            Ok(self
                .logs
                .lock()
                .unwrap()
                .get(key)
                .cloned()
                .unwrap_or_default())
        }
    }

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

    fn lifecycle_backend(root: &Path, client: Arc<FakeKubernetesClient>) -> KubernetesBackend {
        let descriptor = KubernetesCapabilityDescriptor {
            context: "staging".to_string(),
            authorized_namespaces: ["workspace-dev".to_string()].into_iter().collect(),
            ingress_classes: ["nginx".to_string()].into_iter().collect(),
            gateway_classes: ["egress".to_string()].into_iter().collect(),
            dns_zones: ["example.test".to_string()].into_iter().collect(),
        };
        KubernetesBackend::with_client(
            root.to_path_buf(),
            Arc::new(FakeKubernetesDiscovery {
                descriptors: [("staging".to_string(), descriptor)].into_iter().collect(),
            }),
            client,
        )
    }

    fn deployment_key(name: &str) -> KubernetesResourceKey {
        KubernetesResourceKey {
            kind: KubernetesResourceKind::Deployment,
            namespace: "workspace-dev".to_string(),
            name: name.to_string(),
        }
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

    #[test]
    fn lifecycle_apply_is_idempotent_and_changed_apply_reconciles_a_new_record() {
        let root = tempdir().unwrap();
        let client = Arc::new(FakeKubernetesClient::new("staging"));
        let backend = lifecycle_backend(root.path(), client.clone());
        let first = backend.apply(ApplyRequest { plan: plan() }).unwrap();
        assert_eq!(first.record.spec.status, DeploymentStatus::Observed);
        let applied = client.applies();
        let unchanged = backend.apply(ApplyRequest { plan: plan() }).unwrap();
        assert_eq!(unchanged.record, first.record);
        assert_eq!(
            client.applies(),
            applied,
            "unchanged apply must not mutate Kubernetes"
        );

        let mut changed = plan();
        let DesiredDeploymentAction::DeployFleet { fleet, .. } =
            changed.actions.last_mut().unwrap()
        else {
            unreachable!()
        };
        fleet.spec.image.digest = format!("sha256:{}", "b".repeat(64));
        changed.desired_spec_digest = format!("sha256:{}", "c".repeat(64));
        let updated = backend.apply(ApplyRequest { plan: changed }).unwrap();
        assert_ne!(
            updated.record.spec.deployment_id,
            first.record.spec.deployment_id
        );
        assert!(client.applies() > applied);
    }

    #[test]
    fn interrupted_apply_persists_a_recoverable_receipt_then_converges() {
        let root = tempdir().unwrap();
        let client = Arc::new(FakeKubernetesClient::new("staging"));
        client.fail_after(Some(1));
        let backend = lifecycle_backend(root.path(), client.clone());
        let rendered = render(&plan()).unwrap();
        assert_eq!(
            backend
                .apply(ApplyRequest { plan: plan() })
                .unwrap_err()
                .code,
            ContractErrorCode::Mutation
        );
        assert_eq!(
            KubernetesDeploymentStore::new(root.path().to_path_buf())
                .load(&rendered.deployment_id)
                .unwrap()
                .unwrap()
                .spec
                .status,
            DeploymentStatus::Unavailable
        );
        client.fail_after(None);
        assert_eq!(
            backend
                .apply(ApplyRequest { plan: plan() })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Observed
        );
    }

    #[test]
    fn status_distinguishes_degraded_unavailable_and_orphaned_resources() {
        let root = tempdir().unwrap();
        let client = Arc::new(FakeKubernetesClient::new("staging"));
        let backend = lifecycle_backend(root.path(), client.clone());
        let record = backend.apply(ApplyRequest { plan: plan() }).unwrap().record;
        let key = deployment_key("workspace");
        client.mutate(&key, |resource| {
            resource.health = KubernetesResourceHealth::Degraded
        });
        assert_eq!(
            backend
                .status(StatusRequest {
                    deployment_id: record.spec.deployment_id.clone()
                })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Degraded
        );
        client.mutate(&key, |resource| {
            resource.health = KubernetesResourceHealth::Unavailable
        });
        assert_eq!(
            backend
                .status(StatusRequest {
                    deployment_id: record.spec.deployment_id.clone()
                })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Unavailable
        );
        client.mutate(&key, |resource| {
            resource.labels.remove(LABEL_IMAGE_DIGEST);
        });
        assert_eq!(
            backend
                .status(StatusRequest {
                    deployment_id: record.spec.deployment_id
                })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Orphaned
        );
    }

    #[test]
    fn status_classifies_missing_expected_resources_as_drift_and_an_empty_set_as_unavailable() {
        let root = tempdir().unwrap();
        let client = Arc::new(FakeKubernetesClient::new("staging"));
        let backend = lifecycle_backend(root.path(), client.clone());
        let record = backend.apply(ApplyRequest { plan: plan() }).unwrap().record;
        client.remove(&deployment_key("workspace"));
        assert_eq!(
            backend
                .status(StatusRequest {
                    deployment_id: record.spec.deployment_id.clone()
                })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Degraded
        );
        client.remove(&KubernetesResourceKey {
            kind: KubernetesResourceKind::Service,
            namespace: "workspace-dev".to_string(),
            name: "workspace".to_string(),
        });
        assert_eq!(
            backend
                .status(StatusRequest {
                    deployment_id: record.spec.deployment_id
                })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Unavailable
        );
    }

    #[test]
    fn reconstructs_durable_record_from_verified_remote_metadata() {
        let root = tempdir().unwrap();
        let client = Arc::new(FakeKubernetesClient::new("staging"));
        let backend = lifecycle_backend(root.path(), client.clone());
        let record = backend.apply(ApplyRequest { plan: plan() }).unwrap().record;
        let store = KubernetesDeploymentStore::new(root.path().to_path_buf());
        fs::remove_file(store.record_path(&record.spec.deployment_id)).unwrap();
        let reconstructed = backend
            .status(StatusRequest {
                deployment_id: record.spec.deployment_id.clone(),
            })
            .unwrap()
            .record;
        assert_eq!(reconstructed, record);
        assert_eq!(
            store.load(&record.spec.deployment_id).unwrap().unwrap(),
            record
        );
    }

    #[test]
    fn reconstruction_refuses_incomplete_remote_ownership_metadata() {
        let root = tempdir().unwrap();
        let client = Arc::new(FakeKubernetesClient::new("staging"));
        let backend = lifecycle_backend(root.path(), client.clone());
        let record = backend.apply(ApplyRequest { plan: plan() }).unwrap().record;
        client.mutate(&deployment_key("workspace"), |resource| {
            resource.annotations.remove(ANNOTATION_RECORD);
        });
        let store = KubernetesDeploymentStore::new(root.path().to_path_buf());
        fs::remove_file(store.record_path(&record.spec.deployment_id)).unwrap();
        assert_eq!(
            backend
                .status(StatusRequest {
                    deployment_id: record.spec.deployment_id
                })
                .unwrap_err()
                .code,
            ContractErrorCode::Ownership
        );
    }

    #[test]
    fn destroy_refuses_foreign_unlabelled_and_digest_mismatched_resources_then_is_repeatable() {
        for mutation in ["foreign", "unlabelled", "digest"] {
            let root = tempdir().unwrap();
            let client = Arc::new(FakeKubernetesClient::new("staging"));
            let backend = lifecycle_backend(root.path(), client.clone());
            let record = backend.apply(ApplyRequest { plan: plan() }).unwrap().record;
            client.mutate(&deployment_key("workspace"), |resource| match mutation {
                "foreign" => {
                    resource
                        .labels
                        .insert(LABEL_DEPLOYMENT_ID.to_string(), "foreign".to_string());
                }
                "unlabelled" => resource.labels.clear(),
                "digest" => {
                    resource
                        .labels
                        .insert(LABEL_IMAGE_DIGEST.to_string(), "sha256:foreign".to_string());
                }
                _ => unreachable!(),
            });
            assert_eq!(
                backend
                    .destroy(DestroyRequest {
                        deployment_id: record.spec.deployment_id
                    })
                    .unwrap_err()
                    .code,
                ContractErrorCode::Ownership,
                "{mutation}"
            );
            assert_eq!(client.deletes(), 0, "{mutation} must not delete a resource");
        }

        let root = tempdir().unwrap();
        let client = Arc::new(FakeKubernetesClient::new("staging"));
        let backend = lifecycle_backend(root.path(), client.clone());
        let record = backend.apply(ApplyRequest { plan: plan() }).unwrap().record;
        assert_eq!(
            backend
                .destroy(DestroyRequest {
                    deployment_id: record.spec.deployment_id.clone()
                })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Destroyed
        );
        let deletes = client.deletes();
        assert_eq!(
            backend
                .destroy(DestroyRequest {
                    deployment_id: record.spec.deployment_id
                })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Destroyed
        );
        assert_eq!(
            client.deletes(),
            deletes,
            "repeat destroy must not call Kubernetes"
        );
    }

    #[test]
    fn logs_are_owned_and_service_scoped() {
        let root = tempdir().unwrap();
        let client = Arc::new(FakeKubernetesClient::new("staging"));
        let backend = lifecycle_backend(root.path(), client.clone());
        let record = backend.apply(ApplyRequest { plan: plan() }).unwrap().record;
        client.set_logs(deployment_key("workspace"), &["one", "two"]);
        assert_eq!(
            backend
                .logs(LogsRequest {
                    deployment_id: record.spec.deployment_id,
                    service: Some("workspace".to_string())
                })
                .unwrap()
                .lines,
            vec!["one", "two"]
        );
    }

    #[test]
    fn lifecycle_resources_keep_the_full_rendered_workload_semantics() {
        let root = tempdir().unwrap();
        let client = Arc::new(FakeKubernetesClient::new("staging"));
        let backend = lifecycle_backend(root.path(), client.clone());
        let record = backend.apply(ApplyRequest { plan: plan() }).unwrap().record;
        let deployment = client
            .resources
            .lock()
            .unwrap()
            .get(&deployment_key("workspace"))
            .cloned()
            .unwrap();
        assert_eq!(
            deployment
                .manifest
                .pointer("/spec/template/spec/containers/0/image")
                .and_then(serde_json::Value::as_str),
            Some("ghcr.io/acme/demo@sha256:image")
        );
        let embedded: DeploymentRecord = serde_json::from_str(
            deployment
                .manifest
                .pointer("/metadata/annotations/aibox.projectious.work~1deployment-record")
                .and_then(serde_json::Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(embedded.spec.deployment_id, record.spec.deployment_id);
        assert_eq!(embedded.spec.status, DeploymentStatus::Desired);
        assert!(
            deployment
                .manifest
                .pointer("/spec/template/spec/containers/0/ports/0/containerPort")
                .is_some()
        );
    }
}
