---
name: kubernetes-basics
description: Kubernetes cluster management, resource definitions, networking, storage, Helm, and troubleshooting. Use when working with Kubernetes manifests, kubectl commands, Helm charts, or debugging pod/service issues.
allowed-tools: Bash(kubectl:*) Bash(helm:*) Bash(k9s:*) Read Write
---

# Kubernetes Basics

## When to Use

- Writing or editing Kubernetes YAML manifests
- Running kubectl or helm commands
- Debugging pods, services, networking, or storage
- Setting up ingress, DNS, or network policies
- Managing ConfigMaps, Secrets, or PVCs
- Creating or modifying Helm charts

## Instructions

### Cluster Context

Always confirm the active cluster and namespace before making changes:

```bash
kubectl config current-context
kubectl config get-contexts
kubectl get ns
```

Switch context/namespace explicitly rather than relying on defaults:

```bash
kubectl config use-context <name>
kubectl config set-context --current --namespace=<ns>
```

### Core Resources

Use Deployments for stateless workloads, StatefulSets for stateful ones. Never create bare Pods in production -- they are not rescheduled on failure.

- **Pod** -- smallest deployable unit, one or more containers
- **Deployment** -- manages ReplicaSets, handles rolling updates
- **StatefulSet** -- stable network identity and persistent storage per replica
- **DaemonSet** -- one pod per node (logging, monitoring agents)
- **Job / CronJob** -- run-to-completion and scheduled tasks

See `references/resource-cheatsheet.md` for YAML patterns.

### Networking

Services expose pods internally or externally:

- **ClusterIP** (default) -- internal only
- **NodePort** -- exposes on each node's IP at a static port
- **LoadBalancer** -- provisions external LB (cloud providers)

Ingress routes external HTTP/HTTPS to services by host/path. Always set `ingressClassName`. Pods resolve services via DNS: `<svc>.<ns>.svc.cluster.local`.

Network Policies default to allow-all. Define explicit ingress/egress rules to restrict traffic between namespaces or pods.

### Storage

- **PersistentVolume (PV)** -- cluster-level storage resource
- **PersistentVolumeClaim (PVC)** -- namespace-scoped request for storage
- **StorageClass** -- dynamic provisioning template

Use `storageClassName` in PVCs. For StatefulSets, use `volumeClaimTemplates` to create per-replica PVCs automatically. Always set `accessModes` and `resources.requests.storage`.

### Configuration

- **ConfigMap** -- non-sensitive key-value config, mount as files or env vars
- **Secret** -- base64-encoded sensitive data, same mount options

Prefer mounting as volumes over env vars for config files. Use `kubectl create secret generic` for quick creation. In production, use an external secrets operator or sealed-secrets.

### Helm

Helm manages templated releases:

```bash
helm repo add <name> <url>
helm repo update
helm search repo <chart>
helm install <release> <chart> -n <ns> --create-namespace -f values.yaml
helm upgrade <release> <chart> -n <ns> -f values.yaml
helm rollback <release> <revision> -n <ns>
helm list -n <ns>
helm uninstall <release> -n <ns>
```

Inspect before installing: `helm template` to render, `helm show values` for defaults. Pin chart versions in CI with `--version`.

### Troubleshooting

Follow this order when debugging:

1. `kubectl get events --sort-by=.lastTimestamp` -- cluster-level events
2. `kubectl describe pod <pod>` -- scheduling, image pull, probe failures
3. `kubectl logs <pod> [-c container] [--previous]` -- app logs
4. `kubectl exec -it <pod> -- sh` -- interactive shell
5. `kubectl top pod` / `kubectl top node` -- resource usage

See `references/troubleshooting.md` for specific failure patterns.

### Applying Changes Safely

Always use `kubectl diff` before `kubectl apply` to preview changes. Prefer declarative `apply` over imperative `create`/`edit`. Use `--dry-run=client -o yaml` to validate manifests without submitting.

For production changes, use `kubectl rollout status` to watch deployments and `kubectl rollout undo` if something goes wrong.

## Examples

### Deploy a web app with service and ingress

```bash
kubectl create namespace myapp
kubectl apply -f deployment.yaml -n myapp
kubectl apply -f service.yaml -n myapp
kubectl apply -f ingress.yaml -n myapp
kubectl rollout status deployment/myapp -n myapp
```

### Debug a pod stuck in CrashLoopBackOff

```bash
kubectl describe pod <pod> -n <ns>        # check events for OOM, probe failures
kubectl logs <pod> -n <ns> --previous     # logs from the crashed container
kubectl get pod <pod> -n <ns> -o yaml     # inspect resource limits, commands
```

### Helm upgrade with rollback safety

```bash
helm upgrade myrelease mychart/app -n prod -f prod-values.yaml --atomic --timeout 5m
# --atomic auto-rolls back on failure
helm history myrelease -n prod
```
