# V1 adoption pilots

LLMS index: [llms.txt](/aibox/v1.x/llms.txt)

---

# V1 adoption pilots

Stable-v1 readiness requires four repeatable journeys against the exact
candidate and binary:

1. a new Compose workspace can compile and render a deterministic plan;
2. a representative v0 project can preview, apply, and roll back a reviewed
   v1 intent without exposing secrets or touching v1 deployment records;
3. an existing Kubernetes target passes the complete live M7c lifecycle;
4. exact-pinned processkit install, verify, unchanged update, recovery, and
   uninstall pass through the direct opaque boundary.

After the live M7c and M5 producer evidence has been generated, run:

```sh
RELEASE_CANDIDATE_SHA="$(git rev-parse HEAD)" \
AIBOX_RELEASE_BINARY_SHA256="sha256:<tested-binary-digest>" \
  ./scripts/test-v1-adoption-pilots.sh
```

The harness refuses missing or candidate-mismatched prerequisites. It executes
the local Compose and migration journeys, verifies the live Kubernetes and
direct-processkit scenario sets, and writes
`.aibox/release-evidence/v1-readiness/adoption-pilots.json`. Stable publication
reruns this automatically and verifies the retained log digest.

This automated evidence establishes repeatability, not user sentiment. Record
configuration friction, plan comprehension, recovery steps, terminology
confusion, and documentation gaps from external pilots in their tracking
issues. Do not convert an unrun or unsuccessful external pilot into a passing
release marker.
