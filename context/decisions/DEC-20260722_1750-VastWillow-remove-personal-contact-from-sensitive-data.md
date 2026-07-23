---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260722_1750-VastWillow-remove-personal-contact-from-sensitive-data
  created: '2026-07-22T17:50:49+00:00'
  updated: '2026-07-22T17:52:55+00:00'
spec:
  title: Remove personal contact from sensitive-data allowlist example
  state: accepted
  decision: Remove the confirmed personal contact address from the pk-doctor sensitive-data
    allowlist example and retain the synthetic local-domain example.
  context: Redacted successor to an earlier decision record that inadvertently repeated
    the address.
  rationale: The owner confirmed the contact must be redacted. Synthetic examples
    and non-phone numeric literals remain unchanged.
  decided_at: '2026-07-22T17:50:49+00:00'
  supersedes: DEC-20260722_1744-SnappyHare-remove-personal-contact-from-sensitive-data
---
