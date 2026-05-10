"""Tests for team-manager skill.

Run with:

    uv run --with pyyaml --with jsonschema --with pytest --with mcp \
        pytest context/skills/processkit/team-manager/scripts/test_team_manager.py -v

Covers:
  - create/get/update/deactivate/reactivate roundtrip
  - list with all filter combos
  - reserve/release/suggest/list_available_names
  - off-pool rejection for ai-agent
  - pool exhaustion
  - init_memory_tree creates all expected subdirs
  - Each of the 10 consistency checks: passing + failing case
  - export + import roundtrip preserves expected structure
  - export redacts confidential/pii files
  - export excludes journal/, relations/, private/
"""
from __future__ import annotations

import json
import math
import os
import shutil
import sys
from pathlib import Path

import pytest

# --- Path bootstrap so we can import scripts and the processkit lib -----------
_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

# Find the processkit lib (mirrors server.py's _find_lib).
def _find_lib() -> Path:
    here = _SCRIPTS_DIR
    while True:
        for c in (here / "src" / "lib", here / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


_LIB = _find_lib()
if str(_LIB) not in sys.path:
    sys.path.insert(0, str(_LIB))

import consistency  # noqa: E402
import export_import  # noqa: E402
import memory_tree  # noqa: E402
import name_pool  # noqa: E402
from processkit import entity as _entity_mod  # noqa: E402


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

@pytest.fixture()
def project_root(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """Create a throwaway project root with AGENTS.md + schemas symlinked in."""
    (tmp_path / "AGENTS.md").write_text("# test project\n")
    (tmp_path / "context").mkdir()

    # Mirror just the team-member schema into the consumer layout so
    # processkit.schema.load_schema("TeamMember") works.
    schemas_dir = tmp_path / "context" / "schemas"
    schemas_dir.mkdir()
    src_schema = _find_repo_root() / "context" / "schemas" / "team-member.yaml"
    shutil.copy(src_schema, schemas_dir / "team-member.yaml")
    # Also copy role schema (for completeness)
    role_schema = _find_repo_root() / "context" / "schemas" / "role.yaml"
    if role_schema.is_file():
        shutil.copy(role_schema, schemas_dir / "role.yaml")
    # RoleSlot + Binding + Scope schemas — needed by the Phase A
    # team-creator v2 RoleSlot tools (DEC-20260509_1906-CoolBadger).
    for extra in ("roleslot.yaml", "binding.yaml", "scope.yaml"):
        src_extra = _find_repo_root() / "context" / "schemas" / extra
        if src_extra.is_file():
            shutil.copy(src_extra, schemas_dir / extra)

    monkeypatch.chdir(tmp_path)
    # Force processkit.paths.find_project_root to discover AGENTS.md in tmp_path
    # (it walks from cwd).
    # Invalidate schema LRU cache so the test schema is reloaded fresh.
    from processkit import schema as _schema_mod
    _schema_mod.load_schema.cache_clear()
    return tmp_path


def _find_repo_root() -> Path:
    here = Path(__file__).resolve()
    for ancestor in [here, *here.parents]:
        if (ancestor / "AGENTS.md").is_file() and (ancestor / "src").is_dir():
            return ancestor
    raise RuntimeError("repo root not found")


@pytest.fixture()
def pool_path(tmp_path: Path) -> Path:
    """Copy a fresh name-pool.yaml into tmp_path/data/ so tests mutate a throwaway file."""
    src = Path(__file__).resolve().parent.parent / "data" / "name-pool.yaml"
    dest = tmp_path / "data" / "name-pool.yaml"
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(src, dest)
    return dest


@pytest.fixture()
def assets_dir() -> Path:
    return Path(__file__).resolve().parent.parent / "assets"


@pytest.fixture()
def server_mod(project_root: Path):
    """Import the MCP server fresh so its tool callables bind to project_root.

    The module caches FastMCP state at import time, so we reset sys.modules
    and re-import for each test to keep them isolated.
    """
    for mod in list(sys.modules):
        if mod in {"server", "consistency", "memory_tree", "name_pool", "export_import"}:
            # Keep helper modules; only reload server.
            if mod == "server":
                del sys.modules[mod]
    mcp_dir = Path(__file__).resolve().parent.parent / "mcp"
    if str(mcp_dir) not in sys.path:
        sys.path.insert(0, str(mcp_dir))
    import server  # noqa: F401
    # Point the skill's pool_path helper at a throwaway pool for create_team_member
    pool_src = Path(__file__).resolve().parent.parent / "data" / "name-pool.yaml"
    pool_copy = project_root / "pool.yaml"
    shutil.copy(pool_src, pool_copy)
    server._pool_path = lambda: pool_copy  # type: ignore[attr-defined]
    return server


# ---------------------------------------------------------------------------
# name_pool
# ---------------------------------------------------------------------------

def test_name_pool_list_available_all(pool_path: Path):
    pool = name_pool.load_pool(pool_path)
    names = name_pool.available(pool)
    assert len(names) > 50
    assert "Alice" in names


def test_name_pool_filter_by_kind(pool_path: Path):
    pool = name_pool.load_pool(pool_path)
    fem = name_pool.available(pool, "feminine")
    masc = name_pool.available(pool, "masculine")
    neut = name_pool.available(pool, "neutral")
    assert "Alice" in fem and "Alice" not in masc
    assert "Adam" in masc and "Adam" not in fem
    assert "Kai" in neut and "Kai" not in fem


def test_name_pool_reserve_release(pool_path: Path):
    name_pool.reserve(pool_path, "Alice", "alice-chen")
    pool = name_pool.load_pool(pool_path)
    assert name_pool.is_reserved(pool, "Alice") == "alice-chen"
    assert "Alice" not in name_pool.available(pool)

    name_pool.release(pool_path, "Alice")
    pool = name_pool.load_pool(pool_path)
    assert name_pool.is_reserved(pool, "Alice") is None
    assert "Alice" in name_pool.available(pool)


def test_name_pool_reserve_not_in_pool(pool_path: Path):
    with pytest.raises(name_pool.NamePoolError):
        name_pool.reserve(pool_path, "Zorblax", "x")


def test_name_pool_double_reserve_conflict(pool_path: Path):
    name_pool.reserve(pool_path, "Alice", "alice-a")
    with pytest.raises(name_pool.NamePoolError):
        name_pool.reserve(pool_path, "Alice", "alice-b")


def test_name_pool_reserve_same_slug_idempotent(pool_path: Path):
    name_pool.reserve(pool_path, "Alice", "alice-a")
    name_pool.reserve(pool_path, "Alice", "alice-a")


def test_name_pool_suggest(pool_path: Path):
    pool = name_pool.load_pool(pool_path)
    n = name_pool.suggest(pool, "feminine")
    assert n in name_pool.available(pool, "feminine")


def test_name_pool_exhaustion(pool_path: Path):
    pool = name_pool.load_pool(pool_path)
    fem = name_pool.available(pool, "feminine")
    for idx, n in enumerate(fem):
        name_pool.reserve(pool_path, n, f"slug-{idx}")
    pool = name_pool.load_pool(pool_path)
    assert name_pool.suggest(pool, "feminine") is None
    assert name_pool.available(pool, "feminine") == []


def test_name_pool_release_unreserved_is_noop(pool_path: Path):
    name_pool.release(pool_path, "Alice")  # never reserved — should not raise


# ---------------------------------------------------------------------------
# memory_tree
# ---------------------------------------------------------------------------

def test_init_memory_tree_creates_subdirs(tmp_path: Path):
    tm_dir = tmp_path / "atlas"
    created = memory_tree.init_tree(tm_dir, "atlas")
    for tier in memory_tree.TIER_SUBDIRS + ("private",):
        assert (tm_dir / tier).is_dir()
        assert (tm_dir / tier / ".gitkeep").is_file()
    assert (tm_dir / "persona.md").is_file()
    assert (tm_dir / "card.json").is_file()
    # Idempotent second call
    created2 = memory_tree.init_tree(tm_dir, "atlas")
    assert created2 == []


def test_init_memory_tree_uses_entity_name_and_role(tmp_path: Path):
    tm_dir = tmp_path / "atlas"
    ent = _entity_mod.new(
        "TeamMember",
        "TEAMMEMBER-atlas",
        {
            "type": "ai-agent",
            "name": "Atlas",
            "slug": "atlas",
            "default_role": "ROLE-architect",
            "default_seniority": "expert",
            "active": True,
        },
    )
    memory_tree.init_tree(tm_dir, "atlas", ent)
    card = json.loads((tm_dir / "card.json").read_text())
    assert card["name"] == "Atlas"
    assert card["role"] == "ROLE-architect"
    assert card["seniority"] == "expert"


# ---------------------------------------------------------------------------
# Lifecycle (via server module)
# ---------------------------------------------------------------------------

def test_create_human_roundtrip(server_mod, project_root: Path):
    r = server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        email="alice@example.com",
        default_role="ROLE-developer",
        default_seniority="senior",
    )
    assert r.get("id") == "TEAMMEMBER-alice-chen"
    assert Path(r["path"]).is_file()

    got = server_mod.get_team_member("alice-chen")
    assert got["name"] == "Alice Chen"
    assert got["type"] == "human"
    assert got["slug"] == "alice-chen"
    assert got["email"] == "alice@example.com"
    assert got["default_seniority"] == "senior"


def test_create_ai_agent_from_pool_auto_reserves(server_mod, project_root: Path):
    r = server_mod.create_team_member(
        name="Alice", type="ai-agent", slug="alice",
        default_role="ROLE-architect",
    )
    assert "error" not in r, r
    pool = name_pool.load_pool(server_mod._pool_path())
    assert name_pool.is_reserved(pool, "Alice") == "alice"


def test_create_ai_agent_off_pool_rejected(server_mod, project_root: Path):
    r = server_mod.create_team_member(
        name="Atlas", type="ai-agent", slug="atlas",
    )
    assert "error" in r
    assert "name pool" in r["error"]


def test_create_ai_agent_in_pool(server_mod, project_root: Path):
    r = server_mod.create_team_member(
        name="Alice", type="ai-agent", slug="alice",
    )
    assert r.get("id") == "TEAMMEMBER-alice", r
    pool = name_pool.load_pool(server_mod._pool_path())
    assert name_pool.is_reserved(pool, "Alice") == "alice"


def test_create_invalid_type_rejected(server_mod):
    r = server_mod.create_team_member(name="x", type="bogus", slug="x")
    assert "error" in r


def test_create_invalid_slug_rejected(server_mod):
    r = server_mod.create_team_member(name="x", type="human", slug="Not-Valid!")
    assert "error" in r


def test_create_duplicate_slug_rejected(server_mod):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    r = server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    assert "error" in r
    assert "already exists" in r["error"]


def test_update_team_member(server_mod):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    r = server_mod.update_team_member(
        "alice-chen", email="a@b.com", handle="alice", default_seniority="expert"
    )
    assert r.get("ok") is True
    assert set(r["updated"]) == {"email", "handle", "default_seniority"}
    got = server_mod.get_team_member("alice-chen")
    assert got["email"] == "a@b.com"
    assert got["handle"] == "alice"
    assert got["default_seniority"] == "expert"


def test_deactivate_reactivate(server_mod):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    r = server_mod.deactivate_team_member("alice-chen")
    assert r.get("ok") is True
    got = server_mod.get_team_member("alice-chen")
    assert got["active"] is False
    assert got["left_at"]
    r2 = server_mod.reactivate_team_member("alice-chen")
    assert r2.get("ok") is True
    got = server_mod.get_team_member("alice-chen")
    assert got["active"] is True
    assert got["left_at"] is None


def test_list_team_members_filters(server_mod):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    server_mod.create_team_member(name="Bob Lee", type="human", slug="bob-lee",
                                  default_role="ROLE-developer")
    server_mod.create_team_member(name="Alice", type="ai-agent", slug="alice")
    server_mod.deactivate_team_member("bob-lee")

    all_active = server_mod.list_team_members()
    slugs = {m["slug"] for m in all_active}
    assert "alice-chen" in slugs
    assert "alice" in slugs
    assert "bob-lee" not in slugs

    all_including_inactive = server_mod.list_team_members(active_only=False)
    assert "bob-lee" in {m["slug"] for m in all_including_inactive}

    humans = server_mod.list_team_members(type="human")
    assert {m["type"] for m in humans} == {"human"}

    agents = server_mod.list_team_members(type="ai-agent")
    assert {m["type"] for m in agents} == {"ai-agent"}

    by_role = server_mod.list_team_members(active_only=False, role="ROLE-developer")
    assert [m["slug"] for m in by_role] == ["bob-lee"]


def test_get_nonexistent(server_mod):
    r = server_mod.get_team_member("nobody")
    assert "error" in r


def test_active_interlocutor_roundtrip(server_mod, project_root: Path):
    server_mod.create_team_member(
        name="Alice", type="ai-agent", slug="alice",
        default_role="ROLE-product-manager",
        default_seniority="senior",
    )
    r = server_mod.set_active_interlocutor("alice")
    assert r["ok"] is True
    assert r["interlocutor"]["speaker_prefix"] == "Alice [TEAMMEMBER-alice]"

    got = server_mod.get_active_interlocutor()
    assert got["configured"] is True
    assert got["interlocutor"]["id"] == "TEAMMEMBER-alice"
    assert (project_root / "context" / "team" / "session-identity.json").is_file()


def test_interlocutor_runtime_binding_reports_mismatch(
    server_mod,
    monkeypatch: pytest.MonkeyPatch,
):
    server_mod.create_team_member(
        name="Alice", type="ai-agent", slug="alice",
        default_role="ROLE-product-manager",
        default_seniority="senior",
    )
    server_mod.set_active_interlocutor("alice")

    class Candidate:
        model_id = "ART-20260503_1424-ModelSpec-openai-gpt-5"
        version_id = "5"
        effort = "medium"
        rank = 1
        source_layer = 2
        rationale = "team-member preference"
        profile_id = "ART-20260503_1832-ModelProfile-general-balanced"
        profile_rank = 1

    class Resolver:
        @staticmethod
        def _runtime_context(task_hints=None):
            return {
                "harnesses": ["codex"],
                "allowed_providers": ["openai"],
                "preferred_providers": ["openai"],
                "provider_source": "aibox.toml ai.harnesses",
            }

        @staticmethod
        def resolve(**kwargs):
            return [Candidate()], [{"step": 2, "action": "team_member_preference"}]

    monkeypatch.setattr(server_mod, "_load_model_resolver", lambda: Resolver)
    monkeypatch.setattr(
        server_mod,
        "_model_spec_from_id",
        lambda model_id: {
            "provider": "openai",
            "family": "gpt-5",
            "profile_ids": ["gpt-5"],
            "versions": [{"version_id": "5"}],
        },
    )

    got = server_mod.get_interlocutor_runtime_binding(
        observed_model="gpt-5.5",
        observed_effort="high",
    )

    assert got["configured"] is True
    assert got["binding"]["policy"] == "capability-negotiated"
    assert got["binding"]["mode"] == "launch-conform"
    assert got["binding"]["capabilities"]["subagent_mcp_supported"] is False
    assert got["binding"]["resolved"]["model_id"].endswith("openai-gpt-5")
    assert got["binding"]["mismatch"]["severity"] == "warn"
    assert got["binding"]["mismatch"]["model"] is False
    assert got["binding"]["mismatch"]["effort"] is False


def test_active_interlocutor_rejects_inactive(server_mod):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    server_mod.deactivate_team_member("alice-chen")
    r = server_mod.set_active_interlocutor("alice-chen")
    assert "error" in r
    assert "inactive" in r["error"]


# ---------------------------------------------------------------------------
# init_memory_tree tool
# ---------------------------------------------------------------------------

def test_init_memory_tree_tool(server_mod, project_root: Path):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    r = server_mod.init_memory_tree("alice-chen")
    p = Path(r["path"])
    for tier in memory_tree.TIER_SUBDIRS:
        assert (p / tier).is_dir()
        assert (p / tier / ".gitkeep").is_file()


# ---------------------------------------------------------------------------
# Name pool tools
# ---------------------------------------------------------------------------

def test_suggest_and_reserve_tool(server_mod):
    r = server_mod.suggest_name("feminine")
    assert r["kind"] == "feminine"
    name = r["name"]
    s = server_mod.reserve_name(name, f"{name.lower()}-01")
    assert s["ok"] is True
    avail = server_mod.list_available_names("feminine")
    assert name not in avail
    rr = server_mod.release_name(name)
    assert rr["ok"] is True
    assert name in server_mod.list_available_names("feminine")


# ---------------------------------------------------------------------------
# Consistency checks — 10 codes, one passing + one failing each
# ---------------------------------------------------------------------------

def _setup_alice(server_mod, project_root: Path):
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role=None, default_seniority="specialist",
    )
    server_mod.init_memory_tree("alice-chen")
    return project_root / "context" / "team-members" / "alice-chen"


