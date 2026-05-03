---
apiVersion: processkit.projectious.work/v2
kind: Role
metadata:
  id: ROLE-embedded-engineer
  created: 2026-04-22 00:00:00+00:00
spec:
  name: embedded-engineer
  description: Builds system software for constrained devices (firmware, RTOS, peripherals).
  responsibilities:
  - Design and implement firmware for resource-constrained targets
  - Own the device bring-up and board-support layer
  - Measure and tune footprint, power, and real-time behaviour
  - Write drivers and HAL abstractions with tests
  skills_required:
  - c-cpp
  - rtos
  - embedded-linux
  - hardware-protocols
  - low-level-debugging
  default_scope: permanent
  default_seniority: senior
  function_group: engineering-software
---
