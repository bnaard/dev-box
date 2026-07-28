//! Typed Kubernetes connection and network-reconciliation adapters.
//!
//! The adapters in this module deliberately operate on already-provisioned
//! Kubernetes facilities.  They never create a cluster, namespace,
//! IngressClass, GatewayClass, controller, or DNS zone. Deployment-owned
//! Gateway and HTTPRoute objects are reconciled only after their GatewayClass
//! is proved to exist. `kubectl` is only used as the execution transport after
//! the typed target has been validated.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::deployment_backend::BackendError;
use crate::deployment_contract::{
    ConnectionTarget, ConnectionTransport, ContractErrorCode, CredentialReference,
};

pub const LABEL_MANAGED_BY: &str = "app.kubernetes.io/managed-by";
pub const LABEL_DEPLOYMENT_ID: &str = "aibox.projectious.work/deployment-id";
pub const LABEL_SPEC_DIGEST: &str = "aibox.projectious.work/desired-spec-digest";
pub const MANAGED_BY_VALUE: &str = "aibox";

/// A validated command for a Kubernetes connection.  The argv never contains
/// credential material: credentials remain in the contract as references.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KubernetesConnectionCommand {
    pub argv: Vec<String>,
    pub lifecycle: ConnectionLifecycle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionLifecycle {
    /// `kubectl exec` terminates when the remote command terminates. Its exit
    /// status is therefore the connection command's exit status.
    RemoteCommand,
    /// `kubectl port-forward` stays alive until it is cancelled. The caller
    /// owns the child process and must terminate it when the local client ends.
    ManagedPortForward,
}

/// Build exact, shell-free `kubectl` argv for a typed Kubernetes target.
///
/// Interactive exec gets both `--stdin` and `--tty`; non-interactive exec gets
/// neither, which keeps output machine-readable and preserves the remote exit
/// status.  Port-forward always binds loopback only.
pub fn connection_command(
    target: &ConnectionTarget,
) -> Result<KubernetesConnectionCommand, BackendError> {
    match target.spec.transport {
        ConnectionTransport::KubernetesExec => kubernetes_exec_command(target),
        ConnectionTransport::KubernetesPortForward => kubernetes_port_forward_command(target),
        _ => Err(connection_error(
            "Kubernetes backend received a non-Kubernetes connection transport",
        )),
    }
}

fn kubernetes_exec_command(
    target: &ConnectionTarget,
) -> Result<KubernetesConnectionCommand, BackendError> {
    let endpoint = KubernetesExecEndpoint::parse(&target.spec.endpoint)?;
    if endpoint.service != target.spec.service {
        return Err(connection_error(
            "Kubernetes exec endpoint service does not match connection service",
        ));
    }
    let command = if target.spec.invocation.is_empty() {
        vec!["/bin/sh".to_string()]
    } else {
        target.spec.invocation.clone()
    };
    validate_argv(&command)?;

    let mut argv = kubectl_prefix(&endpoint.context, &endpoint.namespace);
    argv.push("exec".to_string());
    if target.spec.interactive {
        argv.extend(["--stdin".to_string(), "--tty".to_string()]);
    }
    argv.push(format!("deployment/{}", endpoint.service));
    argv.push("--".to_string());
    argv.extend(command);
    Ok(KubernetesConnectionCommand {
        argv,
        lifecycle: ConnectionLifecycle::RemoteCommand,
    })
}

fn kubernetes_port_forward_command(
    target: &ConnectionTarget,
) -> Result<KubernetesConnectionCommand, BackendError> {
    if target.spec.interactive {
        return Err(connection_error(
            "Kubernetes port-forward targets are non-interactive; connect a local client separately",
        ));
    }
    if !target.spec.invocation.is_empty() {
        return Err(connection_error(
            "Kubernetes port-forward targets do not accept a remote invocation",
        ));
    }
    let endpoint = KubernetesPortForwardEndpoint::parse(&target.spec.endpoint)?;
    if endpoint.service != target.spec.service {
        return Err(connection_error(
            "Kubernetes port-forward endpoint service does not match connection service",
        ));
    }
    let mut argv = kubectl_prefix(&endpoint.context, &endpoint.namespace);
    argv.extend([
        "port-forward".to_string(),
        "--address".to_string(),
        endpoint.address.clone(),
        format!("service/{}", endpoint.service),
        format!("{}:{}", endpoint.local_port, endpoint.remote_port),
    ]);
    Ok(KubernetesConnectionCommand {
        argv,
        lifecycle: ConnectionLifecycle::ManagedPortForward,
    })
}

fn kubectl_prefix(context: &str, namespace: &str) -> Vec<String> {
    vec![
        "kubectl".to_string(),
        "--context".to_string(),
        context.to_string(),
        "--namespace".to_string(),
        namespace.to_string(),
    ]
}

