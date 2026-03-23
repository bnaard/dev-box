---
name: context-archiving
description: Manages context/archive/ — moves completed backlog items, old decisions, finished projects, and old session notes from active context files to their archive counterparts.
---
# Context Archiving

## When to use

When active context files grow large, when closing out completed work, or during periodic maintenance. Archive keeps active files focused while preserving full history.

## Instructions

1. Read the active context file and its archive counterpart in `context/archive/`
2. Identify items eligible for archiving:
   - **Backlog:** status = `done` or `archived`
   - **Decisions:** older than the most recent ~6 entries, or explicitly superseded
   - **Projects:** status = `complete` or `archived`
   - **Session notes:** older than the 3 most recent sessions
3. Move items from active to archive, preserving format (tables stay tables, headings stay headings)
4. Verify cross-references: each active file links to its archive (`[archive/X.md](archive/X.md)`) and vice versa (`[../X.md](../X.md)`)
5. Never delete items — only move between active and archive
6. After archiving, confirm the active file's "Next ID" counter is unchanged
