"""Focused tests for the processkit gateway stdio proxy helper."""

from __future__ import annotations

import asyncio
import importlib
import importlib.util
import inspect
import sys
import types
from pathlib import Path

import pytest


SKILL_ROOT = Path(__file__).resolve().parents[1]
LIB_ROOT = SKILL_ROOT.parents[1] / "_lib"


def _load_proxy():
    if str(LIB_ROOT) not in sys.path:
        sys.path.insert(0, str(LIB_ROOT))
    return importlib.import_module("processkit.gateway.proxy")


def test_proxy_config_validates_url_and_applies_default_headers():
    proxy = _load_proxy()

    config = proxy.ProxyConfig.from_values(
        url=" https://example.test/mcp ",
        headers={"Authorization": "Bearer token"},
    )

    assert config.url == "https://example.test/mcp"
    assert config.headers["Accept"] == "application/json, text/event-stream"
    assert config.headers["User-Agent"].startswith(
        "processkit-gateway-stdio-proxy/"
    )
    assert config.headers["Authorization"] == "Bearer token"

    for invalid_url in (
        "",
        "file:///tmp/server",
        "http://",
        "https://user:pass@example.test/mcp",
        "https://example.test/mcp#fragment",
    ):
        with pytest.raises(proxy.ProxyConfigError):
            proxy.ProxyConfig.from_values(url=invalid_url)


def test_parse_header_args_rejects_malformed_headers():
    proxy = _load_proxy()

    assert proxy.parse_header_args([
        "Authorization: Bearer token",
        "X-Trace-Id: abc",
    ]) == {
        "Authorization": "Bearer token",
        "X-Trace-Id": "abc",
    }

    with pytest.raises(proxy.ProxyConfigError):
        proxy.parse_header_args(["missing separator"])
    with pytest.raises(proxy.ProxyConfigError):
        proxy.parse_header_args([": missing name"])


def test_proxy_import_does_not_import_processkit_source_servers(monkeypatch):
    before = {
        name
        for name in sys.modules
        if name.startswith("processkit_gateway_")
    }
    sys.modules.pop("processkit.gateway.proxy", None)

    original_spec_from_file_location = importlib.util.spec_from_file_location

    def guarded_spec_from_file_location(name, location, *args, **kwargs):
        if str(location).endswith("/mcp/server.py"):
            raise AssertionError("proxy import must not import source servers")
        return original_spec_from_file_location(
            name,
            location,
            *args,
            **kwargs,
        )

    monkeypatch.setattr(
        importlib.util,
        "spec_from_file_location",
        guarded_spec_from_file_location,
    )

    _load_proxy()

    after = {
        name
        for name in sys.modules
        if name.startswith("processkit_gateway_")
    }
    assert after == before


def test_load_mcp_sdk_prefers_current_streamable_http_client(monkeypatch):
    proxy = _load_proxy()

    class FakeClientSession:
        pass

    def fake_streamable_http_client():
        pass

    def fake_stdio_server():
        pass

    mcp_module = types.ModuleType("mcp")
    mcp_module.ClientSession = FakeClientSession
    client_module = types.ModuleType("mcp.client")
    streamable_module = types.ModuleType("mcp.client.streamable_http")
    streamable_module.streamable_http_client = fake_streamable_http_client
    server_module = types.ModuleType("mcp.server")
    stdio_module = types.ModuleType("mcp.server.stdio")
    stdio_module.stdio_server = fake_stdio_server

    monkeypatch.setitem(sys.modules, "mcp", mcp_module)
    monkeypatch.setitem(sys.modules, "mcp.client", client_module)
    monkeypatch.setitem(
        sys.modules,
        "mcp.client.streamable_http",
        streamable_module,
    )
    monkeypatch.setitem(sys.modules, "mcp.server", server_module)
    monkeypatch.setitem(sys.modules, "mcp.server.stdio", stdio_module)

    sdk = proxy.load_mcp_sdk()

    assert sdk.client_session is FakeClientSession
    assert sdk.stdio_server is fake_stdio_server
    assert sdk.streamable_client is fake_streamable_http_client
    assert sdk.streamable_client_uses_http_client is True


def test_async_entrypoint_is_cli_callable(monkeypatch):
    proxy = _load_proxy()
    captured = {}
    sentinel_sdk = object()

    async def fake_run_stdio_proxy(
        config,
        *,
        sdk=None,
        stdin=None,
        stdout=None,
    ):
        captured["config"] = config
        captured["sdk"] = sdk
        captured["stdin"] = stdin
        captured["stdout"] = stdout
        return 0

    monkeypatch.setattr(proxy, "run_stdio_proxy", fake_run_stdio_proxy)

    assert inspect.iscoroutinefunction(proxy.async_main)

    result = asyncio.run(proxy.async_main(
        [
            "--url",
            "http://127.0.0.1:8765/mcp",
            "--header",
            "Authorization: Bearer token",
            "--no-terminate-on-close",
        ],
        sdk=sentinel_sdk,
    ))

    assert result == 0
    assert captured["sdk"] is sentinel_sdk
    assert captured["config"].url == "http://127.0.0.1:8765/mcp"
    assert captured["config"].headers["Authorization"] == "Bearer token"
    assert captured["config"].terminate_on_close is False