def _check_codes(findings, code):
    return [f for f in findings if f["code"] == code]


def test_check_schema_drift_pass(server_mod, project_root: Path, pool_path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.schema") == []


def test_check_schema_drift_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    # Break the entity: remove required 'type' field
    tm_md = tm_dir / "team-member.md"
    ent = _entity_mod.load(tm_md)
    ent.spec.pop("type")
    ent.write()
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.schema")


def test_check_tier_missing_pass(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.tier_missing") == []


def test_check_tier_missing_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    shutil.rmtree(tm_dir / "knowledge")
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.tier_missing")


def test_check_dangling_ref_fail(server_mod, project_root: Path, assets_dir):
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-ghost",
    )
    tm_dir = project_root / "context" / "team-members" / "alice-chen"
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.dangling_ref")


def test_check_dangling_ref_pass(server_mod, project_root: Path, assets_dir):
    # Create a role file so the reference resolves
    roles_dir = project_root / "context" / "roles"
    roles_dir.mkdir(parents=True, exist_ok=True)
    (roles_dir / "ROLE-developer.md").write_text(
        "---\n"
        "apiVersion: processkit.projectious.work/v2\n"
        "kind: Role\n"
        "metadata:\n"
        "  id: ROLE-developer\n"
        "  created: 2026-04-22T00:00:00+00:00\n"
        "spec:\n"
        "  name: developer\n"
        "---\n"
    )
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-developer",
    )
    tm_dir = project_root / "context" / "team-members" / "alice-chen"
    server_mod.init_memory_tree("alice-chen")
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.dangling_ref") == []


