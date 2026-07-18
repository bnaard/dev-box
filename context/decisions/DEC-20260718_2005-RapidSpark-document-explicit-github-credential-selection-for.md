---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260718_2005-RapidSpark-document-explicit-github-credential-selection-for
  created: '2026-07-18T20:05:41+00:00'
spec:
  title: Document explicit GitHub credential selection for containers
  state: accepted
  decision: 'Document two GitHub CLI authentication patterns: (1) persistent gh auth
    login using insecure file storage under the aibox-managed persistent home, with
    GH_TOKEN and GITHUB_TOKEN absent because they take precedence; and (2) least-privilege
    PATs provisioned through .aibox-local.toml, keeping one default GH_TOKEN and selecting
    additional repository-specific tokens explicitly per gh invocation.'
  context: Derived projects may need an AI agent to act on repositories owned by different
    GitHub resource owners. A single fine-grained PAT cannot necessarily cover all
    of them, while a broad OAuth login can grant the container more of the human user's
    authority than intended.
  rationale: Explicit token selection preserves persistent, non-interactive authentication
    while allowing the human owner to grant each container and AI agent only the repository
    permissions it needs. The OAuth login option remains documented for users who
    accept its broader authorization.
  consequences: Documentation must explain persistence through .aibox-home, environment-variable
    precedence, plaintext credential implications, generated env_file behavior, and
    explicit commands such as GH_TOKEN="$PROJECT_ISSUES_TOKEN" gh issue create.
  decided_at: '2026-07-18T20:05:41+00:00'
---
