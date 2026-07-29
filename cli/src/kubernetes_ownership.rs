//! Kubernetes ownership metadata, reconstruction, and destructive preflight.
//!
//! This module is deliberately separate from rendering and lifecycle
//! orchestration. Every destructive caller must complete these read-only
//! checks before its first target mutation.

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use crate::deployment_backend::BackendError;
use crate::deployment_contract::{BackendKind, ContractErrorCode, DeploymentRecord};
use crate::kubernetes_plan::{KubernetesClient, KubernetesResource, KubernetesResourceKind};

pub(crate) const LABEL_DEPLOYMENT_ID: &str = "aibox.projectious.work/deployment-id";
pub(crate) const LABEL_SPEC_DIGEST: &str = "aibox.projectious.work/desired-spec-digest";
pub(crate) const LABEL_IMAGE_DIGEST: &str = "aibox.projectious.work/image-digest";
pub(crate) const LABEL_NAMESPACE: &str = "aibox.projectious.work/namespace";
pub(crate) const LABEL_OWNER_ID: &str = "aibox.projectious.work/owner-id";
pub(crate) const ANNOTATION_RECORD: &str = "aibox.projectious.work/deployment-record";
pub(crate) const ANNOTATION_RECORD_DIGEST: &str = "aibox.projectious.work/deployment-record-digest";

pub(crate) fn record_annotations(
    record: &DeploymentRecord,
) -> Result<BTreeMap<String, String>, BackendError> {
    let body = serde_json::to_string(record).map_err(|error| BackendError {
        code: ContractErrorCode::Planning,
        message: format!("could not render Kubernetes plan: {error}"),
    })?;
    let mut annotations = BTreeMap::new();
    annotations.insert(ANNOTATION_RECORD.to_string(), body.clone());
    annotations.insert(
        ANNOTATION_RECORD_DIGEST.to_string(),
        sha256_digest(body.as_bytes()),
    );
    Ok(annotations)
}

pub(crate) fn reconstruct(
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

pub(crate) fn assert_owned(
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

pub(crate) fn discover_destroy_workloads(
    client: &dyn KubernetesClient,
    record: &DeploymentRecord,
) -> Result<Vec<KubernetesResource>, BackendError> {
    let service_names = record
        .spec
        .services
        .iter()
        .map(|service| service.name.as_str())
        .collect::<BTreeSet<_>>();
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
    assert_owned(record, &resources)?;
    let deployments = resources
        .iter()
        .filter(|resource| resource.key.kind == KubernetesResourceKind::Deployment)
        .map(|resource| resource.key.name.as_str())
        .collect::<BTreeSet<_>>();
    if deployments != service_names {
        return Err(ownership_error(
            "refusing destroy: recorded Kubernetes workload identities are incomplete",
        ));
    }
    Ok(resources)
}

pub(crate) fn verify_destroy_workloads_absent(
    client: &dyn KubernetesClient,
    record: &DeploymentRecord,
    planned: &[KubernetesResource],
) -> Result<(), BackendError> {
    let planned_keys = planned
        .iter()
        .map(|resource| &resource.key)
        .collect::<BTreeSet<_>>();
    let remaining = client
        .list_namespace(&record.spec.target.scope)?
        .into_iter()
        .any(|resource| planned_keys.contains(&resource.key));
    if remaining {
        return Err(BackendError {
            code: ContractErrorCode::Mutation,
            message: "Kubernetes workload resource remained after destroy".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn ownership_error(message: &str) -> BackendError {
    BackendError {
        code: ContractErrorCode::Ownership,
        message: message.to_string(),
    }
}

pub(crate) fn digest_label_value(digest: &str) -> String {
    let canonical_hex = digest
        .strip_prefix("sha256:")
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| {
            Sha256::digest(digest.as_bytes())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect()
        });
    format!("s256-{}", &canonical_hex[..58])
}

fn sha256_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{hex}")
}