def test_check_name_collision(server_mod, project_root: Path, assets_dir):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-a")
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-b")
    server_mod.init_memory_tree("alice-a")
    server_mod.init_memory_tree("alice-b")
    report = consistency.check_all(
        project_root,
        project_root / "context" / "team-members",
        server_mod._pool_path(),
        assets_dir,
    )
    collisions = [
        f for findings in report["members"].values()
        for f in findings
        if f["code"] == "team.name_collision"
    ]
    assert collisions


def test_check_name_collision_pass(server_mod, project_root: Path, assets_dir):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    server_mod.create_team_member(name="Bob Lee", type="human", slug="bob-lee")
    server_mod.init_memory_tree("alice-chen")
    server_mod.init_memory_tree("bob-lee")
    report = consistency.check_all(
        project_root,
        project_root / "context" / "team-members",
        server_mod._pool_path(),
        assets_dir,
    )
    collisions = [
        f for findings in report["members"].values()
        for f in findings
        if f["code"] == "team.name_collision"
    ]
    assert collisions == []


def test_check_name_off_pool_fail(server_mod, project_root: Path, assets_dir):
    # Hand-craft an ai-agent entity with an off-pool name, bypassing create_team_member.
    tm_dir = project_root / "context" / "team-members" / "zz"
    tm_dir.mkdir(parents=True)
    memory_tree.init_tree(tm_dir, "zz")
    ent = _entity_mod.new(
        "TeamMember",
        "TEAMMEMBER-zz",
        {"type": "ai-agent", "name": "Zzz", "slug": "zz", "active": True},
    )
    ent.write(tm_dir / "team-member.md")
    findings = consistency.check_one(project_root, tm_dir, "zz",
                                     server_mod._pool_path(), assets_dir)
    off = _check_codes(findings, "team.name.off_pool")
    assert off


def test_check_name_off_pool_pass(server_mod, project_root: Path, assets_dir):
    server_mod.create_team_member(name="Alice", type="ai-agent", slug="alice")
    tm_dir = project_root / "context" / "team-members" / "alice"
    server_mod.init_memory_tree("alice")
    findings = consistency.check_one(project_root, tm_dir, "alice",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.name.off_pool") == []


def test_check_orphan_file_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    (tm_dir / "random.txt").write_text("stray")
    (tm_dir / "stray_dir").mkdir()
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    orphans = _check_codes(findings, "team.drift.orphan_file")
    assert any("random.txt" in f["path"] for f in orphans)
    assert any("stray_dir" in f["path"] for f in orphans)


def test_check_orphan_file_pass(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.orphan_file") == []


def test_check_leak_risk_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    bad = tm_dir / "knowledge" / "secrets.md"
    bad.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: confidential\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "secret\n"
    )
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.sensitivity.leak_risk")


def test_check_leak_risk_pass(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    ok = tm_dir / "knowledge" / "stack.md"
    ok.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: public\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "ok\n"
    )
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.sensitivity.leak_risk") == []


def test_check_private_not_ignored_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    # No .gitignore at all → expect warning
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.private.not_ignored")


def test_check_private_ignored_pass(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    (project_root / ".gitignore").write_text("**/private/\n")
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.private.not_ignored") == []


def test_check_memory_bad_header_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    bad = tm_dir / "knowledge" / "note.md"
    bad.write_text("no frontmatter at all\n")
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.memory.bad_header")


def test_check_memory_bad_header_pass(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    good = tm_dir / "knowledge" / "note.md"
    good.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: public\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "body\n"
    )
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.memory.bad_header") == []


def test_check_card_stale_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    card = tm_dir / "card.json"
    card.write_text(json.dumps({
        "schemaVersion": "a2a/v0.3",
        "name": "WRONG",
        "role": "ROLE-other",
        "seniority": "junior",
        "skills": [],
        "signature": {"alg": "none", "value": ""},
    }, indent=2))
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.card.stale")


def test_check_card_stale_pass(server_mod, project_root: Path, assets_dir):
    # Freshly-initted card matches entity — should pass.
    tm_dir = _setup_alice(server_mod, project_root)
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.card.stale") == []


# ---------------------------------------------------------------------------
# Export / Import
# ---------------------------------------------------------------------------

def test_export_roundtrip_excludes_journal_relations_private(
    server_mod, project_root: Path, tmp_path: Path
):
    tm_dir = _setup_alice(server_mod, project_root)
    # Populate each tier with a file
    for tier in ("knowledge", "skills", "lessons", "journal", "relations"):
        f = tm_dir / tier / "note.md"
        f.write_text(
            "---\n"
            f"tier: {tier if tier != 'knowledge' else 'semantic'}\n"
            "source: conversation\n"
            "sensitivity: public\n"
            "created: 2026-04-22T00:00:00+00:00\n"
            "---\n"
            f"body {tier}\n"
        )
    (tm_dir / "private" / "secret.md").write_text("secret\n")

    out = tmp_path / "bundle.tar.gz"
    summary = export_import.export(tm_dir, out)
    assert out.is_file()

    # Inspect contents
    import tarfile
    with tarfile.open(out) as tar:
        names = tar.getnames()
    # Should include knowledge/, skills/, lessons/ and top-level files
    assert any("alice-chen/knowledge/note.md" in n for n in names)
    assert any("alice-chen/skills/note.md" in n for n in names)
    assert any("alice-chen/lessons/note.md" in n for n in names)
    # Should NOT include journal/, relations/, private/
    assert not any("alice-chen/journal/" in n for n in names), names
    assert not any("alice-chen/relations/" in n for n in names), names
    assert not any("alice-chen/private/" in n for n in names), names


def test_export_redacts_confidential(server_mod, project_root: Path, tmp_path: Path):
    tm_dir = _setup_alice(server_mod, project_root)
    public = tm_dir / "knowledge" / "public.md"
    public.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: public\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "public\n"
    )
    confidential = tm_dir / "knowledge" / "secret.md"
    confidential.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: confidential\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "hush\n"
    )
    pii = tm_dir / "knowledge" / "pii.md"
    pii.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: pii\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "1234\n"
    )

    out = tmp_path / "bundle.tar.gz"
    summary = export_import.export(tm_dir, out)
    import tarfile
    with tarfile.open(out) as tar:
        names = tar.getnames()
    assert any("knowledge/public.md" in n for n in names)
    assert not any("knowledge/secret.md" in n for n in names)
    assert not any("knowledge/pii.md" in n for n in names)
    assert any("secret.md" in r for r in summary["redacted"])
    assert any("pii.md" in r for r in summary["redacted"])


