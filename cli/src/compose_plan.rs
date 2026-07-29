//! Pure Compose/devcontainer renderer for canonical v1 deployment plans.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::deployment_backend::{
    ApplyRequest, ApplyResponse, Backend, BackendError, ConnectionRequest, ConnectionResponse,
    DestroyRequest, DestroyResponse, LogsRequest, LogsResponse, PlanRequest, PlanResponse,
    RenderedDeploymentArtifacts, RenderedDeploymentPlan, StatusRequest, StatusResponse,
    ValidateRequest, ValidateResponse,
};
use crate::deployment_compiler::{DesiredDeploymentAction, DesiredDeploymentPlan};
use crate::deployment_contract::{
    ApiVersion, BackendCapability, BackendKind, ConnectionTransport, ContractErrorCode,
    CredentialReferenceKind, DeployedService, DeploymentOwnership, DeploymentRecord,
    DeploymentRecordKind, DeploymentRecordSpec, DeploymentStatus, ObjectMeta, PortProtocol,
    WorkspaceService,
};

pub const LABEL_DEPLOYMENT_ID: &str = "aibox.projectious.work/deployment-id";
pub const LABEL_SPEC_DIGEST: &str = "aibox.projectious.work/desired-spec-digest";
pub const LABEL_IMAGE_DIGEST: &str = "aibox.projectious.work/image-digest";

/// A small command boundary keeps lifecycle tests independent of Docker or
/// Podman.  It deliberately transports argv as a vector, never through a
/// shell, so connection invocations cannot be reinterpreted by a shell.
pub trait ComposeExecutor: Send + Sync {
    fn run(&self, argv: &[String], cwd: &Path) -> Result<CommandOutput, BackendError>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CommandOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Default)]
