"""Supply-chain audit check.

Offline, deterministic supply-chain hygiene checks for pk-doctor.

The check prefers data from the local supply-chain-audit core library when
available, but degrades gracefully if that library is not present.
"""

from __future__ import annotations

import importlib.util
import os
from pathlib import Path
from typing import Any

import yaml

from .common import CheckResult


CATEGORY = "supply_chain"

_POLICY_REL = ".processkit/supply-chain-policy.yaml"

_CORE_LIBRARY_PATHS = (
    "src/context/skills/processkit/supply-chain-audit/scripts/supply_chain_audit.py",
    "src/context/skills/processkit/supply_chain_audit/scripts/supply_chain_audit.py",
    "context/skills/processkit/supply-chain-audit/scripts/supply_chain_audit.py",
    "context/skills/processkit/supply_chain_audit/scripts/supply_chain_audit.py",
    "src/context/skills/processkit/supply_chain_audit.py",
    "src/context/skills/processkit/supply-chain-audit.py",
)

_CORE_FUNCTIONS = (
    "run_audit",
    "run_phase1",
    "collect_supply_chain_report",
    "collect_supply_chain_audit",
    "collect_report",
    "collect_data",
    "run_supply_chain_audit",
    "scan_supply_chain",
    "build_report",
    "read_report",
)

_SKIP_DIRS = {
    ".git",
    ".venv",
    "node_modules",
    "dist",
    "build",
    ".pytest_cache",
    ".mypy_cache",
    "__pycache__",
}

_KNOWN_MANIFESTS = {
    "package.json",
    "pyproject.toml",
    "Pipfile",
    "requirements.txt",
    "requirements-dev.txt",
    "Cargo.toml",
    "go.mod",
    "Gemfile",
    "mix.exs",
}

_MANIFEST_LOCK_RULES: dict[str, dict[str, set[str]]] = {
    "Node": {
        "manifests": {"package.json"},
        "lockfiles": {"package-lock.json", "npm-shrinkwrap.json", "yarn.lock", "pnpm-lock.yaml"},
    },
    "Rust": {
        "manifests": {"Cargo.toml"},
        "lockfiles": {"Cargo.lock"},
    },
    "Go": {
        "manifests": {"go.mod"},
        "lockfiles": {"go.sum"},
    },
    "Python": {
        "manifests": {"Pipfile"},
        "lockfiles": {"Pipfile.lock"},
    },
    "Ruby": {
        "manifests": {"Gemfile"},
        "lockfiles": {"Gemfile.lock"},
    },
    "Elixir": {
        "manifests": {"mix.exs"},
        "lockfiles": {"mix.lock"},
    },
}

_KNOWN_SBOM_NAMES = {
    "sbom.json",
    "sbom.xml",
    "cyclonedx.json",
    "cyclonedx.xml",
    "spdx.json",
    "spdx.xml",
    "dependency-tree.json",
}
_SBOM_KEYWORDS = ("sbom", "cyclonedx", "spdx", "bom")


def _normalize(value: Any) -> str:
    if value is None:
        return ""
    return str(value).strip().lower()


def _split_license(value: str) -> list[str]:
    normalized = _normalize(value).replace("(", " ").replace(")", " ")
    return [token.strip() for token in normalized.replace(";", " ").split()]


def _policy_lists(raw: dict[str, Any] | None, key: str) -> set[str]:
    if not raw:
        return set()
    licenses = raw.get("licenses")
    if not isinstance(licenses, dict):
        return set()
    values = licenses.get(key)
    if not isinstance(values, list):
        return set()
    return {_normalize(v) for v in values if isinstance(v, str)}


def _load_yaml(path: Path) -> tuple[dict[str, Any] | None, str | None]:
    try:
        payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    except Exception as exc:
        return None, f"{type(exc).__name__}: {exc}"
    return payload if isinstance(payload, dict) else None, None


def _policy(repo_root: Path) -> tuple[dict[str, Any] | None, list[CheckResult]]:
    path = repo_root / _POLICY_REL
    if not path.is_file():
        return None, [CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="supply_chain.no-policy",
            message=(
                "no supply-chain policy file was found in .processkit/"
                "supply-chain-policy.yaml; policy checks are advisory until "
                "the project opts into enforcement"
            ),
        )]

    payload, error = _load_yaml(path)
    if error is not None:
        return None, [CheckResult(
            severity="WARN",
            category=CATEGORY,
            id="supply_chain.policy-load-failed",
            message=f"could not read supply-chain policy {path}: {error}",
        )]
    if not isinstance(payload, dict):
        return None, [CheckResult(
            severity="WARN",
            category=CATEGORY,
            id="supply_chain.policy-invalid",
            message=(
                f"supply-chain policy {path} is malformed; policy checks "
                "will remain advisory"
            ),
        )]
    return payload, []


