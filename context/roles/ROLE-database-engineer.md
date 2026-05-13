---
apiVersion: processkit.projectious.work/v2
kind: Role
metadata:
  id: ROLE-database-engineer
  created: 2026-04-22 00:00:00+00:00
spec:
  name: database-engineer
  description: Designs schemas, operates databases, and optimises data-path performance.
  responsibilities:
  - Author schemas and migrations with rollback plans
  - Tune queries, indexes, and replication
  - Operate HA/DR and backup strategies
  - Investigate and fix data-path incidents
  skills_required:
  - sql
  - schema-design
  - query-optimization
  - replication
  - backup-restore
  default_scope: permanent
  default_seniority: senior
  function_group: platform-infra
---
