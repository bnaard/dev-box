"""Check registry for pk-doctor.

Adding a Phase 2 check: drop a new module next to these, then append it
to REGISTRY. Order matters only for report ordering on stdout; the
aggregator doesn't otherwise sequence checks.
"""

from __future__ import annotations

from . import (
    schema_filename,
    schema_vocabulary,
    v2_contracts,
    v1_entity_drift,
    sharding,
    entity_storage_hygiene,
    migrations,
    migration_integrity,
    drift,
    team_consistency,
    team_member_exports,
    release_integrity,
    commands_consistency,
    mcp_config_drift,
    mcp_gateway,
    server_header_drift,
    preauth_applied,
    skill_dag,
    context_consumption,
    context_hygiene,
    runtime_health,
    doctor_boundary,
    id_vocabulary,
    agents_md_hygiene,
)

# (name, module). Name is the --category= token and the key in the
# per-category tally block of the doctor.report LogEntry.
REGISTRY = [
    ("schema_filename", schema_filename),
    ("schema_vocabulary", schema_vocabulary),
    ("v2_contracts", v2_contracts),
    ("v1_entity_drift", v1_entity_drift),
    ("sharding", sharding),
    ("entity_storage_hygiene", entity_storage_hygiene),
    ("migrations", migrations),
    ("migration_integrity", migration_integrity),
    ("drift", drift),
    ("team_consistency", team_consistency),
    ("team_member_exports", team_member_exports),
    ("release_integrity", release_integrity),
    ("commands_consistency", commands_consistency),
    ("mcp_config_drift", mcp_config_drift),
    ("mcp_gateway", mcp_gateway),
    ("server_header_drift", server_header_drift),
    ("preauth_applied", preauth_applied),
    ("skill_dag", skill_dag),
    ("context_consumption", context_consumption),
    ("context_hygiene", context_hygiene),
    ("runtime_health", runtime_health),
    ("doctor_boundary", doctor_boundary),
    ("id_vocabulary", id_vocabulary),
    ("agents_md_hygiene", agents_md_hygiene),
]


def get(name: str):
    for n, mod in REGISTRY:
        if n == name:
            return mod
    raise KeyError(f"unknown check: {name}")


def names() -> list[str]:
    return [n for n, _ in REGISTRY]
