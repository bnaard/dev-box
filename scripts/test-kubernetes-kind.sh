#!/usr/bin/env bash
# Live, release-gated M7c verification. Runs inside the SSH companion.
set -euo pipefail

: "${AIBOX_M7C_COMMIT:?missing release candidate commit}"
: "${AIBOX_M7C_BINARY_SHA256:?missing release candidate binary digest}"
: "${AIBOX_BIN:?missing aibox binary path}"
: "${AIBOX_ADDONS_DIR:?missing addon directory}"

[[ "${AIBOX_M7C_COMMIT}" =~ ^[0-9a-f]{40}$ ]] \
  || { echo "invalid release candidate commit" >&2; exit 2; }
[[ "${AIBOX_M7C_BINARY_SHA256}" =~ ^sha256:[0-9a-f]{64}$ ]] \
  || { echo "invalid release candidate binary digest" >&2; exit 2; }
[[ -f "${AIBOX_BIN}" && -x "${AIBOX_BIN}" && ! -L "${AIBOX_BIN}" ]] \
  || { echo "candidate binary must be an executable regular file" >&2; exit 2; }
actual_binary_sha256="sha256:$(sha256sum "${AIBOX_BIN}" | awk '{print $1}')"
[[ "${actual_binary_sha256}" == "${AIBOX_M7C_BINARY_SHA256}" ]] \
  || { echo "deployed candidate binary digest does not match the release candidate" >&2; exit 2; }

name="aibox-m7c-${RANDOM}-${RANDOM}"
namespace="aibox-m7c"
class="aibox-m7c-${RANDOM}"
workspace="/workspaces/${name}"
attestation="/tmp/${name}-evidence.json"
context="kind-${name}"
image_digest="sha256:208b70eefac13ee9be00e486f79c695b15cef861c680527171a27d253d834be9"
scenarios=()

