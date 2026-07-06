"""AGENTS.md hygiene check.

AGENTS.md is the startup policy surface most likely to drift in derived
projects because it combines processkit-owned instructions with local project
preferences. This check keeps processkit-owned guidance explicit:

* processkit-managed blocks must be present and well-formed;
* required references to processkit routing/storage/team/MCP contracts must be
  present;
* known stale or provider-specific anti-patterns should be reconciled; and
* provider-specific pointer files must stay thin pointers back to AGENTS.md.
"""

from __future__ import annotations

import hashlib
import re
from pathlib import Path

from .common import CheckResult


CATEGORY = "agents_md_hygiene"
REQUIRED_MANAGED_BLOCKS = {
    "pk-compliance-contract-v2": "processkit compliance contract",
    "pk-commands": "project command adapter block",
}
LEGACY_BLOCK_MARKERS = {
    "pk-compliance-contract-v2": (
        "<!-- pk-compliance-contract v2 BEGIN -->",
        "<!-- pk-compliance-contract v2 END -->",
    ),
    "pk-commands": ("<!-- pk-commands BEGIN -->", "<!-- pk-commands END -->"),
}
COMMAND_KEYS = ("build", "test", "lint", "fmt", "typecheck")
POINTER_FILES = (
    Path("CLAUDE.md"),
    Path("CODEX.md"),
    Path(".cursor/rules"),
    Path(".cursorrules"),
)

REFERENCE_GROUPS = {
    "session-start": (
        "session start should point agents at the project status briefing",
        ("pk-resume",),
    ),
    "skill-routing": (
        "domain work should route through skill-finder/task-router",
        ("find_skill", "route_task"),
    ),
    "entity-io": (
        "entity reads/writes should use processkit MCP/index entry points",
        ("index-management", "MCP tool", "context/templates/"),
    ),
    "decision-capture": (
        "accepted cross-cutting recommendations need same-turn decision capture",
        ("record_decision", "skip_decision_record"),
    ),
    "migration-flow": (
        "pending migrations should route through migration-management",
        ("migration-management", "context/migrations/pending"),
    ),
    "subagent-dispatch": (
        "sub-agent dispatch should preserve team and model-class routing",
        ("recommended_team_member_slug", "recommended_model_class"),
    ),
    "team-model-routing": (
        "team/model text should stay provider-neutral and binding-driven",
        ("TeamMember", "model-profile", "model-spec"),
    ),
    "mcp-topology": (
        "MCP merge guidance should mention manifest-driven re-merge",
        (".processkit-mcp-manifest.json", "mcp-config.json"),
    ),
    "provider-pointers": (
        "provider-specific files should be thin pointers to AGENTS.md",
        ("CLAUDE.md", "CODEX.md", ".cursor/rules", "AGENTS.md"),
    ),
    "commands-block": (
        "command adapters should live in the pk-commands block",
        ("pk-commands",),
    ),
}

ANTI_PATTERNS = (
    (
        "legacy-model-tier-table",
        re.compile(r"\|\s*Role\s*\|\s*Model tier\s*\|", re.IGNORECASE),
        "legacy Team table binds roles directly to model tiers",
    ),
    (
        "target-orientation-mix",
        re.compile(r"\bTarget orientation mix\b", re.IGNORECASE),
        "legacy orientation mix wording predates TeamMember defaults",
    ),
    (
        "legacy-aibox-extension",
        re.compile(r"\bspec\.x_aibox\b"),
        "legacy spec.x_aibox extension conflicts with processkit bindings",
    ),
    (
        "legacy-actors-root",
        re.compile(r"\bcontext/actors/\b"),
        "legacy Actor references should be reconciled to TeamMember/Role text",
    ),
    (
        "template-actors",
        re.compile(r"\bTemplate Actors?\b", re.IGNORECASE),
        "template actor language predates the TeamMember model",
    ),
    (
        "morning-briefing",
        re.compile(r"\bmorning-briefing\b", re.IGNORECASE),
        "morning-briefing was replaced by status-briefing/session-start flows",
    ),
    (
        "aibox-skill",
        re.compile(r"\baibox skill\b", re.IGNORECASE),
        "aibox skill wording should be reconciled to processkit skills",
    ),
)


