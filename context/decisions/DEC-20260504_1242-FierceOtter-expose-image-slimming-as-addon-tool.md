---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260504_1242-FierceOtter-expose-image-slimming-as-addon-tool
  created: '2026-05-04T12:42:42+00:00'
spec:
  title: Expose Image Slimming as Addon Tool Switches
  state: accepted
  decision: Implement safe build-context slimming directly, and implement capability slimming by exposing finer-grained addon tool switches in aibox.toml. Defaults must preserve current high-value behavior unless a migration explicitly changes a default with owner-visible guidance. For LaTeX, keep common authoring support such as TikZ available by default, but make heavier ancillary tools like Inkscape/SVG conversion explicit switches where feasible.
  context: Image slimming opportunities were identified in the base image build context and in addons such as Python, Rust, LaTeX, Node/docs, preview, and cloud tooling. Some opportunities are pure build hygiene, while others would remove capabilities that existing users may rely on. The project model treats aibox.toml as the declarative component selector, so slimming should preserve that model rather than split images or silently remove expected tools.
  rationale: This reduces image size for users who do not need heavyweight tools, while keeping the selected-components model intact and avoiding surprising regressions in existing projects.
  alternatives:
  - option: Create multiple capability-specific image streams
    status: rejected
    reason: Conflicts with aibox.toml as the single declarative selector.
  - option: Remove heavyweight defaults globally
    status: rejected
    reason: Would break existing workflows, especially LaTeX and preview-heavy projects.
  - option: Do nothing until image sizes are measured from built images
    status: rejected
    reason: Some changes, such as build-context exclusions and cache cleanup, are low-risk and independently justified.
  consequences: Addon manifests need finer tool modeling and generator tests showing switches affect Dockerfile output. Release notes and config migrations must explain newly available switches. Build hygiene changes such as .dockerignore exclusions can be applied without new user-facing configuration.
  decided_at: '2026-05-04T12:42:42+00:00'
---
