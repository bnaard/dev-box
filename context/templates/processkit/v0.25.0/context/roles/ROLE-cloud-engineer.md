---
apiVersion: processkit.projectious.work/v2
kind: Role
metadata:
  id: ROLE-cloud-engineer
  created: 2026-04-22 00:00:00+00:00
spec:
  name: cloud-engineer
  description: Builds and operates cloud-native infrastructure (IaC, networking, identity).
  responsibilities:
  - Design cloud topologies aligned to cost and reliability goals
  - Own IaC modules and promotion pipelines
  - Implement networking, IAM, and security controls
  - Tune cloud costs alongside capacity
  skills_required:
  - aws-gcp-azure
  - iac-terraform
  - cloud-networking
  - iam
  - cost-optimization
  default_scope: permanent
  default_seniority: senior
  function_group: platform-infra
---