def test_import_roundtrip(server_mod, project_root: Path, tmp_path: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    # Ensure the card has a signature field
    card = json.loads((tm_dir / "card.json").read_text())
    card["signature"] = {"alg": "none", "value": ""}
    (tm_dir / "card.json").write_text(json.dumps(card, indent=2))

    out = tmp_path / "bundle.tar.gz"
    export_import.export(tm_dir, out)

    # Remove the original so import into the same project succeeds
    shutil.rmtree(tm_dir)
    assert not (project_root / "context" / "team-members" / "alice-chen").exists()

    result = export_import.import_bundle(
        out, project_root / "context" / "team-members", assets_dir,
    )
    assert result["slug"] == "alice-chen"
    assert (project_root / "context" / "team-members" / "alice-chen" / "team-member.md").is_file()


def test_import_rejects_missing_signature(server_mod, project_root: Path, tmp_path: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    # Strip signature field
    card = json.loads((tm_dir / "card.json").read_text())
    card.pop("signature", None)
    (tm_dir / "card.json").write_text(json.dumps(card, indent=2))

    out = tmp_path / "bundle.tar.gz"
    export_import.export(tm_dir, out)
    shutil.rmtree(tm_dir)

    with pytest.raises(export_import.ImportError):
        export_import.import_bundle(
            out, project_root / "context" / "team-members", assets_dir,
        )


# ---------------------------------------------------------------------------
# SUB-3: Consultant type + engagement window resolver
# ---------------------------------------------------------------------------

_WINDOW_PAST = {
    "starts_at": "2020-01-01T00:00:00+00:00",
    "ends_at": "2020-01-02T00:00:00+00:00",
}
_WINDOW_FUTURE = {
    "starts_at": "2099-01-01T00:00:00+00:00",
    "ends_at": "2099-12-31T23:59:59+00:00",
}
_WINDOW_ACTIVE = {
    "starts_at": "2020-01-01T00:00:00+00:00",
    "ends_at": "2099-12-31T23:59:59+00:00",
}


# --- Schema conditional-field validation ---

def test_consultant_requires_engaged_for(server_mod):
    """Creating a consultant without engaged_for must be rejected."""
    r = server_mod.create_team_member(
        name="Con Sultant", type="consultant", slug="con-sultant",
        default_role="ROLE-software-engineer",
        engagement_window=_WINDOW_ACTIVE,
        # engaged_for intentionally omitted
    )
    assert "error" in r, f"expected error, got: {r}"
    joined = " ".join(str(v) for v in r.values()).lower()
    assert "engaged_for" in joined, f"expected 'engaged_for' in error, got: {r}"


def test_consultant_requires_engagement_window(server_mod):
    """Creating a consultant without engagement_window must be rejected."""
    r = server_mod.create_team_member(
        name="Con Sultant", type="consultant", slug="con-sultant",
        default_role="ROLE-software-engineer",
        engaged_for="SCOPE-q2-2026",
        # engagement_window intentionally omitted
    )
    assert "error" in r, f"expected error, got: {r}"
    joined = " ".join(str(v) for v in r.values()).lower()
    assert "engagement_window" in joined, f"expected 'engagement_window' in error, got: {r}"


def test_non_consultant_rejects_engaged_for(server_mod):
    """Creating a human/ai-agent/service with engaged_for must be rejected."""
    r = server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        engaged_for="SCOPE-q2-2026",
    )
    assert "error" in r, f"expected error, got: {r}"
    joined = " ".join(str(v) for v in r.values()).lower()
    assert "engaged_for" in joined, f"expected 'engaged_for' in error, got: {r}"


def test_consultant_created_successfully(server_mod):
    """Creating a valid consultant with engaged_for + window should succeed."""
    r = server_mod.create_team_member(
        name="Con Sultant", type="consultant", slug="con-sultant",
        default_role="ROLE-software-engineer",
        default_seniority="senior",
        engaged_for="SCOPE-q2-2026",
        engagement_window=_WINDOW_ACTIVE,
    )
    assert "error" not in r, f"unexpected error: {r}"
    assert r.get("id") == "TEAMMEMBER-con-sultant"


# --- Resolver window filter in list_team_members ---

def test_consultant_within_window_resolves(server_mod):
    """Active consultant inside engagement_window appears in list_team_members."""
    server_mod.create_team_member(
        name="Con Sultant", type="consultant", slug="con-sultant",
        default_role="ROLE-software-engineer",
        engaged_for="SCOPE-q2-2026",
        engagement_window=_WINDOW_ACTIVE,
    )
    members = server_mod.list_team_members()
    ids = {m["id"] for m in members}
    assert "TEAMMEMBER-con-sultant" in ids, "in-window consultant should appear in active list"


def test_consultant_outside_window_not_resolved(server_mod):
    """Active consultant outside engagement_window is excluded from list_team_members."""
    server_mod.create_team_member(
        name="Old Con", type="consultant", slug="old-con",
        default_role="ROLE-software-engineer",
        engaged_for="SCOPE-q2-2026",
        engagement_window=_WINDOW_PAST,
    )
    members = server_mod.list_team_members()
    ids = {m["id"] for m in members}
    assert "TEAMMEMBER-old-con" not in ids, (
        "out-of-window consultant must NOT appear in active list"
    )
    # But should still appear when active_only=False
    all_members = server_mod.list_team_members(active_only=False)
    all_ids = {m["id"] for m in all_members}
    assert "TEAMMEMBER-old-con" in all_ids, (
        "out-of-window consultant must still appear with active_only=False"
    )


def test_non_consultant_always_resolves(server_mod):
    """Non-consultant TeamMember always appears regardless of any time fields."""
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-developer",
    )
    members = server_mod.list_team_members()
    ids = {m["id"] for m in members}
    assert "TEAMMEMBER-alice-chen" in ids


# --- Auto-deactivate on Scope close ---

def _make_consultant(server_mod, slug, scope_id, auto_deactivate=True):
    r = server_mod.create_team_member(
        name=slug.replace("-", " ").title(),
        type="consultant",
        slug=slug,
        default_role="ROLE-software-engineer",
        engaged_for=scope_id,
        engagement_window=_WINDOW_ACTIVE,
        auto_deactivate_on_scope_close=auto_deactivate,
    )
    assert "error" not in r, f"create failed: {r}"
    return r


def test_auto_deactivate_on_scope_close(server_mod, project_root: Path):
    """Consultant with auto_deactivate_on_scope_close=true is deactivated
    when _auto_deactivate_consultants_for_scope is called."""
    scope_id = "SCOPE-q2-2026"
    _make_consultant(server_mod, "con-auto", scope_id, auto_deactivate=True)

    # Call the internal helper directly (mirrors the scope-management hook).
    from processkit import paths as _paths
    root = _paths.find_project_root()
    deactivations = server_mod._auto_deactivate_consultants_for_scope(root, scope_id)

    deactivated_ids = {d["team_member_id"] for d in deactivations if d.get("deactivated")}
    assert "TEAMMEMBER-con-auto" in deactivated_ids

    # Entity should now be inactive.
    got = server_mod.get_team_member("con-auto")
    assert got["active"] is False, "consultant should be inactive after auto-deactivate"


def test_auto_deactivate_opt_out_leaves_active(server_mod, project_root: Path):
    """Consultant with auto_deactivate_on_scope_close=false is NOT deactivated
    when the Scope closes."""
    scope_id = "SCOPE-q2-2026"
    _make_consultant(server_mod, "con-optout", scope_id, auto_deactivate=False)

    from processkit import paths as _paths
    root = _paths.find_project_root()
    deactivations = server_mod._auto_deactivate_consultants_for_scope(root, scope_id)

    skipped_ids = {d["team_member_id"] for d in deactivations if d.get("skipped_reason")}
    assert "TEAMMEMBER-con-optout" in skipped_ids

    # Entity should still be active.
    got = server_mod.get_team_member("con-optout")
    assert got["active"] is True, "opted-out consultant should remain active"


# --- pk-team-review: expired-but-active finding ---

def test_query_consultant_findings_surfaces_expired(server_mod):
    """query_consultant_findings returns team.consultant.expired_but_active
    for consultants whose window has ended but are still active."""
    server_mod.create_team_member(
        name="Expired Con", type="consultant", slug="expired-con",
        default_role="ROLE-software-engineer",
        engaged_for="SCOPE-q2-2026",
        engagement_window=_WINDOW_PAST,
    )
    result = server_mod.query_consultant_findings()
    assert result["count"] > 0
    codes = {f["code"] for f in result["findings"]}
    assert "team.consultant.expired_but_active" in codes
    ids_in_findings = {f["team_member_id"] for f in result["findings"]}
    assert "TEAMMEMBER-expired-con" in ids_in_findings
    # Check action prompt is present.
    for f in result["findings"]:
        if f["team_member_id"] == "TEAMMEMBER-expired-con":
            assert "update_team_member" in f["action"]


def test_query_consultant_findings_clean_when_no_expired(server_mod):
    """query_consultant_findings returns no findings when all active consultants
    are within their window."""
    server_mod.create_team_member(
        name="Active Con", type="consultant", slug="active-con",
        default_role="ROLE-software-engineer",
        engaged_for="SCOPE-q2-2026",
        engagement_window=_WINDOW_ACTIVE,
    )
    result = server_mod.query_consultant_findings()
    expired_findings = [
        f for f in result["findings"]
        if f["team_member_id"] == "TEAMMEMBER-active-con"
    ]
    assert expired_findings == [], "in-window consultant must not appear in findings"


def test_export_claude_subagent_adapter(server_mod, project_root: Path):
    server_mod.create_team_member(
        name="Alice", type="ai-agent", slug="alice",
        default_role="ROLE-software-engineer",
        default_seniority="expert",
    )
    server_mod.init_memory_tree("alice")
    persona = project_root / "context" / "team-members" / "alice" / "persona.md"
    persona.write_text("# Alice\n\nPragmatic implementation specialist.\n")

    r = server_mod.export_claude_subagent("alice")
    assert r["written"] is True
    out = project_root / ".claude" / "agents" / "alice.md"
    text = out.read_text()
    assert "name: alice\n" in text
    assert "model: inherit\n" in text
    assert "tools:" not in text
    assert "TeamMember TEAMMEMBER-alice" in text
    assert "Pragmatic implementation specialist." in text
    # DaringRaven rec 6: header comment bakes TeamMember identity into
    # the exported adapter so the file is self-describing.
    assert "<!--" in text
    assert "TeamMember:  TEAMMEMBER-alice" in text
    assert "Slug:        alice" in text
    assert "Role:        ROLE-software-engineer" in text
    assert "Seniority:   expert" in text
    assert "Model policy: inherit" in text


def test_export_claude_subagent_resolved_model_policy(
    server_mod,
    project_root: Path,
    monkeypatch: pytest.MonkeyPatch,
):
    server_mod.create_team_member(
        name="Alice", type="ai-agent", slug="alice",
        default_role="ROLE-software-engineer",
        default_seniority="expert",
    )

    class Candidate:
        model_id = "ART-20260503_1424-ModelSpec-anthropic-claude-sonnet"
        version_id = "4.5"
        effort = "high"
        rank = 1
        source_layer = 5
        rationale = "role binding"
        profile_id = None
        profile_rank = 1

    class Resolver:
        @staticmethod
        def _runtime_context(task_hints=None):
            return {
                "harnesses": ["claude"],
                "allowed_providers": ["anthropic"],
                "preferred_providers": ["anthropic"],
                "provider_source": "task_hints.available_providers",
            }

        @staticmethod
        def resolve(**kwargs):
            return [Candidate()], []

    monkeypatch.setattr(server_mod, "_load_model_resolver", lambda: Resolver)
    monkeypatch.setattr(
        server_mod,
        "_model_spec_from_id",
        lambda model_id: {
            "provider": "anthropic",
            "family": "claude-sonnet",
            "profile_ids": ["claude-sonnet-4.5"],
            "versions": [{"version_id": "4.5"}],
        },
    )

    r = server_mod.export_claude_subagent("alice", model_policy="resolved")

    assert r["written"] is True
    assert r["model_policy"] == "resolved"
    text = (project_root / ".claude" / "agents" / "alice.md").read_text()
    assert "model: claude-sonnet-4.5\n" in text
    assert "effort: high\n" in text
    # DaringRaven rec 6: header comment surfaces the resolved binding
    # so the adapter file is auditable without re-resolving.
    assert "Model policy: resolved" in text
    assert "Resolved model: claude-sonnet-4.5" in text
    assert "Provider: anthropic" in text
    assert "Effort: high" in text