def _iter_files(root: Path, names: set[str]) -> list[Path]:
    wanted = {_normalize(n) for n in names}
    found: list[Path] = []
    if not root.is_dir():
        return found

    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [
            d
            for d in dirnames
            if d not in _SKIP_DIRS and not d.startswith(".")
        ]
        for filename in filenames:
            if _normalize(filename) in wanted:
                found.append(Path(dirpath) / filename)
    return sorted(found)


def _collect_inventory(
    repo_root: Path,
) -> tuple[dict[str, int], list[Path], list[tuple[Path, str]], bool]:
    manifests = _iter_files(repo_root, set(_KNOWN_MANIFESTS))
    lockfile_names: set[str] = set()
    for rule in _MANIFEST_LOCK_RULES.values():
        lockfile_names.update(rule["lockfiles"])
    lockfiles = _iter_files(repo_root, lockfile_names)
    counts: dict[str, int] = {}
    missing: list[tuple[Path, str]] = []

    manifest_index: dict[str, list[Path]] = {}
    for manifest in manifests:
        manifest_index.setdefault(manifest.name, []).append(manifest)
        counts.setdefault(manifest.name, 0)
        counts[manifest.name] = counts[manifest.name] + 1

    for family, rule in _MANIFEST_LOCK_RULES.items():
        for manifest_name in rule["manifests"]:
            for manifest in manifest_index.get(manifest_name, []):
                counts[family] = counts.get(family, 0) + 1
                if not any((manifest.parent / lock).exists() for lock in rule["lockfiles"]):
                    missing.append((manifest, family))

    pyproject_paths = manifest_index.get("pyproject.toml", [])
    for manifest in pyproject_paths:
        counts["Python"] = counts.get("Python", 0) + 1
        if not ((manifest.parent / "poetry.lock").exists() or (manifest.parent / "uv.lock").exists()):
            missing.append((manifest, "Python"))

    dependency_present = bool(manifests or lockfiles)
    return counts, sorted(lockfiles), missing, dependency_present


def _find_sbom_files(repo_root: Path) -> list[Path]:
    found: list[Path] = []
    seen: set[Path] = set()
    for path in _iter_files(repo_root, set(_KNOWN_SBOM_NAMES)):
        if path not in seen:
            seen.add(path)
            found.append(path)

    for dirpath, dirnames, filenames in os.walk(repo_root):
        dirnames[:] = [
            d
            for d in dirnames
            if d not in _SKIP_DIRS and not d.startswith(".")
        ]
        for filename in filenames:
            lower = filename.lower()
            if not lower.endswith((".json", ".xml", ".yaml", ".yml")):
                continue
            if not any(token in lower for token in _SBOM_KEYWORDS):
                continue
            path = Path(dirpath) / filename
            if path not in seen:
                seen.add(path)
                found.append(path)
    return sorted(found)


def _load_module(path: Path, module_name: str):
    spec = importlib.util.spec_from_file_location(module_name, path)
    if not spec or not spec.loader:
        return None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _coerce_payload(raw: Any) -> dict[str, Any] | None:
    if raw is None:
        return None
    if hasattr(raw, "dict") and callable(raw.dict):
        try:
            raw = raw.dict()  # type: ignore[call-overload]
        except Exception:
            pass
    if hasattr(raw, "to_dict") and callable(raw.to_dict):
        try:
            raw = raw.to_dict()  # type: ignore[call-overload]
        except Exception:
            pass
    return raw if isinstance(raw, dict) else None


def _call_core_fn(fn, repo_root: Path) -> dict[str, Any] | None:
    attempts: list[tuple[tuple[Any, ...], dict[str, Any]]] = [
        ((), {}),
        ((repo_root,), {}),
        ((str(repo_root),), {}),
        ((), {"repo_root": repo_root}),
        ((), {"project_root": repo_root}),
        ((), {"root": repo_root}),
        ((), {"path": repo_root}),
    ]
    for args, kwargs in attempts:
        try:
            result = fn(*args, **kwargs)
        except TypeError:
            continue
        except Exception:
            return None
        payload = _coerce_payload(result)
        if payload is not None:
            return payload
    return None


def _load_core_payload(repo_root: Path) -> dict[str, Any] | None:
    for rel in _CORE_LIBRARY_PATHS:
        path = repo_root / rel
        if not path.is_file():
            continue
        module_name = (
            "_pk_doctor_supply_chain_core_"
            + path.as_posix().replace("/", "_").replace("-", "_").replace(".", "_")
        )
        try:
            module = _load_module(path, module_name)
        except Exception:
            continue
        if module is None:
            continue
        for fn_name in _CORE_FUNCTIONS:
            fn = getattr(module, fn_name, None)
            if not callable(fn):
                continue
            payload = _call_core_fn(fn, repo_root)
            if payload is not None:
                return payload
    return None


