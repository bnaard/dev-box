#!/usr/bin/env python3
from __future__ import annotations

import json
import tempfile
from pathlib import Path

import supply_chain_audit as sca


def check(label: str, condition: bool, detail: str = "") -> None:
    if not condition:
        raise AssertionError(f"{label}: {detail}")
    print(f"PASS {label}")


with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    app = root / "app"
    app.mkdir()
    (app / "package.json").write_text(json.dumps({
        "name": "demo",
        "private": True,
        "dependencies": {"left-pad": "1.3.0"},
        "devDependencies": {"copyleft": "1.0.0"},
    }), encoding="utf-8")
    (app / "package-lock.json").write_text(json.dumps({
        "lockfileVersion": 3,
        "packages": {
            "": {"name": "demo", "version": "1.0.0"},
            "node_modules/left-pad": {
                "version": "1.3.0",
                "license": "MIT",
            },
            "node_modules/copyleft": {
                "version": "1.0.0",
                "license": "GPL-3.0-only",
                "dev": True,
            },
            "node_modules/mystery": {
                "version": "2.0.0",
            },
        },
    }), encoding="utf-8")

    audit = sca.run_audit(root)
    ids = {finding["id"] for finding in audit["findings"]}
    check("discovers npm manifest", audit["summary"]["manifest_count"] == 1)
    check("inventories lockfile packages", audit["summary"]["component_count"] == 3)
    check("detects denied license", "supply-chain.license-denied" in ids)
    check("detects unknown license", "supply-chain.license-unknown" in ids)
    check("security skipped by default", "supply-chain.security-skipped" in ids)
    sbom = sca.export_sbom(audit)
    check("exports CycloneDX-like SBOM", sbom["bomFormat"] == "CycloneDX")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    (root / "package.json").write_text(json.dumps({"private": True}), encoding="utf-8")
    audit = sca.run_audit(root)
    ids = {finding["id"] for finding in audit["findings"]}
    check("missing app lockfile is error", "supply-chain.lockfile-missing" in ids)
