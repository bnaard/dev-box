"""team_member_exports check — Claude sub-agent export drift.

Per BACK-20260509_1317-WildPanda P3: when force-dispatching sub-agents
through TeamMember identity (compliance contract sub-agent-dispatch
clause), every active TeamMember needs a Claude sub-agent export file
under `.claude/agents/<slug>.md` (the convention emitted by
`team-manager.export_claude_subagent`). A missing export means the
harness can't invoke that TeamMember as a `subagent_type`; a stale
export points at a slug that no longer maps to an active TeamMember.

Detect logic
------------
- WARN per active TeamMember without `.claude/agents/<slug>.md`.
- WARN per `.claude/agents/<slug>.md` whose slug no longer maps to an
  active TeamMember.
- INFO when the active roster and the export directory are in sync.

Detect-only. The fix is to re-run `team-manager.export_claude_subagent`
(or `export_claude_subagents` for the whole roster); doctor does not
auto-export.
"""
from __future__ import annotations

from pathlib import Path

import yaml

from .common import CheckResult


# Default Claude sub-agent export dir — matches `_claude_output_dir`
# in team-manager/mcp/server.py.
EXPORT_DIR = Path(".claude") / "agents"


def _active_team_member_slugs(repo_root: Path) -> set[str]:
    """Read context/team-members/<slug>/team-member.md frontmatter and
    return the set of active TeamMember slugs. Mirrors the cheap roster
    read used by task-router's _list_active_team_members().
    """
    tm_root = repo_root / "context" / "team-members"
    if not tm_root.is_dir():
        return set()
    out: set[str] = set()
    for child in sorted(tm_root.iterdir()):
        if not child.is_dir():
            continue
        p = child / "team-member.md"
        if not p.is_file():
            continue
        try:
            text = p.read_text(encoding="utf-8")
        except OSError:
            continue
        if not text.startswith("---"):
            continue
        parts = text.split("---", 2)
        if len(parts) < 3:
            continue
        try:
            data = yaml.safe_load(parts[1])
        except yaml.YAMLError:
            continue
        if not isinstance(data, dict):
            continue
        spec = data.get("spec") if isinstance(data.get("spec"), dict) else {}
        if not spec.get("active", True):
            continue
        # Mirror team-manager.export_claude_subagent: skip non-exportable
        # TeamMembers (e.g. owner with type='human' carrying exportable=false).
        # The export is impossible by design — flagging it as missing is noise.
        if spec.get("exportable") is False:
            continue
        slug = spec.get("slug") or child.name
        if isinstance(slug, str) and slug:
            out.add(slug)
    return out


def _exported_slugs(repo_root: Path) -> set[str]:
    """Return the set of slugs present in `.claude/agents/<slug>.md`."""
    export_dir = repo_root / EXPORT_DIR
    if not export_dir.is_dir():
        return set()
    return {p.stem for p in export_dir.glob("*.md") if p.is_file()}


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    results: list[CheckResult] = []

    active = _active_team_member_slugs(repo_root)
    exported = _exported_slugs(repo_root)

    export_dir_rel = str(EXPORT_DIR.as_posix())

    # Active TeamMembers without an export file.
    for slug in sorted(active - exported):
        results.append(CheckResult(
            severity="WARN",
            category="team_member_exports",
            id="team_member_exports.missing-export",
            message=(
                f"active TeamMember {slug!r} has no Claude sub-agent "
                f"export at {export_dir_rel}/{slug}.md; harnesses cannot "
                f"dispatch it as subagent_type"
            ),
            entity_ref=f"TEAMMEMBER-{slug}",
            fixable=True,
            suggested_fix=(
                f"run team-manager.export_claude_subagent(slug={slug!r}) "
                "to regenerate the export"
            ),
            fix_mcp_tool=(
                "mcp__processkit-team-manager__export_claude_subagent"
            ),
        ))

    # Export files whose slug no longer maps to an active TeamMember.
    for slug in sorted(exported - active):
        results.append(CheckResult(
            severity="WARN",
            category="team_member_exports",
            id="team_member_exports.stale-export",
            message=(
                f"{export_dir_rel}/{slug}.md has no active TeamMember "
                f"with slug {slug!r}; remove or regenerate the export"
            ),
            entity_ref=f"{export_dir_rel}/{slug}.md",
            fixable=False,
            suggested_fix=(
                "delete the stale export file or reactivate the "
                "underlying TeamMember"
            ),
        ))

    if not results:
        synced = len(active)
        results.append(CheckResult(
            severity="INFO",
            category="team_member_exports",
            id="team_member_exports.in-sync",
            message=(
                f"{synced} active TeamMember(s) match {synced} export "
                f"file(s) under {export_dir_rel}/"
            ),
        ))

    return results
