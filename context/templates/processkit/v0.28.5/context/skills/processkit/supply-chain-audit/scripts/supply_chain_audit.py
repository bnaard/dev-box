#!/usr/bin/env python3
"""Offline-first supply-chain audit helpers.

The module intentionally has no third-party runtime dependency. Phase 1 is
deterministic and lockfile-driven; Phase 2/3 surfaces are represented as
structured skipped/advisory findings unless a future caller supplies scanner
results.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


DEFAULT_POLICY = {
    "licenses": {
        "allow": [
            "0BSD",
            "Apache-2.0",
            "BSD-2-Clause",
            "BSD-3-Clause",
            "ISC",
            "MIT",
            "Unicode-3.0",
        ],
        "review": [
            "CDDL-1.0",
            "EPL-1.0",
            "EPL-2.0",
            "LGPL-2.1-only",
            "LGPL-2.1-or-later",
            "LGPL-3.0-only",
            "LGPL-3.0-or-later",
            "MPL-2.0",
        ],
        "deny": [
            "AGPL-3.0-only",
            "AGPL-3.0-or-later",
            "GPL-2.0-only",
            "GPL-2.0-or-later",
            "GPL-3.0-only",
            "GPL-3.0-or-later",
        ],
    },
    "security": {
        "block_severity": "high",
        "warn_severity": "medium",
    },
}

KNOWN_LICENSES = set(
    DEFAULT_POLICY["licenses"]["allow"]
    + DEFAULT_POLICY["licenses"]["review"]
    + DEFAULT_POLICY["licenses"]["deny"]
    + ["CC0-1.0", "Unlicense"]
)


@dataclass
class Manifest:
    ecosystem: str
    manifest_path: str
    lockfile_path: str | None
    package_name: str | None = None
    package_version: str | None = None
    application: bool = True


@dataclass
class Component:
    ecosystem: str
    name: str
    version: str
    purl: str
    license: str | None = None
    scope: str = "runtime"
    direct: bool = False
    manifest_path: str | None = None
    lockfile_path: str | None = None
    license_bucket: str = "unknown"


@dataclass
class Finding:
    severity: str
    id: str
    message: str
    component: str | None = None
    path: str | None = None
    extra: dict[str, Any] = field(default_factory=dict)


def run_audit(
    repo_root: Path | str,
    *,
    include_security: bool = False,
    include_quality: bool = False,
) -> dict[str, Any]:
    root = Path(repo_root).resolve()
    policy = load_policy(root)
    manifests = discover_manifests(root)
    components = inventory_components(root, manifests)
    findings = _license_findings(components, policy)
    findings += _manifest_findings(manifests)
    security, skipped_scanners, security_findings = _security_phase(
        root,
        manifests,
        include_security,
    )
    outdated = _outdated_phase(root, manifests, include_security)
    supplier_quality, quality_findings = _quality_phase(
        components,
        include_quality,
    )
    findings += security_findings
    findings += quality_findings

    by_bucket = _license_bucket_counts(components, policy)
    return {
        "audit_version": "1.0.0",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "repo_root": str(root),
        "policy": {
            "path": str(_policy_path(root)) if _policy_path(root).is_file() else None,
            "licenses": policy["licenses"],
        },
        "manifests": [asdict(x) for x in manifests],
        "components": [asdict(x) for x in components],
        "summary": {
            "manifest_count": len(manifests),
            "component_count": len(components),
            "direct_component_count": sum(1 for c in components if c.direct),
            "transitive_component_count": sum(1 for c in components if not c.direct),
            "license_buckets": by_bucket,
            "findings": _finding_counts(findings),
        },
        "security": security,
        "skipped_scanners": skipped_scanners,
        "outdated": outdated,
        "supplier_quality": supplier_quality,
        "phases": {
            "phase1": {
                "status": "complete",
                "objective": "offline license inventory and usage risk",
            },
            "phase2": {
                "status": "enabled" if include_security else "skipped",
                "objective": "vulnerability and freshness review",
                "scanner_findings": security,
                "outdated": outdated,
            },
            "phase3": {
                "status": "enabled" if include_quality else "skipped",
                "objective": "supplier stability and quality advisory",
                "quality_findings": supplier_quality,
            },
        },
        "findings": [asdict(x) for x in findings],
    }


def discover_manifests(repo_root: Path | str) -> list[Manifest]:
    root = Path(repo_root).resolve()
    manifests: list[Manifest] = []
    for package_json in sorted(root.rglob("package.json")):
        if _skip_path(root, package_json):
            continue
        rel_manifest = _rel(root, package_json)
        lock = package_json.with_name("package-lock.json")
        data = _read_json(package_json)
        manifests.append(Manifest(
            ecosystem="npm",
            manifest_path=rel_manifest,
            lockfile_path=_rel(root, lock) if lock.is_file() else None,
            package_name=data.get("name") if isinstance(data, dict) else None,
            package_version=data.get("version") if isinstance(data, dict) else None,
            application=bool(data.get("private", True)) if isinstance(data, dict) else True,
        ))
    return manifests


def inventory_components(
    repo_root: Path | str,
    manifests: list[Manifest] | None = None,
) -> list[Component]:
    root = Path(repo_root).resolve()
    manifests = manifests if manifests is not None else discover_manifests(root)
    components: list[Component] = []
    seen: set[tuple[str, str, str]] = set()
    for manifest in manifests:
        if manifest.ecosystem != "npm" or not manifest.lockfile_path:
            continue
        lockfile = root / manifest.lockfile_path
        data = _read_json(lockfile)
        direct_names = _npm_direct_dependency_names(root / manifest.manifest_path)
        for name, entry in _npm_packages(data).items():
            version = str(entry.get("version") or "")
            if not name or not version:
                continue
            key = ("npm", name, version)
            if key in seen:
                continue
            seen.add(key)
            license_expr = entry.get("license")
            if isinstance(license_expr, dict):
                license_expr = license_expr.get("type")
            component = Component(
                ecosystem="npm",
                name=name,
                version=version,
                purl=_purl("npm", name, version),
                license=str(license_expr) if license_expr else None,
                scope="dev" if entry.get("dev") else "runtime",
                direct=name in direct_names,
                manifest_path=manifest.manifest_path,
                lockfile_path=manifest.lockfile_path,
            )
            component.license_bucket = classify_license(component, load_policy(root))
            components.append(component)
    return sorted(components, key=lambda c: (c.ecosystem, c.name, c.version))


def load_policy(repo_root: Path | str) -> dict[str, Any]:
    root = Path(repo_root).resolve()
    policy = json.loads(json.dumps(DEFAULT_POLICY))
    path = _policy_path(root)
    if not path.is_file():
        return policy
    parsed = _read_simple_yaml(path)
    licenses = parsed.get("licenses", {}) if isinstance(parsed, dict) else {}
    for key in ("allow", "review", "deny"):
        value = licenses.get(key)
        if isinstance(value, list):
            policy["licenses"][key] = [str(x) for x in value]
    security = parsed.get("security", {}) if isinstance(parsed, dict) else {}
    if isinstance(security, dict):
        policy["security"].update({k: str(v) for k, v in security.items()})
    return policy


def classify_license(component: Component, policy: dict[str, Any]) -> str:
    expression = (component.license or "").strip()
    if not expression:
        return "unknown"
    tokens = _license_tokens(expression)
    if not tokens:
        return "unknown"
    if any(token in set(policy["licenses"].get("deny", [])) for token in tokens):
        return "deny"
    if any(token in set(policy["licenses"].get("review", [])) for token in tokens):
        return "review"
    allowed = set(policy["licenses"].get("allow", []))
    if all(token in allowed for token in tokens):
        return "allow"
    if any(token not in KNOWN_LICENSES for token in tokens):
        return "unknown"
    return "review"


def export_sbom(audit: dict[str, Any]) -> dict[str, Any]:
    components = audit.get("components", [])
    return {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "version": 1,
        "metadata": {
            "timestamp": audit.get("generated_at"),
            "tools": [{
                "vendor": "processkit",
                "name": "supply-chain-audit",
                "version": audit.get("audit_version", "1.0.0"),
            }],
        },
        "components": [
            {
                "type": "library",
                "name": c["name"],
                "version": c["version"],
                "purl": c["purl"],
                "licenses": (
                    [{"license": {"id": c["license"]}}]
                    if c.get("license") else []
                ),
                "scope": c.get("scope", "required"),
            }
            for c in components
        ],
    }


def _license_findings(
    components: list[Component],
    policy: dict[str, Any],
) -> list[Finding]:
    findings: list[Finding] = []
    for component in components:
        bucket = classify_license(component, policy)
        if bucket == "deny":
            findings.append(Finding(
                severity="ERROR",
                id="supply-chain.license-denied",
                component=component.purl,
                path=component.lockfile_path,
                message=(
                    f"{component.name}@{component.version} uses denied "
                    f"license {component.license or 'UNKNOWN'}"
                ),
            ))
        elif bucket == "review":
            findings.append(Finding(
                severity="WARN",
                id="supply-chain.license-review",
                component=component.purl,
                path=component.lockfile_path,
                message=(
                    f"{component.name}@{component.version} uses review-needed "
                    f"license {component.license or 'UNKNOWN'}"
                ),
            ))
        elif bucket == "unknown":
            findings.append(Finding(
                severity="WARN",
                id="supply-chain.license-unknown",
                component=component.purl,
                path=component.lockfile_path,
                message=f"{component.name}@{component.version} has unknown license metadata",
            ))
    return findings


def _manifest_findings(manifests: list[Manifest]) -> list[Finding]:
    findings: list[Finding] = []
    for manifest in manifests:
        if manifest.application and not manifest.lockfile_path:
            findings.append(Finding(
                severity="ERROR",
                id="supply-chain.lockfile-missing",
                path=manifest.manifest_path,
                message=f"{manifest.manifest_path} has no committed lockfile",
            ))
    return findings


def _security_phase(
    repo_root: Path,
    manifests: list[Manifest],
    enabled: bool,
) -> tuple[list[dict[str, Any]], list[dict[str, str]], list[Finding]]:
    if not enabled:
        return [], [], [Finding(
            severity="INFO",
            id="supply-chain.security-skipped",
            message="Vulnerability/outdated probes skipped; offline license audit only.",
        )]

    scanner_results: list[dict[str, Any]] = []
    skipped: list[dict[str, str]] = []
    findings: list[Finding] = []
    npm = shutil.which("npm")
    npm_manifests = [m for m in manifests if m.ecosystem == "npm" and m.lockfile_path]
    if npm is None:
        skipped.append({"name": "npm audit", "reason": "not-installed"})
    for manifest in npm_manifests:
        if npm is None:
            continue
        cwd = repo_root / Path(manifest.manifest_path).parent
        result = subprocess.run(
            [npm, "audit", "--json"],
            cwd=str(cwd),
            capture_output=True,
            text=True,
            timeout=60,
        )
        if not result.stdout.strip():
            skipped.append({
                "name": "npm audit",
                "reason": f"no-json-output:{result.returncode}",
            })
            continue
        try:
            payload = json.loads(result.stdout)
        except json.JSONDecodeError:
            skipped.append({"name": "npm audit", "reason": "invalid-json-output"})
            continue
        scanner_results.extend(_npm_audit_findings(payload, manifest))

    for item in scanner_results:
        severity = str(item.get("severity", "")).lower()
        findings.append(Finding(
            severity="ERROR" if severity in {"critical", "high"} else "WARN",
            id="supply-chain.vulnerability",
            message=(
                f"{item.get('package', '<unknown>')} has "
                f"{severity or 'unknown'} vulnerability {item.get('id', '<unknown>')}"
            ),
            component=item.get("package"),
            path=item.get("path"),
            extra=item,
        ))
    if not scanner_results and not skipped:
        findings.append(Finding(
            severity="INFO",
            id="supply-chain.security-clean",
            message="Requested vulnerability scanners reported no findings.",
        ))
    return scanner_results, skipped, findings


def _outdated_phase(
    repo_root: Path,
    manifests: list[Manifest],
    enabled: bool,
) -> list[dict[str, Any]]:
    if not enabled:
        return []
    npm = shutil.which("npm")
    if npm is None:
        return []
    results: list[dict[str, Any]] = []
    for manifest in manifests:
        if manifest.ecosystem != "npm" or not manifest.lockfile_path:
            continue
        cwd = repo_root / Path(manifest.manifest_path).parent
        completed = subprocess.run(
            [npm, "outdated", "--json"],
            cwd=str(cwd),
            capture_output=True,
            text=True,
            timeout=60,
        )
        if not completed.stdout.strip():
            continue
        try:
            payload = json.loads(completed.stdout)
        except json.JSONDecodeError:
            continue
        for name, item in payload.items():
            if isinstance(item, dict):
                results.append({
                    "name": name,
                    "current": item.get("current"),
                    "wanted": item.get("wanted"),
                    "latest": item.get("latest"),
                    "path": manifest.manifest_path,
                })
    return results


def _quality_phase(
    components: list[Component],
    enabled: bool,
) -> tuple[list[dict[str, Any]], list[Finding]]:
    if not enabled:
        return [], [Finding(
            severity="INFO",
            id="supply-chain.quality-skipped",
            message="Supplier quality probes skipped; they are advisory and opt-in.",
        )]
    quality: list[dict[str, Any]] = []
    for component in components:
        signals: list[str] = []
        if component.license_bucket == "unknown":
            signals.append("unknown-license-metadata")
        if component.scope == "runtime" and not component.direct:
            signals.append("runtime-transitive-dependency")
        if signals:
            quality.append({
                "supplier": component.purl,
                "risk": "medium" if "unknown-license-metadata" in signals else "low",
                "signals": signals,
            })
    findings = [Finding(
        severity="INFO",
        id="supply-chain.quality-advisory",
        message=f"Supplier quality advisory generated {len(quality)} signal(s).",
        extra={"supplier_quality": quality},
    )]
    return quality, findings


def _npm_audit_findings(
    payload: dict[str, Any],
    manifest: Manifest,
) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    vulnerabilities = payload.get("vulnerabilities")
    if isinstance(vulnerabilities, dict):
        for name, item in vulnerabilities.items():
            if not isinstance(item, dict):
                continue
            via = item.get("via")
            advisory_id = None
            if isinstance(via, list):
                for via_item in via:
                    if isinstance(via_item, dict):
                        advisory_id = via_item.get("source") or via_item.get("url")
                        break
            out.append({
                "id": str(advisory_id or name),
                "package": name,
                "severity": item.get("severity"),
                "path": manifest.lockfile_path,
                "range": item.get("range"),
                "fix_available": item.get("fixAvailable"),
            })
    advisories = payload.get("advisories")
    if isinstance(advisories, dict):
        for advisory_id, item in advisories.items():
            if not isinstance(item, dict):
                continue
            out.append({
                "id": str(advisory_id),
                "package": item.get("module_name"),
                "severity": item.get("severity"),
                "path": manifest.lockfile_path,
                "title": item.get("title"),
            })
    return out


def _license_bucket_counts(
    components: list[Component],
    policy: dict[str, Any],
) -> dict[str, int]:
    counts = {"allow": 0, "review": 0, "deny": 0, "unknown": 0}
    for component in components:
        counts[classify_license(component, policy)] += 1
    return counts


def _finding_counts(findings: list[Finding]) -> dict[str, int]:
    counts = {"ERROR": 0, "WARN": 0, "INFO": 0}
    for finding in findings:
        counts[finding.severity] = counts.get(finding.severity, 0) + 1
    return counts


def _npm_direct_dependency_names(package_json: Path) -> set[str]:
    data = _read_json(package_json)
    names: set[str] = set()
    for key in ("dependencies", "devDependencies", "optionalDependencies", "peerDependencies"):
        value = data.get(key) if isinstance(data, dict) else None
        if isinstance(value, dict):
            names.update(str(name) for name in value)
    return names


def _npm_packages(data: dict[str, Any]) -> dict[str, dict[str, Any]]:
    packages = data.get("packages")
    if isinstance(packages, dict):
        out: dict[str, dict[str, Any]] = {}
        for path, entry in packages.items():
            if not isinstance(entry, dict) or path == "":
                continue
            name = entry.get("name") or _npm_name_from_path(path)
            if name:
                out[str(name)] = entry
        return out
    deps = data.get("dependencies")
    if isinstance(deps, dict):
        return {str(name): entry for name, entry in deps.items() if isinstance(entry, dict)}
    return {}


def _npm_name_from_path(path: str) -> str | None:
    marker = "node_modules/"
    if marker not in path:
        return None
    tail = path.rsplit(marker, 1)[1]
    parts = tail.split("/")
    if not parts:
        return None
    if parts[0].startswith("@") and len(parts) > 1:
        return f"{parts[0]}/{parts[1]}"
    return parts[0]


def _purl(ecosystem: str, name: str, version: str) -> str:
    escaped = name.replace("@", "%40") if name.startswith("@") else name
    return f"pkg:{ecosystem}/{escaped}@{version}"


def _license_tokens(expression: str) -> list[str]:
    cleaned = expression.replace("(", " ").replace(")", " ")
    tokens = re.split(r"\s+(?:AND|OR|WITH)\s+|\s+", cleaned)
    return [
        token.strip()
        for token in tokens
        if token.strip() and token.strip().upper() not in {"AND", "OR", "WITH"}
    ]


def _policy_path(root: Path) -> Path:
    return root / ".processkit" / "supply-chain-policy.yaml"


def _read_json(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}


def _read_simple_yaml(path: Path) -> dict[str, Any]:
    """Read the small policy subset without requiring PyYAML.

    Supports nested mappings with scalar lists, which is enough for the
    documented policy contract. Invalid or richer YAML falls back to defaults.
    """
    result: dict[str, Any] = {}
    stack: list[tuple[int, dict[str, Any] | list[Any]]] = [(-1, result)]
    last_key_by_indent: dict[int, str] = {}
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return result
    for raw in lines:
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip(" "))
        text = raw.strip()
        while stack and indent <= stack[-1][0]:
            stack.pop()
        parent = stack[-1][1]
        if text.startswith("- "):
            if isinstance(parent, list):
                parent.append(text[2:].strip().strip("\"'"))
            continue
        if ":" not in text or not isinstance(parent, dict):
            continue
        key, value = text.split(":", 1)
        key = key.strip()
        value = value.strip()
        if value:
            parent[key] = value.strip("\"'")
            continue
        child: dict[str, Any] | list[Any] = {}
        parent[key] = child
        last_key_by_indent[indent] = key
        stack.append((indent, child))
        # If the next meaningful line is a list, convert lazily later.
        if key in {"allow", "review", "deny"}:
            parent[key] = []
            stack[-1] = (indent, parent[key])
    return result


def _skip_path(root: Path, path: Path) -> bool:
    rel = path.resolve().relative_to(root)
    return any(
        part in {".git", "node_modules", ".venv", "dist", "build"}
        or part.startswith(".")
        for part in rel.parts
    )


def _rel(root: Path, path: Path) -> str:
    try:
        return str(path.resolve().relative_to(root))
    except ValueError:
        return str(path)


def _main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="pk-supply-chain")
    parser.add_argument("command", choices=["inventory", "licenses", "audit", "sbom"])
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--include-security", action="store_true")
    parser.add_argument("--include-quality", action="store_true")
    args = parser.parse_args(argv)
    audit = run_audit(
        args.repo_root,
        include_security=args.include_security,
        include_quality=args.include_quality,
    )
    if args.command == "sbom":
        print(json.dumps(export_sbom(audit), indent=2, sort_keys=True))
    elif args.command == "inventory":
        print(json.dumps({
            "manifests": audit["manifests"],
            "components": audit["components"],
            "summary": audit["summary"],
        }, indent=2, sort_keys=True))
    else:
        print(json.dumps(audit, indent=2, sort_keys=True))
    return 1 if audit["summary"]["findings"]["ERROR"] else 0


if __name__ == "__main__":
    raise SystemExit(_main(__import__("sys").argv[1:]))