def _is_high_or_critical(severity: Any) -> bool:
    return _normalize(severity) in {
        "critical",
        "high",
        "critical+",
        "high+",
        "c",
        "h",
    }


def _license_findings(core_payload: dict[str, Any] | None, policy: dict[str, Any] | None) -> list[CheckResult]:
    deny = _policy_lists(policy, "deny")
    review = _policy_lists(policy, "review")
    allowed = _policy_lists(policy, "allow")

    findings: list[CheckResult] = []
    raw_components = core_payload.get("components") if isinstance(core_payload, dict) else []
    for item in raw_components if isinstance(raw_components, list) else []:
        if not isinstance(item, dict):
            continue
        name = str(item.get("name") or item.get("package") or item.get("purl") or "<unknown>")
        license_value = item.get("license") or item.get("license_expression") or ""
        bucket = _normalize(
            item.get("license_bucket")
            or item.get("license_class")
            or item.get("license_status"),
        )

        if bucket not in {"allow", "review", "deny", "unknown"}:
            tokens = {t for t in _split_license(str(license_value)) if t}
            if deny and any(token in deny for token in tokens):
                bucket = "deny"
            elif review and any(token in review for token in tokens):
                bucket = "review"
            elif allowed and tokens and all(token in allowed for token in tokens):
                bucket = "allow"
            elif tokens:
                bucket = "unknown"
            else:
                bucket = "unknown"

        if bucket == "deny":
            findings.append(CheckResult(
                severity="ERROR",
                category=CATEGORY,
                id="supply_chain.denied-license",
                message=(
                    f"{name} is blocked by denied policy because its license is "
                    f"{license_value or '<unknown>'}"
                ),
                entity_ref=name,
                extra={"license": str(license_value) if license_value else None, "license_bucket": bucket},
            ))
            continue

        if bucket == "review":
            findings.append(CheckResult(
                severity="WARN",
                category=CATEGORY,
                id="supply_chain.review-license",
                message=(f"{name} uses a review-needed license policy ({license_value or '<unknown>'})"),
                entity_ref=name,
                extra={"license": str(license_value) if license_value else None, "license_bucket": bucket},
            ))
            continue

        if bucket == "unknown":
            findings.append(CheckResult(
                severity="WARN",
                category=CATEGORY,
                id="supply_chain.unknown-license",
                message=(
                    f"{name} has unknown license metadata "
                    f"({license_value or '<unknown>'})"
                ),
                entity_ref=name,
                extra={"license": str(license_value) if license_value else None},
            ))

    return findings


def _missing_lockfile_findings(missing: list[tuple[Path, str]], root: Path) -> list[CheckResult]:
    return [CheckResult(
        severity="ERROR",
        category=CATEGORY,
        id="supply_chain.missing-lockfile",
        message=f"{manifest.relative_to(root)} has no application lockfile for {family}",
        entity_ref=str(manifest.relative_to(root)),
        extra={"manifest": str(manifest.relative_to(root)), "family": family},
    ) for manifest, family in missing]


def _inventory_events(
    counts: dict[str, int],
    lockfiles: list[Path],
    missing: list[tuple[Path, str]],
    root: Path,
    sbom_files: list[Path],
) -> list[CheckResult]:
    events: list[CheckResult] = [CheckResult(
        severity="INFO",
        category=CATEGORY,
        id="supply_chain.inventory",
        message=(
            f"Found {sum(counts.values())} manifest file(s), "
            f"{len(lockfiles)} lockfile(s), "
            f"{len(sbom_files)} SBOM artifact(s)"
        ),
        extra={
            "manifest_counts": dict(sorted(counts.items())),
            "lockfiles": [str(path.relative_to(root)) for path in lockfiles],
            "sbom_files": [str(path.relative_to(root)) for path in sbom_files],
        },
    )]

    if sbom_files:
        events.append(CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="supply_chain.sbom-found",
            message=(
                "SBOM files found: "
                + ", ".join(str(path.relative_to(root)) for path in sbom_files[:3])
            ),
            extra={"sbom_files": [str(path.relative_to(root)) for path in sbom_files]},
        ))

    events.extend(_missing_lockfile_findings(missing, root))
    return events


def _iter_security_findings(core_payload: dict[str, Any]) -> list[dict[str, Any]]:
    findings: list[dict[str, Any]] = []
    for key in ("security", "vulnerabilities", "scanner_findings", "security_findings"):
        items = core_payload.get(key)
        if isinstance(items, list):
            for item in items:
                if isinstance(item, dict):
                    findings.append(item)

    phases = core_payload.get("phases")
    if isinstance(phases, dict):
        phase2 = phases.get("phase2")
        if isinstance(phase2, dict):
            raw = phase2.get("scanner_findings")
            if isinstance(raw, list):
                for item in raw:
                    if isinstance(item, dict):
                        findings.append(item)
    return findings


