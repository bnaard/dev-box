---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260509_2124-SolidField-grandhawk-follow-up-rewrite-zellij-ref
  created: '2026-05-09T21:24:35+00:00'
spec:
  title: 'GrandHawk follow-up: rewrite Zellij ref in NOTE-GrandCrow line 68'
  body: See Markdown body below.
  type: reference
  state: permanent
---
## Background

`context/notes/NOTE-20260411_2335-GrandCrow-competitive-analysis-dev-environments.md` has frontmatter `apiVersion: processkit.projectious.work/v1` — MCP-owned, so the T3 GrandHawk agent did not edit it inline.

## Single edit needed (line 68 of the note)

**Current:**
```
4. Terminal-first toolchain integration (Zellij + Yazi + Vim + lazygit)
```

**Proposed:**
```
4. Terminal-first toolchain integration (tmux + Yazi + Vim + lazygit)
```

## Rationale

The note is a competitive-analysis reference describing aibox's current unique position. v0.25.x NobleCrane migration replaced Zellij with tmux as the multiplexer. The line as-written misrepresents current state.

## Action

When the v1→v2 note migration tooling lands (or when the note is otherwise re-rendered through MCP), apply the one-character substitution. Until then this fleeting note records the pending edit.

## Provenance

Originally written by the T3 GrandHawk agent as `context/migrations/pending/grandhawk-grandcrow.md` because the sandbox blocked `/tmp/` writes. That misfiled propose-only file has been promoted to this Note and the original deleted.
