#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
# ]
# ///
"""processkit supply-chain-audit MCP server."""

from __future__ import annotations

import importlib.util
import inspect
import json
import os
import sys
from pathlib import Path
from typing import Any

_CORE_LIB_NAME = "supply_chain_audit_core"
_CORE_LIBRARY_CANDIDATES: tuple[tuple[str, ...], ...] = (
    ("scripts", "supply_chain_audit.py"),
    (
        "src",
        "context",
        "skills",
        "processkit",
        "supply-chain-audit",
        "scripts",
        "supply_chain_audit.py",
    ),
    (
        "context",
        "skills",
        "processkit",
        "supply-chain-audit",
        "scripts",
        "supply_chain_audit.py",
    ),
)
_CORE_FN_ALTERNATES = {
    "run_supply_chain_audit": ("run_supply_chain_audit", "run_audit"),
    "export_supply_chain_sbom": (
        "export_supply_chain_sbom",
        "export_sbom",
        "build_cyclonedx_sbom",
        "build_spdx_sbom",
    ),
}
_CORE_FN_ALIASES: dict[str, dict[str, tuple[str, ...]]] = {
    "discover_manifests": {
        "project_root": ("repo_root", "root", "root_dir", "path"),
        "manifest_globs": ("globs", "glob_patterns", "patterns"),
        "include_vendor_dir": (
            "scan_vendor",
            "include_vendor",
            "vendor_scope",
        ),
        "include_ci_manifests": (
            "scan_ci",
            "include_ci",
            "ci_manifests",
        ),
    },
    "run_supply_chain_audit": {
        "project_root": ("repo_root", "root", "root_dir", "path"),
        "manifest_paths": (
            "paths",
            "manifests",
            "manifest_files",
            "inputs",
        ),
        "run_security_checks": (
            "include_security",
            "security_checks",
            "run_security",
            "do_security",
        ),
        "run_quality_checks": (
            "include_quality",
            "quality_checks",
            "run_quality",
            "do_quality",
        ),
        "network_enabled": (
            "enable_network",
            "run_network",
            "include_network",
        ),
        "write_output": ("write", "write_results", "persist"),
        "output_path": ("output_file", "report_path", "result_path"),
    },
    "run_audit": {
        "project_root": ("repo_root", "root", "root_dir", "path"),
        "run_security_checks": (
            "include_security",
            "security_checks",
            "run_security",
            "do_security",
        ),
        "run_quality_checks": (
            "include_quality",
            "quality_checks",
            "run_quality",
            "do_quality",
        ),
    },
    "export_supply_chain_sbom": {
        "project_root": ("repo_root", "root", "root_dir", "path"),
        "manifest_paths": (
            "paths",
            "manifests",
            "manifest_files",
            "inputs",
        ),
        "format": ("sbom_format", "output_format", "format_name"),
        "write_output": ("write", "write_file", "persist"),
        "output_path": ("output_file", "output", "path"),
    },
    "export_sbom": {
        "summary": ("audit", "audit_payload"),
    },
    "build_cyclonedx_sbom": {"format": ("sbom_format", "output_format")},
    "build_spdx_sbom": {"format": ("sbom_format", "output_format")},
}

_core_module = None


