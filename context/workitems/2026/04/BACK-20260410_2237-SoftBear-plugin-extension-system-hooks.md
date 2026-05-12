---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260410_2237-SoftBear-plugin-extension-system-hooks
  created: '2026-04-10T22:37:15+00:00'
  labels:
    old_id: BACK-007
    area: features
spec:
  title: "Plugin / extension system \u2014 hooks, custom template overrides, community features"
  state: backlog
  type: task
  priority: medium
  description: "Scope: (1) user-installable extensions that add commands, modify generation, or provide new features (oh-my-zsh model); (2) hook system \u2014 pre/post hooks for lifecycle commands (pre-build, post-start, etc.); (3) custom template overrides \u2014 user-provided Dockerfile.j2, docker-compose.yml.j2, layout .kdl templates; (4) community features \u2014 like devcontainer features but for aibox. Open questions: MVP plugin interface; shell scripts vs Rust dylibs vs WASM; toml config integration; distribution and versioning model. Old ID: BACK-007."
---
