"""Focused tests for the aggregate processkit MCP server.

Run with:

    uv run --with mcp --with pyyaml --with jsonschema --with httpx \
        --with sqlite-vec --with pytest pytest \
        src/context/skills/processkit/aggregate-mcp/scripts -v
"""
from __future__ import annotations

import asyncio
import importlib
import importlib.util
import json
import os
import sys
from pathlib import Path

import pytest


SKILL_ROOT = Path(__file__).resolve().parents[1]
SERVER_PATH = SKILL_ROOT / "mcp" / "server.py"
CONFIG_PATH = SKILL_ROOT / "mcp" / "mcp-config.aggregate.json"


def _load_server(unique_suffix: str = ""):
    """Import the aggregate-mcp server fresh.

    Each call gets a unique module name so eager and lazy variants do
    not collide in ``sys.modules`` (the server runs work at import
    time).
    """
    name = f"aggregate_mcp_server{('_' + unique_suffix) if unique_suffix else ''}"
    sys.modules.pop(name, None)
    spec = importlib.util.spec_from_file_location(name, SERVER_PATH)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _purge_per_skill_modules() -> set[str]:
    """Drop any per-skill modules a previous import seeded into sys.modules.

    Returns the set of module names removed so callers can assert which
    skill modules a subsequent import (re)loads.
    """
    removed: set[str] = set()
    for name in list(sys.modules):
        if name.startswith("processkit_aggregate_") or name.startswith(
            "processkit_gateway_"
        ):
            sys.modules.pop(name, None)
            removed.add(name)
    return removed


@pytest.fixture
def clean_env(monkeypatch):
    monkeypatch.delenv("PROCESSKIT_MCP_LAZY", raising=False)
    monkeypatch.delenv("PROCESSKIT_MCP_MODE", raising=False)
    monkeypatch.delenv("PROCESSKIT_MCP_CATALOG_PATH", raising=False)
    yield monkeypatch


def test_list_aggregate_tools_reports_gateway_metadata(clean_env):
    aggregate = _load_server("eager_default")

    result = aggregate.list_aggregate_tools()

    assert result["count"] == len(result["tools"])
    assert result["source_server_count"] > 0
    assert result["runtime"]["lazy_daemon"] is False
    assert result["runtime"]["import_mode"] in {
        "eager",
        "gateway_registry",
    }
    assert result["runtime"]["registry_backend"] in {
        "eager_import",
        "gateway_registry",
    }
    assert isinstance(
        result["runtime"]["canonical_gateway_available"],
        bool,
    )

    first_tool = result["tools"][0]
    assert {
        "name",
        "source_skill",
        "source_server_path",
        "source_tool",
        "annotations",
        "collision_status",
        "collision_sources",
        "deduplicated",
    } <= set(first_tool)
    assert first_tool["source_server_path"].endswith("/mcp/server.py")
    assert first_tool["collision_status"] in {
        "unique",
        "primary",
        "namespaced_duplicate",
    }


def test_aggregate_uses_gateway_registry_when_available(clean_env):
    aggregate = _load_server("eager_gateway")

    result = aggregate.list_aggregate_tools()

    assert result["runtime"]["canonical_gateway_available"] is True
    assert result["runtime"]["registry_backend"] == "gateway_registry"
    assert result["runtime"]["import_mode"] == "gateway_registry"


def test_list_aggregate_tools_metadata_is_json_serializable(clean_env):
    aggregate = _load_server("eager_json")

    json.dumps(aggregate.list_aggregate_tools())


def test_opt_in_aggregate_config_fragment_registers_single_server():
    config = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))

    assert list(config["mcpServers"]) == ["processkit-aggregate-mcp"]
    server_config = config["mcpServers"]["processkit-aggregate-mcp"]
    assert server_config["command"] == "uv"
    assert server_config["args"] == [
        "run",
        "context/skills/processkit/aggregate-mcp/mcp/server.py",
    ]
    assert server_config["env"]["PROCESSKIT_MCP_MODE"] == "aggregate"


# -- lazy_catalog mode ------------------------------------------------


def test_lazy_mode_skips_per_skill_imports_until_first_call(clean_env):
    """Opt into lazy_catalog: per-skill modules must NOT be imported eagerly."""
    clean_env.setenv("PROCESSKIT_MCP_LAZY", "1")
    _purge_per_skill_modules()

    aggregate = _load_server("lazy_first")

    # No per-skill ``processkit_aggregate_*`` or gateway-loader
    # ``processkit_gateway_*`` modules should be in sys.modules yet.
    leaked = {
        name for name in sys.modules
        if name.startswith("processkit_aggregate_")
        or name.startswith("processkit_gateway_")
    }
    assert leaked == set(), (
        f"lazy mode must not import per-skill modules at startup; got {leaked}"
    )

    result = aggregate.list_aggregate_tools()
    assert result["runtime"]["import_mode"] == "lazy_catalog"
    assert result["runtime"]["lazy_daemon"] is True
    assert result["runtime"]["canonical_gateway_available"] is True
    assert "catalog_path" in result["runtime"]
    assert result["count"] > 0


