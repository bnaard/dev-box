---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260724_1717-WildLynx-define-aibox-v1-deployment-state-and
  created: '2026-07-24T17:17:11+00:00'
spec:
  title: Define aibox v1 deployment state and infrastructure boundaries
  state: accepted
  decision: 'Use split DeploymentRecord representation: a complete project-local record,
    reconstructible ownership metadata on target resources, and an optional minimal
    non-secret target receipt. Support v0/v1 operational coexistence and reversible
    configuration rollback, but do not require v0 to manage v1-created deployments.
    Reconcile DNS records only in pre-existing zones and consume existing IngressClass
    or GatewayClass resources; aibox does not provision DNS zones, ingress controllers,
    or other infrastructure.'
  context: 'Final owner refinement of the aibox v1 workplan associated with GitHub
    issue #179 and discussions DISC-20260724_1653-BuoyantOtter and DISC-20260724_1705-ToughOwl.'
  rationale: The split state model permits recovery and safe ownership checks without
    introducing a controller or central database. The rollback boundary protects v0
    users without coupling maintenance-mode v0 code to v1 deployment concepts. DNS
    and ingress remain useful deployment capabilities while respecting the no-infrastructure-provisioning
    boundary.
  alternatives:
  - option: Project-local DeploymentRecord only
    reason_rejected: Insufficient recovery and multi-operator evidence if local state
      is lost.
  - option: Target-side authoritative controller/database
    reason_rejected: Adds infrastructure and operational ownership outside v1 scope.
  - option: Require v0 to manage v1 deployments
    reason_rejected: Would force v1 architecture into the patch-only v0 line.
  - option: Provision DNS zones or ingress controllers
    reason_rejected: Violates the agreed boundary that aibox deploys onto pre-existing
      infrastructure.
  consequences: V1 requires local record persistence and recovery, target ownership
    labels/annotations, reversible config migration with v0-compatible backup, explicit
    refusal by v0 to manage v1 records, typed DNS record adapters for existing zones,
    and validation of existing ingress or gateway classes before mutation.
  decided_at: '2026-07-24T17:17:11+00:00'
---
