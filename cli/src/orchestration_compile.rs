//! Adapter from validated `aibox.toml` orchestration intent to canonical v1 contracts.
//!
//! This module contains no backend discovery or mutation. It makes the config/compiler
//! boundary explicit and keeps apply-time image builds disabled.

use anyhow::{Context, Result};

use crate::cli::{CompileOutputFormat, DeployPlanOutputFormat};
use crate::config::{
    AiboxConfig, CredentialReferenceKind as ConfigCredentialKind, CredentialReferenceSection,
    OrchestrationBackend, OrchestrationPortProtocol,
};
use crate::deployment_backend::{
    ApplyRequest, BackendRegistry, ConnectionRequest, DestroyRequest, LogsRequest, PlanRequest,
    StatusRequest, preflight,
};
use crate::deployment_compiler::{
    CompileRequest, DesiredDeploymentPlan, ImageBuildIntent, compile,
};
use crate::deployment_contract::BackendCapability;
use crate::deployment_contract::{
    ApiVersion, BackendKind, ConnectionTarget, ConnectionTargetKind, ConnectionTargetSpec,
    ConnectionTransport, CredentialReference, CredentialReferenceKind, DeploymentTarget,
    DeploymentTargetKind, DeploymentTargetSpec, EnvironmentReference, ImmutableImageReference,
    KubernetesReconciliationIntent, ObjectMeta, OwnershipReference, PortProtocol, PortSpec,
    WorkspaceFleetSpec, WorkspaceFleetSpecBody, WorkspaceFleetSpecKind, WorkspaceService,
};

/// Print a deterministic deployment plan without performing discovery or mutation.
pub fn cmd_config_compile(config_path: &Option<String>, format: CompileOutputFormat) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let plan = compile_config(&config)?;

    match format {
        CompileOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        CompileOutputFormat::Human => {
            let backend = match plan.target.backend {
                BackendKind::Compose => "compose",
                BackendKind::Kubernetes => "kubernetes",
            };
            let image_build = plan.actions.iter().any(|action| {
                matches!(
                    action,
                    crate::deployment_compiler::DesiredDeploymentAction::BuildImage { .. }
                )
            });
            println!("Deployment plan");
            println!("  target: {backend}:{}", plan.target.target_ref);
            println!("  scope: {}", plan.target.scope);
            println!("  desired spec digest: {}", plan.desired_spec_digest);
            println!(
                "  image build: {}",
                if image_build {
                    "explicitly enabled"
                } else {
                    "disabled"
                }
            );
            println!("  actions: {}", plan.actions.len());
        }
    }
    Ok(())
}

/// Render a backend-specific deployment plan without runtime discovery, file writes, or mutation.
pub fn cmd_deploy_plan(config_path: &Option<String>, format: DeployPlanOutputFormat) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let plan = compile_config(&config)?;
    let registry = BackendRegistry::built_in();
    let backend = registry
        .get(&plan.target.backend)
        .map_err(anyhow::Error::msg)?;
    preflight(backend, BackendCapability::Plan).map_err(anyhow::Error::msg)?;
    let rendered = backend
        .plan(PlanRequest { plan })
        .map_err(anyhow::Error::msg)?
        .rendered;
    match format {
        DeployPlanOutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rendered)?),
        DeployPlanOutputFormat::Human => {
            println!("Deployment plan");
            println!(
                "  backend: {}",
                match rendered.backend {
                    BackendKind::Compose => "compose",
                    BackendKind::Kubernetes => "kubernetes",
                }
            );
            println!("  deployment id: {}", rendered.deployment_id);
            println!("  desired spec digest: {}", rendered.desired_spec_digest);
            println!("  image digest: {}", rendered.image_digest);
            match rendered.backend {
                BackendKind::Compose => {
                    println!("  artifacts: docker-compose.yml, devcontainer.json")
                }
                BackendKind::Kubernetes => {
                    println!("  artifacts: kubernetes.yaml, kubernetes.json")
                }
            }
        }
    }
    Ok(())
}

pub fn cmd_deploy_apply(config_path: &Option<String>) -> Result<()> {
    let plan = load_plan(config_path)?;
    let registry = BackendRegistry::built_in();
    let backend = registry
        .get(&plan.target.backend)
        .map_err(anyhow::Error::msg)?;
    preflight(backend, BackendCapability::Apply).map_err(anyhow::Error::msg)?;
    let record = backend
        .apply(ApplyRequest { plan })
        .map_err(anyhow::Error::msg)?
        .record;
    println!(
        "deployment {}: {:?}",
        record.spec.deployment_id, record.spec.status
    );
    Ok(())
}