def _rel(path: Path, repo_root: Path) -> str:
    try:
        return path.relative_to(repo_root).as_posix()
    except ValueError:
        return path.as_posix()


def _brief(kind: str, steps: list[str]) -> dict:
    return {
        "kind": kind,
        "project_agent_briefing": " ".join(steps),
    }


def _normalize_block(text: str) -> str:
    lines = [line.rstrip() for line in text.strip().splitlines()]
    return "\n".join(lines).strip() + "\n"


def _managed_blocks(text: str) -> dict[str, str]:
    pattern = re.compile(
        r"<!--\s*pk-managed:([A-Za-z0-9_.-]+)\s+BEGIN\s*-->"
        r"(.*?)"
        r"<!--\s*pk-managed:\1\s+END\s*-->",
        re.DOTALL,
    )
    return {match.group(1): match.group(2) for match in pattern.finditer(text)}


def _reference_agents_md(repo_root: Path, agents_path: Path) -> tuple[Path, str] | None:
    candidates = [repo_root / "src" / "AGENTS.md"]
    templates_root = repo_root / "context" / "templates" / "processkit"
    if templates_root.is_dir():
        candidates.extend(
            path / "AGENTS.md"
            for path in sorted(templates_root.iterdir(), reverse=True)
            if path.is_dir()
        )
    for candidate in candidates:
        if candidate == agents_path:
            continue
        try:
            return candidate, candidate.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
    return None


def _missing_reference_groups(text: str) -> list[tuple[str, str, tuple[str, ...]]]:
    lower = text.lower()
    missing = []
    for group, (description, terms) in REFERENCE_GROUPS.items():
        if not all(term.lower() in lower for term in terms):
            missing.append((group, description, terms))
    return missing


def _commands_schema_findings(
    repo_root: Path,
    text: str,
) -> list[CheckResult]:
    match = re.search(
        r"<!--\s*pk-commands BEGIN\s*-->(.*?)<!--\s*pk-commands END\s*-->",
        text,
        re.DOTALL,
    )
    if not match:
        return [CheckResult(
            severity="WARN",
            category=CATEGORY,
            id=f"{CATEGORY}.commands-block-missing",
            message="AGENTS.md is missing the pk-commands block",
            entity_ref="AGENTS.md",
            suggested_fix=(
                "restore the pk-commands block with build/test/lint/fmt/"
                "typecheck keys"
            ),
            extra={
                "briefing": _brief("missing_reference", [
                    "Restore the processkit command adapter block.",
                    "Keep local command values inside the block and preserve all",
                    "five keys so command shims can read it consistently.",
                ])
            },
        )]

    block = match.group(1)
    missing = [key for key in COMMAND_KEYS if f"{key}:" not in block]
    if not missing:
        return []
    return [CheckResult(
        severity="WARN",
        category=CATEGORY,
        id=f"{CATEGORY}.commands-schema",
        message=(
            "AGENTS.md pk-commands block is missing keys: "
            + ", ".join(missing)
        ),
        entity_ref="AGENTS.md",
        suggested_fix="add the missing command keys, using empty strings when unset",
        extra={
            "missing_keys": missing,
            "briefing": _brief("safe_replace", [
                "Reconcile the pk-commands block rather than moving commands",
                "into prose.",
                "Use empty string values for unsupported commands so wrappers",
                "and doctor checks keep a stable schema.",
            ]),
        },
    )]


