---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260511_2130-CleverMoss-doctor-report
  created: '2026-05-11T21:30:34+00:00'
spec:
  event_type: doctor.report
  timestamp: '2026-05-11T21:30:34+00:00'
  summary: "/pk-doctor \u2014 0 ERROR / 0 WARN / 78 INFO"
  actor: TEAMMEMBER-20260508_2042-MigratedMember-avery
  details:
    doctor_version: 1.0.0
    invocation: /pk-doctor
    categories:
      schema_filename:
        ERROR: 0
        WARN: 0
        INFO: 1
      schema_vocabulary:
        ERROR: 0
        WARN: 0
        INFO: 1
      v2_contracts:
        ERROR: 0
        WARN: 0
        INFO: 1
      v1_entity_drift:
        ERROR: 0
        WARN: 0
        INFO: 1
      sharding:
        ERROR: 0
        WARN: 0
        INFO: 1
      migrations:
        ERROR: 0
        WARN: 0
        INFO: 1
      migration_integrity:
        ERROR: 0
        WARN: 0
        INFO: 1
      drift:
        ERROR: 0
        WARN: 0
        INFO: 1
      team_consistency:
        ERROR: 0
        WARN: 0
        INFO: 1
      team_member_exports:
        ERROR: 0
        WARN: 0
        INFO: 1
      release_integrity:
        ERROR: 0
        WARN: 0
        INFO: 50
      commands_consistency:
        ERROR: 0
        WARN: 0
        INFO: 1
      mcp_config_drift:
        ERROR: 0
        WARN: 0
        INFO: 1
      mcp_gateway:
        ERROR: 0
        WARN: 0
        INFO: 2
      server_header_drift:
        ERROR: 0
        WARN: 0
        INFO: 1
      preauth_applied:
        ERROR: 0
        WARN: 0
        INFO: 2
      skill_dag:
        ERROR: 0
        WARN: 0
        INFO: 1
      context_consumption:
        ERROR: 0
        WARN: 0
        INFO: 1
      context_hygiene:
        ERROR: 0
        WARN: 0
        INFO: 9
    top_findings: []
    fixes_applied: []
    duration_ms: 6451
---