fn validate_argv(argv: &[String]) -> Result<(), BackendError> {
    if argv
        .iter()
        .any(|argument| argument.is_empty() || argument.contains('\0'))
    {
        return Err(connection_error(
            "Kubernetes connection invocation contains an empty or NUL argument",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct KubernetesExecEndpoint {
    context: String,
    namespace: String,
    service: String,
}
impl KubernetesExecEndpoint {
    fn parse(value: &str) -> Result<Self, BackendError> {
        let parts = value
            .strip_prefix("kubernetes://")
            .ok_or_else(|| {
                connection_error(
                    "Kubernetes exec endpoint must be kubernetes://<context>/<namespace>/<service>",
                )
            })?
            .split('/')
            .collect::<Vec<_>>();
        match parts.as_slice() {
            [context, namespace, service]
                if valid_name(context) && valid_name(namespace) && valid_name(service) =>
            {
                Ok(Self {
                    context: (*context).to_string(),
                    namespace: (*namespace).to_string(),
                    service: (*service).to_string(),
                })
            }
            _ => Err(connection_error(
                "Kubernetes exec endpoint must be kubernetes://<context>/<namespace>/<service>",
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct KubernetesPortForwardEndpoint {
    context: String,
    namespace: String,
    service: String,
    address: String,
    local_port: u16,
    remote_port: u16,
}
impl KubernetesPortForwardEndpoint {
    fn parse(value: &str) -> Result<Self, BackendError> {
        let value = value
            .strip_prefix("kubernetes-port-forward://")
            .ok_or_else(|| {
                connection_error(
                    "Kubernetes port-forward endpoint must use kubernetes-port-forward://",
                )
            })?;
        let (path, ports) = value.rsplit_once('/').ok_or_else(|| {
            connection_error("Kubernetes port-forward endpoint must include local:remote ports")
        })?;
        let parts = path.split('/').collect::<Vec<_>>();
        let [context, namespace, service, address] = parts.as_slice() else {
            return Err(connection_error(
                "Kubernetes port-forward endpoint must be kubernetes-port-forward://<context>/<namespace>/<service>/<loopback-address>/<local-port>:<remote-port>",
            ));
        };
        if !valid_name(context) || !valid_name(namespace) || !valid_name(service) {
            return Err(connection_error(
                "Kubernetes port-forward endpoint has an invalid context, namespace, or service",
            ));
        }
        if !matches!(*address, "127.0.0.1" | "localhost" | "::1") {
            return Err(connection_error(
                "Kubernetes port-forward may bind only 127.0.0.1, localhost, or ::1",
            ));
        }
        let (local_port, remote_port) = ports.split_once(':').ok_or_else(|| {
            connection_error("Kubernetes port-forward endpoint must use <local-port>:<remote-port>")
        })?;
        let local_port = parse_port(local_port)?;
        let remote_port = parse_port(remote_port)?;
        Ok(Self {
            context: (*context).to_string(),
            namespace: (*namespace).to_string(),
            service: (*service).to_string(),
            address: (*address).to_string(),
            local_port,
            remote_port,
        })
    }
}

fn parse_port(value: &str) -> Result<u16, BackendError> {
    value
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
        .ok_or_else(|| {
            connection_error("Kubernetes port-forward ports must be between 1 and 65535")
        })
}
fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '-'
                || character == '.'
        })
        && !value.starts_with(['-', '.'])
        && !value.ends_with(['-', '.'])
}
fn connection_error(message: &str) -> BackendError {
    BackendError {
        code: ContractErrorCode::Connection,
        message: message.to_string(),
    }
}

/// Ownership identity attached to all aibox-managed network resources.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkOwnership {
    pub deployment_id: String,
    pub desired_spec_digest: String,
    /// The complete non-secret ownership identity copied from the deployment
    /// record.  Older records did not carry this field, so the three required
    /// aibox labels remain the minimum compatible identity.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub resource_labels: BTreeMap<String, String>,
}
impl NetworkOwnership {
    pub fn labels(&self) -> BTreeMap<String, String> {
        let mut labels = self.resource_labels.clone();
        labels.extend(BTreeMap::from([
            (LABEL_MANAGED_BY.to_string(), MANAGED_BY_VALUE.to_string()),
            (LABEL_DEPLOYMENT_ID.to_string(), self.deployment_id.clone()),
            (
                LABEL_SPEC_DIGEST.to_string(),
                self.desired_spec_digest.clone(),
            ),
        ]));
        labels
    }
    fn matches(&self, labels: &BTreeMap<String, String>) -> bool {
        self.labels()
            .iter()
            .all(|(key, value)| labels.get(key) == Some(value))
    }
}

/// A discovered, deployment-owned network resource which is safe to delete
/// only after the entire destroy plan has been validated.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ManagedNetworkResourceKey {
    pub kind: String,
    pub namespace: String,
    pub name: String,
}

/// Desired bindings to facilities which must be present before reconciliation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkReconciliationRequest {
    pub namespace: String,
    pub service: String,
    pub hostname: String,
    pub ownership: NetworkOwnership,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingress_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_zone: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dns_credentials: Vec<CredentialReference>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedNetworkResource {
    pub kind: String,
    pub namespace: String,
    pub name: String,
    pub labels: BTreeMap<String, String>,
    pub hostname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
}

/// Narrow typed adapter boundary. Provider implementations are allowed to
/// create deployment-owned Ingress, Gateway, HTTPRoute, and DNSRecord objects,
/// but cannot provision a backing class, controller, cluster, or DNS zone.
pub trait KubernetesNetworkApi: Send + Sync {
    fn ingress_classes(&self) -> Result<BTreeSet<String>, BackendError>;
    fn gateway_classes(&self) -> Result<BTreeSet<String>, BackendError>;
    fn dns_zones(&self) -> Result<BTreeSet<String>, BackendError>;
    fn get(
        &self,
        kind: &str,
        namespace: &str,
        name: &str,
    ) -> Result<Option<ManagedNetworkResource>, BackendError>;
    fn apply(&self, resource: ManagedNetworkResource) -> Result<(), BackendError>;
    fn delete(&self, kind: &str, namespace: &str, name: &str) -> Result<(), BackendError>;
}

/// Production network adapter.  It sends typed manifests to kubectl with a
/// direct argv and stdin boundary; no shell interpolation is involved.
///
/// External DNS is deliberately absent: DNS providers need their own explicit
/// adapter and credential-resolution policy.  Returning no zones makes DNS
/// intent fail capability preflight before any workload mutation.
#[derive(Clone, Debug)]
pub struct KubectlNetworkApi {
    context: String,
}
impl KubectlNetworkApi {
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
        }
    }
    fn prefix(&self, namespace: &str) -> Vec<String> {
        vec![
            "--context".to_string(),
            self.context.clone(),
            "--namespace".to_string(),
            namespace.to_string(),
        ]
    }
    fn output(&self, args: Vec<String>) -> Result<std::process::Output, BackendError> {
        Command::new("kubectl")
            .args(args)
            .output()
            .map_err(|error| BackendError {
                code: ContractErrorCode::Observation,
                message: format!("could not execute kubectl: {error}"),
            })
    }
    fn names(&self, resource: &str) -> Result<BTreeSet<String>, BackendError> {
        let mut args = vec![
            "--context".to_string(),
            self.context.clone(),
            "get".to_string(),
            resource.to_string(),
            "-o".to_string(),
            "json".to_string(),
        ];
        let output = self.output(std::mem::take(&mut args))?;
        if !output.status.success() {
            return Err(BackendError {
                code: ContractErrorCode::CapabilityUnsupported,
                message: format!(
                    "kubectl cannot discover {resource}: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            });
        }
        let json: serde_json::Value =
            serde_json::from_slice(&output.stdout).map_err(|error| BackendError {
                code: ContractErrorCode::Observation,
                message: format!("invalid kubectl discovery response: {error}"),
            })?;
        Ok(json
            .get("items")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|item| {
                item.pointer("/metadata/name")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
            })
            .collect())
    }
    fn manifest(resource: &ManagedNetworkResource) -> Result<serde_json::Value, BackendError> {
        let metadata = serde_json::json!({"name": resource.name, "namespace": resource.namespace, "labels": resource.labels});
        match resource.kind.as_str() {
            "Ingress" => Ok(serde_json::json!({"apiVersion":"networking.k8s.io/v1","kind":"Ingress","metadata":metadata,"spec":{"ingressClassName":resource.class,"rules":[{"host":resource.hostname,"http":{"paths":[{"path":"/","pathType":"Prefix","backend":{"service":{"name":resource.service,"port":{"number":80}}}}]}}]}})),
            "Gateway" => Ok(serde_json::json!({"apiVersion":"gateway.networking.k8s.io/v1","kind":"Gateway","metadata":metadata,"spec":{"gatewayClassName":resource.class,"listeners":[{"name":"http","hostname":resource.hostname,"port":80,"protocol":"HTTP"}]}})),
            "HTTPRoute" => Ok(serde_json::json!({"apiVersion":"gateway.networking.k8s.io/v1","kind":"HTTPRoute","metadata":metadata,"spec":{"parentRefs":[{"name":resource.parent}],"hostnames":[resource.hostname],"rules":[{"backendRefs":[{"name":resource.service,"port":80}]}]}})),
            "DNSRecord" => Err(BackendError { code: ContractErrorCode::CapabilityUnsupported, message: "external DNS requires an explicit provider adapter; generic kubectl cannot create DNS records".to_string() }),
            _ => Err(BackendError { code: ContractErrorCode::Validation, message: format!("unsupported managed network resource kind '{}'", resource.kind) }),
        }
    }
}
impl KubernetesNetworkApi for KubectlNetworkApi {
    fn ingress_classes(&self) -> Result<BTreeSet<String>, BackendError> {
        self.names("ingressclass")
    }
    fn gateway_classes(&self) -> Result<BTreeSet<String>, BackendError> {
        self.names("gatewayclass.gateway.networking.k8s.io")
    }
    fn dns_zones(&self) -> Result<BTreeSet<String>, BackendError> {
        Ok(BTreeSet::new())
    }
    fn get(
        &self,
        kind: &str,
        namespace: &str,
        name: &str,
    ) -> Result<Option<ManagedNetworkResource>, BackendError> {
        if kind == "DNSRecord" {
            return Ok(None);
        }
        let resource = kind.to_ascii_lowercase();
        let mut args = self.prefix(namespace);
        args.extend([
            "get".to_string(),
            format!("{resource}/{name}"),
            "-o".to_string(),
            "json".to_string(),
        ]);
        let output = self.output(args)?;
        if !output.status.success() {
            if String::from_utf8_lossy(&output.stderr).contains("NotFound") {
                return Ok(None);
            }
            return Err(BackendError {
                code: ContractErrorCode::Observation,
                message: format!(
                    "kubectl get {kind} failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            });
        }
        let value: serde_json::Value =
            serde_json::from_slice(&output.stdout).map_err(|e| BackendError {
                code: ContractErrorCode::Observation,
                message: format!("invalid kubectl {kind} response: {e}"),
            })?;
        let labels = value
            .pointer("/metadata/labels")
            .and_then(serde_json::Value::as_object)
            .map(|labels| {
                labels
                    .iter()
                    .filter_map(|(key, value)| {
                        value.as_str().map(|value| (key.clone(), value.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Some(ManagedNetworkResource {
            kind: kind.to_string(),
            namespace: namespace.to_string(),
            name: name.to_string(),
            labels,
            hostname: String::new(),
            class: None,
            service: None,
            parent: None,
            zone: None,
        }))
    }
    fn apply(&self, resource: ManagedNetworkResource) -> Result<(), BackendError> {
        let namespace = resource.namespace.clone();
        let manifest =
            serde_json::to_vec(&Self::manifest(&resource)?).map_err(|e| BackendError {
                code: ContractErrorCode::Mutation,
                message: format!("could not serialize network manifest: {e}"),
            })?;
        let mut args = self.prefix(&namespace);
        args.extend([
            "apply".to_string(),
            "--server-side".to_string(),
            "--field-manager=aibox".to_string(),
            "-f".to_string(),
            "-".to_string(),
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
        child
            .stdin
            .take()
            .ok_or_else(|| BackendError {
                code: ContractErrorCode::Mutation,
                message: "could not open kubectl stdin".to_string(),
            })?
            .write_all(&manifest)
            .map_err(|e| BackendError {
                code: ContractErrorCode::Mutation,
                message: format!("could not write network manifest: {e}"),
            })?;
        let output = child.wait_with_output().map_err(|e| BackendError {
            code: ContractErrorCode::Mutation,
            message: format!("could not await kubectl: {e}"),
        })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(BackendError {
                code: ContractErrorCode::Mutation,
                message: format!(
                    "kubectl network apply failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            })
        }
    }
    fn delete(&self, kind: &str, namespace: &str, name: &str) -> Result<(), BackendError> {
        if kind == "DNSRecord" {
            return Err(BackendError {
                code: ContractErrorCode::CapabilityUnsupported,
                message: "external DNS requires an explicit provider adapter".to_string(),
            });
        }
        let mut args = self.prefix(namespace);
        args.extend([
            "delete".to_string(),
            format!("{}/{}", kind.to_ascii_lowercase(), name),
            "--ignore-not-found=true".to_string(),
        ]);
        let output = self.output(args)?;
        if output.status.success() {
            Ok(())
        } else {
            Err(BackendError {
                code: ContractErrorCode::Mutation,
                message: format!(
                    "kubectl network delete failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            })
        }
    }
}

/// Reconcile namespaced workload entry resources and a record in an existing
/// zone. An IngressClass creates an Ingress; a GatewayClass creates a
/// deployment-owned Gateway and HTTPRoute whose parent ref is that Gateway.
/// Neither path can create the backing controller, class, or DNS zone.
pub fn reconcile_network(
    api: &dyn KubernetesNetworkApi,
    request: &NetworkReconciliationRequest,
) -> Result<Vec<ManagedNetworkResource>, BackendError> {
    validate_network_request(api, request)?;
    let mut applied = Vec::new();
    for resource in expected_network_resources(request) {
        api.apply(resource.clone())?;
        applied.push(resource);
    }
    Ok(applied)
}

/// Confirm that every externally visible endpoint was created with the exact
/// request and is still owned by this deployment.  This check deliberately
/// happens after workload observation so a failed workload cannot publish a
/// route or DNS record.
pub fn verify_network(
    api: &dyn KubernetesNetworkApi,
    request: &NetworkReconciliationRequest,
) -> Result<(), BackendError> {
    for expected in expected_network_resources(request) {
        let actual = api
            .get(&expected.kind, &expected.namespace, &expected.name)?
            .ok_or_else(|| BackendError {
                code: ContractErrorCode::Observation,
                message: format!(
                    "managed {} '{}/{}' was not observed after reconciliation",
                    expected.kind, expected.namespace, expected.name
                ),
            })?;
        if actual != expected || !request.ownership.matches(&actual.labels) {
            return Err(BackendError {
                code: ContractErrorCode::Ownership,
                message: format!(
                    "managed {} '{}/{}' does not match the verified deployment endpoint",
                    expected.kind, expected.namespace, expected.name
                ),
            });
        }
    }
    Ok(())
}

fn expected_network_resources(
    request: &NetworkReconciliationRequest,
) -> Vec<ManagedNetworkResource> {
    let name = network_name(&request.ownership.deployment_id, &request.service);
    let mut resources = Vec::new();
    if let Some(class) = &request.ingress_class {
        resources.push(ManagedNetworkResource {
            kind: "Ingress".to_string(),
            namespace: request.namespace.clone(),
            name: name.clone(),
            labels: request.ownership.labels(),
            hostname: request.hostname.clone(),
            class: Some(class.clone()),
            service: Some(request.service.clone()),
            parent: None,
            zone: None,
        });
    }
    if let Some(class) = &request.gateway_class {
        resources.push(ManagedNetworkResource {
            kind: "Gateway".to_string(),
            namespace: request.namespace.clone(),
            name: name.clone(),
            labels: request.ownership.labels(),
            hostname: request.hostname.clone(),
            class: Some(class.clone()),
            service: None,
            parent: None,
            zone: None,
        });
        resources.push(ManagedNetworkResource {
            kind: "HTTPRoute".to_string(),
            namespace: request.namespace.clone(),
            name: name.clone(),
            labels: request.ownership.labels(),
            hostname: request.hostname.clone(),
            class: None,
            service: Some(request.service.clone()),
            parent: Some(name.clone()),
            zone: None,
        });
    }
    if let Some(zone) = &request.dns_zone {
        resources.push(ManagedNetworkResource {
            kind: "DNSRecord".to_string(),
            namespace: request.namespace.clone(),
            name,
            labels: request.ownership.labels(),
            hostname: request.hostname.clone(),
            class: None,
            service: Some(request.service.clone()),
            parent: None,
            zone: Some(zone.clone()),
        });
    }
    resources
}

pub fn destroy_network(
    api: &dyn KubernetesNetworkApi,
    request: &NetworkReconciliationRequest,
) -> Result<(), BackendError> {
    let plan = plan_network_destroy(api, request)?;
    execute_network_destroy(api, &plan)?;
    verify_network_absent(api, &plan)
}

/// Discover and validate every network resource for one service without
/// mutating it.  Callers that destroy several services must collect every
/// plan before executing any of them.
pub fn plan_network_destroy(
    api: &dyn KubernetesNetworkApi,
    request: &NetworkReconciliationRequest,
) -> Result<Vec<ManagedNetworkResourceKey>, BackendError> {
    let name = network_name(&request.ownership.deployment_id, &request.service);
    let mut owned = Vec::new();
    for kind in ["HTTPRoute", "Gateway", "Ingress", "DNSRecord"] {
        let Some(existing) = api.get(kind, &request.namespace, &name)? else {
            continue;
        };
        if !request.ownership.matches(&existing.labels) {
            return Err(BackendError {
                code: ContractErrorCode::Ownership,
                message: format!(
                    "refusing to delete {kind} '{}/{}': ownership labels do not match deployment record",
                    request.namespace, name
                ),
            });
        }
        if existing.kind != kind || existing.namespace != request.namespace || existing.name != name
        {
            return Err(BackendError {
                code: ContractErrorCode::Ownership,
                message: format!(
                    "refusing to delete {kind} '{}/{}': discovered identity does not match deployment record",
                    request.namespace, name
                ),
            });
        }
        owned.push(ManagedNetworkResourceKey {
            kind: kind.to_string(),
            namespace: request.namespace.clone(),
            name: name.clone(),
        });
    }
    Ok(owned)
}

/// Execute a previously validated network destroy plan.  This deliberately
/// accepts only concrete discovered keys, never a selector.
pub fn execute_network_destroy(
    api: &dyn KubernetesNetworkApi,
    plan: &[ManagedNetworkResourceKey],
) -> Result<(), BackendError> {
    for resource in plan {
        api.delete(&resource.kind, &resource.namespace, &resource.name)?;
    }
    Ok(())
}

/// Verify a completed network destroy plan before the deployment record is
/// marked destroyed.
pub fn verify_network_absent(
    api: &dyn KubernetesNetworkApi,
    plan: &[ManagedNetworkResourceKey],
) -> Result<(), BackendError> {
    for resource in plan {
        if api
            .get(&resource.kind, &resource.namespace, &resource.name)?
            .is_some()
        {
            return Err(BackendError {
                code: ContractErrorCode::Mutation,
                message: format!(
                    "Kubernetes network resource {}/{} remained after destroy",
                    resource.kind, resource.name
                ),
            });
        }
    }
    Ok(())
}

fn validate_network_request(
    api: &dyn KubernetesNetworkApi,
    request: &NetworkReconciliationRequest,
) -> Result<(), BackendError> {
    if !valid_name(&request.namespace)
        || !valid_name(&request.service)
        || request.hostname.is_empty()
    {
        return Err(BackendError {
            code: ContractErrorCode::Validation,
            message: "network reconciliation requires valid namespace, service, and hostname"
                .to_string(),
        });
    }
    if request.ingress_class.is_some() && request.gateway_class.is_some() {
        return Err(BackendError {
            code: ContractErrorCode::Validation,
            message: "select either an existing IngressClass or GatewayClass, not both".to_string(),
        });
    }
    if let Some(class) = &request.ingress_class
        && !api.ingress_classes()?.contains(class)
    {
        return Err(BackendError {
            code: ContractErrorCode::CapabilityUnsupported,
            message: format!("requested existing IngressClass '{class}' is unavailable"),
        });
    }
    if let Some(class) = &request.gateway_class
        && !api.gateway_classes()?.contains(class)
    {
        return Err(BackendError {
            code: ContractErrorCode::CapabilityUnsupported,
            message: format!("requested existing GatewayClass '{class}' is unavailable"),
        });
    }
    if let Some(zone) = &request.dns_zone {
        if !api.dns_zones()?.contains(zone) {
            return Err(BackendError {
                code: ContractErrorCode::CapabilityUnsupported,
                message: format!("requested existing DNS zone '{zone}' is unavailable"),
            });
        }
        if request.hostname != *zone && !request.hostname.ends_with(&format!(".{zone}")) {
            return Err(BackendError {
                code: ContractErrorCode::Validation,
                message: format!(
                    "hostname '{}' is not inside existing DNS zone '{zone}'",
                    request.hostname
                ),
            });
        }
    }
    Ok(())
}

/// Validate every facility before the first managed network resource is
/// created.  Lifecycle callers use this to guarantee a missing DNS provider,
/// class, or zone cannot leave partially applied workloads behind.
pub fn preflight_network(
    api: &dyn KubernetesNetworkApi,
    request: &NetworkReconciliationRequest,
) -> Result<(), BackendError> {
    validate_network_request(api, request)
}
fn network_name(deployment_id: &str, service: &str) -> String {
    format!("{deployment_id}-{service}")
        .chars()
        .take(63)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::deployment_contract::{
        ApiVersion, ConnectionTargetKind, ObjectMeta, OwnershipReference,
    };

    fn target(
        transport: ConnectionTransport,
        endpoint: &str,
        interactive: bool,
    ) -> ConnectionTarget {
        ConnectionTarget {
            api_version: ApiVersion::V1Alpha1,
            kind: ConnectionTargetKind::V1Alpha1,
            metadata: ObjectMeta {
                name: "shell".to_string(),
                owner: OwnershipReference {
                    owner_id: "team-a".to_string(),
                },
                labels: BTreeMap::new(),
            },
            spec: crate::deployment_contract::ConnectionTargetSpec {
                deployment_id: "deploy-a".to_string(),
                service: "workspace".to_string(),
                transport,
                interactive,
                endpoint: endpoint.to_string(),
                invocation: vec![],
                credentials: vec![],
            },
        }
    }

    #[test]
    fn interactive_exec_uses_tty_and_default_shell() {
        let command = connection_command(&target(
            ConnectionTransport::KubernetesExec,
            "kubernetes://staging/workspace-dev/workspace",
            true,
        ))
        .unwrap();
        assert_eq!(command.lifecycle, ConnectionLifecycle::RemoteCommand);
        assert_eq!(
            command.argv,
            vec![
                "kubectl",
                "--context",
                "staging",
                "--namespace",
                "workspace-dev",
                "exec",
                "--stdin",
                "--tty",
                "deployment/workspace",
                "--",
                "/bin/sh"
            ]
        );
    }

    #[test]
    fn noninteractive_exec_omits_tty_and_preserves_argument_boundaries() {
        let mut target = target(
            ConnectionTransport::KubernetesExec,
            "kubernetes://staging/workspace-dev/workspace",
            false,
        );
        target.spec.invocation = vec![
            "sh".to_string(),
            "-c".to_string(),
            "printf '%s' \"two words\"".to_string(),
        ];
        let command = connection_command(&target).unwrap();
        assert!(!command.argv.contains(&"--tty".to_string()));
        assert_eq!(
            command.argv.last(),
            Some(&"printf '%s' \"two words\"".to_string())
        );
    }

    #[test]
    fn connection_argv_never_receives_credential_references_or_secret_canaries() {
        let mut target = target(
            ConnectionTransport::KubernetesExec,
            "kubernetes://staging/workspace-dev/workspace",
            false,
        );
        target.spec.credentials = vec![CredentialReference {
            kind: crate::deployment_contract::CredentialReferenceKind::EnvironmentVariable,
            reference: "super-secret-canary".to_string(),
        }];
        let command = connection_command(&target).unwrap();
        assert!(
            command
                .argv
                .iter()
                .all(|argument| !argument.contains("super-secret-canary"))
        );
    }

    #[test]
    fn port_forward_is_loopback_managed_and_has_no_remote_command() {
        let command = connection_command(&target(
            ConnectionTransport::KubernetesPortForward,
            "kubernetes-port-forward://staging/workspace-dev/workspace/127.0.0.1/18080:8080",
            false,
        ))
        .unwrap();
        assert_eq!(command.lifecycle, ConnectionLifecycle::ManagedPortForward);
        assert_eq!(
            command.argv,
            vec![
                "kubectl",
                "--context",
                "staging",
                "--namespace",
                "workspace-dev",
                "port-forward",
                "--address",
                "127.0.0.1",
                "service/workspace",
                "18080:8080"
            ]
        );
    }

    #[test]
    fn port_forward_rejects_public_bind_and_interactive_mode() {
        assert_eq!(
            connection_command(&target(
                ConnectionTransport::KubernetesPortForward,
                "kubernetes-port-forward://staging/workspace-dev/workspace/0.0.0.0/18080:8080",
                false
            ))
            .unwrap_err()
            .code,
            ContractErrorCode::Connection
        );
        assert_eq!(
            connection_command(&target(
                ConnectionTransport::KubernetesPortForward,
                "kubernetes-port-forward://staging/workspace-dev/workspace/127.0.0.1/18080:8080",
                true
            ))
            .unwrap_err()
            .code,
            ContractErrorCode::Connection
        );
    }

    #[derive(Default)]
    struct FakeNetworkApi {
        ingress_classes: BTreeSet<String>,
        gateway_classes: BTreeSet<String>,
        dns_zones: BTreeSet<String>,
        resources: Mutex<BTreeMap<(String, String, String), ManagedNetworkResource>>,
    }
    impl KubernetesNetworkApi for FakeNetworkApi {
        fn ingress_classes(&self) -> Result<BTreeSet<String>, BackendError> {
            Ok(self.ingress_classes.clone())
        }
        fn gateway_classes(&self) -> Result<BTreeSet<String>, BackendError> {
            Ok(self.gateway_classes.clone())
        }
        fn dns_zones(&self) -> Result<BTreeSet<String>, BackendError> {
            Ok(self.dns_zones.clone())
        }
        fn get(
            &self,
            kind: &str,
            namespace: &str,
            name: &str,
        ) -> Result<Option<ManagedNetworkResource>, BackendError> {
            Ok(self
                .resources
                .lock()
                .unwrap()
                .get(&(kind.to_string(), namespace.to_string(), name.to_string()))
                .cloned())
        }
        fn apply(&self, resource: ManagedNetworkResource) -> Result<(), BackendError> {
            self.resources.lock().unwrap().insert(
                (
                    resource.kind.clone(),
                    resource.namespace.clone(),
                    resource.name.clone(),
                ),
                resource,
            );
            Ok(())
        }
        fn delete(&self, kind: &str, namespace: &str, name: &str) -> Result<(), BackendError> {
            self.resources.lock().unwrap().remove(&(
                kind.to_string(),
                namespace.to_string(),
                name.to_string(),
            ));
            Ok(())
        }
    }
    fn request() -> NetworkReconciliationRequest {
        NetworkReconciliationRequest {
            namespace: "workspace-dev".to_string(),
            service: "workspace".to_string(),
            hostname: "workspace.example.test".to_string(),
            ownership: NetworkOwnership {
                deployment_id: "workspace-12345678".to_string(),
                desired_spec_digest: "sha256:abc".to_string(),
                resource_labels: BTreeMap::new(),
            },
            ingress_class: Some("nginx".to_string()),
            gateway_class: None,
            dns_zone: Some("example.test".to_string()),
            dns_credentials: vec![CredentialReference {
                kind: crate::deployment_contract::CredentialReferenceKind::EnvironmentVariable,
                reference: "super-secret-canary".to_string(),
            }],
        }
    }

    #[test]
    fn reconciler_uses_existing_facilities_and_never_serializes_secret_canary() {
        let api = FakeNetworkApi {
            ingress_classes: ["nginx".to_string()].into_iter().collect(),
            dns_zones: ["example.test".to_string()].into_iter().collect(),
            ..Default::default()
        };
        let applied = reconcile_network(&api, &request()).unwrap();
        assert_eq!(applied.len(), 2);
        let rendered = serde_json::to_string(&applied).unwrap();
        assert!(!rendered.contains("super-secret-canary"));
        assert!(
            applied
                .iter()
                .all(|resource| resource.labels.get(LABEL_MANAGED_BY)
                    == Some(&MANAGED_BY_VALUE.to_string()))
        );
    }

    #[test]
    fn reconciler_rejects_missing_facilities_and_out_of_zone_hosts() {
        let api = FakeNetworkApi::default();
        assert_eq!(
            reconcile_network(&api, &request()).unwrap_err().code,
            ContractErrorCode::CapabilityUnsupported
        );
        let api = FakeNetworkApi {
            ingress_classes: ["nginx".to_string()].into_iter().collect(),
            dns_zones: ["example.test".to_string()].into_iter().collect(),
            ..Default::default()
        };
        let mut bad = request();
        bad.hostname = "outside.invalid".to_string();
        assert_eq!(
            reconcile_network(&api, &bad).unwrap_err().code,
            ContractErrorCode::Validation
        );
    }

    #[test]
    fn gateway_class_reconciles_an_owned_gateway_and_route() {
        let api = FakeNetworkApi {
            gateway_classes: ["shared".to_string()].into_iter().collect(),
            dns_zones: ["example.test".to_string()].into_iter().collect(),
            ..Default::default()
        };
        let mut gateway = request();
        gateway.ingress_class = None;
        gateway.gateway_class = Some("shared".to_string());
        let applied = reconcile_network(&api, &gateway).unwrap();
        assert_eq!(
            applied
                .iter()
                .map(|resource| resource.kind.as_str())
                .collect::<Vec<_>>(),
            vec!["Gateway", "HTTPRoute", "DNSRecord"]
        );
        let name = network_name("workspace-12345678", "workspace");
        let route = api
            .resources
            .lock()
            .unwrap()
            .get(&(
                "HTTPRoute".to_string(),
                "workspace-dev".to_string(),
                name.clone(),
            ))
            .unwrap()
            .clone();
        assert_eq!(route.parent.as_deref(), Some(name.as_str()));
        assert_eq!(route.service.as_deref(), Some("workspace"));
        assert!(gateway.ownership.matches(&route.labels));
    }

    #[test]
    fn guarded_network_destroy_refuses_foreign_resources() {
        let api = FakeNetworkApi {
            ingress_classes: ["nginx".to_string()].into_iter().collect(),
            dns_zones: ["example.test".to_string()].into_iter().collect(),
            ..Default::default()
        };
        reconcile_network(&api, &request()).unwrap();
        let name = network_name("workspace-12345678", "workspace");
        api.resources
            .lock()
            .unwrap()
            .get_mut(&("Ingress".to_string(), "workspace-dev".to_string(), name))
            .unwrap()
            .labels
            .insert(LABEL_DEPLOYMENT_ID.to_string(), "foreign".to_string());
        assert_eq!(
            destroy_network(&api, &request()).unwrap_err().code,
            ContractErrorCode::Ownership
        );
        assert!(api.resources.lock().unwrap().contains_key(&(
            "DNSRecord".to_string(),
            "workspace-dev".to_string(),
            network_name("workspace-12345678", "workspace"),
        )));
    }

    #[test]
    fn guarded_network_destroy_is_idempotent_for_owned_resources() {
        let api = FakeNetworkApi {
            ingress_classes: ["nginx".to_string()].into_iter().collect(),
            dns_zones: ["example.test".to_string()].into_iter().collect(),
            ..Default::default()
        };
        reconcile_network(&api, &request()).unwrap();
        destroy_network(&api, &request()).unwrap();
        destroy_network(&api, &request()).unwrap();
    }

    #[test]
    fn endpoint_verification_rejects_a_missing_or_mismatched_resource() {
        let api = FakeNetworkApi {
            ingress_classes: ["nginx".to_string()].into_iter().collect(),
            dns_zones: ["example.test".to_string()].into_iter().collect(),
            ..Default::default()
        };
        reconcile_network(&api, &request()).unwrap();
        verify_network(&api, &request()).unwrap();

        let name = network_name("workspace-12345678", "workspace");
        api.resources
            .lock()
            .unwrap()
            .get_mut(&("Ingress".to_string(), "workspace-dev".to_string(), name))
            .unwrap()
            .hostname = "foreign.example.test".to_string();
        assert_eq!(
            verify_network(&api, &request()).unwrap_err().code,
            ContractErrorCode::Ownership
        );
    }

    #[test]
    fn kubectl_network_manifest_is_typed_and_never_contains_credential_material() {
        let resource = ManagedNetworkResource {
            kind: "Ingress".to_string(),
            namespace: "workspace-dev".to_string(),
            name: "workspace".to_string(),
            labels: NetworkOwnership {
                deployment_id: "deployment".to_string(),
                desired_spec_digest: "sha256:spec".to_string(),
                resource_labels: BTreeMap::new(),
            }
            .labels(),
            hostname: "workspace.example.test".to_string(),
            class: Some("nginx".to_string()),
            service: Some("workspace".to_string()),
            parent: None,
            zone: None,
        };
        let manifest = KubectlNetworkApi::manifest(&resource).unwrap();
        assert_eq!(
            manifest
                .pointer("/spec/ingressClassName")
                .and_then(serde_json::Value::as_str),
            Some("nginx")
        );
        assert_eq!(
            manifest
                .pointer("/spec/rules/0/http/paths/0/backend/service/name")
                .and_then(serde_json::Value::as_str),
            Some("workspace")
        );
        let encoded = serde_json::to_string(&manifest).unwrap();
        assert!(!encoded.contains("credential") && !encoded.contains("secret-token"));
        assert_eq!(
            KubectlNetworkApi::manifest(&ManagedNetworkResource {
                kind: "DNSRecord".to_string(),
                ..resource
            })
            .unwrap_err()
            .code,
            ContractErrorCode::CapabilityUnsupported
        );
    }
}