def _pointer_findings(repo_root: Path) -> list[CheckResult]:
    findings: list[CheckResult] = []
    for rel in POINTER_FILES:
        path = repo_root / rel
        if not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        if "AGENTS.md" in text and len(text.splitlines()) <= 40:
            continue
        findings.append(CheckResult(
            severity="WARN",
            category=CATEGORY,
            id=f"{CATEGORY}.provider-pointer-not-thin",
            message=(
                f"{rel.as_posix()} should be a thin provider-specific pointer "
                "to AGENTS.md, not a second policy surface"
            ),
            entity_ref=rel.as_posix(),
            suggested_fix=(
                "move processkit policy text into AGENTS.md and replace the "
                "provider-specific file with a short pointer"
            ),
            extra={
                "briefing": _brief("local_rehome", [
                    f"Review {rel.as_posix()} for duplicated processkit policy.",
                    "Keep only harness-specific bootstrap text there and point",
                    "agents back to AGENTS.md for the canonical contract.",
                ])
            },
        ))
    return findings


def _managed_block_findings(
    repo_root: Path,
    agents_path: Path,
    text: str,
) -> list[CheckResult]:
    findings: list[CheckResult] = []
    begin_count = len(re.findall(r"<!--\s*pk-managed:[A-Za-z0-9_.-]+\s+BEGIN\s*-->", text))
    end_count = len(re.findall(r"<!--\s*pk-managed:[A-Za-z0-9_.-]+\s+END\s*-->", text))
    if begin_count != end_count:
        findings.append(CheckResult(
            severity="ERROR",
            category=CATEGORY,
            id=f"{CATEGORY}.managed-block-malformed",
            message=(
                "AGENTS.md has mismatched pk-managed BEGIN/END markers "
                f"({begin_count} BEGIN, {end_count} END)"
            ),
            entity_ref="AGENTS.md",
            suggested_fix="repair pk-managed block boundaries before syncing",
            extra={
                "briefing": _brief("manual_merge", [
                    "Repair managed block markers first.",
                    "Do not attempt automated replacement until each",
                    "pk-managed BEGIN has a matching END with the same id.",
                ])
            },
        ))

    blocks = _managed_blocks(text)
    for block_id, description in REQUIRED_MANAGED_BLOCKS.items():
        if block_id in blocks:
            continue
        legacy_begin, legacy_end = LEGACY_BLOCK_MARKERS[block_id]
        if legacy_begin in text and legacy_end in text:
            message = (
                f"AGENTS.md has the {description} but it is not wrapped in "
                f"pk-managed:{block_id}"
            )
        else:
            message = f"AGENTS.md is missing the {description}"
        findings.append(CheckResult(
            severity="WARN",
            category=CATEGORY,
            id=f"{CATEGORY}.managed-block-missing",
            message=message,
            entity_ref="AGENTS.md",
            suggested_fix=(
                f"restore or wrap the block with "
                f"<!-- pk-managed:{block_id} BEGIN --> / END markers"
            ),
            extra={
                "block_id": block_id,
                "briefing": _brief("safe_replace", [
                    f"Restore the {description} from the current processkit",
                    "template, then preserve local project notes outside the",
                    "managed block.",
                ]),
            },
        ))

    reference = _reference_agents_md(repo_root, agents_path)
    if not reference:
        return findings
    reference_path, reference_text = reference
    reference_blocks = _managed_blocks(reference_text)
    for block_id in sorted(set(blocks) & set(reference_blocks)):
        local_hash = hashlib.sha256(
            _normalize_block(blocks[block_id]).encode("utf-8")
        ).hexdigest()
        reference_hash = hashlib.sha256(
            _normalize_block(reference_blocks[block_id]).encode("utf-8")
        ).hexdigest()
        if local_hash == reference_hash:
            continue
        findings.append(CheckResult(
            severity="WARN",
            category=CATEGORY,
            id=f"{CATEGORY}.managed-block-drift",
            message=(
                f"AGENTS.md managed block {block_id!r} differs from "
                f"{_rel(reference_path, repo_root)}"
            ),
            entity_ref="AGENTS.md",
            suggested_fix=(
                "replace the managed block from the current processkit source "
                "unless the project records an explicit exception"
            ),
            extra={
                "block_id": block_id,
                "local_sha256": local_hash,
                "reference_sha256": reference_hash,
                "reference_path": _rel(reference_path, repo_root),
                "briefing": _brief("manual_merge", [
                    f"Compare AGENTS.md block {block_id} with",
                    f"{_rel(reference_path, repo_root)}.",
                    "Keep project-specific guidance outside the managed block;",
                    "inside it, prefer the processkit source text or document a",
                    "project exception.",
                ]),
            },
        ))
    return findings


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    agents_path = repo_root / "AGENTS.md"
    try:
        text = agents_path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return [CheckResult(
            severity="WARN",
            category=CATEGORY,
            id=f"{CATEGORY}.missing",
            message="AGENTS.md not found at repo root",
            suggested_fix="create AGENTS.md from the current processkit template",
            extra={
                "briefing": _brief("safe_replace", [
                    "Create AGENTS.md from the shipped processkit template.",
                    "Then add local project notes outside managed blocks.",
                ])
            },
        )]
    except UnicodeDecodeError:
        return [CheckResult(
            severity="ERROR",
            category=CATEGORY,
            id=f"{CATEGORY}.unreadable",
            message="AGENTS.md is not valid UTF-8",
            entity_ref="AGENTS.md",
            suggested_fix="convert AGENTS.md to UTF-8 text",
        )]

    findings: list[CheckResult] = []
    findings.extend(_managed_block_findings(repo_root, agents_path, text))

    missing_groups = _missing_reference_groups(text)
    if missing_groups:
        findings.append(CheckResult(
            severity="WARN",
            category=CATEGORY,
            id=f"{CATEGORY}.references-missing",
            message=(
                "AGENTS.md is missing processkit reference groups: "
                + ", ".join(group for group, _, _ in missing_groups)
            ),
            entity_ref="AGENTS.md",
            suggested_fix=(
                "reconcile AGENTS.md with the current processkit template and "
                "restore missing processkit references"
            ),
            extra={
                "missing_groups": [
                    {
                        "group": group,
                        "description": description,
                        "required_terms": list(terms),
                    }
                    for group, description, terms in missing_groups
                ],
                "briefing": _brief("manual_merge", [
                    "AGENTS.md lacks processkit routing, storage, team, MCP,",
                    "migration, or provider-pointer references that current",
                    "processkit regulates.",
                    "Merge the current template guidance and keep local policy",
                    "outside managed blocks.",
                ]),
            },
        ))

    matched_patterns = []
    for slug, pattern, description in ANTI_PATTERNS:
        if pattern.search(text):
            matched_patterns.append({"pattern": slug, "description": description})
    if matched_patterns:
        findings.append(CheckResult(
            severity="WARN",
            category=CATEGORY,
            id=f"{CATEGORY}.semantic-cleanup",
            message=(
                "AGENTS.md contains stale processkit/aibox guidance: "
                + ", ".join(item["pattern"] for item in matched_patterns)
            ),
            entity_ref="AGENTS.md",
            suggested_fix=(
                "replace stale guidance with current processkit TeamMember, "
                "binding, skill, and migration language"
            ),
            extra={
                "matched_patterns": matched_patterns,
                "briefing": _brief("semantic_cleanup", [
                    "Review the matched stale guidance.",
                    "Remove direct provider/model/team-member shortcuts and",
                    "legacy aibox extensions when processkit now owns that",
                    "behavior through roles, TeamMembers, bindings, skills, or",
                    "MCP tools.",
                ]),
            },
        ))

    findings.extend(_commands_schema_findings(repo_root, text))
    findings.extend(_pointer_findings(repo_root))

    if findings:
        return findings
    return [CheckResult(
        severity="INFO",
        category=CATEGORY,
        id=f"{CATEGORY}.clean",
        message="AGENTS.md managed blocks and processkit references look current",
    )]
