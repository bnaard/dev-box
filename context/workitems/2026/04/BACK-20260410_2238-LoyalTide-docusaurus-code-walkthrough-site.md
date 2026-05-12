---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260410_2238-LoyalTide-docusaurus-code-walkthrough-site
  created: '2026-04-10T22:38:14+00:00'
  labels:
    old_id: BACK-072
    area: docs
spec:
  title: Docusaurus code walkthrough site for Rust newcomers
  state: backlog
  type: task
  priority: medium
  description: "Documentation site (or subsite under docs-site/) explaining the CLI codebase for someone new to Rust but experienced in programming. Scope: (1) Architecture overview \u2014 module graph, data flow from aibox init \u2192 generated files, key abstractions. (2) Per-module walkthrough \u2014 config parsing, template rendering, addon system, seed/sync lifecycle, runtime detection. (3) Rust idioms used \u2014 Result/Option chains, serde patterns, builder pattern, error handling. (4) How to contribute \u2014 adding an addon, adding a command, writing tests. (5) Maintenance strategy \u2014 auto-generate from doc comments vs hand-written? Old ID: BACK-072."
---
