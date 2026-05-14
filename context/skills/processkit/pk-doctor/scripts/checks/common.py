"""Shared dataclasses + helpers for pk-doctor checks.

Every check module exports:

    def run(ctx) -> list[CheckResult]: ...

and optionally:

    def run_fix(ctx, results) -> list[dict]: ...

`ctx` is a dict produced by doctor.py with at minimum:

    {
      "repo_root":  Path to the repo root,
      "since":      Optional[str] git ref, or None,
      "yes":        bool, auto-confirm non-data-loss prompts,
      "interactive": bool, sys.stdin.isatty(),
    }

CheckResult is the single vocabulary for findings.
"""

from __future__ import annotations

from dataclasses import dataclass, field, asdict
from typing import Literal, Optional


Severity = Literal["ERROR", "WARN", "INFO"]
ActionKind = Literal[
    "safe_fix",
    "migration_needed",
    "archive_needed",
    "policy_decision_needed",
    "user_confirmation_needed",
    "external_dependency",
]
DefaultAgentAction = Literal[
    "fix_now",
    "create_workitem",
    "create_migration",
    "ask_user",
    "defer_with_reason",
]
AcceptableResolution = Literal[
    "fixed",
    "migrated",
    "archived",
    "linked_tracking_item",
]


@dataclass
class CheckResult:
    severity: Severity
    category: str
    id: str
    message: str
    entity_ref: Optional[str] = None
    fixable: bool = False
    suggested_fix: Optional[str] = None
    fix_mcp_tool: Optional[str] = None
    data_loss: bool = False
    action_required: Optional[bool] = None
    action_kind: Optional[ActionKind] = None
    default_agent_action: Optional[DefaultAgentAction] = None
    requires_user_confirmation: bool = False
    acceptable_resolution: Optional[AcceptableResolution] = None
    extra: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        d = asdict(self)
        action_required = self._action_required()
        d["action_required"] = action_required
        d["action_kind"] = self._action_kind() if action_required else None
        d["default_agent_action"] = (
            self._default_agent_action() if action_required else None
        )
        d["requires_user_confirmation"] = (
            self.requires_user_confirmation
            or d["action_kind"] in {
                "migration_needed",
                "archive_needed",
                "policy_decision_needed",
                "user_confirmation_needed",
            }
            or self.data_loss
        )
        d["acceptable_resolution"] = (
            self._acceptable_resolution() if action_required else None
        )
        if not d["extra"]:
            d.pop("extra")
        return d

    def _action_required(self) -> bool:
        if self.action_required is not None:
            return self.action_required
        return bool(
            self.severity in {"ERROR", "WARN"}
            or self.fixable
            or self.fix_mcp_tool
            or self.suggested_fix
        )

    def _action_kind(self) -> Optional[ActionKind]:
        if self.action_kind:
            return self.action_kind
        hint = " ".join(
            str(part).lower()
            for part in (
                self.id,
                self.message,
                self.suggested_fix,
                self.fix_mcp_tool,
            )
            if part
        )
        if "archive" in hint:
            return "archive_needed"
        if "migration" in hint or "migrate" in hint:
            return "migration_needed"
        if "external" in hint or "gh cli" in hint or "authenticated" in hint:
            return "external_dependency"
        if "policy" in hint or "grandfather" in hint:
            return "policy_decision_needed"
        if self.fixable and not self.data_loss:
            return "safe_fix"
        return "user_confirmation_needed"

    def _default_agent_action(self) -> DefaultAgentAction:
        if self.default_agent_action:
            return self.default_agent_action
        kind = self._action_kind()
        if kind == "safe_fix":
            return "fix_now"
        if kind == "migration_needed":
            return "create_migration"
        if kind in {
            "archive_needed",
            "policy_decision_needed",
            "user_confirmation_needed",
        }:
            return "ask_user"
        return "create_workitem"

    def _acceptable_resolution(self) -> AcceptableResolution:
        if self.acceptable_resolution:
            return self.acceptable_resolution
        kind = self._action_kind()
        if kind == "migration_needed":
            return "migrated"
        if kind == "archive_needed":
            return "archived"
        if kind == "policy_decision_needed":
            return "linked_tracking_item"
        if kind == "external_dependency":
            return "linked_tracking_item"
        return "fixed"


def tally(results: list[CheckResult]) -> dict[str, int]:
    """Return {'ERROR': n, 'WARN': n, 'INFO': n} for a list of results."""
    out = {"ERROR": 0, "WARN": 0, "INFO": 0}
    for r in results:
        out[r.severity] = out.get(r.severity, 0) + 1
    return out


def action_tally(results: list[CheckResult]) -> dict[str, int]:
    """Return machine-actionability counts for a list of findings."""
    out = {
        "actionable": 0,
        "needs_user_confirmation": 0,
        "needs_tracking": 0,
        "safe_fix": 0,
        "migration_needed": 0,
        "archive_needed": 0,
        "policy_decision_needed": 0,
        "external_dependency": 0,
    }
    for result in results:
        data = result.to_dict()
        if not data["action_required"]:
            continue
        out["actionable"] += 1
        kind = data.get("action_kind")
        if kind in out:
            out[kind] += 1
        if data.get("requires_user_confirmation"):
            out["needs_user_confirmation"] += 1
        if data.get("default_agent_action") in {
            "create_workitem",
            "defer_with_reason",
        }:
            out["needs_tracking"] += 1
    return out
