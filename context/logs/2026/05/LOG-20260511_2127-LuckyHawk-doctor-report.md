---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260511_2127-LuckyHawk-doctor-report
  created: '2026-05-11T21:27:14+00:00'
spec:
  event_type: doctor.report
  timestamp: '2026-05-11T21:27:14+00:00'
  summary: "/pk-doctor \u2014 134 ERROR / 132 WARN / 20 INFO"
  actor: TEAMMEMBER-20260508_2042-MigratedMember-avery
  details:
    doctor_version: 1.0.0
    invocation: /pk-doctor
    categories:
      schema_filename:
        ERROR: 100
        WARN: 123
        INFO: 1
      schema_vocabulary:
        ERROR: 34
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
        INFO: 1
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
        WARN: 9
        INFO: 0
    top_findings_summary:
    - schema_filename has 100 errors and 123 warnings, mostly legacy filename date mismatches plus invalid binding scope/note/log frontmatter
    - schema_vocabulary has 34 errors for undeclared WorkItem, LogEntry, and Migration vocabulary values
    - context_hygiene has 9 warnings for demoted content, mixed binding filename styles, suspicious _0000 artifact timestamp, and applied migration archive candidates
    fixes_applied: []
    duration_ms: 31363
---
