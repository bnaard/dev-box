---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260410_2234-LivelyQuail-xdg-open-shim-in
  created: '2026-04-10T22:34:03+00:00'
spec:
  title: xdg-open shim in base image for headless OAuth flows
  state: accepted
  decision: "The aibox base image ships a 30-line POSIX shim at `/usr/local/bin/xdg-open` that prints the URL with framed copy-to-host instructions and exits 0, instead of attempting to launch a real browser. Any tool calling `xdg-open` (`gh auth login`, git credential helpers, opencode, claude code device-flow, etc.) sees a successful browser launch; the user copies the URL into their host browser. The shim is the canonical solution for headless OAuth in aibox containers \u2014 no host-forwarding daemon (lemonade) in v0.17.x."
  context: Headless containers have no browser. Tools calling `xdg-open` fail with "executable file not found in $PATH", breaking OAuth polling entirely. Users had no path to authenticate gh, git credentials, or AI provider CLIs inside the container.
  rationale: "The shim is universal (works on any *nix host), zero-dependency, and gives the user the URL in a framed, copy-friendly format. Tools think the browser opened and continue OAuth polling. A host-forwarding daemon (lemonade) was researched and rejected for v0.17.x \u2014 too big for a patch release, and the user explicitly said \"no further implementation for v0.17.0\". The shim is the right baseline; host forwarding is a future v0.18+ option."
  alternatives:
  - option: "lemonade \u2014 cross-platform host browser forwarding daemon"
    rejected_because: Too large for a patch release; requires a host-side daemon; user explicitly deferred to v0.18+
  consequences: All tools that call xdg-open in the container get a print-URL flow instead of an error. OAuth polling continues successfully. The shim is in `images/base-debian/config/bin/xdg-open.sh`.
  deciders:
  - ACTOR-20260410_2209-SnappyFrog-bernhard
  decided_at: '2026-04-10T22:34:03+00:00'
---
