# state-machines/

Runtime state machine definitions for primitives that have lifecycle states
(WorkItem, DecisionRecord, Migration, Scope, Discussion, Note). Each is a YAML
file describing states, allowed transitions, and optional transition guards.

Projects may override these defaults by providing their own state machines
with the same name in their `context/state-machines/` directory.

## Shape

```yaml
apiVersion: processkit.projectious.work/v2
kind: StateMachine
metadata:
  id: SM-<name>
  name: <name>
spec:
  initial: <state>
  terminal: [<state>, ...]
  states:
    <state>:
      description: "What this state means."
      transitions:
        - to: <other-state>
          guard: "optional condition for the transition"
```

`kind: StateMachine` is a loader marker for these runtime definitions, not a
first-class v2 primitive schema.

## Shipped state machines

| File | Kind | Lifecycle |
|---|---|---|
| `workitem.yaml` | WorkItem | backlog → in-progress → review → done (+ blocked, cancelled) |
| `decisionrecord.yaml` | DecisionRecord | proposed → accepted → (superseded) |
| `migration.yaml` | Migration | pending → in-progress → applied (+ rejected) |
| `scope.yaml` | Scope | planned → active → completed (+ cancelled) |
| `discussion.yaml` | Discussion | active ↔ resolved → archived |
| `note.yaml` | Note | fleeting → promoted / discarded |

Other primitives (Actor, Role, Gate, Binding, Category, Constraint, Context,
Artifact, TeamMember) do not have lifecycle states by design. Their schemas
have `state_machine: null`.
