"""Stdio-to-streamable-http proxy helpers for the processkit gateway."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from contextlib import asynccontextmanager
from dataclasses import dataclass
from functools import partial
from pathlib import Path
from typing import Any, Callable, Mapping, Sequence
from urllib.parse import urlparse


DEFAULT_HEADERS: Mapping[str, str] = {
    "Accept": "application/json, text/event-stream",
    "User-Agent": "processkit-gateway-stdio-proxy/0.1",
}


class ProxyConfigError(ValueError):
    """Raised when stdio proxy configuration is invalid."""


@dataclass(frozen=True)
class ProxyConfig:
    """Validated configuration for a stdio-to-streamable-http proxy."""

    url: str
    headers: Mapping[str, str]
    timeout: float = 30.0
    sse_read_timeout: float = 300.0
    terminate_on_close: bool = True
    daemon_command: Sequence[str] | None = None
    daemon_start_timeout: float = 30.0
    daemon_log_path: Path | None = None

    @classmethod
    def from_values(
        cls,
        *,
        url: str,
        headers: Mapping[str, str] | None = None,
        timeout: float = 30.0,
        sse_read_timeout: float = 300.0,
        terminate_on_close: bool = True,
        daemon_command: Sequence[str] | None = None,
        daemon_start_timeout: float = 30.0,
        daemon_log_path: str | Path | None = None,
    ) -> "ProxyConfig":
        merged = dict(DEFAULT_HEADERS)
        if headers:
            merged.update(headers)
        return cls(
            url=validate_streamable_http_url(url),
            headers=merged,
            timeout=timeout,
            sse_read_timeout=sse_read_timeout,
            terminate_on_close=terminate_on_close,
            daemon_command=tuple(daemon_command) if daemon_command else None,
            daemon_start_timeout=daemon_start_timeout,
            daemon_log_path=Path(daemon_log_path) if daemon_log_path else None,
        )


@dataclass(frozen=True)
class McpSdkBindings:
    """Runtime imports from the optional MCP SDK."""

    client_session: type[Any]
    stdio_server: Callable[..., Any]
    streamable_client: Callable[..., Any]
    streamable_client_uses_http_client: bool
    async_http_client: type[Any] | None = None


def validate_streamable_http_url(url: str) -> str:
    """Return a normalized streamable-http URL or raise ``ProxyConfigError``."""

    candidate = url.strip()
    if not candidate:
        raise ProxyConfigError("upstream URL is required")

    parsed = urlparse(candidate)
    if parsed.scheme not in {"http", "https"}:
        raise ProxyConfigError("upstream URL must use http or https")
    if not parsed.netloc:
        raise ProxyConfigError("upstream URL must include a host")
    if parsed.username or parsed.password:
        raise ProxyConfigError("upstream URL must not embed credentials")
    if parsed.fragment:
        raise ProxyConfigError("upstream URL must not include a fragment")
    return candidate


def parse_header_args(values: Sequence[str] | None) -> dict[str, str]:
    """Parse repeated ``Name: value`` CLI headers."""

    headers: dict[str, str] = {}
    for raw_value in values or ():
        name, separator, value = raw_value.partition(":")
        if not separator:
            raise ProxyConfigError(
                f"header must be in 'Name: value' form: {raw_value!r}"
            )
        name = name.strip()
        value = value.strip()
        if not name:
            raise ProxyConfigError("header name must not be empty")
        if any(ord(char) < 33 or char in ":()" for char in name):
            raise ProxyConfigError(f"invalid header name: {name!r}")
        headers[name] = value
    return headers


def _local_http_binding(url: str) -> tuple[str, int, str] | None:
    """Return local daemon host, port, path if the URL is locally owned."""

    parsed = urlparse(validate_streamable_http_url(url))
    if parsed.scheme != "http":
        return None
    if parsed.hostname not in {"127.0.0.1", "localhost", "::1"}:
        return None
    port = parsed.port or 80
    return parsed.hostname, port, parsed.path or "/"


async def _tcp_ready(host: str, port: int, *, timeout: float) -> bool:
    """Return whether a TCP listener is available within ``timeout``."""

    import anyio

    deadline = anyio.current_time() + max(timeout, 0.0)
    while True:
        try:
            stream = await anyio.connect_tcp(host, port)
            await stream.aclose()
            return True
        except OSError:
            if anyio.current_time() >= deadline:
                return False
            await anyio.sleep(0.1)


def _spawn_daemon(
    command: Sequence[str],
    *,
    log_path: Path | None = None,
) -> subprocess.Popen[Any]:
    """Start a gateway daemon without contaminating MCP stdio."""

    target = log_path or Path("/tmp/processkit-gateway-daemon.log")
    target.parent.mkdir(parents=True, exist_ok=True)
    log_file = target.open("ab")
    try:
        return subprocess.Popen(
            list(command),
            stdin=subprocess.DEVNULL,
            stdout=log_file,
            stderr=subprocess.STDOUT,
            close_fds=True,
            start_new_session=True,
        )
    except Exception:
        log_file.close()
        raise


async def ensure_upstream_daemon(
    config: ProxyConfig,
) -> subprocess.Popen[Any] | None:
    """Start the local daemon if this proxy owns startup for the URL."""

    if not config.daemon_command:
        return None

    binding = _local_http_binding(config.url)
    if binding is None:
        return None

    host, port, _path = binding
    if await _tcp_ready(host, port, timeout=0.2):
        return None

    process = _spawn_daemon(
        config.daemon_command,
        log_path=config.daemon_log_path,
    )
    if await _tcp_ready(host, port, timeout=config.daemon_start_timeout):
        return process

    exit_code = process.poll()
    message = f"gateway daemon did not become ready at {host}:{port}"
    if exit_code is not None:
        message += f"; daemon exited with code {exit_code}"
    else:
        process.terminate()
    if config.daemon_log_path is not None:
        message += f"; see {config.daemon_log_path}"
    raise RuntimeError(message)


def load_mcp_sdk() -> McpSdkBindings:
    """Import MCP transport bindings without importing processkit servers."""

    from mcp import ClientSession
    from mcp.client import streamable_http
    from mcp.server.stdio import stdio_server

    current_client = getattr(streamable_http, "streamable_http_client", None)
    if current_client is not None:
        try:
            from httpx import AsyncClient
        except ImportError:  # pragma: no cover - mcp[cli] brings httpx.
            AsyncClient = None  # type: ignore[assignment]
        return McpSdkBindings(
            client_session=ClientSession,
            stdio_server=stdio_server,
            streamable_client=current_client,
            streamable_client_uses_http_client=True,
            async_http_client=AsyncClient,
        )

    legacy_client = getattr(streamable_http, "streamablehttp_client", None)
    if legacy_client is None:
        raise RuntimeError(
            "mcp.client.streamable_http does not expose a streamable "
            "HTTP client"
        )
    return McpSdkBindings(
        client_session=ClientSession,
        stdio_server=stdio_server,
        streamable_client=legacy_client,
        streamable_client_uses_http_client=False,
    )


@asynccontextmanager
async def _upstream_streams(
    config: ProxyConfig,
    sdk: McpSdkBindings,
):
    if sdk.streamable_client_uses_http_client:
        if sdk.async_http_client is None:
            async with sdk.streamable_client(
                config.url,
                terminate_on_close=config.terminate_on_close,
            ) as streams:
                yield streams
            return

        async with sdk.async_http_client(
            headers=dict(config.headers),
            timeout=config.timeout,
        ) as http_client:
            async with sdk.streamable_client(
                config.url,
                http_client=http_client,
                terminate_on_close=config.terminate_on_close,
            ) as streams:
                yield streams
            return

    async with sdk.streamable_client(
        config.url,
        headers=dict(config.headers),
        timeout=config.timeout,
        sse_read_timeout=config.sse_read_timeout,
        terminate_on_close=config.terminate_on_close,
    ) as streams:
        yield streams


async def _forward_messages(
    read_stream: Any,
    write_stream: Any,
    cancel_scope: Any,
) -> None:
    try:
        async for message in read_stream:
            if isinstance(message, Exception):
                raise message
            await write_stream.send(message)
    finally:
        close = getattr(write_stream, "aclose", None)
        if close is not None:
            await close()
        cancel_scope.cancel()


async def run_stdio_proxy(
    config: ProxyConfig,
    *,
    sdk: McpSdkBindings | None = None,
    stdin: Any | None = None,
    stdout: Any | None = None,
) -> int:
    """Relay MCP session messages between stdio and streamable HTTP."""

    sdk = sdk or load_mcp_sdk()
    daemon_process = await ensure_upstream_daemon(config)
    if daemon_process is not None:
        print(
            f"processkit-gateway daemon started with pid "
            f"{daemon_process.pid}",
            file=sys.stderr,
        )

    import anyio

    async with sdk.stdio_server(stdin=stdin, stdout=stdout) as local_streams:
        async with _upstream_streams(config, sdk) as upstream_streams:
            local_read, local_write = local_streams
            upstream_read, upstream_write, _get_session_id = upstream_streams
            async with anyio.create_task_group() as task_group:
                task_group.start_soon(
                    _forward_messages,
                    local_read,
                    upstream_write,
                    task_group.cancel_scope,
                )
                task_group.start_soon(
                    _forward_messages,
                    upstream_read,
                    local_write,
                    task_group.cancel_scope,
                )
    return 0


def build_arg_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="processkit-gateway-stdio-proxy")
    parser.add_argument("--url", required=True)
    parser.add_argument(
        "--header",
        action="append",
        default=[],
        help="HTTP header in 'Name: value' form; may be repeated.",
    )
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--sse-read-timeout", type=float, default=300.0)
    parser.add_argument(
        "--no-terminate-on-close",
        action="store_true",
        help="Do not send the streamable-http session DELETE on exit.",
    )
    return parser


async def async_main(
    argv: Sequence[str] | None = None,
    *,
    sdk: McpSdkBindings | None = None,
    stdin: Any | None = None,
    stdout: Any | None = None,
    daemon_command: Sequence[str] | None = None,
) -> int:
    """CLI-callable async entrypoint for the stdio proxy."""

    args = build_arg_parser().parse_args(argv)
    config = ProxyConfig.from_values(
        url=args.url,
        headers=parse_header_args(args.header),
        timeout=args.timeout,
        sse_read_timeout=args.sse_read_timeout,
        terminate_on_close=not args.no_terminate_on_close,
        daemon_command=daemon_command,
        daemon_log_path=os.environ.get(
            "PROCESSKIT_GATEWAY_DAEMON_LOG",
            "/tmp/processkit-gateway-daemon.log",
        ),
    )
    return await run_stdio_proxy(
        config,
        sdk=sdk,
        stdin=stdin,
        stdout=stdout,
    )


def main(
    argv: Sequence[str] | None = None,
    *,
    daemon_command: Sequence[str] | None = None,
) -> int:
    """Synchronous CLI wrapper for ``async_main``."""

    import anyio

    return anyio.run(partial(
        async_main,
        argv,
        daemon_command=daemon_command,
    ))


__all__ = [
    "DEFAULT_HEADERS",
    "McpSdkBindings",
    "ProxyConfig",
    "ProxyConfigError",
    "async_main",
    "build_arg_parser",
    "load_mcp_sdk",
    "main",
    "parse_header_args",
    "run_stdio_proxy",
    "ensure_upstream_daemon",
    "validate_streamable_http_url",
]