def test_export_claude_subagents_skips_humans_by_default(server_mod, project_root: Path):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    server_mod.create_team_member(name="Alice", type="ai-agent", slug="alice")

    r = server_mod.export_claude_subagents()
    assert r["count"] == 1
    assert (project_root / ".claude" / "agents" / "alice.md").is_file()
    assert not (project_root / ".claude" / "agents" / "alice-chen.md").exists()
    assert any(item.get("skipped") == "human" for item in r["results"])


def test_export_claude_subagent_rejects_output_outside_project(
    server_mod, tmp_path: Path
):
    server_mod.create_team_member(name="Alice", type="ai-agent", slug="alice")
    outside = tmp_path.parent / "outside-project"
    r = server_mod.export_claude_subagent("alice", output_dir=str(outside))
    assert "error" in r
    assert "project root" in r["error"]


# ---------------------------------------------------------------------------
# check_all summary
# ---------------------------------------------------------------------------

def test_check_all_aggregate(server_mod, project_root: Path, assets_dir):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    server_mod.create_team_member(name="Bob Lee", type="human", slug="bob-lee")
    server_mod.init_memory_tree("alice-chen")
    server_mod.init_memory_tree("bob-lee")
    report = consistency.check_all(
        project_root,
        project_root / "context" / "team-members",
        server_mod._pool_path(),
        assets_dir,
    )
    assert report["summary"]["count"] == 2
    assert "alice-chen" in report["members"]
    assert "bob-lee" in report["members"]


# ---------------------------------------------------------------------------
# RoleSlot tools — Phase A team-creator v2
# DEC-20260509_1906-CoolBadger / ART-20260509_1836-SmartPanda
# ---------------------------------------------------------------------------

def _open_slot(server_mod, **overrides):
    """Helper: create a baseline RoleSlot for further tests to mutate."""
    payload = dict(
        scope="SCOPE-q2-2026",
        role="ROLE-software-engineer",
        seniority="senior",
        rank=1,
        rationale="primary backend implementer for q2",
    )
    payload.update(overrides)
    return server_mod.create_role_slot(**payload)


def test_role_slot_create_get_happy_path(server_mod, project_root: Path):
    r = _open_slot(server_mod)
    assert r.get("state") == "open", r
    expected_id = "SLOT-q2-2026-software-engineer-1"
    assert r["id"] == expected_id
    assert Path(r["path"]).is_file()

    got = server_mod.get_role_slot(expected_id)
    assert got["id"] == expected_id
    assert got["scope"] == "SCOPE-q2-2026"
    assert got["role"] == "ROLE-software-engineer"
    assert got["seniority"] == "senior"
    assert got["rank"] == 1
    assert got["state"] == "open"
    assert got["filled_by"] is None
    assert got["rationale"] == "primary backend implementer for q2"
    assert got["created"]


def test_role_slot_create_validates_inputs(server_mod, project_root: Path):
    bad_scope = server_mod.create_role_slot(
        scope="bad", role="ROLE-x", seniority="senior", rank=1, rationale="r",
    )
    assert "error" in bad_scope and "SCOPE" in bad_scope["error"]

    bad_role = server_mod.create_role_slot(
        scope="SCOPE-x", role="bad", seniority="senior", rank=1, rationale="r",
    )
    assert "error" in bad_role and "ROLE" in bad_role["error"]

    bad_sen = server_mod.create_role_slot(
        scope="SCOPE-x", role="ROLE-x", seniority="ninja", rank=1, rationale="r",
    )
    assert "error" in bad_sen and "seniority" in bad_sen["error"]

    bad_rank = server_mod.create_role_slot(
        scope="SCOPE-x", role="ROLE-x", seniority="senior", rank=0, rationale="r",
    )
    assert "error" in bad_rank and "rank" in bad_rank["error"]

    bad_rationale = server_mod.create_role_slot(
        scope="SCOPE-x", role="ROLE-x", seniority="senior", rank=1, rationale="",
    )
    assert "error" in bad_rationale and "rationale" in bad_rationale["error"]


def test_role_slot_create_rejects_duplicate(server_mod, project_root: Path):
    r1 = _open_slot(server_mod)
    assert "error" not in r1, r1
    r2 = _open_slot(server_mod)
    assert "error" in r2 and "already exists" in r2["error"]


def test_list_role_slots_filters(server_mod, project_root: Path):
    _open_slot(server_mod)  # SLOT-q2-2026-software-engineer-1 (open, senior)
    _open_slot(server_mod, rank=2, rationale="parallel slot")
    _open_slot(
        server_mod, role="ROLE-product-manager", seniority="senior",
        rationale="single PM",
    )

    all_slots = server_mod.list_role_slots()
    assert {s["id"] for s in all_slots} == {
        "SLOT-q2-2026-software-engineer-1",
        "SLOT-q2-2026-software-engineer-2",
        "SLOT-q2-2026-product-manager-1",
    }

    by_role = server_mod.list_role_slots(role="ROLE-software-engineer")
    assert len(by_role) == 2

    by_state_open = server_mod.list_role_slots(state="open")
    assert len(by_state_open) == 3

    by_state_filled = server_mod.list_role_slots(state="filled")
    assert by_state_filled == []


def test_fill_role_slot_happy_path(server_mod, project_root: Path):
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-software-engineer",
        default_seniority="senior",
    )
    open_r = _open_slot(server_mod)
    slot_id = open_r["id"]

    fill_r = server_mod.fill_role_slot(
        id=slot_id,
        team_member_slug="alice-chen",
        valid_from="2026-05-09",
        valid_until="2026-08-01",
        rationale="lead engineer for the q2 charter",
    )
    assert fill_r.get("ok") is True, fill_r
    assert fill_r["state"] == "filled"
    assert fill_r["filled_by"] == "TEAMMEMBER-alice-chen"
    assert fill_r["binding_id"].startswith("BIND-")
    assert Path(fill_r["binding_path"]).is_file()

    # Slot file reflects filled state
    got = server_mod.get_role_slot(slot_id)
    assert got["state"] == "filled"
    assert got["filled_by"] == "TEAMMEMBER-alice-chen"


def test_fill_role_slot_rejects_inactive_team_member(server_mod, project_root: Path):
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-software-engineer",
        default_seniority="senior",
    )
    server_mod.deactivate_team_member("alice-chen")
    r = _open_slot(server_mod)
    fill = server_mod.fill_role_slot(id=r["id"], team_member_slug="alice-chen")
    assert "error" in fill
    assert "inactive" in fill["error"]


def test_fill_role_slot_rejects_unknown_team_member(server_mod, project_root: Path):
    r = _open_slot(server_mod)
    fill = server_mod.fill_role_slot(id=r["id"], team_member_slug="nobody")
    assert "error" in fill
    assert "not found" in fill["error"]


def test_close_role_slot_terminal(server_mod, project_root: Path):
    r = _open_slot(server_mod)
    close = server_mod.close_role_slot(r["id"], reason="charter closed early")
    assert close.get("ok") is True
    assert close["state"] == "closed"
    assert close["closed_at"]
    assert close["close_reason"] == "charter closed early"

    # Idempotent close
    close2 = server_mod.close_role_slot(r["id"])
    assert close2.get("ok") is True
    assert close2.get("already_closed") is True

    # Reverse transition rejected
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-software-engineer",
        default_seniority="senior",
    )
    fill_after_close = server_mod.fill_role_slot(
        id=r["id"], team_member_slug="alice-chen",
    )
    assert "error" in fill_after_close
    assert "transition" in fill_after_close["error"]


def test_fill_role_slot_rejects_double_fill(server_mod, project_root: Path):
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-software-engineer",
        default_seniority="senior",
    )
    server_mod.create_team_member(
        name="Bob Lee", type="human", slug="bob-lee",
        default_role="ROLE-software-engineer",
        default_seniority="senior",
    )
    r = _open_slot(server_mod)
    server_mod.fill_role_slot(id=r["id"], team_member_slug="alice-chen")

    second = server_mod.fill_role_slot(id=r["id"], team_member_slug="bob-lee")
    assert "error" in second
    assert "already filled" in second["error"]


def test_resolver_pre_step_returns_filled_slot(
    server_mod,
    project_root: Path,
    monkeypatch: pytest.MonkeyPatch,
):
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-software-engineer",
        default_seniority="senior",
    )
    server_mod.set_active_interlocutor("alice-chen")
    open_r = _open_slot(server_mod)
    server_mod.fill_role_slot(id=open_r["id"], team_member_slug="alice-chen")

    # Stub the model resolver so the existing 8-layer code path runs
    # cleanly and we can inspect the pre-step on top.
    class Resolver:
        @staticmethod
        def _runtime_context(task_hints=None):
            return {"harnesses": [], "allowed_providers": []}

        @staticmethod
        def resolve(**kwargs):
            return [], []

    monkeypatch.setattr(server_mod, "_load_model_resolver", lambda: Resolver)
    got = server_mod.get_interlocutor_runtime_binding(scope="SCOPE-q2-2026")

    assert got["configured"] is True
    pre = got["binding"].get("roleslot_pre_step")
    assert pre is not None, got["binding"]
    assert pre["slot"]["id"] == open_r["id"]
    assert pre["slot"]["state"] == "filled"
    assert pre["team_member"]["id"] == "TEAMMEMBER-alice-chen"
    # TeamMember.default_seniority overrides slot.seniority for model
    # resolution if set (design §"Resolver impact" step 3).
    assert pre["applied_seniority"] == "senior"


