"""Focused tests for the aggregate processkit MCP server.

Run with:

    uv run --with mcp --with pyyaml --with jsonschema --with httpx \
        --with sqlite-vec --with pytest pytest \
        src/context/skills/processkit/aggregate-mcp/scripts -v
"""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path


SKILL_ROOT = Path(__file__).resolve().parents[1]
SERVER_PATH = SKILL_ROOT / "mcp" / "server.py"
CONFIG_PATH = SKILL_ROOT / "mcp" / "mcp-config.aggregate.json"


def _load_server():
    spec = importlib.util.spec_from_file_location(
        "aggregate_mcp_server",
        SERVER_PATH,
    )
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_list_aggregate_tools_reports_gateway_metadata():
    aggregate = _load_server()

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


def test_aggregate_uses_gateway_registry_when_available():
    aggregate = _load_server()

    result = aggregate.list_aggregate_tools()

    assert result["runtime"]["canonical_gateway_available"] is True
    assert result["runtime"]["registry_backend"] == "gateway_registry"
    assert result["runtime"]["import_mode"] == "gateway_registry"


def test_list_aggregate_tools_metadata_is_json_serializable():
    aggregate = _load_server()

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