pub fn cmd_deploy_status(config_path: &Option<String>) -> Result<()> {
    let (plan, deployment_id) = plan_and_id(config_path)?;
    let registry = BackendRegistry::built_in();
    let backend = registry
        .get(&plan.target.backend)
        .map_err(anyhow::Error::msg)?;
    preflight(backend, BackendCapability::Status).map_err(anyhow::Error::msg)?;
    let record = backend
        .status(StatusRequest { deployment_id })
        .map_err(anyhow::Error::msg)?
        .record;
    println!(
        "deployment {}: {:?}",
        record.spec.deployment_id, record.spec.status
    );
    Ok(())
}

pub fn cmd_deploy_destroy(config_path: &Option<String>) -> Result<()> {
    let (plan, deployment_id) = plan_and_id(config_path)?;
    let registry = BackendRegistry::built_in();
    let backend = registry
        .get(&plan.target.backend)
        .map_err(anyhow::Error::msg)?;
    preflight(backend, BackendCapability::Destroy).map_err(anyhow::Error::msg)?;
    let record = backend
        .destroy(DestroyRequest { deployment_id })
        .map_err(anyhow::Error::msg)?
        .record;
    println!(
        "deployment {}: {:?}",
        record.spec.deployment_id, record.spec.status
    );
    Ok(())
}

pub fn cmd_deploy_logs(config_path: &Option<String>, service: Option<String>) -> Result<()> {
    let (plan, deployment_id) = plan_and_id(config_path)?;
    let registry = BackendRegistry::built_in();
    let backend = registry
        .get(&plan.target.backend)
        .map_err(anyhow::Error::msg)?;
    preflight(backend, BackendCapability::Logs).map_err(anyhow::Error::msg)?;
    for line in backend
        .logs(LogsRequest {
            deployment_id,
            service,
        })
        .map_err(anyhow::Error::msg)?
        .lines
    {
        println!("{line}");
    }
    Ok(())
}

pub fn cmd_connect(config_path: &Option<String>, name: &str, command: Vec<String>) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let plan = compile_config(&config)?;
    let rendered = BackendRegistry::built_in()
        .get(&plan.target.backend)
        .map_err(anyhow::Error::msg)?
        .plan(PlanRequest { plan: plan.clone() })
        .map_err(anyhow::Error::msg)?
        .rendered;
    let connection = config
        .orchestration
        .connections
        .iter()
        .find(|connection| connection.name == name)
        .context("orchestration connection not found")?;
    let endpoint = match &connection.endpoint {
        Some(endpoint) => endpoint.clone(),
        None => default_connection_endpoint(&config.orchestration, &plan, connection)?,
    };
    let target = ConnectionTarget {
        api_version: ApiVersion::V1Alpha1,
        kind: ConnectionTargetKind::V1Alpha1,
        metadata: ObjectMeta {
            name: connection.name.clone(),
            owner: OwnershipReference {
                owner_id: config
                    .orchestration
                    .deployment
                    .as_ref()
                    .context("orchestration.deployment is required")?
                    .owner_id
                    .clone(),
            },
            labels: Default::default(),
        },
        spec: ConnectionTargetSpec {
            deployment_id: rendered.deployment_id,
            service: connection.service.clone(),
            transport: match connection.transport {
                crate::config::ConnectionTransport::ComposeExec => ConnectionTransport::ComposeExec,
                crate::config::ConnectionTransport::KubernetesExec => {
                    ConnectionTransport::KubernetesExec
                }
                crate::config::ConnectionTransport::KubernetesPortForward => {
                    ConnectionTransport::KubernetesPortForward
                }
                crate::config::ConnectionTransport::Ssh => ConnectionTransport::Ssh,
            },
            interactive: connection.interactive,
            endpoint,
            invocation: if command.is_empty() {
                connection.invocation.clone()
            } else {
                command
            },
            credentials: connection
                .credentials
                .iter()
                .map(credential_reference)
                .collect(),
        },
    };
    let registry = BackendRegistry::built_in();
    let backend = registry
        .get(&plan.target.backend)
        .map_err(anyhow::Error::msg)?;
    let capability = match target.spec.transport {
        ConnectionTransport::KubernetesPortForward => BackendCapability::PortForward,
        _ => BackendCapability::Exec,
    };
    preflight(backend, capability).map_err(anyhow::Error::msg)?;
    let argv = backend
        .connection(ConnectionRequest { target })
        .map_err(anyhow::Error::msg)?
        .command;
    let (program, args) = argv.split_first().context("empty connection command")?;
    let status = std::process::Command::new(program)
        .args(args)
        .status()
        .context("could not start connection")?;
    if status.success() {
        return Ok(());
    }
    // `kubectl exec` reports the remote command's code.  Do not flatten it
    // to aibox's generic error status: callers need it for scripts.  A
    // cancelled port-forward has no numeric status; its signal is preserved
    // by the terminal/process group and is represented as a generic failure
    // only if this process remains alive to observe it.
    std::process::exit(status.code().unwrap_or(1));
}