def test_lazy_mode_imports_only_owning_skill_on_first_call(clean_env):
    """First tool call imports only the owning skill's module.

    The shared ``processkit.gateway.loader.import_server`` helper memoizes
    its work via ``functools.lru_cache``. We use ``cache_info().currsize``
    as a deterministic counter for "how many distinct skill modules has
    the gateway imported so far" — a much more reliable signal than
    poking ``sys.modules`` (the lazy module loader does not register
    modules under their own name there).
    """
    clean_env.setenv("PROCESSKIT_MCP_LAZY", "1")

    # Pre-clear the import_server LRU cache so prior tests don't bleed
    # into our import counter.
    sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "_lib"))
    from processkit.gateway import loader as gateway_loader

    gateway_loader.import_server.cache_clear()

    aggregate = _load_server("lazy_isolation")
    registry = aggregate._LAZY_REGISTRY
    assert registry is not None

    pre_size = gateway_loader.import_server.cache_info().currsize
    assert pre_size == 0, (
        f"lazy-catalog mode must not import any source server at startup; "
        f"import_server cache size was {pre_size}"
    )

    target = next(
        item for item in registry.tools
        if item["source_skill"] not in {"processkit-gateway", "aggregate-mcp"}
    )
    target_skill = target["source_skill"]

    try:
        asyncio.run(
            registry.call_tool(target["name"], {"__lazy_probe__": True})
        )
    except Exception:
        # Tool likely complained about missing args — we only care that
        # the import side-effect ran on the way in.
        pass

    post_size = gateway_loader.import_server.cache_info().currsize
    assert post_size == 1, (
        f"first tool call must import exactly one source server, "
        f"got cache size {post_size}"
    )

    # Confirm the cache key really is the target skill's server path.
    cached_paths = list(gateway_loader.import_server.cache_parameters() or {})
    # cache_parameters only describes config; instead inspect by calling
    # again and verifying no new import happened.
    asyncio.run(_safely_call(registry, target["name"]))
    repeat_size = gateway_loader.import_server.cache_info().currsize
    assert repeat_size == 1, (
        "repeat call to same tool should hit the import cache"
    )

    # And cross-check: the source path of an unrelated skill is not yet
    # in the cache. Pick a different skill and verify a call adds 1.
    other = next(
        item for item in registry.tools
        if item["source_skill"] not in {
            target_skill, "processkit-gateway", "aggregate-mcp",
        }
    )
    asyncio.run(_safely_call(registry, other["name"]))
    after_other = gateway_loader.import_server.cache_info().currsize
    assert after_other == 2, (
        f"calling a tool on a second skill should import exactly one more "
        f"source server, got cache size {after_other}"
    )


async def _safely_call(registry, tool_name: str) -> None:
    try:
        await registry.call_tool(tool_name, {})
    except Exception:
        pass


def test_list_tools_fast_in_both_modes(clean_env):
    """ListTools surface size is identical between eager and lazy modes."""
    clean_env.delenv("PROCESSKIT_MCP_LAZY", raising=False)
    eager = _load_server("compare_eager").list_aggregate_tools()

    clean_env.setenv("PROCESSKIT_MCP_LAZY", "1")
    _purge_per_skill_modules()
    lazy = _load_server("compare_lazy").list_aggregate_tools()

    eager_names = sorted(item["name"] for item in eager["tools"])
    lazy_names = sorted(item["name"] for item in lazy["tools"])
    assert eager_names == lazy_names

    assert eager["source_server_count"] == lazy["source_server_count"]
    assert eager["runtime"]["import_mode"] in {"eager", "gateway_registry"}
    assert lazy["runtime"]["import_mode"] == "lazy_catalog"


def test_lazy_mode_falls_back_when_catalog_missing(clean_env, tmp_path):
    """Pointing the catalog override at a missing file falls back to eager."""
    clean_env.setenv("PROCESSKIT_MCP_LAZY", "1")
    clean_env.setenv(
        "PROCESSKIT_MCP_CATALOG_PATH",
        str(tmp_path / "does-not-exist.json"),
    )
    _purge_per_skill_modules()

    aggregate = _load_server("lazy_fallback")
    result = aggregate.list_aggregate_tools()
    # No catalog → fall through to eager. lazy_daemon must reflect that.
    assert result["runtime"]["import_mode"] in {"eager", "gateway_registry"}
    assert result["runtime"]["lazy_daemon"] is False
