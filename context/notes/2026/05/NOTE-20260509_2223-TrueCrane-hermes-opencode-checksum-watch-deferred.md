---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260509_2223-TrueCrane-hermes-opencode-checksum-watch-deferred
  created: '2026-05-09T22:23:19+00:00'
spec:
  title: "BraveCrow checksum-watch v0.25.7: Hermes + OpenCode \u2014 no upstream checksums published"
  body: See Markdown body below.
  type: reference
  state: permanent
---
## Watch result — 2026-05-10

**WorkItem:** BACK-20260508_2257-BraveCrow-hermes-opencode-checksum-upstream-watch  
**Branch:** v0.25.7/bravecrow-checksum-watch  
**Scope:** ai-hermes (Nous Research) · ai-opencode (opencode-ai/opencode)

---

### Findings

**Hermes (ai-hermes v1.0.0)**

- Source: `https://github.com/nousresearch/hermes/releases`
- Current aibox posture: versioned binary download, no version pin in aibox.toml (commented out), falls back to `latest` redirect.
- Upstream checksum material: **none published** as of this watch pass. No `SHA256SUMS`, no `*.asc`, no `.sha256` files on the nousresearch/hermes releases page (per WorkItem body, unchanged since 2026-05-08).
- TODO(sec) block in addon: still accurate and present.
- **Decision: no bump. Defer.**

**OpenCode (ai-opencode v1.0.0)**

- Source: `https://github.com/opencode-ai/opencode/releases`
- Current aibox posture: versioned tar.gz download, no version pin in aibox.toml (commented out), falls back to `latest` redirect.
- Upstream checksum material: **none published** as of this watch pass. No per-release SHA-256 files or GPG signatures (per WorkItem body, unchanged since 2026-05-08).
- TODO(sec) block in addon: still accurate and present.
- **Decision: no bump. Defer.**

---

### Rationale for no-change

Computing and pinning a locally-computed SHA-256 of the downloaded binary would only catch accidental transport corruption, not a malicious release substitution. The WorkItem body explicitly documents this reasoning (DEC-20260508_2235-CuriousBadger). The current posture (version-pinned release asset download) remains the best available option.

---

### Next-check trigger

Re-run this watch when either:
1. `https://github.com/nousresearch/hermes/releases` adds a `SHA256SUMS` or `*.asc` artifact to any release.
2. `https://github.com/opencode-ai/opencode/releases` adds a checksum file.

Recommended calendar next-check: **2026-06-10** (monthly cadence), or sooner if a new major release of either tool is announced.

---

### Action if triggered

Per WorkItem spec:
1. Update the relevant addon YAML `runtime:` block to add `sha256sum -c` verification (mirror the pattern in `addons/tools/infrastructure.yaml`).
2. Remove the `TODO(sec)` block.
3. Bump the addon `version:`.
4. File a Migration entity if downstream projects need to re-apply.
