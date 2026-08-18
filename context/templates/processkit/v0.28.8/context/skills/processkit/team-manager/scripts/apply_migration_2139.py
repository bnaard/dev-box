"""Apply script for MIG-20260509T213904-roleslot-phase-a (Phase A back-fill).

Walks the project's `context/roles/` and `context/bindings/` directories,
emits one RoleSlot per legacy ``Role.clone_cap`` unit, and writes a
parallel ``role-slot-fill`` Binding for every active
``role-assignment`` Binding it finds. Old v1 entities are left in place
(read-only).

The script is **idempotent**: a re-run notices already-emitted RoleSlots
and ``role-slot-fill`` Bindings (matched by their deterministic IDs and
``labels.migration`` stamp) and skips them.

On a v2-native project (no v1 archetype Roles, no ``role-assignment``
Bindings) the script is a no-op — its value is for derived projects
(aibox, etc.) when they upgrade across the v0.25.8 → v0.26.0 boundary.

Usage::

    uv run --with pyyaml --with jsonschema --with mcp \\
        python context/skills/processkit/team-manager/scripts/apply_migration_2139.py \\
        --chartering-scope SCOPE-<id>            # required
        [--project-root <path>]                  # default: cwd-walk
        [--dry-run]                              # print plan; write nothing

The chartering Scope must already exist; if it does not, the script
errors before any write.
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any


_THIS = Path(__file__).resolve()
_TM_SCRIPTS = _THIS.parent
_TM_MCP = _TM_SCRIPTS.parent / "mcp"


def _bootstrap_paths() -> None:
    """Ensure both the team-manager MCP server module and the
    processkit lib are importable."""
    here = _THIS
    while True:
        for c in (here / "src" / "lib", here / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                if str(c) not in sys.path:
                    sys.path.insert(0, str(c))
                break
        if here.parent == here:
            break
        here = here.parent
    for p in (str(_TM_MCP), str(_TM_SCRIPTS)):
        if p not in sys.path:
            sys.path.insert(0, p)


_bootstrap_paths()


# Imports happen after path bootstrap so the project lib is found.
import server as team_manager_mcp  # noqa: E402
from processkit import entity, paths  # noqa: E402


_MIGRATION_ID = "MIG-20260509T213904-roleslot-phase-a"


# v0.19.0 archetype set used to recognise legacy archetype-spawned Roles.
_LEGACY_ARCHETYPES = {
    "project-manager",
    "senior-architect",
    "senior-researcher",
    "junior-architect",
    "developer",
    "junior-researcher",
    "junior-developer",
    "assistant",
}


def _iter_v1_archetype_roles(root: Path):
    """Yield (Entity, archetype_name, clone_cap_int) for legacy Roles.

    A legacy Role is one whose name matches a v0.19.0 archetype. We
    treat ``spec.clone_cap`` if present; otherwise default to 1
    (every legacy Role is at least one slot).
    """
    role_dir = root / "context" / "roles"
    if not role_dir.is_dir():
        return
    for path in sorted(role_dir.glob("ROLE-*.md")):
        try:
            ent = entity.load(path)
        except Exception:
            continue
        if ent.kind != "Role":
            continue
        spec = ent.spec or {}
        name = spec.get("name") or ent.id[len("ROLE-"):]
        if name not in _LEGACY_ARCHETYPES:
            # Could also be the matching catalog Role; only legacy
            # archetype-spawned Roles get back-filled. Catalog Roles
            # are by convention named after their job title
            # (e.g. ROLE-product-manager) — never match the archetype
            # set.
            continue
        try:
            cap = int(spec.get("clone_cap", 1))
        except (TypeError, ValueError):
            cap = 1
        if cap < 1:
            cap = 1
        yield ent, name, cap


def _iter_active_role_assignment_bindings(root: Path):
    """Yield Entity objects for active Binding(type=role-assignment)."""
    bind_dir = root / "context" / "bindings"
    if not bind_dir.is_dir():
        return
    for path in sorted(bind_dir.glob("BIND-*.md")):
        try:
            ent = entity.load(path)
        except Exception:
            continue
        if ent.kind != "Binding":
            continue
        spec = ent.spec or {}
        if spec.get("type") != "role-assignment":
            continue
        # Skip ended bindings.
        if spec.get("ended_at") or spec.get("valid_until_passed"):
            continue
        yield ent


def _scope_slug(scope_id: str) -> str:
    return scope_id[len("SCOPE-"):] if scope_id.startswith("SCOPE-") else scope_id


def _existing_role_slot_fill_for(root: Path, slot_id: str,
                                  team_member_id: str) -> bool:
    """True iff a role-slot-fill Binding already exists for (slot, member)."""
    bind_dir = root / "context" / "bindings"
    if not bind_dir.is_dir():
        return False
    for path in bind_dir.glob("BIND-*.md"):
        try:
            ent = entity.load(path)
        except Exception:
            continue
        if ent.kind != "Binding":
            continue
        spec = ent.spec or {}
        if spec.get("type") != "role-slot-fill":
            continue
        if spec.get("target") == slot_id and spec.get("subject") == team_member_id:
            return True
    return False


def _scope_exists(root: Path, scope_id: str) -> bool:
    if not scope_id.startswith("SCOPE-"):
        return False
    scope_dir = root / "context" / "scopes"
    if not scope_dir.is_dir():
        return False
    return any(p.name == f"{scope_id}.md" for p in scope_dir.rglob("SCOPE-*.md"))


def _seniority_for_role(role_id: str, role_spec: dict[str, Any]) -> str:
    """Best-effort seniority extraction.

    v1 archetype Roles did not carry an explicit seniority field — we
    derive a sensible default from the archetype name; a project may
    override after migration.
    """
    name = role_spec.get("name") or role_id[len("ROLE-"):]
    if name.startswith("junior-"):
        return "junior"
    if name in {"project-manager", "senior-architect", "senior-researcher",
                "developer"}:
        return "senior"
    return "specialist"


def apply(
    project_root: Path,
    chartering_scope: str,
    *,
    dry_run: bool = False,
) -> dict[str, Any]:
    """Run the back-fill.

    Returns a summary dict with counts: ``slots_created``,
    ``slots_skipped`` (already present), ``fills_created``,
    ``fills_skipped``, ``warnings``.
    """
    project_root = Path(project_root).resolve()
    if not chartering_scope.startswith("SCOPE-"):
        return {
            "error": (
                f"chartering_scope must be a SCOPE-* id; got "
                f"{chartering_scope!r}"
            )
        }
    if not _scope_exists(project_root, chartering_scope):
        return {
            "error": (
                f"chartering_scope {chartering_scope!r} not found under "
                f"{project_root / 'context' / 'scopes'}"
            )
        }

    summary = {
        "migration_id": _MIGRATION_ID,
        "chartering_scope": chartering_scope,
        "project_root": str(project_root),
        "dry_run": bool(dry_run),
        "slots_created": 0,
        "slots_skipped": 0,
        "fills_created": 0,
        "fills_skipped": 0,
        "warnings": [],
        "actions": [],
    }

    # Make team-manager find the project root we want.
    paths.find_project_root.__wrapped__ if hasattr(paths.find_project_root,
                                                   "__wrapped__") else None
    # Override discovery by chdir'ing — the team-manager server walks up
    # from cwd looking for AGENTS.md.
    import os
    prev_cwd = os.getcwd()
    os.chdir(project_root)
    try:
        # --- Phase A.2: emit RoleSlots ---------------------------------
        archetype_to_slot_rank1: dict[str, str] = {}
        for role_ent, archetype, cap in _iter_v1_archetype_roles(project_root):
            seniority = _seniority_for_role(role_ent.id, role_ent.spec or {})
            scope_slug = _scope_slug(chartering_scope)
            for rank in range(1, cap + 1):
                slot_id = (
                    f"SLOT-{scope_slug}-{role_ent.id[len('ROLE-'):]}-{rank}"
                )
                slot_path = (
                    project_root / "context" / "roleslots" / f"{slot_id}.md"
                )
                if slot_path.is_file():
                    summary["slots_skipped"] += 1
                    summary["actions"].append({
                        "kind": "skip-slot-exists",
                        "slot": slot_id,
                    })
                    if rank == 1:
                        archetype_to_slot_rank1[role_ent.id] = slot_id
                    continue
                if dry_run:
                    summary["slots_created"] += 1
                    summary["actions"].append({
                        "kind": "would-create-slot",
                        "slot": slot_id,
                        "archetype": archetype,
                        "role": role_ent.id,
                        "seniority": seniority,
                        "rank": rank,
                    })
                    if rank == 1:
                        archetype_to_slot_rank1[role_ent.id] = slot_id
                    continue
                rationale = (
                    f"Phase A back-fill for {_MIGRATION_ID}: "
                    f"archetype={archetype} (was Role.clone_cap={cap})"
                )
                result = team_manager_mcp.create_role_slot(
                    scope=chartering_scope,
                    role=role_ent.id,
                    seniority=seniority,
                    rank=rank,
                    rationale=rationale,
                )
                if "error" in result:
                    summary["warnings"].append({
                        "kind": "create_role_slot-failed",
                        "role": role_ent.id,
                        "rank": rank,
                        "detail": result["error"],
                    })
                    continue
                summary["slots_created"] += 1
                summary["actions"].append({
                    "kind": "created-slot",
                    "slot": result["id"],
                    "archetype": archetype,
                    "role": role_ent.id,
                    "rank": rank,
                })
                if rank == 1:
                    archetype_to_slot_rank1[role_ent.id] = result["id"]

        # --- Phase A.3: emit role-slot-fill Bindings -------------------
        for ba in _iter_active_role_assignment_bindings(project_root):
            spec = ba.spec or {}
            target = spec.get("target")
            subject = spec.get("subject")
            if not target or not target.startswith("ROLE-"):
                summary["warnings"].append({
                    "kind": "skip-binding-non-role-target",
                    "binding": ba.id,
                    "target": target,
                })
                continue
            slot_id = archetype_to_slot_rank1.get(target)
            if not slot_id:
                summary["warnings"].append({
                    "kind": "skip-binding-no-rank1-slot",
                    "binding": ba.id,
                    "role": target,
                })
                continue
            # We require a TeamMember subject for role-slot-fill (the v2
            # schema says so). v1 may have used Actors as subjects;
            # those cannot be migrated automatically.
            if not str(subject).startswith("TEAMMEMBER-"):
                summary["warnings"].append({
                    "kind": "skip-binding-non-teammember-subject",
                    "binding": ba.id,
                    "subject": subject,
                })
                continue
            if _existing_role_slot_fill_for(project_root, slot_id, subject):
                summary["fills_skipped"] += 1
                summary["actions"].append({
                    "kind": "skip-fill-exists",
                    "slot": slot_id,
                    "team_member": subject,
                })
                continue
            if dry_run:
                summary["fills_created"] += 1
                summary["actions"].append({
                    "kind": "would-fill-slot",
                    "slot": slot_id,
                    "team_member": subject,
                })
                continue
            tm_slug = subject[len("TEAMMEMBER-"):]
            result = team_manager_mcp.fill_role_slot(
                id=slot_id,
                team_member_slug=tm_slug,
                rationale=(
                    f"Phase A back-fill for {_MIGRATION_ID}: "
                    f"parallel role-slot-fill for v1 {ba.id}"
                ),
            )
            if "error" in result:
                summary["warnings"].append({
                    "kind": "fill_role_slot-failed",
                    "slot": slot_id,
                    "team_member": subject,
                    "detail": result["error"],
                })
                continue
            summary["fills_created"] += 1
            summary["actions"].append({
                "kind": "filled-slot",
                "slot": slot_id,
                "team_member": subject,
                "binding": result.get("binding", {}).get("id"),
            })
    finally:
        os.chdir(prev_cwd)

    return summary


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Apply MIG-20260509T213904-roleslot-phase-a Phase A "
            "back-fill (idempotent)."
        )
    )
    parser.add_argument("--chartering-scope", required=True,
                        help="SCOPE-<id> the back-filled RoleSlots belong to")
    parser.add_argument("--project-root", default=".",
                        help="Project root (defaults to cwd)")
    parser.add_argument("--dry-run", action="store_true",
                        help="Print the plan without writing")
    args = parser.parse_args(argv)
    summary = apply(
        Path(args.project_root),
        args.chartering_scope,
        dry_run=args.dry_run,
    )
    if "error" in summary:
        print(f"ERROR: {summary['error']}", file=sys.stderr)
        return 2
    import json
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
