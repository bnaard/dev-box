//! Backend-neutral v1 deployment operations.
//!
//! The interface deliberately makes planning separate from mutation.  Built-in
//! backends can be selected by contract kind without exposing runtime commands
//! to callers, and unsupported operations always have stable error codes.

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::compose_plan::{ComposeBackend, RenderedDeploymentPlan};
use crate::deployment_compiler::DesiredDeploymentPlan;
use crate::deployment_contract::{
    BackendCapability, BackendKind, ConnectionTarget, ContractErrorCode, DeploymentRecord,
};
use crate::kubernetes_plan::KubernetesBackend;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateRequest {
    pub plan: DesiredDeploymentPlan,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateResponse {
    pub valid: bool,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRequest {
    pub plan: DesiredDeploymentPlan,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanResponse {
    pub rendered: RenderedDeploymentPlan,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyRequest {
    pub plan: DesiredDeploymentPlan,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResponse {
    pub record: DeploymentRecord,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusRequest {
    pub deployment_id: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub record: DeploymentRecord,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DestroyRequest {
    pub deployment_id: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DestroyResponse {
    pub record: DeploymentRecord,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRequest {
    pub target: ConnectionTarget,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionResponse {
    pub command: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogsRequest {
    pub deployment_id: String,
    pub service: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogsResponse {
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendError {
    pub code: ContractErrorCode,
    pub message: String,
}
impl BackendError {
    pub fn unsupported(operation: &str) -> Self {
        Self {
            code: ContractErrorCode::CapabilityUnsupported,
            message: format!("backend does not support {operation}"),
        }
    }
}
impl Display for BackendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}
impl std::error::Error for BackendError {}

pub trait Backend: Send + Sync {
    fn kind(&self) -> BackendKind;
    fn capabilities(&self) -> Vec<BackendCapability>;
    fn validate(&self, request: ValidateRequest) -> Result<ValidateResponse, BackendError>;
    fn plan(&self, request: PlanRequest) -> Result<PlanResponse, BackendError>;
    fn apply(&self, _request: ApplyRequest) -> Result<ApplyResponse, BackendError> {
        Err(BackendError::unsupported("apply"))
    }
    fn status(&self, _request: StatusRequest) -> Result<StatusResponse, BackendError> {
        Err(BackendError::unsupported("status"))
    }
    fn destroy(&self, _request: DestroyRequest) -> Result<DestroyResponse, BackendError> {
        Err(BackendError::unsupported("destroy"))
    }
    fn connection(&self, _request: ConnectionRequest) -> Result<ConnectionResponse, BackendError> {
        Err(BackendError::unsupported("connection"))
    }
    fn logs(&self, _request: LogsRequest) -> Result<LogsResponse, BackendError> {
        Err(BackendError::unsupported("logs"))
    }
}

/// Built-in backend registry. No plugin loading or runtime discovery occurs here.
pub struct BackendRegistry {
    backends: Vec<Box<dyn Backend>>,
}
impl BackendRegistry {
    pub fn built_in() -> Self {
        Self {
            backends: vec![
                Box::new(ComposeBackend::for_current_dir()),
                Box::new(KubernetesBackend::for_project(
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
                )),
            ],
        }
    }

    /// Construct the built-ins against an explicit project root.  This is
    /// primarily useful for callers and tests that must not inherit the
    /// process current directory.
    pub fn built_in_for(project_dir: std::path::PathBuf) -> Self {
        Self {
            backends: vec![
                Box::new(ComposeBackend::for_project(project_dir.clone())),
                Box::new(KubernetesBackend::for_project(project_dir)),
            ],
        }
    }
    pub fn get(&self, kind: &BackendKind) -> Result<&dyn Backend, BackendError> {
        self.backends
            .iter()
            .find(|backend| backend.kind() == *kind)
            .map(Box::as_ref)
            .ok_or_else(|| BackendError {
                code: ContractErrorCode::CapabilityUnsupported,
                message: "no built-in backend is registered for requested target".to_string(),
            })
    }
}

pub fn preflight(backend: &dyn Backend, capability: BackendCapability) -> Result<(), BackendError> {
    if backend.capabilities().contains(&capability) {
        Ok(())
    } else {
        Err(BackendError::unsupported(match capability {
            BackendCapability::Validate => "validate",
            BackendCapability::Plan => "plan",
            BackendCapability::Apply => "apply",
            BackendCapability::Status => "status",
            BackendCapability::Destroy => "destroy",
            BackendCapability::Logs => "logs",
            BackendCapability::Exec => "connection",
            BackendCapability::PortForward => "port-forward",
            BackendCapability::ReconcileIngress => "reconcile-ingress",
            BackendCapability::ReconcileDns => "reconcile-dns",
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deployment_contract::BackendCapability;

    #[test]
    fn compose_is_registered_for_lifecycle_operations() {
        let registry = BackendRegistry::built_in();
        let backend = registry.get(&BackendKind::Compose).unwrap();
        assert!(preflight(backend, BackendCapability::Plan).is_ok());
        assert!(preflight(backend, BackendCapability::Apply).is_ok());
    }

    #[test]
    fn kubernetes_is_registered_for_lifecycle_operations() {
        let registry = BackendRegistry::built_in();
        let backend = registry.get(&BackendKind::Kubernetes).unwrap();
        assert!(preflight(backend, BackendCapability::Plan).is_ok());
        assert!(preflight(backend, BackendCapability::Apply).is_ok());
    }

    #[test]
    fn untracked_destroy_has_stable_ownership_error_code() {
        let registry = BackendRegistry::built_in();
        let backend = registry.get(&BackendKind::Compose).unwrap();
        let error = backend
            .destroy(DestroyRequest {
                deployment_id: "d".to_string(),
            })
            .unwrap_err();
        assert_eq!(error.code, ContractErrorCode::Ownership);
    }
}
