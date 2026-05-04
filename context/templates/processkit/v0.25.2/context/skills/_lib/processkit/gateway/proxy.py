"""Stdio-to-streamable-http proxy helpers for the processkit gateway."""

from __future__ import annotations

import argparse
from contextlib import asynccontextmanager
from dataclasses import dataclass
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

    @classmethod
    def from_values(
        cls,
        *,
        url: str,
        headers: Mapping[str, str] | None = None,
        timeout: float = 30.0,
        sse_read_timeout: float = 300.0,
        terminate_on_close: bool = True,
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
) -> int:
    """CLI-callable async entrypoint for the stdio proxy."""

    args = build_arg_parser().parse_args(argv)
    config = ProxyConfig.from_values(
        url=args.url,
        headers=parse_header_args(args.header),
        timeout=args.timeout,
        sse_read_timeout=args.sse_read_timeout,
        terminate_on_close=not args.no_terminate_on_close,
    )
    return await run_stdio_proxy(
        config,
        sdk=sdk,
        stdin=stdin,
        stdout=stdout,
    )


def main(argv: Sequence[str] | None = None) -> int:
    """Synchronous CLI wrapper for ``async_main``."""

    import anyio

    return anyio.run(async_main, argv)


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
    "validate_streamable_http_url",
]
