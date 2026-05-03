---
apiVersion: processkit.projectious.work/v2
kind: Role
metadata:
  id: ROLE-machine-learning-engineer
  created: 2026-04-22 00:00:00+00:00
spec:
  name: machine-learning-engineer
  description: Trains, evaluates, and productionises machine-learning models at scale.
  responsibilities:
  - Build training and evaluation pipelines
  - Own model serving, versioning, and rollback
  - Monitor model quality and drift in production
  - Collaborate with research on productionisation
  skills_required:
  - ml-frameworks
  - feature-engineering
  - mlops
  - model-serving
  - distributed-training
  default_scope: permanent
  default_seniority: senior
  function_group: data-ml
---
