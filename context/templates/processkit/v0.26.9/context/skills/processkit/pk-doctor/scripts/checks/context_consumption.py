"""context_consumption check — estimate processkit context footprint.

This check is intentionally INFO-only. It measures likely startup and
processkit-adapter surfaces without adding a new MCP tool whose schema would
itself increase harness context.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

try:
    from .common import CheckResult
except ImportError:  # pragma: no cover - direct script invocation
    from common import CheckResult  # type: ignore


_STARTUP_PATHS = [
    Path("AGENTS.md"),
    Path(".claude/settings.json"),
    Path(".codex/config.toml"),
    Path("context/.processkit-mcp-manifest.json"),
]
_CHECKPOINT_DIR = Path("context/.state/context-consumption/checkpoints")
_LABEL_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,79}$")
_HEURISTIC = "ceil(utf8_bytes / 4)"
_BILLING_NOTICE = (
    "Local estimates only; not provider-billed token usage. Reports compare "
    "observed processkit payloads, not the counterfactual prompt delta a "
    "provider charged for."
)


def _measure_file(path: Path) -> dict | None:
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return None
    data = text.encode("utf-8")
    words = len(text.split())
    return {
        "path": path.as_posix(),
        "bytes": len(data),
        "lines": text.count("\n") + (0 if text.endswith("\n") or not text else 1),
        "words": words,
        "estimated_tokens": math.ceil(len(data) / 4),
    }


def _sum(items: list[dict]) -> dict:
    out = {"bytes": 0, "lines": 0, "words": 0, "estimated_tokens": 0}
    for item in items:
        for key in out:
            out[key] += int(item.get(key, 0))
    return out


def _delta(before: dict, after: dict) -> dict:
    keys = ("bytes", "lines", "words", "estimated_tokens")
    return {key: int(after.get(key, 0)) - int(before.get(key, 0)) for key in keys}


def _validate_label(label: str) -> str:
    if not _LABEL_RE.fullmatch(label):
        raise ValueError(
            "checkpoint labels must be 1-80 chars: letters, digits, '.', '_' or '-'"
        )
    return label


def _checkpoint_path(repo_root: Path, label: str) -> Path:
    return repo_root / _CHECKPOINT_DIR / f"{_validate_label(label)}.json"


def _collect_files(repo_root: Path) -> dict[str, list[dict]]:
    startup: list[dict] = []
    for rel in _STARTUP_PATHS:
        item = _measure_file(repo_root / rel)
        if item:
            item["path"] = rel.as_posix()
            startup.append(item)

    commands: list[dict] = []
    for root in (repo_root / ".agents" / "skills", repo_root / ".claude" / "commands"):
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*.md")):
            item = _measure_file(path)
            if item:
                item["path"] = path.relative_to(repo_root).as_posix()
                commands.append(item)

    skill_docs: list[dict] = []
    for path in sorted((repo_root / "context" / "skills" / "processkit").glob("*/*.md")):
        if path.name != "SKILL.md":
            continue
        item = _measure_file(path)
        if item:
            item["path"] = path.relative_to(repo_root).as_posix()
            skill_docs.append(item)

    mcp_configs: list[dict] = []
    for path in sorted((repo_root / "context" / "skills").glob("*/*/mcp/mcp-config.json")):
        item = _measure_file(path)
        if item:
            item["path"] = path.relative_to(repo_root).as_posix()
            mcp_configs.append(item)

    return {
        "startup": startup,
        "commands": commands,
        "skill_docs": skill_docs,
        "mcp_configs": mcp_configs,
    }


def create_snapshot(
    repo_root: Path,
    label: str,
    *,
    now: datetime | None = None,
) -> dict[str, Any]:
    """Create an in-memory context-consumption snapshot."""
    groups = _collect_files(repo_root)
    totals = {name: _sum(items) for name, items in groups.items()}
    overall = _sum(list(totals.values()))
    timestamp = now or datetime.now(timezone.utc)
    return {
        "snapshot_version": 1,
        "label": _validate_label(label),
        "created_at": timestamp.isoformat().replace("+00:00", "Z"),
        "heuristic": _HEURISTIC,
        "billing_notice": _BILLING_NOTICE,
        "totals": totals,
        "overall": overall,
        "groups": groups,
    }


def write_checkpoint(repo_root: Path, label: str) -> tuple[Path, dict[str, Any]]:
    """Persist a named checkpoint under processkit local state."""
    payload = create_snapshot(repo_root, label)
    path = _checkpoint_path(repo_root, label)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n",
                    encoding="utf-8")
    return path, payload


def load_checkpoint(repo_root: Path, label: str) -> dict[str, Any]:
    path = _checkpoint_path(repo_root, label)
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise FileNotFoundError(f"checkpoint not found: {label}") from exc


def _files_by_path(snapshot: dict[str, Any]) -> dict[str, dict[str, Any]]:
    out: dict[str, dict[str, Any]] = {}
    for group, items in snapshot.get("groups", {}).items():
        for item in items:
            row = dict(item)
            row["group"] = group
            out[str(item["path"])] = row
    return out


def compare_checkpoints(repo_root: Path, before: str, after: str) -> dict[str, Any]:
    """Return a provider-neutral delta report for two checkpoint labels."""
    before_snap = load_checkpoint(repo_root, before)
    after_snap = load_checkpoint(repo_root, after)

    group_names = sorted(
        set(before_snap.get("totals", {})) | set(after_snap.get("totals", {}))
    )
    groups = {
        name: {
            "before": before_snap.get("totals", {}).get(name, {}),
            "after": after_snap.get("totals", {}).get(name, {}),
            "delta": _delta(
                before_snap.get("totals", {}).get(name, {}),
                after_snap.get("totals", {}).get(name, {}),
            ),
        }
        for name in group_names
    }

    before_files = _files_by_path(before_snap)
    after_files = _files_by_path(after_snap)
    file_deltas: list[dict[str, Any]] = []
    for path in sorted(set(before_files) | set(after_files)):
        b = before_files.get(path, {})
        a = after_files.get(path, {})
        delta = _delta(b, a)
        status = "changed"
        if not b:
            status = "added"
        elif not a:
            status = "removed"
        if status != "changed" or any(delta.values()):
            file_deltas.append({
                "path": path,
                "group": a.get("group") or b.get("group"),
                "status": status,
                "delta": delta,
                "before": b,
                "after": a,
            })

    file_deltas.sort(
        key=lambda item: abs(item["delta"]["estimated_tokens"]),
        reverse=True,
    )

    return {
        "report_version": 1,
        "heuristic": _HEURISTIC,
        "billing_notice": _BILLING_NOTICE,
        "comparison": {
            "before": {
                "label": before_snap.get("label", before),
                "created_at": before_snap.get("created_at"),
            },
            "after": {
                "label": after_snap.get("label", after),
                "created_at": after_snap.get("created_at"),
            },
        },
        "totals": {
            "before": before_snap.get("overall", {}),
            "after": after_snap.get("overall", {}),
            "delta": _delta(
                before_snap.get("overall", {}),
                after_snap.get("overall", {}),
            ),
        },
        "groups": groups,
        "top_file_deltas": file_deltas[:20],
    }


def _signed(n: int) -> str:
    return f"+{n}" if n > 0 else str(n)


def format_checkpoint(path: Path, payload: dict[str, Any]) -> str:
    overall = payload["overall"]
    return (
        f"checkpoint: {payload['label']}\n"
        f"path:       {path}\n"
        f"estimate:   {overall['estimated_tokens']} tokens, "
        f"{overall['bytes']} bytes across "
        f"{sum(len(v) for v in payload['groups'].values())} files\n"
        f"notice:     {_BILLING_NOTICE}\n"
    )


def format_report(report: dict[str, Any]) -> str:
    lines = [
        "# context_consumption checkpoint report",
        (
            "comparison: "
            f"{report['comparison']['before']['label']} -> "
            f"{report['comparison']['after']['label']}"
        ),
        f"heuristic:  {report['heuristic']}",
        f"notice:     {report['billing_notice']}",
        "",
    ]
    delta = report["totals"]["delta"]
    lines.append(
        "overall:    "
        f"{_signed(delta['estimated_tokens'])} estimated tokens, "
        f"{_signed(delta['bytes'])} bytes, {_signed(delta['lines'])} lines"
    )
    lines.append("")
    lines.append("groups:")
    for name, group in report["groups"].items():
        gd = group["delta"]
        lines.append(
            f"  - {name}: {_signed(gd['estimated_tokens'])} tokens, "
            f"{_signed(gd['bytes'])} bytes"
        )
    if report["top_file_deltas"]:
        lines.append("")
        lines.append("top file deltas:")
        for item in report["top_file_deltas"][:10]:
            fd = item["delta"]
            lines.append(
                f"  - {item['path']} ({item['status']}): "
                f"{_signed(fd['estimated_tokens'])} tokens, "
                f"{_signed(fd['bytes'])} bytes"
            )
    return "\n".join(lines) + "\n"


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    groups = _collect_files(repo_root)
    totals = {name: _sum(items) for name, items in groups.items()}
    overall = _sum(list(totals.values()))

    return [CheckResult(
        severity="INFO",
        category="context_consumption",
        id="context_consumption.estimated",
        message=(
            "estimated processkit context footprint: "
            f"{overall['estimated_tokens']} tokens across "
            f"{sum(len(items) for items in groups.values())} files "
            "(heuristic: ceil(utf8 bytes / 4))"
        ),
        extra={
            "heuristic": _HEURISTIC,
            "billing_notice": _BILLING_NOTICE,
            "checkpoints_dir": _CHECKPOINT_DIR.as_posix(),
            "groups": totals,
        },
    )]


def _parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="context_consumption",
        description="Estimate processkit context footprint and checkpoint deltas.",
    )
    parser.add_argument("--repo-root", default=".",
                        help="Repository root. Defaults to the current directory.")
    parser.add_argument("--json", action="store_true",
                        help="Emit machine-readable JSON.")
    sub = parser.add_subparsers(dest="command", required=True)

    checkpoint = sub.add_parser("checkpoint", help="write a named checkpoint")
    checkpoint.add_argument("label")

    report = sub.add_parser("report", help="compare two named checkpoints")
    report.add_argument("before")
    report.add_argument("after")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv or sys.argv[1:])
    repo_root = Path(args.repo_root).resolve()
    try:
        if args.command == "checkpoint":
            path, payload = write_checkpoint(repo_root, args.label)
            if args.json:
                print(json.dumps({"path": str(path), "checkpoint": payload},
                                 indent=2, sort_keys=True))
            else:
                print(format_checkpoint(path, payload), end="")
            return 0
        if args.command == "report":
            payload = compare_checkpoints(repo_root, args.before, args.after)
            if args.json:
                print(json.dumps(payload, indent=2, sort_keys=True))
            else:
                print(format_report(payload), end="")
            return 0
    except (FileNotFoundError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