fn default_connection_endpoint(
    orchestration: &crate::config::OrchestrationSection,
    plan: &DesiredDeploymentPlan,
    connection: &crate::config::ConnectionIntentSection,
) -> Result<String> {
    match connection.transport {
        crate::config::ConnectionTransport::KubernetesExec => {
            let context = plan
                .target
                .target_ref
                .strip_prefix("kube-context:")
                .context("Kubernetes target must use kube-context:<context>")?;
            Ok(format!(
                "kubernetes://{context}/{}/{}",
                plan.target.scope, connection.service
            ))
        }
        crate::config::ConnectionTransport::KubernetesPortForward => {
            let context = plan
                .target
                .target_ref
                .strip_prefix("kube-context:")
                .context("Kubernetes target must use kube-context:<context>")?;
            let service = orchestration
                .fleet
                .as_ref()
                .and_then(|fleet| {
                    fleet
                        .services
                        .iter()
                        .find(|service| service.name == connection.service)
                })
                .context("Kubernetes port-forward service is missing from orchestration.fleet")?;
            let port = service
                .ports
                .iter()
                .find(|port| port.protocol == crate::config::OrchestrationPortProtocol::Tcp)
                .context("Kubernetes port-forward requires a TCP port on its service")?;
            let local_port = port.host_port.unwrap_or(port.container_port);
            Ok(format!(
                "kubernetes-port-forward://{context}/{}/{}/127.0.0.1/{local_port}:{}",
                plan.target.scope, connection.service, port.container_port
            ))
        }
        _ => Ok(format!(
            "compose://{}/{}",
            plan.target.target_ref, connection.service
        )),
    }
}

fn load_plan(config_path: &Option<String>) -> Result<DesiredDeploymentPlan> {
    compile_config(&AiboxConfig::from_cli_option(config_path)?)
}
fn plan_and_id(config_path: &Option<String>) -> Result<(DesiredDeploymentPlan, String)> {
    let plan = load_plan(config_path)?;
    let rendered = BackendRegistry::built_in()
        .get(&plan.target.backend)
        .map_err(anyhow::Error::msg)?
        .plan(PlanRequest { plan: plan.clone() })
        .map_err(anyhow::Error::msg)?
        .rendered;
    Ok((plan, rendered.deployment_id))
}

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
            kubernetes: (target.backend == OrchestrationBackend::Kubernetes).then(|| {
                KubernetesReconciliationIntent {
                    ingress_class: target.ingress_class.clone(),
                    gateway_class: target.gateway_class.clone(),
                    dns_zone: target.dns_zone.clone(),
                    dns_credentials: target
                        .dns_credentials
                        .iter()
                        .map(credential_reference)
                        .collect(),
                }
            }),
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

    #[test]
    fn kubernetes_connection_defaults_are_typed_endpoints() {
        let config_text = format!(
            "{}{}",
            VALID_CONFIG
                .replace("backend = \"compose\"", "backend = \"kubernetes\"")
                .replace(
                    "reference = \"docker-context:default\"",
                    "reference = \"kube-context:staging\"",
                ),
            r#"

[[orchestration.connections]]
name = "shell"
service = "web"
transport = "kubernetes-exec"
interactive = true

[[orchestration.connections]]
name = "web-forward"
service = "web"
transport = "kubernetes-port-forward"
"#
        );
        let config = AiboxConfig::from_str(&config_text).unwrap();
        let plan = compile_config(&config).unwrap();
        assert_eq!(
            default_connection_endpoint(
                &config.orchestration,
                &plan,
                &config.orchestration.connections[0]
            )
            .unwrap(),
            "kubernetes://staging/workspace/web"
        );
        assert_eq!(
            default_connection_endpoint(
                &config.orchestration,
                &plan,
                &config.orchestration.connections[1]
            )
            .unwrap(),
            "kubernetes-port-forward://staging/workspace/web/127.0.0.1/18080:8080"
        );
    }
}