def test_resolver_pre_step_falls_through_when_no_slot(
    server_mod,
    project_root: Path,
    monkeypatch: pytest.MonkeyPatch,
):
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-software-engineer",
        default_seniority="senior",
    )
    server_mod.set_active_interlocutor("alice-chen")

    class Resolver:
        @staticmethod
        def _runtime_context(task_hints=None):
            return {"harnesses": [], "allowed_providers": []}

        @staticmethod
        def resolve(**kwargs):
            return [], []

    monkeypatch.setattr(server_mod, "_load_model_resolver", lambda: Resolver)
    got = server_mod.get_interlocutor_runtime_binding(scope="SCOPE-q2-2026")

    # No slot exists for (ROLE-software-engineer, senior, SCOPE-q2-2026)
    # so the pre-step is silent and the response is identical to
    # pre-RoleSlot behaviour. Phase A is additive (Q2 deferred).
    assert got["configured"] is True
    assert "roleslot_pre_step" not in got["binding"]


def test_resolver_pre_step_seniority_override_from_team_member(
    server_mod,
    project_root: Path,
    monkeypatch: pytest.MonkeyPatch,
):
    # Slot opens at seniority=senior; TeamMember declares
    # default_seniority=expert. Per design §"Resolver impact" step 3
    # the TeamMember's default_seniority overrides the slot's seniority
    # for model resolution.
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-software-engineer",
        default_seniority="expert",
    )
    server_mod.set_active_interlocutor("alice-chen")
    open_r = server_mod.create_role_slot(
        scope="SCOPE-q2-2026",
        role="ROLE-software-engineer",
        seniority="expert",
        rank=1,
        rationale="expert backend implementer",
    )
    server_mod.fill_role_slot(id=open_r["id"], team_member_slug="alice-chen")

    class Resolver:
        @staticmethod
        def _runtime_context(task_hints=None):
            return {"harnesses": [], "allowed_providers": []}

        @staticmethod
        def resolve(**kwargs):
            return [], []

    monkeypatch.setattr(server_mod, "_load_model_resolver", lambda: Resolver)
    got = server_mod.get_interlocutor_runtime_binding(scope="SCOPE-q2-2026")
    pre = got["binding"]["roleslot_pre_step"]
    assert pre["slot"]["seniority"] == "expert"
    assert pre["applied_seniority"] == "expert"


# ---------------------------------------------------------------------------
# team-creator v2 — archetype-catalog mapping loader (SUB-2 / LuckyWren)
# ---------------------------------------------------------------------------

@pytest.fixture()
def team_creator_lib():
    """Import the team-creator scripts module from the in-repo source path."""
    repo_root = _find_repo_root()
    tc_scripts = (
        repo_root
        / "context"
        / "skills"
        / "processkit"
        / "team-creator"
        / "scripts"
    )
    if str(tc_scripts) not in sys.path:
        sys.path.insert(0, str(tc_scripts))
    if "team_creator_lib" in sys.modules:
        del sys.modules["team_creator_lib"]
    import team_creator_lib  # noqa: F401
    return team_creator_lib


def test_archetype_catalog_mapping_kit_default_loads(team_creator_lib, tmp_path):
    """The shipped kit-default mapping must validate and contain all 8 archetypes."""
    mapping = team_creator_lib.load_archetype_catalog_mapping(tmp_path)
    assert mapping.source == "kit-default"
    assert mapping.semantics is None
    assert mapping.overrides == []
    assert set(mapping.archetypes) == set(team_creator_lib.ARCHETYPES)
    pm = mapping.archetypes["project-manager"]
    assert pm["role"] == "ROLE-product-manager"
    assert pm["seniority"] == "senior"
    assert pm.get("primary_contact") is True
    # Two archetypes share the same catalog Role with different seniority.
    sa = mapping.archetypes["senior-architect"]
    ja = mapping.archetypes["junior-architect"]
    assert sa["role"] == ja["role"] == "ROLE-solutions-architect"
    assert sa["seniority"] == "senior"
    assert ja["seniority"] == "specialist"


def test_archetype_catalog_mapping_project_delta_layers_correctly(
    team_creator_lib, tmp_path
):
    """Project override in delta mode replaces only the listed archetypes."""
    (tmp_path / "context" / "team").mkdir(parents=True)
    proj = tmp_path / "context" / "team" / "archetype-catalog-mapping.yaml"
    proj.write_text(
        "apiVersion: processkit.projectious.work/v2\n"
        "kind: ArchetypeCatalogMapping\n"
        "spec:\n"
        "  archetypes:\n"
        "    developer:\n"
        "      role: ROLE-software-engineer\n"
        "      seniority: principal\n",
        encoding="utf-8",
    )
    mapping = team_creator_lib.load_archetype_catalog_mapping(tmp_path)
    assert mapping.source == "project"
    assert mapping.semantics == "delta"
    # Override applies to the listed archetype:
    assert mapping.archetypes["developer"]["seniority"] == "principal"
    # Untouched archetypes inherit the kit default:
    assert mapping.archetypes["project-manager"]["role"] == "ROLE-product-manager"
    assert mapping.archetypes["assistant"]["role"] == "ROLE-assistant"
    # The delta entry surfaces in `overrides` for the chartering DEC audit:
    delta = [o for o in mapping.overrides
             if o["archetype"] == "developer" and o["field"] == "seniority"]
    assert len(delta) == 1
    assert delta[0]["kit_default"] == "senior"
    assert delta[0]["project_value"] == "principal"


def test_archetype_catalog_mapping_replace_requires_all_archetypes(
    team_creator_lib, tmp_path
):
    """A replace-mode override missing archetypes is a hard error."""
    (tmp_path / "context" / "team").mkdir(parents=True)
    proj = tmp_path / "context" / "team" / "archetype-catalog-mapping.yaml"
    proj.write_text(
        "override_semantics: replace\n"
        "spec:\n"
        "  archetypes:\n"
        "    developer:\n"
        "      role: ROLE-software-engineer\n"
        "      seniority: senior\n",
        encoding="utf-8",
    )
    with pytest.raises(ValueError) as excinfo:
        team_creator_lib.load_archetype_catalog_mapping(tmp_path)
    assert "replace-mode override missing archetypes" in str(excinfo.value)


def test_archetype_catalog_mapping_reverse_lookup(team_creator_lib, tmp_path):
    """archetype_for_role_slot resolves (ROLE, seniority) -> archetype name."""
    mapping = team_creator_lib.load_archetype_catalog_mapping(tmp_path)
    assert team_creator_lib.archetype_for_role_slot(
        "ROLE-software-engineer", "junior", mapping,
    ) == "junior-developer"
    assert team_creator_lib.archetype_for_role_slot(
        "ROLE-software-engineer", "senior", mapping,
    ) == "developer"
    assert team_creator_lib.archetype_for_role_slot(
        "ROLE-product-manager", "senior", mapping,
    ) == "project-manager"
    # No archetype for a fictional pair:
    assert team_creator_lib.archetype_for_role_slot(
        "ROLE-software-engineer", "principal", mapping,
    ) is None


# ---------------------------------------------------------------------------
# Migration apply script — Phase A back-fill (idempotency + smoke test)
# ---------------------------------------------------------------------------

@pytest.fixture()
def apply_migration_2139_module():
    here = Path(__file__).resolve().parent
    if str(here) not in sys.path:
        sys.path.insert(0, str(here))
    # Force a fresh import so the module re-binds ``server`` against the
    # active project_root fixture.
    for name in ("apply_migration_2139",):
        if name in sys.modules:
            del sys.modules[name]
    import apply_migration_2139  # noqa: F401
    return apply_migration_2139


def _seed_v1_archetype_role(project_root: Path, archetype: str, clone_cap: int = 1):
    """Write a minimal v1 archetype-spawned Role file under context/roles/."""
    roles_dir = project_root / "context" / "roles"
    roles_dir.mkdir(parents=True, exist_ok=True)
    rid = f"ROLE-{archetype}"
    body = (
        "---\n"
        "apiVersion: processkit.projectious.work/v2\n"
        "kind: Role\n"
        "metadata:\n"
        f"  id: {rid}\n"
        "  created: 2026-04-01T00:00:00+00:00\n"
        "spec:\n"
        f"  name: {archetype}\n"
        "  description: legacy v1 archetype-spawned role for back-fill test\n"
        f"  clone_cap: {clone_cap}\n"
        "---\n\n"
        f"# {rid}\n"
    )
    (roles_dir / f"{rid}.md").write_text(body, encoding="utf-8")
    return rid


