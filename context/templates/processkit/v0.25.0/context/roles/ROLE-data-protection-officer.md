---
apiVersion: processkit.projectious.work/v2
kind: Role
metadata:
  id: ROLE-data-protection-officer
  created: 2026-04-22 00:00:00+00:00
spec:
  name: data-protection-officer
  description: Oversees data privacy compliance (GDPR, CCPA, and analogues).
  responsibilities:
  - Maintain the privacy programme and RoPA
  - Handle DSARs and privacy incidents
  - Run DPIAs and advise on data-processing changes
  - Manage regulator and supervisory authority contact
  skills_required:
  - privacy-law
  - gdpr-ccpa
  - dpia
  - incident-response
  - vendor-dpa-review
  default_scope: permanent
  default_seniority: senior
  function_group: legal-compliance
---