pub struct SystemComposeExecutor;
impl ComposeExecutor for SystemComposeExecutor {
    fn run(&self, argv: &[String], cwd: &Path) -> Result<CommandOutput, BackendError> {
        let (program, args) = argv.split_first().ok_or_else(|| BackendError {
            code: ContractErrorCode::Mutation,
            message: "empty compose command".to_string(),
        })?;
        let output = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(|error| BackendError {
                code: ContractErrorCode::Mutation,
                message: format!("could not execute compose runtime: {error}"),
            })?;
        Ok(CommandOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

pub struct ComposeBackend {
    project_dir: PathBuf,
    executor: Arc<dyn ComposeExecutor>,
}

impl ComposeBackend {
    pub fn for_current_dir() -> Self {
        Self::for_project(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    pub fn for_project(project_dir: PathBuf) -> Self {
        Self::new(project_dir, Arc::new(SystemComposeExecutor))
    }

    pub fn new(project_dir: PathBuf, executor: Arc<dyn ComposeExecutor>) -> Self {
        Self {
            project_dir,
            executor,
        }
    }

    fn store(&self) -> DeploymentStore {
        DeploymentStore::new(self.project_dir.clone())
    }

    fn command_prefix(&self, deployment_id: &str) -> Vec<String> {
        let compose = self.store().compose_path(deployment_id);
        vec![
            "docker".to_string(),
            "compose".to_string(),
            "--project-name".to_string(),
            deployment_id.to_string(),
            "--file".to_string(),
            compose.display().to_string(),
        ]
    }

    fn execute(&self, argv: Vec<String>, operation: &str) -> Result<CommandOutput, BackendError> {
        let result = self.executor.run(&argv, &self.project_dir)?;
        if result.success {
            Ok(result)
        } else {
            Err(BackendError {
                code: ContractErrorCode::Mutation,
                message: format!("compose {operation} failed: {}", result.stderr.trim()),
            })
        }
    }

    fn observe(&self, record: &DeploymentRecord) -> Result<DeploymentRecord, BackendError> {
        if record.spec.status == DeploymentStatus::Destroyed {
            return Ok(record.clone());
        }
        let mut argv = self.command_prefix(&record.spec.deployment_id);
        argv.extend(["ps".to_string(), "--format".to_string(), "json".to_string()]);
        let output = match self.executor.run(&argv, &self.project_dir) {
            Ok(output) if output.success => output,
            Ok(_) => return Ok(with_status(record, DeploymentStatus::Unavailable, vec![])),
            Err(_) => return Ok(with_status(record, DeploymentStatus::Unavailable, vec![])),
        };
        let rows = parse_compose_rows(&output.stdout).map_err(|error| BackendError {
            code: ContractErrorCode::Observation,
            message: error,
        })?;
        let expected = record
            .spec
            .services
            .iter()
            .map(|service| service.name.as_str())
            .collect::<Vec<_>>();
        let names = rows
            .iter()
            .filter_map(|row| row.get("Service").or_else(|| row.get("Name")))
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        let running = rows
            .iter()
            .filter(|row| row.get("State").and_then(serde_json::Value::as_str) == Some("running"))
            .count();
        let status = if rows.is_empty() {
            DeploymentStatus::Unavailable
        } else if expected.iter().all(|name| names.contains(name)) && running == expected.len() {
            DeploymentStatus::Observed
        } else {
            DeploymentStatus::Degraded
        };
        Ok(with_status(record, status, rows_to_services(rows)))
    }

    fn assert_owned(&self, record: &DeploymentRecord) -> Result<(), BackendError> {
        let mut argv = self.command_prefix(&record.spec.deployment_id);
        argv.extend(["ps".to_string(), "--format".to_string(), "json".to_string()]);
        let output = self
            .executor
            .run(&argv, &self.project_dir)
            .map_err(|error| BackendError {
                code: ContractErrorCode::Ownership,
                message: error.message,
            })?;
        if !output.success {
            return Err(BackendError {
                code: ContractErrorCode::Ownership,
                message: "cannot verify Compose ownership before destroy".to_string(),
            });
        }
        let rows = parse_compose_rows(&output.stdout).map_err(|error| BackendError {
            code: ContractErrorCode::Ownership,
            message: error,
        })?;
        if rows.is_empty() {
            return Err(BackendError {
                code: ContractErrorCode::Ownership,
                message: "refusing to destroy: no labelled Compose resources were found"
                    .to_string(),
            });
        }
        for row in &rows {
            let labels = row_labels(row);
            let matches = labels.get(LABEL_DEPLOYMENT_ID)
                == Some(&record.spec.ownership.deployment_id_label)
                && labels.get(LABEL_SPEC_DIGEST)
                    == Some(&record.spec.ownership.desired_spec_digest_label)
                && labels.get(LABEL_IMAGE_DIGEST)
                    == Some(&record.spec.ownership.image_digest_label)
                && record
                    .metadata
                    .labels
                    .iter()
                    .all(|(key, value)| labels.get(key) == Some(value));
            if !matches {
                return Err(BackendError {
                    code: ContractErrorCode::Ownership,
                    message: "refusing to destroy resources not owned by this deployment record"
                        .to_string(),
                });
            }
        }
        Ok(())
    }
}
impl Backend for ComposeBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Compose
    }
    fn capabilities(&self) -> Vec<BackendCapability> {
        vec![
            BackendCapability::Validate,
            BackendCapability::Plan,
            BackendCapability::Apply,
            BackendCapability::Status,
            BackendCapability::Destroy,
            BackendCapability::Logs,
            BackendCapability::Exec,
        ]
    }
    fn validate(&self, request: ValidateRequest) -> Result<ValidateResponse, BackendError> {
        render(&request.plan).map(|_| ValidateResponse { valid: true })
    }
    fn plan(&self, request: PlanRequest) -> Result<PlanResponse, BackendError> {
        render(&request.plan).map(|rendered| PlanResponse { rendered })
    }
    fn apply(&self, request: ApplyRequest) -> Result<ApplyResponse, BackendError> {
        let rendered = render(&request.plan)?;
        if !is_immutable_digest(&rendered.image_digest) {
            return Err(BackendError {
                code: ContractErrorCode::Validation,
                message: "Compose apply requires a sha256 immutable image digest".to_string(),
            });
        }
        let store = self.store();
        let _lock = store.lock(&rendered.deployment_id)?;
        if let Some(existing) = store.load(&rendered.deployment_id)?
            && existing.spec.status == DeploymentStatus::Observed
            && existing.spec.desired_spec_digest == rendered.desired_spec_digest
            && existing.spec.image.digest == rendered.image_digest
        {
            return Ok(ApplyResponse { record: existing });
        }
        store.write_artifacts(&rendered)?;
        let mut record = record_for(&request.plan, &rendered)?;
        // Persist intent before mutation. An interrupted run therefore leaves
        // a recoverable receipt rather than an untracked runtime.
        store.save(&record)?;
        let mut argv = self.command_prefix(&rendered.deployment_id);
        argv.extend([
            "up".to_string(),
            "--detach".to_string(),
            "--remove-orphans".to_string(),
        ]);
        if let Err(error) = self.execute(argv, "up") {
            record.spec.status = DeploymentStatus::Unavailable;
            store.save(&record)?;
            return Err(error);
        }
        record = self.observe(&record)?;
        store.save(&record)?;
        Ok(ApplyResponse { record })
    }
    fn status(&self, request: StatusRequest) -> Result<StatusResponse, BackendError> {
        let store = self.store();
        let record = store
            .load(&request.deployment_id)?
            .ok_or_else(|| BackendError {
                code: ContractErrorCode::Observation,
                message: "deployment record not found".to_string(),
            })?;
        let observed = self.observe(&record)?;
        store.save(&observed)?;
        Ok(StatusResponse { record: observed })
    }
    fn destroy(&self, request: DestroyRequest) -> Result<DestroyResponse, BackendError> {
        let store = self.store();
        let _lock = store.lock(&request.deployment_id)?;
        let mut record = store
            .load(&request.deployment_id)?
            .ok_or_else(|| BackendError {
                code: ContractErrorCode::Ownership,
                message: "deployment record not found; refusing untracked destroy".to_string(),
            })?;
        if record.spec.status == DeploymentStatus::Destroyed {
            return Ok(DestroyResponse { record });
        }
        self.assert_owned(&record)?;
        let mut argv = self.command_prefix(&record.spec.deployment_id);
        argv.extend(["down".to_string(), "--remove-orphans".to_string()]);
        self.execute(argv, "down")?;
        record.spec.status = DeploymentStatus::Destroyed;
        store.save(&record)?;
        Ok(DestroyResponse { record })
    }
    fn logs(&self, request: LogsRequest) -> Result<LogsResponse, BackendError> {
        let store = self.store();
        let record = store
            .load(&request.deployment_id)?
            .ok_or_else(|| BackendError {
                code: ContractErrorCode::Observation,
                message: "deployment record not found".to_string(),
            })?;
        let mut argv = self.command_prefix(&record.spec.deployment_id);
        argv.push("logs".to_string());
        argv.push("--no-color".to_string());
        if let Some(service) = request.service {
            argv.push(service);
        }
        let output = self.execute(argv, "logs")?;
        Ok(LogsResponse {
            lines: output.stdout.lines().map(ToOwned::to_owned).collect(),
        })
    }
    fn connection(&self, request: ConnectionRequest) -> Result<ConnectionResponse, BackendError> {
        if request.target.spec.transport != ConnectionTransport::ComposeExec {
            return Err(BackendError::unsupported("requested connection transport"));
        }
        let store = self.store();
        let record = store
            .load(&request.target.spec.deployment_id)?
            .ok_or_else(|| BackendError {
                code: ContractErrorCode::Connection,
                message: "deployment record not found".to_string(),
            })?;
        if record.spec.status == DeploymentStatus::Destroyed {
            return Err(BackendError {
                code: ContractErrorCode::Connection,
                message: "deployment has been destroyed".to_string(),
            });
        }
        let argv = compose_exec_argv(
            &self.command_prefix(&record.spec.deployment_id),
            &request.target.spec.service,
            &request.target.spec.invocation,
            request.target.spec.interactive,
        );
        Ok(ConnectionResponse { command: argv })
    }
}

/// Project-local, durable deployment receipt store.  Lock files are acquired
/// with `create_new`, which refuses concurrent mutation without relying on a
/// platform-specific advisory-lock implementation.
struct DeploymentStore {
    root: PathBuf,
}
impl DeploymentStore {
    fn new(project_dir: PathBuf) -> Self {
        Self {
            root: project_dir.join(".aibox").join("deployments"),
        }
    }
    fn record_path(&self, deployment_id: &str) -> PathBuf {
        self.root.join(format!("{deployment_id}.json"))
    }
    fn compose_path(&self, deployment_id: &str) -> PathBuf {
        self.root.join(deployment_id).join("docker-compose.yml")
    }
    fn lock(&self, deployment_id: &str) -> Result<DeploymentLock, BackendError> {
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
        Ok(DeploymentLock { path })
    }
    fn load(&self, deployment_id: &str) -> Result<Option<DeploymentRecord>, BackendError> {
        let path = self.record_path(deployment_id);
        if !path.exists() {
            return Ok(None);
        }
        let body = fs::read_to_string(&path).map_err(store_error)?;
        serde_json::from_str(&body)
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
            &serde_json::to_vec_pretty(record).map_err(|error| BackendError {
                code: ContractErrorCode::Mutation,
                message: error.to_string(),
            })?,
        )
    }
    fn write_artifacts(&self, rendered: &RenderedDeploymentPlan) -> Result<(), BackendError> {
        let directory = self.root.join(&rendered.deployment_id);
        fs::create_dir_all(&directory).map_err(store_error)?;
        let (compose_yaml, devcontainer_json) =
            rendered.artifacts.compose().ok_or_else(|| BackendError {
                code: ContractErrorCode::Planning,
                message: "Compose backend received non-Compose rendered artifacts".to_string(),
            })?;
        atomic_write(
            &directory.join("docker-compose.yml"),
            compose_yaml.as_bytes(),
        )?;
        atomic_write(
            &directory.join("devcontainer.json"),
            devcontainer_json.as_bytes(),
        )
    }
}
struct DeploymentLock {
    path: PathBuf,
}
impl Drop for DeploymentLock {
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
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp)
        .map_err(store_error)?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(store_error)?;
    fs::rename(&temp, path).map_err(store_error)
}
fn store_error(error: std::io::Error) -> BackendError {
    BackendError {
        code: ContractErrorCode::Mutation,
        message: format!("deployment record store error: {error}"),
    }
}

fn record_for(
    plan: &DesiredDeploymentPlan,
    rendered: &RenderedDeploymentPlan,
) -> Result<DeploymentRecord, BackendError> {
    let fleet = plan
        .actions
        .iter()
        .find_map(|action| match action {
            DesiredDeploymentAction::DeployFleet { fleet, .. } => Some(fleet),
            _ => None,
        })
        .ok_or_else(|| BackendError {
            code: ContractErrorCode::Planning,
            message: "plan has no deploy-fleet action".to_string(),
        })?;
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
                    resource_id: format!("{}-{}", rendered.deployment_id, service.name),
                })
                .collect(),
            connections: vec![],
            processkit_result: None,
        },
    })
}
fn with_status(
    record: &DeploymentRecord,
    status: DeploymentStatus,
    services: Vec<DeployedService>,
) -> DeploymentRecord {
    let mut observed = record.clone();
    observed.spec.status = status;
    if !services.is_empty() {
        observed.spec.services = services;
    }
    observed
}
fn is_immutable_digest(digest: &str) -> bool {
    digest.starts_with("sha256:")
        && digest.len() == 71
        && digest[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn parse_compose_rows(body: &str) -> Result<Vec<serde_json::Value>, String> {
    let body = body.trim();
    if body.is_empty() {
        return Ok(vec![]);
    }
    if let Ok(rows) = serde_json::from_str::<Vec<serde_json::Value>>(body) {
        return Ok(rows);
    }
    body.lines()
        .map(|line| {
            serde_json::from_str(line).map_err(|error| format!("invalid Compose ps JSON: {error}"))
        })
        .collect()
}
fn row_labels(row: &serde_json::Value) -> BTreeMap<String, String> {
    let Some(value) = row.get("Labels") else {
        return BTreeMap::new();
    };
    if let Some(map) = value.as_object() {
        return map
            .iter()
            .filter_map(|(key, value)| value.as_str().map(|value| (key.clone(), value.to_string())))
            .collect();
    }
    value
        .as_str()
        .unwrap_or_default()
        .split(',')
        .filter_map(|item| item.split_once('='))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect()
}
fn rows_to_services(rows: Vec<serde_json::Value>) -> Vec<DeployedService> {
    rows.into_iter()
        .filter_map(|row| {
            let name = row
                .get("Service")
                .or_else(|| row.get("Name"))?
                .as_str()?
                .to_string();
            let resource_id = row
                .get("ID")
                .or_else(|| row.get("Id"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&name)
                .to_string();
            Some(DeployedService { name, resource_id })
        })
        .collect()
}
/// Return exactly one argv for the requested mode. Interactive invocations
/// retain a TTY; noninteractive invocations always disable it (`-T`).
pub fn compose_exec_argv(
    prefix: &[String],
    service: &str,
    invocation: &[String],
    interactive: bool,
) -> Vec<String> {
    let mut argv = prefix.to_vec();
    argv.push("exec".to_string());
    argv.push(if interactive {
        "--interactive".to_string()
    } else {
        "-T".to_string()
    });
    argv.push(service.to_string());
    if invocation.is_empty() {
        argv.push("sh".to_string());
    } else {
        argv.extend(invocation.iter().cloned());
    }
    argv
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
                compose_service(
                    service,
                    &format!("{}@{}", fleet.spec.image.reference, fleet.spec.image.digest),
                    &labels,
                ),
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
        artifacts: RenderedDeploymentArtifacts::Compose {
            compose_yaml: serde_yaml::to_string(&compose).map_err(serialization_error)?,
            devcontainer_json: serde_json::to_string_pretty(&devcontainer)
                .map_err(serialization_error)?,
            kubernetes_yaml: None,
            kubernetes_json: None,
        },
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
    use std::sync::Mutex;
    use tempfile::tempdir;

    #[derive(Default)]
    struct FakeExecutor {
        outputs: Mutex<Vec<CommandOutput>>,
        commands: Mutex<Vec<Vec<String>>>,
    }
    impl FakeExecutor {
        fn with(outputs: Vec<CommandOutput>) -> Self {
            Self {
                outputs: Mutex::new(outputs),
                commands: Mutex::new(vec![]),
            }
        }
        fn commands(&self) -> Vec<Vec<String>> {
            self.commands.lock().unwrap().clone()
        }
    }
    impl ComposeExecutor for FakeExecutor {
        fn run(&self, argv: &[String], _cwd: &Path) -> Result<CommandOutput, BackendError> {
            self.commands.lock().unwrap().push(argv.to_vec());
            Ok(self.outputs.lock().unwrap().remove(0))
        }
    }

    fn output(success: bool, stdout: &str) -> CommandOutput {
        CommandOutput {
            success,
            stdout: stdout.to_string(),
            stderr: "runtime failed".to_string(),
        }
    }
    fn plan() -> DesiredDeploymentPlan {
        let mut fleet: WorkspaceFleetSpec = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/workspace-fleet-spec.json"
        ))
        .unwrap();
        fleet.spec.image.digest = format!("sha256:{}", "a".repeat(64));
        let target: DeploymentTarget = serde_json::from_str(include_str!(
            "../contracts/v1alpha1/fixtures/valid/deployment-target.json"
        ))
        .unwrap();
        compile(CompileRequest {
            image: None,
            fleet,
            target,
            image_build: ImageBuildIntent::Disabled,
        })
        .unwrap()
    }
    fn owned_ps(rendered: &RenderedDeploymentPlan) -> String {
        serde_json::json!([{"Service":"workspace", "State":"running", "ID":"abc", "Labels": rendered.ownership_labels}]).to_string()
    }

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
        let (compose_yaml, _) = first.artifacts.compose().unwrap();
        assert_eq!(first, render(&plan).unwrap());
        assert_eq!(
            compose_yaml,
            include_str!("../contracts/v1alpha1/fixtures/valid/compose-plan.yaml")
        );
        assert!(compose_yaml.contains(LABEL_DEPLOYMENT_ID));
        assert!(compose_yaml.contains("${AIBOX_REGISTRY_TOKEN}"));
        assert!(!compose_yaml.contains("secret-token"));
        assert!(compose_yaml.contains("@sha256:"));
    }

    #[test]
    fn apply_is_idempotent_and_persists_an_atomic_record() {
        let plan = plan();
        let rendered = render(&plan).unwrap();
        let root = tempdir().unwrap();
        let fake = Arc::new(FakeExecutor::with(vec![
            output(true, ""),
            output(true, &owned_ps(&rendered)),
        ]));
        let backend = ComposeBackend::new(root.path().to_path_buf(), fake.clone());
        let first = backend.apply(ApplyRequest { plan: plan.clone() }).unwrap();
        let second = backend.apply(ApplyRequest { plan }).unwrap();
        assert_eq!(first.record, second.record);
        assert_eq!(
            fake.commands().len(),
            2,
            "second unchanged apply must not touch Compose"
        );
        let store = backend.store();
        assert!(store.record_path(&rendered.deployment_id).is_file());
        assert!(store.load(&rendered.deployment_id).unwrap().is_some());
        assert!(
            !store
                .root
                .join(format!(
                    "{}.tmp-{}",
                    rendered.deployment_id,
                    std::process::id()
                ))
                .exists()
        );
    }

    #[test]
    fn failed_apply_leaves_recoverable_unavailable_record() {
        let plan = plan();
        let rendered = render(&plan).unwrap();
        let root = tempdir().unwrap();
        let fake = Arc::new(FakeExecutor::with(vec![output(false, "")]));
        let backend = ComposeBackend::new(root.path().to_path_buf(), fake);
        assert_eq!(
            backend.apply(ApplyRequest { plan }).unwrap_err().code,
            ContractErrorCode::Mutation
        );
        assert_eq!(
            backend
                .store()
                .load(&rendered.deployment_id)
                .unwrap()
                .unwrap()
                .spec
                .status,
            DeploymentStatus::Unavailable
        );
    }

    #[test]
    fn concurrent_mutation_is_refused() {
        let plan = plan();
        let rendered = render(&plan).unwrap();
        let root = tempdir().unwrap();
        let backend =
            ComposeBackend::new(root.path().to_path_buf(), Arc::new(FakeExecutor::default()));
        let _lock = backend.store().lock(&rendered.deployment_id).unwrap();
        let error = backend.apply(ApplyRequest { plan }).unwrap_err();
        assert_eq!(error.code, ContractErrorCode::Mutation);
        assert!(error.message.contains("already in progress"));
    }

    #[test]
    fn destroy_refuses_every_ownership_mismatch_and_is_idempotent_after_success() {
        for bad_label in [
            LABEL_DEPLOYMENT_ID,
            LABEL_SPEC_DIGEST,
            LABEL_IMAGE_DIGEST,
            "unlabelled",
        ] {
            let plan = plan();
            let rendered = render(&plan).unwrap();
            let root = tempdir().unwrap();
            let mut labels = rendered.ownership_labels.clone();
            if bad_label == "unlabelled" {
                labels.clear();
            } else {
                labels.insert(bad_label.to_string(), "foreign".to_string());
            }
            let bad_ps =
                serde_json::json!([{"Service":"workspace", "State":"running", "Labels": labels}])
                    .to_string();
            let fake = Arc::new(FakeExecutor::with(vec![
                output(true, ""),
                output(true, &owned_ps(&rendered)),
                output(true, &bad_ps),
            ]));
            let backend = ComposeBackend::new(root.path().to_path_buf(), fake.clone());
            backend.apply(ApplyRequest { plan }).unwrap();
            let error = backend
                .destroy(DestroyRequest {
                    deployment_id: rendered.deployment_id.clone(),
                })
                .unwrap_err();
            assert_eq!(error.code, ContractErrorCode::Ownership, "{bad_label}");
            assert_eq!(
                fake.commands().len(),
                3,
                "{bad_label} must not run compose down"
            );
        }

        let plan = plan();
        let rendered = render(&plan).unwrap();
        let root = tempdir().unwrap();
        let fake = Arc::new(FakeExecutor::with(vec![
            output(true, ""),
            output(true, &owned_ps(&rendered)),
            output(true, &owned_ps(&rendered)),
            output(true, ""),
        ]));
        let backend = ComposeBackend::new(root.path().to_path_buf(), fake.clone());
        backend.apply(ApplyRequest { plan }).unwrap();
        assert_eq!(
            backend
                .destroy(DestroyRequest {
                    deployment_id: rendered.deployment_id.clone()
                })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Destroyed
        );
        assert_eq!(
            backend
                .destroy(DestroyRequest {
                    deployment_id: rendered.deployment_id.clone()
                })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Destroyed
        );
        assert_eq!(
            fake.commands().len(),
            4,
            "repeated destroy must not call runtime"
        );
    }

    #[test]
    fn status_classifies_degraded_and_unavailable_without_secrets() {
        let plan = plan();
        let rendered = render(&plan).unwrap();
        let root = tempdir().unwrap();
        let degraded = serde_json::json!([{"Service":"workspace", "State":"exited"}]).to_string();
        let fake = Arc::new(FakeExecutor::with(vec![
            output(true, ""),
            output(true, &owned_ps(&rendered)),
            output(true, &degraded),
            output(false, ""),
        ]));
        let backend = ComposeBackend::new(root.path().to_path_buf(), fake);
        backend.apply(ApplyRequest { plan }).unwrap();
        assert_eq!(
            backend
                .status(StatusRequest {
                    deployment_id: rendered.deployment_id.clone()
                })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Degraded
        );
        assert_eq!(
            backend
                .status(StatusRequest {
                    deployment_id: rendered.deployment_id.clone()
                })
                .unwrap()
                .record
                .spec
                .status,
            DeploymentStatus::Unavailable
        );
        assert!(
            !std::fs::read_to_string(backend.store().record_path(&rendered.deployment_id))
                .unwrap()
                .contains("secret-token")
        );
    }

    #[test]
    fn compose_exec_uses_separate_tty_safe_argv() {
        let prefix = vec!["docker".to_string(), "compose".to_string()];
        assert_eq!(
            compose_exec_argv(&prefix, "workspace", &["sh".to_string()], true),
            vec![
                "docker",
                "compose",
                "exec",
                "--interactive",
                "workspace",
                "sh"
            ]
        );
        assert_eq!(
            compose_exec_argv(
                &prefix,
                "workspace",
                &["echo".to_string(), "ok".to_string()],
                false
            ),
            vec!["docker", "compose", "exec", "-T", "workspace", "echo", "ok"]
        );
    }
}
