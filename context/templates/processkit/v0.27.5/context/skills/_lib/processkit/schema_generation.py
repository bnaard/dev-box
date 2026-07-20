"""Deterministic composition and rendering for processkit schemas."""

from __future__ import annotations

import os
import tempfile
from copy import deepcopy
from pathlib import Path
from typing import Any

import yaml
from jinja2 import FileSystemLoader, StrictUndefined, TemplateError
from jinja2.sandbox import SandboxedEnvironment


class SchemaGenerationError(ValueError):
    """Raised when schema source composition is invalid."""


def _environment(source_root: Path) -> SandboxedEnvironment:
    environment = SandboxedEnvironment(
        loader=FileSystemLoader(source_root),
        undefined=StrictUndefined,
        autoescape=False,
        keep_trailing_newline=True,
    )
    environment.filters["to_yaml"] = _yaml_dump
    return environment


def _load_yaml(path: Path, source_root: Path) -> dict[str, Any]:
    try:
        relative = _within(path, source_root).relative_to(source_root)
        rendered = _environment(source_root).get_template(
            relative.as_posix()
        ).render()
        value = yaml.safe_load(rendered)
    except (OSError, TemplateError, yaml.YAMLError, ValueError) as exc:
        raise SchemaGenerationError(f"cannot load {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise SchemaGenerationError(f"schema source must be a mapping: {path}")
    return value


def _merge(parent: Any, child: Any, location: str = "root") -> Any:
    if isinstance(child, dict) and "__merge" in child:
        unexpected = set(child) - {"__merge", "value"}
        if unexpected or "value" not in child:
            raise SchemaGenerationError(
                f"invalid merge wrapper at {location}: {sorted(unexpected)}"
            )
        strategy = child["__merge"]
        value = child["value"]
        if strategy == "replace":
            return deepcopy(value)
        if strategy == "concat":
            if not isinstance(parent, list) or not isinstance(value, list):
                raise SchemaGenerationError(
                    f"concat requires lists at {location}"
                )
            return deepcopy(parent) + deepcopy(value)
        if strategy == "name-merge":
            if not isinstance(parent, dict) or not isinstance(value, dict):
                raise SchemaGenerationError(
                    f"name-merge requires mappings at {location}"
                )
            return _merge(parent, value, location)
        raise SchemaGenerationError(
            f"unknown merge strategy at {location}: {strategy}"
        )

    if isinstance(parent, dict) and isinstance(child, dict):
        result = deepcopy(parent)
        for key, value in child.items():
            if key == "extends":
                continue
            nested = f"{location}.{key}"
            result[key] = (
                _merge(result[key], value, nested)
                if key in result
                else deepcopy(value)
            )
        return result
    return deepcopy(child)


def merge_documents(
    parent: dict[str, Any], child: dict[str, Any]
) -> dict[str, Any]:
    """Merge one schema source over another using explicit wrappers."""

    return _merge(parent, child)


def _within(path: Path, root: Path) -> Path:
    resolved = path.resolve()
    try:
        resolved.relative_to(root.resolve())
    except ValueError as exc:
        raise SchemaGenerationError(
            f"path escapes allowed root: {path}"
        ) from exc
    return resolved


def _compose(
    path: Path, source_root: Path, stack: tuple[Path, ...]
) -> dict[str, Any]:
    path = _within(path, source_root)
    if path in stack:
        cycle = " -> ".join(item.name for item in (*stack, path))
        raise SchemaGenerationError(f"schema inheritance cycle: {cycle}")
    document = _load_yaml(path, source_root)
    parents = document.get("extends", [])
    if isinstance(parents, str):
        parents = [parents]
    if not isinstance(parents, list) or not all(
        isinstance(item, str) for item in parents
    ):
        raise SchemaGenerationError(f"extends must be a string or list: {path}")

    result: dict[str, Any] = {}
    for parent in parents:
        parent_path = _within(path.parent / parent, source_root)
        result = _merge(
            result,
            _compose(parent_path, source_root, (*stack, path)),
        )
    return _merge(result, document)


def _yaml_dump(value: Any) -> str:
    return yaml.safe_dump(
        value,
        allow_unicode=False,
        default_flow_style=False,
        sort_keys=False,
        width=79,
    ).rstrip()


def _render(document: dict[str, Any], source_root: Path) -> str:
    ordered: dict[str, Any] = {}
    for key in ("apiVersion", "kind", "metadata", "spec"):
        if key in document:
            ordered[key] = document[key]
    ordered.update(
        (key, value) for key, value in document.items() if key not in ordered
    )
    environment = _environment(source_root)
    template = environment.get_template("templates/schema.yaml.j2")
    return template.render(document=ordered).rstrip() + "\n"


def _atomic_write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(
        prefix=f".{path.name}.", dir=path.parent
    )
    try:
        with os.fdopen(
            descriptor, "w", encoding="utf-8", newline="\n"
        ) as stream:
            stream.write(content)
        os.replace(temporary, path)
    except BaseException:
        Path(temporary).unlink(missing_ok=True)
        raise


def regenerate_schemas(
    schemas_root: Path,
    kinds: list[str] | None = None,
    output_dir: Path | None = None,
    check: bool = False,
) -> dict[str, list[str] | dict[str, str]]:
    """Render all or selected registry kinds and report changed files."""

    schemas_root = schemas_root.resolve()
    source_root = schemas_root / "src"
    registry = _load_yaml(source_root / "registry.yaml", source_root)
    entries = registry.get("kinds")
    interfaces = registry.get("interfaces")
    if not isinstance(entries, dict) or not isinstance(interfaces, list):
        raise SchemaGenerationError("registry requires kinds and interfaces")

    selected = list(entries) if kinds is None else kinds
    unknown = sorted(set(selected) - set(entries))
    if unknown:
        return {
            "rebuilt": [],
            "unchanged": [],
            "errors": {kind: "unknown kind" for kind in unknown},
        }

    output_dir = (output_dir or schemas_root / "_generated").resolve()
    planned: list[tuple[str, Path, str]] = []
    errors: dict[str, str] = {}
    destinations: dict[Path, str] = {}
    for kind in selected:
        entry = entries[kind]
        try:
            if not isinstance(entry, dict) or not isinstance(
                entry.get("source"), str
            ):
                raise SchemaGenerationError("registry entry requires source")
            source = _within(source_root / entry["source"], source_root)
            document = _compose(source, source_root, ())
            metadata = document.get("metadata", {})
            target_kind = str(metadata.get("target_kind", "")).lower()
            if document.get("kind") != "Schema" or target_kind != kind:
                raise SchemaGenerationError(
                    "source kind/metadata.target_kind does not match registry"
                )
            if metadata.get("id") != f"SCHEMA-{kind}":
                raise SchemaGenerationError(
                    f"schema id must be SCHEMA-{kind}"
                )
            declared = document.get("spec", {}).get("interfaces", [])
            if not isinstance(declared, list) or not all(
                isinstance(interface, str) for interface in declared
            ):
                raise SchemaGenerationError("interfaces must be a string list")
            invalid = sorted(set(declared) - set(interfaces))
            if invalid:
                raise SchemaGenerationError(
                    f"unknown interfaces: {', '.join(invalid)}"
                )
            content = _render(document, source_root)
            output = entry.get("output", f"{kind}.yaml")
            if (
                not isinstance(output, str)
                or Path(output).is_absolute()
                or len(Path(output).parts) != 1
                or not output.endswith(".yaml")
            ):
                raise SchemaGenerationError(
                    "output must be a flat YAML filename"
                )
            destination = _within(output_dir / output, output_dir)
            if destination in destinations:
                raise SchemaGenerationError(
                    f"output collides with kind {destinations[destination]}"
                )
            destinations[destination] = kind
            planned.append((kind, destination, content))
        except (OSError, SchemaGenerationError) as exc:
            errors[kind] = str(exc)
    if errors:
        return {"rebuilt": [], "unchanged": [], "errors": errors}

    rebuilt: list[str] = []
    unchanged: list[str] = []
    writes: list[tuple[Path, str]] = []
    for kind, destination, content in planned:
        current = (
            destination.read_text(encoding="utf-8")
            if destination.exists()
            else None
        )
        if current == content:
            unchanged.append(kind)
        else:
            rebuilt.append(kind)
            writes.append((destination, content))
    if not check:
        for destination, content in writes:
            _atomic_write(destination, content)
    return {"rebuilt": rebuilt, "unchanged": unchanged, "errors": errors}
