---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260723_1923-FairFlame-fix-ansible-pip3-infrastructure-addon
  created: '2026-07-23T19:23:13+00:00'
  updated: '2026-07-23T19:23:20+00:00'
spec:
  title: Fix Ansible installer dependency in infrastructure addon
  state: in-progress
  type: bug
  priority: high
  description: Derived projects enable the infrastructure addon by default, which
    renders `pip3 install ansible` although the base runtime image has no pip3. Install
    the required Python pip package before Ansible and port the source change to v0.x
    and v1.x.
  started_at: '2026-07-23T19:23:20+00:00'
---

## Transition note (2026-07-23T19:23:20+00:00)

Investigating derived Dockerfile failure caused by missing pip3 before Ansible installation.
