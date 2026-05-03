"""mcp_gateway check — report gateway-mode MCP config health.

Detect-only. Gateway mode is valid when a derived project's `.mcp.json`
registers `processkit-gateway` instead of the granular per-skill
processkit servers. This check reports whether the gateway config is
present and catches accidental registration of both gateway and granular
servers in the root harness config.
"""

from __future__ import annotations

import json
from pathlib import Path

from .common import CheckResult


GATEWAY_CONFIG_REL = Path(
    "context/skills/processkit/processkit-gateway/mcp/mcp-config.json"
)
SRC_GATEWAY_CONFIG_REL = Path(
    "src/context/skills/processkit/processkit-gateway/mcp/mcp-config.json"
)
GATEWAY_SERVER_NAME = "processkit-gateway"


def _load_json(path: Path) -> tuple[dict | None, str | None]:
    try:
        return json.loads(path.read_text(encoding="utf-8")), None
    except (OSError, json.JSONDecodeError) as e:
        return None, str(e)


def _processkit_server_names(servers: dict) -> list[str]:
    return sorted(
        name for name in servers
        if name.startswith("processkit-") and name != GATEWAY_SERVER_NAME
    )


def _gateway_shape(gateway: dict) -> str:
    args = [str(arg) for arg in gateway.get("args") or []]
    if "stdio-proxy" in args:
        return "stdio-proxy"
    if "serve" in args and "streamable-http" in args:
        return "daemon"
    return "direct-stdio"


def _gateway_config_health(repo_root: Path) -> list[CheckResult]:
    cfg = repo_root / GATEWAY_CONFIG_REL
    cfg_label = GATEWAY_CONFIG_REL.as_posix()
    if not cfg.is_file():
        cfg = repo_root / SRC_GATEWAY_CONFIG_REL
        cfg_label = SRC_GATEWAY_CONFIG_REL.as_posix()
    if not cfg.is_file():
        return [CheckResult(
            severity="INFO",
            category="mcp_gateway",
            id="mcp_gateway.config-absent",
            message="gateway MCP config not present; granular MCP mode only.",
        )]

    data, error = _load_json(cfg)
    if error:
        return [CheckResult(
            severity="ERROR",
            category="mcp_gateway",
            id="mcp_gateway.config-unreadable",
            message=f"could not parse {cfg_label}: {error}",
        )]

    servers = (data or {}).get("mcpServers") or {}
    gateway = servers.get(GATEWAY_SERVER_NAME)
    if not isinstance(gateway, dict):
        return [CheckResult(
            severity="WARN",
            category="mcp_gateway",
            id="mcp_gateway.server-missing",
            message=(
                f"{cfg_label} does not define mcpServers."
                f"{GATEWAY_SERVER_NAME}."
            ),
        )]

    env = gateway.get("env") or {}
    args = gateway.get("args") or []
    if env.get("PROCESSKIT_MCP_MODE") != "gateway":
        return [CheckResult(
            severity="WARN",
            category="mcp_gateway",
            id="mcp_gateway.mode-env-missing",
            message=(
                f"{GATEWAY_SERVER_NAME} config should set "
                "PROCESSKIT_MCP_MODE=gateway."
            ),
        )]
    if not any("processkit-gateway/mcp/server.py" in str(arg) for arg in args):
        return [CheckResult(
            severity="WARN",
            category="mcp_gateway",
            id="mcp_gateway.command-target-unexpected",
            message=(
                f"{GATEWAY_SERVER_NAME} config does not appear to launch "
                "processkit-gateway/mcp/server.py."
            ),
        )]

    shape = _gateway_shape(gateway)
    results = [CheckResult(
        severity="INFO",
        category="mcp_gateway",
        id="mcp_gateway.config-present",
        message=f"gateway MCP config present at {cfg_label} ({shape}).",
    )]
    if shape == "stdio-proxy" and "--url" not in [str(arg) for arg in args]:
        results.append(CheckResult(
            severity="WARN",
            category="mcp_gateway",
            id="mcp_gateway.proxy-url-missing",
            message="gateway stdio-proxy config should include --url.",
        ))
    if shape == "daemon":
        args_text = [str(arg) for arg in args]
        if "--host" in args_text and args_text.index("--host") + 1 < len(args_text):
            host = args_text[args_text.index("--host") + 1]
            if host not in {"127.0.0.1", "localhost", "::1"}:
                results.append(CheckResult(
                    severity="WARN",
                    category="mcp_gateway",
                    id="mcp_gateway.daemon-nonlocal-host",
                    message=(
                        "gateway daemon config should bind locally unless "
                        "a deployment layer adds explicit auth."
                    ),
                ))
    return results


def _harness_health(repo_root: Path) -> list[CheckResult]:
    mcp_json = repo_root / ".mcp.json"
    if not mcp_json.is_file():
        return []

    data, error = _load_json(mcp_json)
    if error:
        return [CheckResult(
            severity="ERROR",
            category="mcp_gateway",
            id="mcp_gateway.mcp-json-unreadable",
            message=f"could not parse .mcp.json: {error}",
        )]

    servers = (data or {}).get("mcpServers") or {}
    if GATEWAY_SERVER_NAME not in servers:
        return [CheckResult(
            severity="INFO",
            category="mcp_gateway",
            id="mcp_gateway.harness-granular-mode",
            message=".mcp.json does not register processkit-gateway.",
        )]

    granular = _processkit_server_names(servers)
    if granular:
        preview = ", ".join(granular[:5])
        if len(granular) > 5:
            preview += f" (+{len(granular) - 5} more)"
        return [CheckResult(
            severity="WARN",
            category="mcp_gateway",
            id="mcp_gateway.mixed-registration",
            message=(
                ".mcp.json registers processkit-gateway and granular "
                f"processkit servers ({preview}); choose one default mode."
            ),
        )]

    shape = _gateway_shape(servers[GATEWAY_SERVER_NAME])
    finding_id = "mcp_gateway.harness-gateway-mode"
    message = ".mcp.json registers processkit-gateway only."
    if shape == "stdio-proxy":
        finding_id = "mcp_gateway.harness-proxy-mode"
        message = ".mcp.json registers processkit-gateway stdio proxy only."

    return [CheckResult(
        severity="INFO",
        category="mcp_gateway",
        id=finding_id,
        message=message,
    )]


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    return [*_gateway_config_health(repo_root), *_harness_health(repo_root)]
