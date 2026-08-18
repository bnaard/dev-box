"""Daemon-runtime tests for processkit gateway helpers."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest


LIB_ROOT = Path(__file__).resolve().parents[3] / "_lib"
sys.path.insert(0, str(LIB_ROOT))

from processkit.gateway.runtime import (  # noqa: E402
    DAEMON_ENV_DEFAULTS,
    DEFAULT_DAEMON_HOST,
    DEFAULT_DAEMON_PATH,
    DEFAULT_DAEMON_PORT,
    DEFAULT_DAEMON_TRANSPORT,
    ENV_DAEMON_HOST,
    ENV_DAEMON_LAZY,
    ENV_DAEMON_PATH,
    ENV_DAEMON_PORT,
    ENV_DAEMON_PROXY,
    ENV_TRANSPORT,
    DaemonConfig,
    RuntimeMetadata,
    daemon_config_from_env,
    daemon_port_from_env,
    normalize_daemon_path,
    runtime_metadata_from_env,
)


def test_daemon_defaults_are_local_only_and_streamable_http():
    config = DaemonConfig()

    assert config.host == "127.0.0.1"
    assert config.host == DEFAULT_DAEMON_HOST
    assert config.port == DEFAULT_DAEMON_PORT
    assert config.path == DEFAULT_DAEMON_PATH
    assert config.transport == DEFAULT_DAEMON_TRANSPORT
    assert config.url == "http://127.0.0.1:8000/mcp"
    assert config.as_env() == DAEMON_ENV_DEFAULTS


def test_daemon_config_reads_env_overrides():
    config = daemon_config_from_env({
        ENV_DAEMON_HOST: "127.0.0.2",
        ENV_DAEMON_PORT: "8123",
        ENV_DAEMON_PATH: "gateway",
        ENV_TRANSPORT: DEFAULT_DAEMON_TRANSPORT,
    })

    assert config.host == "127.0.0.2"
    assert config.port == 8123
    assert config.path == "/gateway"
    assert config.url == "http://127.0.0.2:8123/gateway"
    assert config.as_env()[ENV_DAEMON_PATH] == "/gateway"


@pytest.mark.parametrize(
    ("raw_path", "expected"),
    [
        (None, "/mcp"),
        ("", "/mcp"),
        ("mcp", "/mcp"),
        ("/mcp/", "/mcp"),
        ("/", "/"),
    ],
)
def test_normalize_daemon_path(raw_path, expected):
    assert normalize_daemon_path(raw_path) == expected


@pytest.mark.parametrize("raw_port", ["0", "65536", "abc"])
def test_daemon_port_validation_rejects_invalid_values(raw_port):
    with pytest.raises(ValueError):
        daemon_port_from_env(raw_port)


def test_runtime_metadata_defaults_to_stdio_non_daemon_mode():
    payload = RuntimeMetadata().as_dict()

    assert payload["transport"] == "stdio"
    assert payload["daemon"] is False
    assert payload["proxy"] is False
    assert payload["lazy"] is False
    assert payload["lazy_daemon"] is False
    assert "host" not in payload


def test_runtime_metadata_for_daemon_includes_binding_fields():
    metadata = RuntimeMetadata.for_daemon(
        DaemonConfig(host="127.0.0.3", port=9123, path="mcp"),
        lazy=True,
    ).as_dict()

    assert metadata["transport"] == DEFAULT_DAEMON_TRANSPORT
    assert metadata["daemon"] is True
    assert metadata["proxy"] is False
    assert metadata["lazy"] is True
    assert metadata["lazy_daemon"] is True
    assert metadata["host"] == "127.0.0.3"
    assert metadata["port"] == 9123
    assert metadata["path"] == "/mcp"


def test_runtime_metadata_from_env_detects_daemon_and_proxy_modes():
    daemon_metadata = runtime_metadata_from_env({
        ENV_TRANSPORT: DEFAULT_DAEMON_TRANSPORT,
        ENV_DAEMON_LAZY: "true",
    }).as_dict()
    proxy_metadata = runtime_metadata_from_env({
        ENV_DAEMON_PROXY: "1",
        ENV_DAEMON_PORT: "9001",
    }).as_dict()

    assert daemon_metadata["daemon"] is True
    assert daemon_metadata["lazy"] is True
    assert daemon_metadata["host"] == DEFAULT_DAEMON_HOST
    assert proxy_metadata["transport"] == "stdio"
    assert proxy_metadata["daemon"] is False
    assert proxy_metadata["proxy"] is True
    assert proxy_metadata["port"] == 9001