def _seed_chartering_scope(project_root: Path, scope_id: str = "SCOPE-q2-2026"):
    """Write a minimal Scope file so apply_migration_2139's existence check passes."""
    scopes_dir = project_root / "context" / "scopes"
    scopes_dir.mkdir(parents=True, exist_ok=True)
    body = (
        "---\n"
        "apiVersion: processkit.projectious.work/v2\n"
        "kind: Scope\n"
        "metadata:\n"
        f"  id: {scope_id}\n"
        "  created: 2026-04-01T00:00:00+00:00\n"
        "spec:\n"
        "  title: 2026-Q2 chartering scope (test)\n"
        "  state: active\n"
        "  description: test scope for SUB-2 apply-script smoke test\n"
        "---\n\n"
        f"# {scope_id}\n"
    )
    (scopes_dir / f"{scope_id}.md").write_text(body, encoding="utf-8")
    return scope_id


def _seed_v1_role_assignment_binding(
    project_root: Path,
    binding_id: str,
    subject: str,
    target: str,
):
    """Write a minimal v1 Binding(type=role-assignment) under context/bindings/."""
    bind_dir = project_root / "context" / "bindings"
    bind_dir.mkdir(parents=True, exist_ok=True)
    body = (
        "---\n"
        "apiVersion: processkit.projectious.work/v2\n"
        "kind: Binding\n"
        "metadata:\n"
        f"  id: {binding_id}\n"
        "  created: 2026-04-01T00:00:00+00:00\n"
        "spec:\n"
        "  type: role-assignment\n"
        f"  subject: {subject}\n"
        f"  target: {target}\n"
        "  subject_kind: TeamMember\n"
        "  target_kind: Role\n"
        "  valid_from: '2026-04-01'\n"
        "---\n\n"
        f"# {binding_id}\n"
    )
    (bind_dir / f"{binding_id}.md").write_text(body, encoding="utf-8")
    return binding_id


def test_apply_migration_2139_smoke_creates_slot_and_fill(
    server_mod,                # noqa: ARG001 — establishes project_root + paths
    project_root: Path,
    apply_migration_2139_module,
):
    """1 archetype-spawned Role + 1 role-assignment Binding -> 1 SLOT + 1 fill."""
    scope_id = _seed_chartering_scope(project_root)
    _seed_v1_archetype_role(project_root, "developer", clone_cap=1)
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-software-engineer",
        default_seniority="senior",
    )
    _seed_v1_role_assignment_binding(
        project_root,
        binding_id="BIND-test-developer-alice",
        subject="TEAMMEMBER-alice-chen",
        target="ROLE-developer",
    )

    summary = apply_migration_2139_module.apply(
        project_root, scope_id, dry_run=False,
    )
    assert "error" not in summary, summary
    assert summary["slots_created"] == 1, summary
    assert summary["fills_created"] == 1, summary
    assert summary["slots_skipped"] == 0
    assert summary["fills_skipped"] == 0

    # The new SLOT exists under context/roleslots/
    slot_id = "SLOT-q2-2026-developer-1"
    slot_path = project_root / "context" / "roleslots" / f"{slot_id}.md"
    assert slot_path.is_file(), f"expected {slot_path} to exist"

    # The new role-slot-fill Binding exists under context/bindings/
    bind_dir = project_root / "context" / "bindings"
    fills = []
    for path in bind_dir.glob("BIND-*.md"):
        text = path.read_text(encoding="utf-8")
        if "type: role-slot-fill" in text and slot_id in text:
            fills.append(path)
    assert len(fills) == 1, [p.name for p in bind_dir.glob('*.md')]


def test_apply_migration_2139_is_idempotent(
    server_mod,                # noqa: ARG001
    project_root: Path,
    apply_migration_2139_module,
):
    """Re-running apply produces no-ops (skips both slot create + fill)."""
    scope_id = _seed_chartering_scope(project_root)
    _seed_v1_archetype_role(project_root, "developer", clone_cap=1)
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-software-engineer",
        default_seniority="senior",
    )
    _seed_v1_role_assignment_binding(
        project_root,
        binding_id="BIND-test-developer-alice",
        subject="TEAMMEMBER-alice-chen",
        target="ROLE-developer",
    )

    first = apply_migration_2139_module.apply(project_root, scope_id)
    second = apply_migration_2139_module.apply(project_root, scope_id)
    assert "error" not in second, second
    assert second["slots_created"] == 0
    assert second["fills_created"] == 0
    assert second["slots_skipped"] == first["slots_created"]
    assert second["fills_skipped"] == first["fills_created"]


def test_apply_migration_2139_v2_native_project_is_no_op(
    server_mod,                # noqa: ARG001
    project_root: Path,
    apply_migration_2139_module,
):
    """A project with no v1 archetype Roles produces zero writes."""
    scope_id = _seed_chartering_scope(project_root)
    summary = apply_migration_2139_module.apply(project_root, scope_id)
    assert "error" not in summary, summary
    assert summary["slots_created"] == 0
    assert summary["fills_created"] == 0
    assert summary["slots_skipped"] == 0
    assert summary["fills_skipped"] == 0


def test_apply_migration_2139_dry_run_writes_nothing(
    server_mod,                # noqa: ARG001
    project_root: Path,
    apply_migration_2139_module,
):
    """--dry-run reports the plan but writes no SLOT or fill Binding."""
    scope_id = _seed_chartering_scope(project_root)
    _seed_v1_archetype_role(project_root, "developer", clone_cap=2)
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-software-engineer",
        default_seniority="senior",
    )
    _seed_v1_role_assignment_binding(
        project_root,
        binding_id="BIND-test-developer-alice",
        subject="TEAMMEMBER-alice-chen",
        target="ROLE-developer",
    )

    summary = apply_migration_2139_module.apply(
        project_root, scope_id, dry_run=True,
    )
    assert summary["dry_run"] is True
    assert summary["slots_created"] == 2  # planned, not written
    assert summary["fills_created"] == 1
    # No actual files were written:
    rs_dir = project_root / "context" / "roleslots"
    assert not rs_dir.exists() or not list(rs_dir.glob("SLOT-*.md"))


def test_apply_migration_2139_rejects_missing_scope(
    server_mod,                # noqa: ARG001
    project_root: Path,        # noqa: ARG001 — project_root activates path bootstrap
    apply_migration_2139_module,
):
    """Apply errors when the chartering Scope does not exist."""
    summary = apply_migration_2139_module.apply(
        project_root, "SCOPE-does-not-exist",
    )
    assert "error" in summary


# ---------------------------------------------------------------------------
# Budget projection helpers (Gap 5 — SUB-4 / BACK-20260509_1837-SwiftReef)
# ---------------------------------------------------------------------------

@pytest.fixture()
def team_creator_lib_budget():
    """Import team_creator_lib for budget-projection tests (no server_mod needed)."""
    repo_root = _find_repo_root()
    tc_scripts = (
        repo_root
        / "context"
        / "skills"
        / "processkit"
        / "team-creator"
        / "scripts"
    )
    if str(tc_scripts) not in sys.path:
        sys.path.insert(0, str(tc_scripts))
    for name in ("team_creator_lib",):
        if name in sys.modules:
            del sys.modules[name]
    import team_creator_lib  # noqa: F401
    return team_creator_lib


def _make_budget_projection(tcl, slot_id="SLOT-q2-2026-software-engineer-1",
                             role="ROLE-software-engineer",
                             unit_cost=0.000003, invocations=50,
                             weeks=12, threshold=20.0):
    """Helper: build a canonical budget_projection block for tests."""
    # weeks=12 => scope window 84 days
    import datetime as _dt
    starts = _dt.date.today()
    ends = starts + _dt.timedelta(days=weeks * 7)
    eff_window = (str(starts), str(ends))
    row = tcl.compute_slot_projection(
        slot_id=slot_id,
        role=role,
        seniority="senior",
        model_profile="ART-20260503_1424-ModelProfile-sonnet",
        expected_invocations_per_week=invocations,
        avg_tokens={"input": 8000, "output": 2000},
        unit_cost_usd=unit_cost,
        effective_window=eff_window,
    )
    bp = tcl.build_budget_projection(
        slot_projections=[row],
        scope_window={"starts_at": eff_window[0], "ends_at": eff_window[1]},
        drift_threshold_pct=threshold,
        projection_method="heuristic",
    )
    return bp, row


def test_charter_creates_budget_projection_block(team_creator_lib_budget):
    """build_budget_projection returns a valid budget_projection block."""
    tcl = team_creator_lib_budget
    bp, row = _make_budget_projection(tcl)
    assert bp["currency"] == "USD"
    assert "window" in bp
    assert "projected_total" in bp
    assert bp["projection_method"] == "heuristic"
    assert len(bp["slot_projections"]) == 1
    assert bp["drift_threshold_pct"] == 20.0
    assert "notes" not in bp  # not set when not provided