def _security_findings(core_payload: dict[str, Any] | None) -> list[CheckResult]:
    if not isinstance(core_payload, dict):
        return []

    skipped = core_payload.get("skipped_scanners")
    results: list[CheckResult] = []
    if isinstance(skipped, list):
        for scanner in skipped:
            if not isinstance(scanner, dict):
                continue
            name = scanner.get("name")
            if not name:
                continue
            reason = scanner.get("reason")
            results.append(CheckResult(
                severity="WARN",
                category=CATEGORY,
                id="supply_chain.scanner-skipped",
                message=(
                    f"security scanner {name} was skipped"
                    + (f" ({reason})" if reason else "")
                ),
                extra={"scanner": name, "reason": reason},
            ))

    advisory: list[dict[str, Any]] = []
    findings = _iter_security_findings(core_payload)
    for item in findings:
        severity = _normalize(item.get("severity") or item.get("level") or item.get("risk"))
        vuln_id = item.get("id") or item.get("cve") or item.get("reference") or item.get("name")
        dependency = item.get("dependency") or item.get("package") or item.get("component")
        if not vuln_id:
            vuln_id = "UNKNOWN"
        entry = {
            "id": vuln_id,
            "package": dependency,
            "severity": severity,
        }
        if _is_high_or_critical(severity):
            advisory.append(entry)
            continue
        advisory.append(entry)

    if not findings:
        return results
    high_or_critical = [
        item
        for item in advisory
        if _is_high_or_critical(item.get("severity"))
    ]
    if high_or_critical:
        results.append(CheckResult(
            severity="ERROR",
            category=CATEGORY,
            id="supply_chain.high-or-critical-vulnerability",
            message="high/critical vulnerabilities were detected and require remediation",
            extra={"vulnerabilities": advisory},
        ))
        return results

    results.append(CheckResult(
        severity="INFO",
        category=CATEGORY,
        id="supply_chain.advisory-security",
        message=(
            "security scan findings are advisory and require local policy review; "
            "no automatic networked verification was performed"
        ),
        extra={"vulnerabilities": advisory},
    ))

    return results


def _outdated_findings(core_payload: dict[str, Any] | None) -> list[CheckResult]:
    if not isinstance(core_payload, dict):
        return []
    outdated = core_payload.get("outdated")
    if not isinstance(outdated, list):
        return []
    outdated = [item for item in outdated if isinstance(item, dict)]
    if not outdated:
        return []
    return [CheckResult(
        severity="INFO",
        category=CATEGORY,
        id="supply_chain.advisory-outdated",
        message=f"outdated dependency signal is advisory ({len(outdated)} record(s))",
        extra={"outdated": outdated},
    )]


def _supplier_quality_findings(core_payload: dict[str, Any] | None) -> list[CheckResult]:
    if not isinstance(core_payload, dict):
        return []

    quality: list[dict[str, Any]] = []
    for key in ("supplier_quality", "supplier-quality"):
        value = core_payload.get(key)
        if isinstance(value, list):
            quality.extend(item for item in value if isinstance(item, dict))

    phases = core_payload.get("phases")
    if isinstance(phases, dict):
        phase3 = phases.get("phase3")
        if isinstance(phase3, dict):
            value = phase3.get("quality_findings")
            if isinstance(value, list):
                quality.extend(item for item in value if isinstance(item, dict))

    if not quality:
        return []
    return [CheckResult(
        severity="INFO",
        category=CATEGORY,
        id="supply_chain.advisory-supplier-quality",
        message=f"supplier-quality signal is advisory ({len(quality)} record(s))",
        extra={"supplier_quality": quality},
    )]


def _run_check_payload(core_payload: dict[str, Any] | None, repo_root: Path) -> list[CheckResult]:
    findings: list[CheckResult] = []
    policy, policy_results = _policy(repo_root)
    findings.extend(policy_results)

    findings.extend(_license_findings(core_payload, policy))
    findings.extend(_security_findings(core_payload))
    findings.extend(_outdated_findings(core_payload))
    findings.extend(_supplier_quality_findings(core_payload))
    return findings


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    core_payload = _load_core_payload(repo_root)
    counts, lockfiles, missing_lockfiles, has_dependencies = _collect_inventory(repo_root)
    sbom_files = _find_sbom_files(repo_root)

    if not has_dependencies and core_payload is None:
        return [CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="supply_chain.no-artifacts",
            message="no supported manifests or lockfiles were found for supply-chain auditing",
        )]

    findings = []
    findings.extend(_inventory_events(counts, lockfiles, missing_lockfiles, repo_root, sbom_files))
    findings.extend(_run_check_payload(core_payload, repo_root))
    return findings
