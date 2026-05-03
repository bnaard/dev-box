#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
#   "httpx>=0.27",
#   "sqlite-vec>=0.1.0",
# ]
# ///
"""Provider-neutral processkit MCP gateway."""
from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path
from typing import Any


def _find_lib() -> Path:
    env = os.environ.get("PROCESSKIT_LIB_PATH")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve().parent
    while True:
        for c in (here / "src" / "lib", here / "_lib",
                  here / "context" / "skills" / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


sys.path.insert(0, str(_find_lib()))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402
from processkit.gateway import (  # noqa: E402
    GatewayRegistry,
    RuntimeMetadata,
    default_catalog_path,
    register_gateway_tools,
)
from processkit.gateway.runtime import (  # noqa: E402
    ENV_DAEMON_LAZY,
    normalize_daemon_path,
)

_TRANSPORT = os.environ.get("PROCESSKIT_GATEWAY_TRANSPORT", "stdio")
_IMPORT_MODE = os.environ.get("PROCESSKIT_GATEWAY_IMPORT_MODE", "eager")
_HOST = os.environ.get("PROCESSKIT_GATEWAY_HOST", "127.0.0.1")
_PORT = int(os.environ.get("PROCESSKIT_GATEWAY_PORT", "8000"))
_PATH = normalize_daemon_path(os.environ.get("PROCESSKIT_GATEWAY_PATH", "/mcp"))
_LAZY = (
    _IMPORT_MODE == "lazy-catalog"
    or os.environ.get(ENV_DAEMON_LAZY, "").lower() in {"1", "true", "yes", "on"}
)

server = FastMCP(
    "processkit-gateway",
    host=_HOST,
    port=_PORT,
    streamable_http_path=_PATH,
)

_SELF_SKILL = "processkit-gateway"
_REGISTRY = GatewayRegistry(
    self_skill=_SELF_SKILL,
    exclude_skills={"aggregate-mcp"},
    runtime=RuntimeMetadata(
        import_mode="lazy-catalog" if _LAZY else _IMPORT_MODE,
        lazy=_LAZY,
        lazy_daemon=_LAZY,
        transport=_TRANSPORT,
        daemon=_TRANSPORT == "streamable-http",
        proxy=os.environ.get("PROCESSKIT_GATEWAY_PROXY", "").lower()
        in {"1", "true", "yes", "on"},
        host=_HOST if _TRANSPORT == "streamable-http" else None,
        port=_PORT if _TRANSPORT == "streamable-http" else None,
        path=_PATH if _TRANSPORT == "streamable-http" else None,
    ),
    catalog_path=default_catalog_path(Path(__file__).resolve().parents[3]),
)
register_gateway_tools(server, _REGISTRY)


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_gateway_tools() -> dict:
    """List tools exposed by the processkit gateway."""
    return _REGISTRY.metadata()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def gateway_health() -> dict[str, Any]:
    """Return gateway health and runtime metadata."""
    return {
        "ok": True,
        "server": "processkit-gateway",
        "runtime": _REGISTRY.runtime.as_dict(),
        "tool_count": _REGISTRY.metadata(include_tools=False)["count"],
        "endpoint": {
            "host": _HOST,
            "port": _PORT,
            "path": _PATH,
        },
    }


def _parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="processkit-gateway")
    sub = parser.add_subparsers(dest="command")
    serve = sub.add_parser("serve")
    serve.add_argument(
        "--transport",
        choices=["stdio", "streamable-http"],
        default="stdio",
    )
    serve.add_argument("--host", default=_HOST)
    serve.add_argument("--port", type=int, default=_PORT)
    serve.add_argument("--path", default=_PATH)
    proxy = sub.add_parser("stdio-proxy")
    proxy.add_argument("--url", required=True)
    proxy.add_argument("--header", action="append", default=[])
    proxy.add_argument("--timeout", type=float, default=30.0)
    proxy.add_argument("--sse-read-timeout", type=float, default=300.0)
    proxy.add_argument("--no-terminate-on-close", action="store_true")
    catalog = sub.add_parser("catalog")
    catalog.add_argument("--write", action="store_true")
    catalog.add_argument("--path")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv or sys.argv[1:])
    if args.command in (None, "serve"):
        transport = getattr(args, "transport", "stdio")
        if transport == "streamable-http":
            server.settings.host = args.host
            server.settings.port = args.port
            server.settings.streamable_http_path = normalize_daemon_path(
                args.path
            )
            os.environ["PROCESSKIT_GATEWAY_TRANSPORT"] = "streamable-http"
        server.run(transport=transport)
        return 0
    if args.command == "stdio-proxy":
        from processkit.gateway.proxy import main as proxy_main

        proxy_args = ["--url", args.url]
        for header in args.header:
            proxy_args.extend(["--header", header])
        proxy_args.extend(["--timeout", str(args.timeout)])
        proxy_args.extend(["--sse-read-timeout", str(args.sse_read_timeout)])
        if args.no_terminate_on_close:
            proxy_args.append("--no-terminate-on-close")
        return proxy_main(proxy_args)
    if args.command == "catalog":
        target = Path(args.path) if args.path else None
        path = _REGISTRY.write_catalog(target)
        print(path.as_posix())
        return 0
    raise SystemExit(f"unknown command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
