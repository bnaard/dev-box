"""commands_consistency check.

Every processkit-managed skill that declares
``metadata.processkit.commands`` in its SKILL.md must ship a matching
``commands/<name>.md`` file alongside SKILL.md. The canonical command
files are then projected into harness-specific surfaces:

* ``.claude/commands/<name>.md`` for slash-capable Claude Code.
* ``.agents/skills/<name>/SKILL.md`` for Codex-style natural-language
  skill triggering when custom slash commands are unavailable.

ERROR per missing canonical command file, unprefixed processkit command,
frontmatter argument mismatch, missing projection, or projection-only
command. WARN if a skill has ``commands/*.md`` files that are not
declared in metadata.
"""

from __future__ import annotations

from pathlib import Path

import yaml

from .common import CheckResult


def _parse_frontmatter(path: Path) -> dict | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    if not text.startswith("---"):
        return None
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None
    try:
        data = yaml.safe_load(parts[1])
    except yaml.YAMLError:
        return None
    return data if isinstance(data, dict) else None


def _declared_commands(frontmatter: dict) -> dict[str, str]:
    meta = frontmatter.get("metadata")
    if not isinstance(meta, dict):
        return {}
    pk = meta.get("processkit")
    if not isinstance(pk, dict):
        return {}
    commands = pk.get("commands")
    if not isinstance(commands, list):
        return {}
    declared: dict[str, str] = {}
    for entry in commands:
        if isinstance(entry, dict) and isinstance(entry.get("name"), str):
            args = entry.get("args")
            declared[entry["name"]] = args if isinstance(args, str) else ""
    return declared


def _command_argument_hint(path: Path) -> str | None:
    fm = _parse_frontmatter(path)
    if fm is None:
        return None
    hint = fm.get("argument-hint")
    return hint if isinstance(hint, str) else None


def _projection_stems(root: Path, pattern: str) -> set[str]:
    if not root.is_dir():
        return set()
    return {p.stem for p in root.glob(pattern)}


def _agent_skill_names(root: Path) -> set[str]:
    if not root.is_dir():
        return set()
    return {p.parent.name for p in root.glob("*/SKILL.md")}


def _projection_enabled(root: Path) -> bool:
    return root.exists()


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    skills_root = repo_root / "context" / "skills"
    if not skills_root.is_dir():
        return [CheckResult(
            severity="INFO",
            category="commands_consistency",
            id="commands_consistency.no-skills-dir",
            message=f"{skills_root.relative_to(repo_root)} not found; skipping",
        )]

    findings: list[CheckResult] = []
    canonical: set[str] = set()
    command_args: dict[str, str] = {}

    for skill_md in sorted(skills_root.glob("*/*/SKILL.md")):
        skill_dir = skill_md.parent
        skill_slug = skill_dir.name
        fm = _parse_frontmatter(skill_md)
        if fm is None:
            continue
        declared = _declared_commands(fm)
        commands_dir = skill_dir / "commands"
        existing = set()
        if commands_dir.is_dir():
            existing = {p.stem for p in commands_dir.glob("*.md")}

        for name, args in declared.items():
            command_args[name] = args
            if not name.startswith("pk-"):
                findings.append(CheckResult(
                    severity="ERROR",
                    category="commands_consistency",
                    id="commands_consistency.unprefixed-processkit-command",
                    message=(
                        f"skill '{skill_slug}' declares command '{name}', but "
                        "processkit slash commands must use the reserved 'pk-' "
                        "prefix"
                    ),
                    entity_ref=f"SKILL-{skill_slug}",
                    suggested_fix=(
                        f"rename command to 'pk-{name}' and move the adapter to "
                        f"commands/pk-{name}.md"
                    ),
                ))
            expected = commands_dir / f"{name}.md"
            if not expected.is_file():
                findings.append(CheckResult(
                    severity="ERROR",
                    category="commands_consistency",
                    id="commands_consistency.missing-command-file",
                    message=(
                        f"skill '{skill_slug}' declares command '{name}' in SKILL.md "
                        f"metadata but {expected.relative_to(repo_root)} is missing"
                    ),
                    entity_ref=f"SKILL-{skill_slug}",
                ))
            else:
                canonical.add(name)
                hint = _command_argument_hint(expected)
                if hint != args:
                    findings.append(CheckResult(
                        severity="ERROR",
                        category="commands_consistency",
                        id="commands_consistency.argument-hint-mismatch",
                        message=(
                            f"skill '{skill_slug}' declares command '{name}' "
                            f"args={args!r}, but "
                            f"{expected.relative_to(repo_root)} has "
                            f"argument-hint={hint!r}"
                        ),
                        entity_ref=f"SKILL-{skill_slug}",
                    ))

        declared_set = set(declared)
        for stem in sorted(existing - declared_set):
            findings.append(CheckResult(
                severity="WARN",
                category="commands_consistency",
                id="commands_consistency.undeclared-command-file",
                message=(
                    f"skill '{skill_slug}' ships commands/{stem}.md but '{stem}' is "
                    f"not declared in SKILL.md metadata.processkit.commands"
                ),
                entity_ref=f"SKILL-{skill_slug}",
            ))

    claude_root = repo_root / ".claude" / "commands"
    if _projection_enabled(claude_root):
        claude = _projection_stems(claude_root, "*.md")
        for name in sorted(canonical - claude):
            findings.append(CheckResult(
                severity="ERROR",
                category="commands_consistency",
                id="commands_consistency.missing-claude-projection",
                message=(
                    f"canonical command '{name}' is missing "
                    f"{(claude_root / f'{name}.md').relative_to(repo_root)}"
                ),
            ))
        for name in sorted(claude - canonical):
            findings.append(CheckResult(
                severity="ERROR",
                category="commands_consistency",
                id="commands_consistency.claude-only-command",
                message=(
                    f"{(claude_root / f'{name}.md').relative_to(repo_root)} "
                    "has no matching canonical context/skills command"
                ),
            ))
        for name in sorted(canonical & claude):
            hint = _command_argument_hint(claude_root / f"{name}.md")
            args = command_args.get(name, "")
            if hint != args:
                findings.append(CheckResult(
                    severity="ERROR",
                    category="commands_consistency",
                    id="commands_consistency.claude-argument-hint-mismatch",
                    message=(
                        f".claude projection '{name}' has argument-hint={hint!r}, "
                        f"but metadata args={args!r}"
                    ),
                ))

    agents_root = repo_root / ".agents" / "skills"
    if _projection_enabled(agents_root):
        agents = _agent_skill_names(agents_root)
        for name in sorted(canonical - agents):
            findings.append(CheckResult(
                severity="ERROR",
                category="commands_consistency",
                id="commands_consistency.missing-agent-skill-projection",
                message=(
                    f"canonical command '{name}' is missing "
                    f"{(agents_root / name / 'SKILL.md').relative_to(repo_root)}"
                ),
            ))
        for name in sorted(agents - canonical):
            findings.append(CheckResult(
                severity="ERROR",
                category="commands_consistency",
                id="commands_consistency.agent-only-command",
                message=(
                    f"{(agents_root / name / 'SKILL.md').relative_to(repo_root)} "
                    "has no matching canonical context/skills command"
                ),
            ))

    if not findings:
        return [CheckResult(
            severity="INFO",
            category="commands_consistency",
            id="commands_consistency.ok",
            message="all skill commands: metadata entries have matching commands/*.md files",
        )]
    return findings
