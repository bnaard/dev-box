"""Focused tests for the processkit gateway MCP server.

Run with:

    uv run --with mcp --with pyyaml --with jsonschema --with httpx \
        --with sqlite-vec --with pytest pytest \
        src/context/skills/processkit/processkit-gateway/scripts -v
"""
from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path


SKILL_ROOT = Path(__file__).resolve().parents[1]
SERVER_PATH = SKILL_ROOT / "mcp" / "server.py"
CONFIG_PATH = SKILL_ROOT / "mcp" / "mcp-config.json"


def _load_server():
    spec = importlib.util.spec_from_file_location(
        "processkit_gateway_server",
        SERVER_PATH,
    )
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _load_server_with_env(env: dict[str, str]):
    original = {key: os.environ.get(key) for key in env}
    os.environ.update(env)
    try:
        return _load_server()
    finally:
        for key, value in original.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def test_list_gateway_tools_reports_runtime_metadata():
    gateway = _load_server()

    result = gateway.list_gateway_tools()

    assert result["count"] == len(result["tools"])
    assert result["source_server_count"] > 0
    assert result["runtime"]["import_mode"] == "eager"
    assert result["runtime"]["lazy_daemon"] is False
    assert result["runtime"]["transport"] == "stdio"
    assert "create_workitem" in gateway.server._tool_manager._tools


def test_gateway_metadata_is_json_serializable():
    gateway = _load_server()

    json.dumps(gateway.list_gateway_tools())
    json.dumps(gateway.gateway_health())


def test_gateway_can_write_and_load_lazy_catalog(tmp_path):
    gateway = _load_server()
    catalog = tmp_path / "tool-catalog.json"

    gateway._REGISTRY.write_catalog(catalog)

    lazy = gateway.GatewayRegistry(
        self_skill="processkit-gateway",
        exclude_skills={"aggregate-mcp"},
        runtime=gateway.RuntimeMetadata(
            import_mode="lazy-catalog",
            lazy_daemon=True,
            transport="streamable-http",
            daemon=True,
        ),
        catalog_path=catalog,
    )
    lazy.load()

    metadata = lazy.metadata()
    assert metadata["count"] == gateway.list_gateway_tools()["count"]
    assert metadata["runtime"]["import_mode"] == "lazy-catalog"
    assert "tool" not in metadata["tools"][0]


def test_lazy_gateway_import_mode_registers_catalog_tools(tmp_path):
    eager = _load_server()
    catalog = tmp_path / "tool-catalog.json"
    eager._REGISTRY.write_catalog(catalog)

    lazy_registry = eager.GatewayRegistry(
        self_skill="processkit-gateway",
        exclude_skills={"aggregate-mcp"},
        runtime=eager.RuntimeMetadata(import_mode="lazy-catalog"),
        catalog_path=catalog,
    )
    lazy_server = eager.FastMCP("lazy-test")

    eager.register_gateway_tools(lazy_server, lazy_registry)

    tool = lazy_server._tool_manager._tools["create_workitem"]
    assert tool.parameters["required"] == ["title"]
    assert tool.fn_metadata.arg_model.__name__ == "PassthroughArguments"


def test_gateway_config_registers_single_gateway_server():
    config = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))

    assert list(config["mcpServers"]) == ["processkit-gateway"]
    server_config = config["mcpServers"]["processkit-gateway"]
    assert server_config["command"] == "uv"
    assert server_config["args"] == [
        "run",
        "context/skills/processkit/processkit-gateway/mcp/server.py",
    ]
    assert server_config["env"]["PROCESSKIT_MCP_MODE"] == "gateway"


def test_main_runs_streamable_http_transport(monkeypatch):
    gateway = _load_server()
    captured = {}

    def fake_run(*, transport):
        captured["transport"] = transport
        captured["host"] = gateway.server.settings.host
        captured["port"] = gateway.server.settings.port
        captured["path"] = gateway.server.settings.streamable_http_path

    monkeypatch.setattr(gateway.server, "run", fake_run)

    assert gateway.main([
        "serve",
        "--transport",
        "streamable-http",
        "--host",
        "127.0.0.2",
        "--port",
        "9123",
        "--path",
        "gateway",
    ]) == 0

    assert captured == {
        "transport": "streamable-http",
        "host": "127.0.0.2",
        "port": 9123,
        "path": "/gateway",
    }


def test_main_delegates_stdio_proxy(monkeypatch):
    gateway = _load_server()
    proxy = __import__("processkit.gateway.proxy", fromlist=["main"])
    captured = {}

    def fake_proxy_main(argv):
        captured["argv"] = argv
        return 0

    monkeypatch.setattr(proxy, "main", fake_proxy_main)

    assert gateway.main([
        "stdio-proxy",
        "--url",
        "http://127.0.0.1:8000/mcp",
        "--header",
        "Authorization: Bearer token",
        "--no-terminate-on-close",
    ]) == 0

    assert captured["argv"] == [
        "--url",
        "http://127.0.0.1:8000/mcp",
        "--header",
        "Authorization: Bearer token",
        "--timeout",
        "30.0",
        "--sse-read-timeout",
        "300.0",
        "--no-terminate-on-close",
    ]


def test_main_writes_catalog(tmp_path, capsys):
    gateway = _load_server()
    catalog = tmp_path / "catalog.json"

    assert gateway.main(["catalog", "--write", "--path", str(catalog)]) == 0

    captured = capsys.readouterr()
    assert captured.out.strip() == catalog.as_posix()
    payload = json.loads(catalog.read_text(encoding="utf-8"))
    assert payload["catalog_version"] == 1
    assert len(payload["tools"]) == gateway.list_gateway_tools()["count"]