cleanup() {
  kind delete cluster --name "${name}" >/dev/null 2>&1 || true
  rm -rf "${workspace}" >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

export KIND_EXPERIMENTAL_PROVIDER=podman
systemd-run --user --scope -p Delegate=yes --quiet \
  env KIND_EXPERIMENTAL_PROVIDER=podman \
  kind create cluster --name "${name}" --wait 90s
kubectl --context "${context}" get nodes
kubectl --context "${context}" create namespace "${namespace}"

# The class is a test capability fixture. aibox must only create the
# deployment-owned Ingress and must never install an ingress controller.
kubectl --context "${context}" apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: IngressClass
metadata:
  name: ${class}
spec:
  controller: example.com/aibox-m7c-fixture
EOF

mkdir -p "${workspace}"
cat > "${workspace}/aibox.toml" <<EOF
[container]
name = "m7c-kind"

[orchestration]
enabled = true

[orchestration.image]
reference = "docker.io/library/nginx"
digest = "${image_digest}"
platform = "linux-amd64"

[orchestration.fleet]
name = "workspace"
services = [{ name = "web", ports = [{ container_port = 80, host_port = 18080 }] }]

[orchestration.target]
backend = "kubernetes"
reference = "kube-context:${context}"
scope = "${namespace}"
ingress_class = "${class}"

[orchestration.deployment]
name = "m7c-kind"
owner_id = "release-candidate"

[[orchestration.connections]]
name = "shell"
service = "web"
transport = "kubernetes-exec"
interactive = false

[[orchestration.connections]]
name = "web-forward"
service = "web"
transport = "kubernetes-port-forward"
interactive = false
EOF

cd "${workspace}"
export AIBOX_ADDONS_DIR

# First, unchanged, and changed desired-state reconciliation.
"$AIBOX_BIN" deploy apply --output json > first.json
"$AIBOX_BIN" deploy apply --output json > unchanged.json
jq -e '.spec.status == "observed"' first.json unchanged.json >/dev/null
deployment_id="$(jq -r '.spec.deploymentId' first.json)"
test -n "${deployment_id}" && test "${deployment_id}" != null
kubectl --context "${context}" --namespace "${namespace}" rollout status deployment/web --timeout=120s
kubectl --context "${context}" --namespace "${namespace}" get ingress -o json |
  jq -e --arg id "${deployment_id}" '.items[] | select(.metadata.labels["aibox.projectious.work/deployment-id"] == $id)' >/dev/null
scenarios+=("first-apply")
scenarios+=("unchanged-apply")
scenarios+=("ingress")

# Observability and both typed connection paths exercise a real workload.
"$AIBOX_BIN" deploy status --output json | jq -e '.spec.status == "observed"' >/dev/null
"$AIBOX_BIN" deploy logs --service web --output json | jq -e '.lines | type == "array"' >/dev/null
scenarios+=("status-logs")
"$AIBOX_BIN" connect shell -- /bin/sh -c 'test -f /etc/nginx/nginx.conf'
"$AIBOX_BIN" connect web-forward > port-forward.log 2>&1 &
pf_pid=$!
for _ in $(seq 1 30); do
  curl -fsS http://127.0.0.1:18080/ >/dev/null && break
  sleep 1
done
curl -fsS http://127.0.0.1:18080/ | grep -qi nginx
kill "${pf_pid}" || true
wait "${pf_pid}" 2>/dev/null || true
scenarios+=("exec-port-forward")

sed -i 's/host_port = 18080/host_port = 18081/' aibox.toml
"$AIBOX_BIN" deploy apply --output json > changed.json
test "$(jq -r '.spec.deploymentId' changed.json)" = "${deployment_id}"
test "$(jq -r '.spec.desiredSpecDigest' changed.json)" != "$(jq -r '.spec.desiredSpecDigest' first.json)"
scenarios+=("changed-apply")

# Observed drift must be visible and an apply must restore the deployment.
kubectl --context "${context}" --namespace "${namespace}" delete deployment/web
"$AIBOX_BIN" deploy status --output json | jq -e '.spec.status == "degraded"' >/dev/null
"$AIBOX_BIN" deploy apply --output json > recovered.json
jq -e '.spec.status == "observed"' recovered.json >/dev/null
kubectl --context "${context}" --namespace "${namespace}" rollout status deployment/web --timeout=120s
scenarios+=("drift-recovery")

# A durable per-target lock rejects a concurrent mutation.
touch ".aibox/deployments/${deployment_id}.lock"
if "$AIBOX_BIN" deploy apply --output json > locked.json 2>&1; then
  echo "concurrent apply unexpectedly succeeded" >&2
  exit 1
fi
grep -q 'operation already in progress' locked.json
rm -f ".aibox/deployments/${deployment_id}.lock"

# An ownership spoof must be refused; the workload remains until ownership is
# restored by reconcile. Guarded destroy then removes workload and Ingress.
kubectl --context "${context}" --namespace "${namespace}" label deployment/web \
  aibox.projectious.work/desired-spec-digest=foreign --overwrite
if "$AIBOX_BIN" deploy destroy --output json > foreign-destroy.json 2>&1; then
  echo "foreign destroy unexpectedly succeeded" >&2
  exit 1
fi
grep -q 'refusing resources not owned' foreign-destroy.json
kubectl --context "${context}" --namespace "${namespace}" get deployment/web >/dev/null
scenarios+=("foreign-destroy-refusal")
"$AIBOX_BIN" deploy apply --output json > ownership-recovered.json
"$AIBOX_BIN" deploy destroy --output json | jq -e '.spec.status == "destroyed"' >/dev/null
! kubectl --context "${context}" --namespace "${namespace}" get deployment/web >/dev/null 2>&1
! kubectl --context "${context}" --namespace "${namespace}" get service/web >/dev/null 2>&1
! kubectl --context "${context}" --namespace "${namespace}" get ingress -o name | grep -q .

scenarios_json="$(printf '%s\n' "${scenarios[@]}" | jq -R . | jq -s '[.[] | {id: ., status: "passed"}]')"
jq -n \
  --arg candidateCommit "${AIBOX_M7C_COMMIT}" \
  --arg binarySha256 "${actual_binary_sha256}" \
  --arg cluster "${name}" \
  --arg command 'cargo test --features e2e --test e2e kubernetes_kind -- --ignored --nocapture --test-threads=1' \
  --arg recordedAt "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --argjson scenarios "${scenarios_json}" \
  '{"apiVersion":"aibox.projectious.work/v1alpha1","kind":"DisposableClusterEvidence","status":"passed","candidateCommit":$candidateCommit,"binarySha256":$binarySha256,"cluster":$cluster,"command":$command,"scenarios":$scenarios,"recordedAt":$recordedAt}' > "${attestation}"

# Exercise the Rust readiness parser in the disposable project before the test
# harness copies the evidence to the release checkout.  The producer supplies
# both candidate-bound values; the parser rejects any malformed attestation.
mkdir -p .aibox/release-evidence
cp "${attestation}" .aibox/release-evidence/m7c-live.json
RELEASE_CANDIDATE_SHA="${AIBOX_M7C_COMMIT}" \
  AIBOX_RELEASE_BINARY_SHA256="${actual_binary_sha256}" \
  "$AIBOX_BIN" config release-readiness --output json > readiness.json
jq -e '.ready == true and (.gates[] | select(.id == "m7c-live-disposable-cluster-evidence").status == "passed")' readiness.json >/dev/null

trap - EXIT INT TERM
cleanup
cat "${attestation}"
rm -f "${attestation}"
