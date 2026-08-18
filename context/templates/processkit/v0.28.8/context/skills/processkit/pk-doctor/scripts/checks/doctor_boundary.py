"""Guard host-orchestrator one-directionality in active skill surfaces."""

from __future__ import annotations

from pathlib import Path
import re

from .common import CheckResult


CATEGORY = "doctor_boundary"
_HOST_COMMAND_PATTERNS = [
    re.compile(re.escape("aibox" + " doctor"), re.IGNORECASE),
    re.compile(re.escape("aibox" + " prune"), re.IGNORECASE),
    re.compile(r"suggested_fix=.*" + "ai" + "box", re.IGNORECASE),
    re.compile(
        r"(?:dry_run_command|apply_command).*" + "ai" + "box",
        re.IGNORECASE,
    ),
]
_ACTIVE_SUFFIXES = {".md", ".py", ".json", ".toml"}


def _active_surface_roots(repo_root: Path) -> list[Path]:
    candidates = [
        repo_root / "context" / "skills",
        repo_root / "src" / "context" / "skills",
        repo_root / ".agents" / "skills",
    ]
    return [path for path in candidates if path.is_dir()]


def _iter_active_surface_files(root: Path):
    for path in root.rglob("*"):
        if not path.is_file() or path.suffix not in _ACTIVE_SUFFIXES:
            continue
        if "__pycache__" in path.parts:
            continue
        yield path


def run(ctx) -> list[CheckResult]:
    repo_root = Path(ctx["repo_root"])
    matches: list[str] = []

    for root in _active_surface_roots(repo_root):
        for path in _iter_active_surface_files(root):
            try:
                text = path.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            for lineno, line in enumerate(text.splitlines(), start=1):
                if any(pattern.search(line) for pattern in _HOST_COMMAND_PATTERNS):
                    matches.append(f"{path.relative_to(repo_root)}:{lineno}")

    if matches:
        preview = ", ".join(matches[:8])
        if len(matches) > 8:
            preview += f" (+{len(matches) - 8} more)"
        return [CheckResult(
            severity="ERROR",
            category=CATEGORY,
            id="doctor_boundary.host-orchestrator-command-reference",
            message=(
                "active skill/MCP surfaces bind processkit remediation to a "
                f"host orchestrator command ({preview}); processkit must "
                "stay host-orchestrator neutral inside derived containers"
            ),
            suggested_fix=(
                "replace command-specific remediation with processkit-owned "
                "actions or generic external host-action evidence for the "
                "owner"
            ),
        )]

    return [CheckResult(
        severity="INFO",
        category=CATEGORY,
        id="doctor_boundary.clean",
        message=(
            "active skill/MCP surfaces keep processkit remediation "
            "host-orchestrator neutral and contain no forbidden host-command "
            "binding"
        ),
    )]