def test_projection_sums_slot_projections_correctly(team_creator_lib_budget):
    """projected_total must equal the sum of all slot projected_total_usd values."""
    tcl = team_creator_lib_budget
    import datetime as _dt
    starts = _dt.date.today()
    ends = starts + _dt.timedelta(days=84)
    eff_window = (str(starts), str(ends))

    rows = []
    for i, (slot, role, cost) in enumerate([
        ("SLOT-q2-2026-software-engineer-1", "ROLE-software-engineer", 0.000003),
        ("SLOT-q2-2026-product-manager-1", "ROLE-product-manager", 0.000010),
        ("SLOT-q2-2026-solutions-architect-1", "ROLE-solutions-architect", 0.000008),
    ], start=1):
        rows.append(tcl.compute_slot_projection(
            slot_id=slot,
            role=role,
            seniority="senior",
            model_profile=None,
            expected_invocations_per_week=50,
            avg_tokens={"input": 8000, "output": 2000},
            unit_cost_usd=cost,
            effective_window=eff_window,
        ))
    bp = tcl.build_budget_projection(
        slot_projections=rows,
        scope_window={"starts_at": eff_window[0], "ends_at": eff_window[1]},
    )
    expected = round(sum(r["projected_total_usd"] for r in rows), 6)
    assert bp["projected_total"] == expected, (
        f"projected_total {bp['projected_total']} != sum {expected}"
    )


def test_get_pricing_snapshotted_into_unit_cost(team_creator_lib_budget):
    """unit_cost_usd in the slot row must reflect the value passed (simulating
    get_pricing snapshot at charter time)."""
    tcl = team_creator_lib_budget
    import datetime as _dt
    starts = _dt.date.today()
    ends = starts + _dt.timedelta(days=84)
    live_price = 0.000007  # simulated get_pricing return value
    row = tcl.compute_slot_projection(
        slot_id="SLOT-q2-2026-software-engineer-1",
        role="ROLE-software-engineer",
        seniority="senior",
        model_profile="ART-20260503_1424-ModelProfile-sonnet",
        expected_invocations_per_week=50,
        avg_tokens={"input": 8000, "output": 2000},
        unit_cost_usd=live_price,
        effective_window=(str(starts), str(ends)),
    )
    assert row["unit_cost_usd"] == live_price, (
        "unit_cost_usd must be snapshotted from get_pricing, not recomputed"
    )
    # projected_total_usd must use the snapshotted price
    weeks = math.ceil(84 / 7)
    expected_total = live_price * (8000 + 2000) * 50 * weeks
    assert abs(row["projected_total_usd"] - expected_total) < 1e-9


def test_pk_team_review_skips_drift_when_no_budget_projection(server_mod, project_root):
    """query_budget_drift returns status=no_projection_on_file when the
    chartering DEC has no budget_projection block."""
    # No decision files written — server_mod starts clean.
    result = server_mod.query_budget_drift()
    assert result["status"] == "no_projection_on_file"
    assert result["finding_code"] is None
    assert "skipping drift check" in result["summary"] or "drift check skipped" in result["summary"]


def test_pk_team_review_emits_budget_drift_over_spend(
    server_mod, project_root: Path
):
    """query_budget_drift emits team.budget.drift (warning) when actual exceeds threshold."""
    tcl_path = (
        _find_repo_root()
        / "context" / "skills" / "processkit" / "team-creator" / "scripts"
    )
    if str(tcl_path) not in sys.path:
        sys.path.insert(0, str(tcl_path))
    for name in ("team_creator_lib",):
        if name in sys.modules:
            del sys.modules[name]
    import team_creator_lib as tcl

    bp, row = _make_budget_projection(tcl, unit_cost=0.000003, invocations=50, weeks=12)
    slot_id = row["slot"]

    # Actual cost is 50% over projected → exceeds 20% threshold
    projected = row["projected_total_usd"]
    actual_costs = {slot_id: projected * 1.50}

    drift = tcl.compute_budget_drift(bp, actual_costs)
    assert drift["threshold_exceeded"] is True
    assert drift["finding_code"] == "team.budget.drift"
    assert drift["severity"] == "warning"
    assert drift["drift_direction"] == "over"
    assert drift["drift_pct"] > 20.0


def test_pk_team_review_emits_informational_under_spend(team_creator_lib_budget):
    """compute_budget_drift emits team.budget.drift with severity=info on under-spend."""
    tcl = team_creator_lib_budget
    bp, row = _make_budget_projection(tcl, unit_cost=0.000003, invocations=50, weeks=12)
    slot_id = row["slot"]
    projected = row["projected_total_usd"]
    # Actual is 40% below projected → exceeds threshold but in under direction
    actual_costs = {slot_id: projected * 0.50}

    drift = tcl.compute_budget_drift(bp, actual_costs)
    assert drift["threshold_exceeded"] is True
    assert drift["finding_code"] == "team.budget.drift"
    assert drift["severity"] == "info"
    assert drift["drift_direction"] == "under"
    assert drift["drift_pct"] < -20.0


def test_consultant_slot_uses_engagement_window_intersection(team_creator_lib_budget):
    """Consultant slots use the intersection of engagement_window and Scope window."""
    import datetime as _dt
    tcl = team_creator_lib_budget

    scope_starts = "2026-04-01"
    scope_ends = "2026-06-30"  # 13 weeks

    # Consultant is only engaged for the first 4 weeks of the scope
    consultant_starts = "2026-04-01"
    consultant_ends = "2026-04-28"  # 4 weeks

    intersected = tcl.intersect_windows(
        scope_starts, scope_ends,
        consultant_starts, consultant_ends,
    )
    assert intersected is not None
    assert intersected[0] == consultant_starts
    assert intersected[1] == consultant_ends

    # Compute slot projection using intersected window (4 weeks, not 13)
    row = tcl.compute_slot_projection(
        slot_id="SLOT-q2-2026-solutions-architect-1",
        role="ROLE-solutions-architect",
        seniority="senior",
        model_profile=None,
        expected_invocations_per_week=40,
        avg_tokens={"input": 10000, "output": 3000},
        unit_cost_usd=0.000010,
        effective_window=intersected,
    )
    weeks = math.ceil(
        (_dt.date.fromisoformat(intersected[1]) - _dt.date.fromisoformat(intersected[0])).days / 7
    )
    assert weeks == 4, f"expected 4-week window, got {weeks}"
    expected_total = 0.000010 * (10000 + 3000) * 40 * weeks
    assert abs(row["projected_total_usd"] - expected_total) < 1e-9


def test_intersect_windows_no_overlap_returns_none(team_creator_lib_budget):
    """intersect_windows returns None when the slot window doesn't overlap scope."""
    tcl = team_creator_lib_budget
    result = tcl.intersect_windows(
        "2026-04-01", "2026-06-30",
        "2026-07-01", "2026-09-30",  # after scope ends
    )
    assert result is None


def test_query_budget_drift_mcp_tool_with_injected_costs(
    server_mod, project_root: Path
):
    """query_budget_drift with actual_slot_costs bypasses event-log and returns drift."""
    # Write a minimal chartering DecisionRecord with a budget_projection block.
    import datetime as _dt
    dec_dir = project_root / "context" / "decisions"
    dec_dir.mkdir(parents=True, exist_ok=True)

    tcl_path = (
        _find_repo_root()
        / "context" / "skills" / "processkit" / "team-creator" / "scripts"
    )
    if str(tcl_path) not in sys.path:
        sys.path.insert(0, str(tcl_path))
    for name in ("team_creator_lib",):
        if name in sys.modules:
            del sys.modules[name]
    import team_creator_lib as tcl

    bp, row = _make_budget_projection(tcl, unit_cost=0.000003, invocations=50, weeks=12)
    slot_id = row["slot"]
    projected = row["projected_total_usd"]

    import yaml as _yaml
    dec_body = (
        "---\n"
        "apiVersion: processkit.projectious.work/v2\n"
        "kind: Decision\n"
        "metadata:\n"
        "  id: DEC-20260510_0000-TestDrift-budget-test\n"
        "  created: '2026-05-10T00:00:00+00:00'\n"
        "spec:\n"
        "  title: Budget Drift Test DEC\n"
        "  state: accepted\n"
        "  inputs_snapshot:\n"
        "    chartering_scope: SCOPE-q2-2026\n"
        f"    budget_projection:\n"
        f"      currency: USD\n"
        f"      window:\n"
        f"        starts_at: '{bp['window']['starts_at']}'\n"
        f"        ends_at: '{bp['window']['ends_at']}'\n"
        f"      projected_total: {bp['projected_total']}\n"
        f"      projection_method: heuristic\n"
        f"      drift_threshold_pct: {bp['drift_threshold_pct']}\n"
        f"      slot_projections:\n"
        f"        - slot: {slot_id}\n"
        f"          role: {row['role']}\n"
        f"          seniority: {row['seniority']}\n"
        f"          model_profile: '{row['model_profile']}'\n"
        f"          expected_invocations_per_week: {row['expected_invocations_per_week']}\n"
        f"          avg_tokens_per_invocation:\n"
        f"            input: {row['avg_tokens_per_invocation']['input']}\n"
        f"            output: {row['avg_tokens_per_invocation']['output']}\n"
        f"          unit_cost_usd: {row['unit_cost_usd']}\n"
        f"          projected_total_usd: {row['projected_total_usd']}\n"
        "---\n\n"
        "# Budget Drift Test\n"
    )
    (dec_dir / "DEC-20260510_0000-TestDrift-budget-test.md").write_text(
        dec_body, encoding="utf-8"
    )

    # Inject actual cost: 60% over projected → should trigger warning
    actual = {slot_id: projected * 1.60}
    result = server_mod.query_budget_drift(actual_slot_costs=actual)

    assert result["status"] == "ok", result
    assert result["threshold_exceeded"] is True
    assert result["finding_code"] == "team.budget.drift"
    assert result["severity"] == "warning"
    assert result["drift_direction"] == "over"