def _find_lib() -> Path:
    env = os.environ.get("PROCESSKIT_LIB_PATH")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve().parent
    while True:
        for c in (here / "src" / "lib", here / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


sys.path.insert(0, str(_find_lib()))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402

from processkit import paths  # noqa: E402

server = FastMCP("processkit-supply-chain-audit")


def _find_repo_root() -> Path:
    """Find the repository root for ``scripts/supply_chain_audit.py``."""
    start = Path(__file__).resolve().parent
    for candidate in (start, *start.parents):
        if (candidate / "scripts" / "supply_chain_audit.py").is_file():
            return candidate
        if (candidate / "AGENTS.md").is_file():
            return candidate
        if (candidate / "aibox.toml").is_file():
            return candidate
        if (
            candidate.name == "context"
            and (candidate.parent / "scripts").is_dir()
        ):
            return candidate.parent
        if candidate.name == "src" and (candidate / "context").is_dir():
            return candidate.parent
    return Path.cwd().resolve()


def _coerce_root(project_root: str | None) -> Path:
    if project_root:
        return Path(project_root).expanduser().resolve()
    return paths.find_project_root()


def _normalize_output_path(root: Path, output_path: str | None) -> str | None:
    if not output_path:
        return None
    path = Path(output_path)
    if not path.is_absolute():
        path = root / path
    resolved = path.resolve()
    root_resolved = root.resolve()
    if root_resolved not in resolved.parents and resolved != root_resolved:
        raise ValueError("output_path must stay within project root")
    return str(resolved)


def _write_json_report(payload: dict, output_path: str | None) -> None:
    if not output_path:
        return
    path = Path(output_path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as stream:
        stream.write(json.dumps(payload, indent=2, sort_keys=False))
        stream.write("\n")


def _candidate_lib_paths() -> list[Path]:
    repo_root = _find_repo_root()
    return [repo_root.joinpath(*parts) for parts in _CORE_LIBRARY_CANDIDATES]


def _find_core_library_path() -> Path:
    for path in _candidate_lib_paths():
        if path.is_file():
            return path
    candidates = "\n- ".join(str(path) for path in _candidate_lib_paths())
    raise FileNotFoundError(
        "core library not found. Expected one of:\n- "
        f"{candidates}"
    )


def _load_core_module() -> Any:
    global _core_module
    if _core_module is not None:
        return _core_module

    module_path = _find_core_library_path()
    module_dir = module_path.parent
    if str(module_dir) not in sys.path:
        sys.path.insert(0, str(module_dir))

    spec = importlib.util.spec_from_file_location(_CORE_LIB_NAME, module_path)
    if spec is None or spec.loader is None:
        raise RuntimeError("unable to create import spec for core library")
    module = importlib.util.module_from_spec(spec)
    sys.modules[_CORE_LIB_NAME] = module
    spec.loader.exec_module(module)
    _core_module = module
    return module


def _map_kwargs(
    fn_name: str, fn: Any, kwargs: dict[str, Any]
) -> dict[str, Any]:
    sig = inspect.signature(fn)
    params = sig.parameters
    accepts_var_kwargs = any(
        p.kind == inspect.Parameter.VAR_KEYWORD for p in params.values()
    )
    alias_map = _CORE_FN_ALIASES.get(fn_name, {})
    out: dict[str, Any] = {}

    for key, value in kwargs.items():
        if value is None:
            continue
        if key in params:
            out[key] = value
            continue
        if accepts_var_kwargs:
            out[key] = value
            continue
        replacements = alias_map.get(key, ())
        for alias in replacements:
            if alias in params:
                out[alias] = value
                break

    return out


def _call_mapped_callable(
    fn_name: str,
    fn: Any,
    kwargs: dict[str, Any],
) -> Any:
    """Call a core function using mapped keywords."""
    call_kwargs = _map_kwargs(fn_name, fn, kwargs)
    try:
        return fn(**call_kwargs)
    except TypeError as exc:
        signature = inspect.signature(fn)
        positional: list[Any] = []
        for param_name, param in signature.parameters.items():
            if param.kind == inspect.Parameter.VAR_POSITIONAL:
                break
            if param_name in call_kwargs:
                positional.append(call_kwargs[param_name])
            elif (
                param_name in kwargs
                and param.default is inspect.Parameter.empty
            ):
                raise TypeError(
                    f"{fn_name!r} requires {param_name!r}; "
                    "provide it explicitly"
                ) from exc
        if not positional:
            raise
        return fn(*positional)


def _normalize_core_payload(payload: Any) -> dict:
    return payload if isinstance(payload, dict) else {"result": payload}


def _run_audit_via_core(module: Any, kwargs: dict[str, Any]) -> dict:
    """Run audit through the current API or legacy names."""
    for candidate in _CORE_FN_ALTERNATES["run_supply_chain_audit"]:
        fn = getattr(module, candidate, None)
        if not callable(fn):
            continue
        if candidate == "run_audit":
            result = _call_mapped_callable(
                candidate,
                fn,
                {
                    "project_root": kwargs.get("project_root"),
                    "run_security_checks": bool(
                        kwargs.get("network_enabled")
                        or kwargs.get("run_security_checks")
                    ),
                    "run_quality_checks": bool(
                        kwargs.get("network_enabled")
                        or kwargs.get("run_quality_checks")
                    ),
                },
            )
            payload = _normalize_core_payload(result)
            payload.setdefault("compatibility_mode", candidate)
            payload.setdefault(
                "network_checks_skipped_by_default",
                not bool(
                    kwargs.get("network_enabled")
                    or kwargs.get("run_security_checks")
                    or kwargs.get("run_quality_checks")
                ),
            )
            return payload

        return _normalize_core_payload(
            _call_mapped_callable(candidate, fn, kwargs),
        )

    return {"error": "No runnable audit function was found in core library"}


def _finalize_output_payload(
    sbom: Any,
    kwargs: dict[str, Any],
    base: dict[str, Any] | None = None,
) -> dict:
    output_path = kwargs.get("output_path")
    write_output = bool(kwargs.get("write_output"))
    payload = dict(base or {})
    payload.setdefault("sbom", sbom)
    payload.setdefault("format", kwargs.get("format", "json"))
    payload.setdefault("written", write_output and bool(output_path))
    payload.setdefault("output_path", output_path)
    if write_output and output_path:
        _write_json_report(payload, output_path)
    return payload


def _export_sbom_via_core(module: Any, kwargs: dict[str, Any]) -> dict:
    """Export SBOM via current API, legacy API, or legacy builders."""
    for candidate in _CORE_FN_ALTERNATES["export_supply_chain_sbom"]:
        fn = getattr(module, candidate, None)
        if not callable(fn):
            continue

        if candidate == "build_cyclonedx_sbom":
            audit_payload = _run_audit_via_core(module, kwargs)
            if "error" in audit_payload:
                return audit_payload
            sbom = _call_mapped_callable(
                candidate,
                fn,
                {"audit": audit_payload},
            )
            return _finalize_output_payload(
                sbom,
                kwargs,
                {"compatibility_mode": candidate},
            )

        if candidate == "build_spdx_sbom":
            audit_payload = _run_audit_via_core(module, kwargs)
            if "error" in audit_payload:
                return audit_payload
            sbom = _call_mapped_callable(
                candidate,
                fn,
                {"audit": audit_payload},
            )
            return _finalize_output_payload(
                sbom,
                kwargs,
                {"compatibility_mode": candidate},
            )

        if candidate == "export_sbom":
            audit_payload = _run_audit_via_core(module, kwargs)
            if "error" in audit_payload:
                return audit_payload
            sbom = _call_mapped_callable(
                candidate,
                fn,
                {"summary": audit_payload},
            )
            return _finalize_output_payload(
                _normalize_core_payload(sbom).get("sbom", sbom),
                kwargs,
                {"compatibility_mode": candidate},
            )

        raw_sbom = _call_mapped_callable(candidate, fn, kwargs)
        normalized = _normalize_core_payload(raw_sbom)
        return _finalize_output_payload(
            normalized.get("sbom", raw_sbom),
            kwargs,
        )

    return {"error": "No runnable SBOM function was found in core library"}


def _invoke_core(fn_name: str, **kwargs: Any) -> dict:
    try:
        module = _load_core_module()
        if fn_name == "run_supply_chain_audit":
            return _run_audit_via_core(module, kwargs)
        if fn_name == "export_supply_chain_sbom":
            return _export_sbom_via_core(module, kwargs)

        fn = getattr(module, fn_name, None)
        if fn is None or not callable(fn):
            return {"error": f"core function {fn_name!r} not found"}
        return _normalize_core_payload(
            _call_mapped_callable(fn_name, fn, kwargs),
        )
    except Exception as exc:
        return {"error": f"{fn_name!r} setup failed: {exc}"}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def discover_manifests(
    project_root: str | None = None,
    manifest_globs: list[str] | None = None,
    include_vendor_dir: bool = True,
    include_ci_manifests: bool = False,
) -> dict:
    """Discover supply-chain manifests in the project."""
    root = _coerce_root(project_root)
    return _invoke_core(
        "discover_manifests",
        project_root=str(root),
        manifest_globs=manifest_globs,
        include_vendor_dir=include_vendor_dir,
        include_ci_manifests=include_ci_manifests,
    )


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def run_supply_chain_audit(
    project_root: str | None = None,
    manifest_paths: list[str] | None = None,
    run_security_checks: bool = False,
    run_quality_checks: bool = False,
    network_enabled: bool = False,
    write_output: bool = False,
    output_path: str | None = None,
) -> dict:
    """Run supply-chain audit.

    Network-dependent probes are opt-in and off by default.
    """
    root = _coerce_root(project_root)
    output_path = _normalize_output_path(root, output_path)
    payload = _invoke_core(
        "run_supply_chain_audit",
        project_root=str(root),
        manifest_paths=manifest_paths,
        run_security_checks=run_security_checks,
        run_quality_checks=run_quality_checks,
        network_enabled=network_enabled
        or run_security_checks
        or run_quality_checks,
        write_output=write_output,
        output_path=output_path,
    )
    if write_output and output_path and "error" not in payload:
        _write_json_report(payload, output_path)
        payload.setdefault("written", True)
        payload.setdefault("output_path", output_path)
    payload.setdefault("network_checks_enabled", {
        "security": bool(run_security_checks),
        "quality": bool(run_quality_checks),
        "network_enabled_flag": bool(network_enabled),
    })
    payload.setdefault(
        "network_checks_skipped_by_default",
        not bool(
            network_enabled or run_security_checks or run_quality_checks
        ),
    )
    return payload


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def export_supply_chain_sbom(
    project_root: str | None = None,
    manifest_paths: list[str] | None = None,
    format: str = "json",
    write_output: bool = False,
    output_path: str | None = None,
) -> dict:
    """Export a SBOM built from discoverable supply-chain manifests."""
    root = _coerce_root(project_root)
    output_path = _normalize_output_path(root, output_path)
    payload = _invoke_core(
        "export_supply_chain_sbom",
        project_root=str(root),
        manifest_paths=manifest_paths,
        format=format,
        write_output=write_output,
        output_path=output_path,
    )
    payload.setdefault("requested_format", format)
    payload.setdefault("write_output", write_output)
    return payload


if __name__ == "__main__":
    server.run(transport="stdio")
