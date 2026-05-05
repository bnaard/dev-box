#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "pyyaml>=6.0",
# ]
# ///
"""Create provider-neutral model profiles and repoint role/team bindings."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml


TIMESTAMP = "20260503_1832"
DECISION_ID = "DEC-20260503_1829-LoyalComet"

MODEL = "ART-20260503_1424-ModelSpec-{}"
PROFILE = "ART-{}-ModelProfile-{}"

PROFILES: dict[str, dict[str, Any]] = {
    "general-fast": {
        "name": "General fast model profile",
        "description": "Low-latency general assistant work.",
        "selection": {
            "model_classes": ["fast"],
            "min_dimensions": {"speed": 4, "reliability": 3},
            "effort_floor": "none",
            "effort_ceiling": "medium",
        },
        "candidates": [
            ("anthropic-claude-haiku", "anthropic"),
            ("openai-o4-mini", "openai"),
            ("google-gemini-2-5-flash", "google"),
            ("xai-grok-4-1-fast", "xai"),
            ("mistral-mistral-small", "mistral"),
        ],
    },
    "general-balanced": {
        "name": "General balanced model profile",
        "description": "General-purpose reasoning, planning, and synthesis.",
        "selection": {
            "model_classes": ["standard", "powerful"],
            "min_dimensions": {"reasoning": 4, "reliability": 4},
            "effort_floor": "medium",
            "effort_ceiling": "high",
        },
        "candidates": [
            ("anthropic-claude-sonnet", "anthropic"),
            ("openai-gpt-5", "openai"),
            ("google-gemini-2-5-pro", "google"),
            ("mistral-mistral-medium", "mistral"),
            ("xai-grok-4", "xai"),
        ],
    },
    "general-deep": {
        "name": "General deep model profile",
        "description": "High-effort cross-domain reasoning.",
        "selection": {
            "model_classes": ["powerful"],
            "min_dimensions": {"reasoning": 5, "reliability": 4},
            "effort_floor": "high",
            "effort_ceiling": "max",
        },
        "candidates": [
            ("anthropic-claude-opus", "anthropic"),
            ("openai-gpt-5-pro", "openai"),
            ("google-gemini-3-1-pro", "google"),
            ("xai-grok-4-heavy", "xai"),
            ("deepseek-deepseek-r", "deepseek"),
        ],
    },
    "code-fast": {
        "name": "Code fast model profile",
        "description": "Routine code edits, tests, and small reviews.",
        "selection": {
            "model_classes": ["fast", "standard"],
            "task_classes": ["coding", "debugging", "code_review"],
            "min_dimensions": {"engineering": 3, "speed": 4},
            "effort_floor": "low",
            "effort_ceiling": "medium",
        },
        "candidates": [
            ("anthropic-claude-haiku", "anthropic"),
            ("openai-o4-mini", "openai"),
            ("google-gemini-2-5-flash", "google"),
            ("alibaba-qwen2-5-coder-32b", "alibaba"),
            ("mistral-codestral", "mistral"),
        ],
    },
    "code-balanced": {
        "name": "Code balanced model profile",
        "description": "Repository-scale implementation and review.",
        "selection": {
            "model_classes": ["standard", "powerful"],
            "task_classes": ["coding", "architecture", "debugging"],
            "min_dimensions": {"engineering": 4, "reliability": 4},
            "effort_floor": "medium",
            "effort_ceiling": "high",
        },
        "candidates": [
            ("anthropic-claude-sonnet", "anthropic"),
            ("openai-gpt-5", "openai"),
            ("openai-gpt-5-2-codex", "openai"),
            ("google-gemini-2-5-pro", "google"),
            ("alibaba-qwen3-coder", "alibaba"),
            ("mistral-codestral", "mistral"),
        ],
    },
    "code-deep": {
        "name": "Code deep model profile",
        "description": "Architecture-scale engineering and complex debugging.",
        "selection": {
            "model_classes": ["powerful"],
            "task_classes": ["architecture", "coding", "debugging"],
            "min_dimensions": {"engineering": 5, "reasoning": 4},
            "effort_floor": "high",
            "effort_ceiling": "max",
        },
        "candidates": [
            ("anthropic-claude-opus", "anthropic"),
            ("openai-gpt-5-pro", "openai"),
            ("openai-gpt-5-2-codex", "openai"),
            ("google-gemini-3-1-pro", "google"),
            ("xai-grok-4-heavy", "xai"),
        ],
    },
    "research-deep": {
        "name": "Research deep model profile",
        "description": "Scientific, market, and technical research synthesis.",
        "selection": {
            "model_classes": ["powerful"],
            "task_classes": ["research", "scientific_reasoning", "summarization"],
            "min_dimensions": {"reasoning": 5, "breadth": 4},
            "effort_floor": "high",
            "effort_ceiling": "max",
        },
        "candidates": [
            ("anthropic-claude-opus", "anthropic"),
            ("openai-gpt-5-pro", "openai"),
            ("google-gemini-3-1-pro", "google"),
            ("deepseek-deepseek-r", "deepseek"),
            ("xai-grok-4-heavy", "xai"),
        ],
    },
    "writing-balanced": {
        "name": "Writing balanced model profile",
        "description": "Style-sensitive documentation and narrative synthesis.",
        "selection": {
            "model_classes": ["standard", "powerful"],
            "task_classes": ["summarization", "documentation", "writing"],
            "min_dimensions": {"reliability": 4, "breadth": 4},
            "effort_floor": "low",
            "effort_ceiling": "high",
        },
        "candidates": [
            ("anthropic-claude-sonnet", "anthropic"),
            ("openai-gpt-5", "openai"),
            ("google-gemini-2-5-pro", "google"),
            ("mistral-mistral-medium", "mistral"),
            ("cohere-command-r-plus", "cohere"),
        ],
    },
    "governed-deep": {
        "name": "Governed deep model profile",
        "description": "Security, compliance, and privacy-sensitive deep work.",
        "selection": {
            "model_classes": ["powerful"],
            "task_classes": ["security", "architecture", "privacy_sensitive"],
            "min_dimensions": {"reasoning": 4, "governance": 5},
            "effort_floor": "high",
            "effort_ceiling": "max",
        },
        "candidates": [
            ("anthropic-claude-opus", "anthropic"),
            ("anthropic-claude-sonnet", "anthropic"),
            ("meta-llama-4-maverick", "meta"),
            ("mistral-mistral-large", "mistral"),
            ("openai-gpt-5-pro", "openai"),
        ],
    },
}


ROLE_PROFILE = {
    "ROLE-software-engineer": {
        "junior": "code-fast",
        "specialist": "code-fast",
        "expert": "code-balanced",
        "senior": "code-balanced",
        "principal": "code-deep",
    },
    "ROLE-database-engineer": {"senior": "code-balanced"},
    "ROLE-machine-learning-engineer": {"senior": "code-deep"},
    "ROLE-qa-engineer": {
        "junior": "code-fast",
        "specialist": "code-fast",
        "expert": "code-balanced",
        "senior": "code-balanced",
        "principal": "code-deep",
    },
    "ROLE-technical-writer": {
        "junior": "general-fast",
        "specialist": "general-fast",
        "expert": "writing-balanced",
        "senior": "writing-balanced",
        "principal": "writing-balanced",
    },
    "ROLE-product-manager": {
        "junior": "general-fast",
        "specialist": "general-fast",
        "expert": "general-balanced",
        "senior": "general-balanced",
        "principal": "general-deep",
    },
    "ROLE-assistant": {
        "junior": "general-fast",
        "specialist": "general-fast",
        "expert": "general-fast",
        "senior": "general-balanced",
        "principal": "general-balanced",
    },
    "ROLE-data-scientist": {
        "junior": "general-fast",
        "specialist": "general-fast",
        "expert": "research-deep",
        "senior": "general-balanced",
        "principal": "research-deep",
    },
    "ROLE-research-scientist": {
        "junior": "research-deep",
        "specialist": "research-deep",
        "expert": "research-deep",
        "senior": "research-deep",
        "principal": "research-deep",
    },
    "ROLE-ai-research-scientist": {
        "junior": "research-deep",
        "specialist": "research-deep",
        "expert": "research-deep",
        "senior": "research-deep",
        "principal": "research-deep",
    },
    "ROLE-security-architect": {
        "junior": "governed-deep",
        "specialist": "governed-deep",
        "expert": "governed-deep",
        "senior": "governed-deep",
        "principal": "governed-deep",
    },
    "ROLE-solutions-architect": {
        "junior": "general-balanced",
        "specialist": "general-balanced",
        "expert": "general-deep",
        "senior": "general-deep",
        "principal": "general-deep",
    },
}

TEAM_PROFILE = {
    "TEAMMEMBER-cora": "general-balanced",
}


def profile_id(slug: str) -> str:
    return PROFILE.format(TIMESTAMP, slug)


def model_id(slug: str) -> str:
    return MODEL.format(slug)


def _frontmatter(entity: dict[str, Any]) -> str:
    return "---\n" + yaml.safe_dump(entity, sort_keys=False) + "---\n"


def write_profiles(repo_root: Path) -> None:
    for tree in ("context", "src/context"):
        artifact_dir = repo_root / tree / "artifacts"
        artifact_dir.mkdir(parents=True, exist_ok=True)
        for slug, data in PROFILES.items():
            art_id = profile_id(slug)
            candidates = []
            for rank, (model_slug, provider) in enumerate(data["candidates"], start=1):
                candidates.append({
                    "rank": rank,
                    "provider": provider,
                    "model_spec": model_id(model_slug),
                })
            entity = {
                "apiVersion": "processkit.projectious.work/v2",
                "kind": "Artifact",
                "metadata": {
                    "id": art_id,
                    "created": "2026-05-03T18:32:00Z",
                },
                "spec": {
                    "name": data["name"],
                    "kind": "model-profile",
                    "profile_id": slug,
                    "description": data["description"],
                    "selection": data["selection"],
                    "candidates": candidates,
                    "produced_by": DECISION_ID,
                    "tags": ["model-routing", "provider-neutral"],
                },
            }
            body = (
                f"# {data['name']}\n\n"
                f"{data['description']}\n\n"
                "This artifact is provider-neutral. Concrete provider/model "
                "choices live only in the candidate list and are selected "
                "after runtime access gates are applied.\n"
            )
            (artifact_dir / f"{art_id}.md").write_text(
                _frontmatter(entity) + body,
                encoding="utf-8",
            )


def _load_entity(path: Path) -> dict[str, Any] | None:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return None
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None
    data = yaml.safe_load(parts[1])
    return data if isinstance(data, dict) else None


def _write_entity(path: Path, data: dict[str, Any]) -> None:
    path.write_text(_frontmatter(data), encoding="utf-8")


def _binding_profile(subject: str, conditions: dict[str, Any]) -> str | None:
    if subject in TEAM_PROFILE:
        return TEAM_PROFILE[subject]
    seniority = conditions.get("seniority")
    role_profiles = ROLE_PROFILE.get(subject)
    if role_profiles and seniority in role_profiles:
        return role_profiles[seniority]
    return None


def rewrite_binding_file(path: Path) -> bool:
    data = _load_entity(path)
    if not data:
        return False
    spec = data.get("spec") if isinstance(data.get("spec"), dict) else {}
    if spec.get("type") != "model-assignment":
        return False
    subject = str(spec.get("subject") or "")
    target = str(spec.get("target") or "")
    if "-ModelSpec-" not in target and "-ModelProfile-" not in target:
        return False
    conditions = spec.get("conditions")
    if not isinstance(conditions, dict):
        conditions = {}
        spec["conditions"] = conditions
    profile_slug = _binding_profile(subject, conditions)
    if not profile_slug:
        if subject == "processkit" or subject.startswith("SCOPE-"):
            conditions.setdefault("direct_model_pin", True)
        return True
    spec["target"] = profile_id(profile_slug)
    spec["target_kind"] = "Artifact"
    seniority = conditions.get("seniority") or "default"
    conditions["rationale"] = (
        f"Provider-neutral {profile_slug} routing for {subject} "
        f"{seniority}; concrete model selected by runtime access gates."
    )
    spec["description"] = (
        f"Provider-neutral {profile_slug} model assignment for {subject} "
        f"{seniority}"
    )
    _write_entity(path, data)
    return True


def rewrite_bindings(repo_root: Path) -> None:
    for tree in ("context/bindings", "src/context/bindings"):
        root = repo_root / tree
        if not root.is_dir():
            continue
        for path in sorted(root.glob("*.md")):
            rewrite_binding_file(path)


def rewrite_manifest(repo_root: Path) -> None:
    path = (
        repo_root
        / "context/skills/processkit/model-recommender/default-bindings/MANIFEST.yaml"
    )
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    data["spec"]["description"] = (
        "Shipped defaults mapping (role, seniority) to "
        "Artifact(kind=model-profile) at rank 1. Each profile expands to "
        "provider-specific Artifact(kind=model-spec) candidates after runtime "
        "access gates are applied. Projects override by creating their own "
        "bindings or provider/access policy. See DEC-20260503_1829-LoyalComet."
    )
    for binding in data["spec"]["bindings"]:
        if binding.get("type") != "model-assignment":
            continue
        subject = binding.get("subject")
        conditions = binding.get("conditions") or {}
        profile_slug = _binding_profile(str(subject), conditions)
        if profile_slug:
            binding["target"] = profile_id(profile_slug)
    path.write_text(yaml.safe_dump(data, sort_keys=False), encoding="utf-8")

    src_path = (
        repo_root
        / "src/context/skills/processkit/model-recommender/default-bindings/MANIFEST.yaml"
    )
    src_path.write_text(path.read_text(encoding="utf-8"), encoding="utf-8")


def main() -> int:
    repo_root = Path(__file__).resolve().parents[5]
    write_profiles(repo_root)
    rewrite_bindings(repo_root)
    rewrite_manifest(repo_root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
