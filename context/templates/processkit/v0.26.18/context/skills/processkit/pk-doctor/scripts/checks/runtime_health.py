"""Container-local runtime health probes for pk-doctor.

These checks intentionally live in processkit rather than host-side
installer diagnostics because they inspect binaries, mounted runtime-home
paths, cgroups, and process state from inside the running development
container.
"""

from __future__ import annotations

import shutil
import sqlite3
import subprocess
import tempfile
from pathlib import Path

from .common import CheckResult


CATEGORY = "runtime_health"
POWERKIT_ROOT = Path("/usr/local/share/aibox/tmux/plugins/tmux-powerkit")
POWERKIT_ENTRYPOINT = POWERKIT_ROOT / "tmux-powerkit.tmux"


def _read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def _is_container() -> bool:
    if Path("/.dockerenv").exists() or Path("/run/.containerenv").exists():
        return True
    cgroup = _read_text(Path("/proc/1/cgroup"))
    return any(token in cgroup for token in ("docker", "containerd", "kubepods"))


def _aibox_toml(repo_root: Path) -> str:
    return _read_text(repo_root / "aibox.toml")


def _codex_enabled_from_text(text: str) -> bool:
    return "harness = \"codex\"" in text and "enable = true" in text


def _aibox_runtime_markers(repo_root: Path) -> bool:
    text = _aibox_toml(repo_root)
    return (
        "apiVersion = \"aibox.projectious.work/v1\"" in text
        or (repo_root / ".aibox-home").exists()
        or (Path.home() / ".config" / "tmux" / "tmux.conf").exists()
    )


def _codex_enabled(repo_root: Path) -> bool:
    return _codex_enabled_from_text(_aibox_toml(repo_root))


def _configured_lnav(repo_root: Path) -> bool:
    tmux_conf_candidates = [
        repo_root / ".aibox-home" / ".config" / "tmux" / "tmux.conf",
        Path.home() / ".config" / "tmux" / "tmux.conf",
    ]
    if (repo_root / ".aibox-home" / ".config" / "lnav").exists():
        return True
    return any("lnav" in _read_text(path).lower() for path in tmux_conf_candidates)


def _powerkit_configured(repo_root: Path) -> bool:
    candidates = [
        repo_root / ".aibox-home" / ".config" / "tmux" / "tmux.conf",
        Path.home() / ".config" / "tmux" / "tmux.conf",
    ]
    return any("tmux-powerkit" in _read_text(path) for path in candidates)


def _sqlite_vec_probe() -> tuple[bool, str]:
    try:
        import sqlite_vec  # type: ignore

        conn = sqlite3.connect(":memory:")
        try:
            conn.enable_load_extension(True)
            sqlite_vec.load(conn)
            conn.enable_load_extension(False)
        finally:
            conn.close()
        return True, "sqlite-vec import/load probe passed"
    except Exception as exc:
        return False, f"{type(exc).__name__}: {exc}"


def _bwrap_smoke() -> tuple[bool, str]:
    bwrap = shutil.which("bwrap") or shutil.which("bubblewrap")
    if not bwrap:
        return False, "bwrap/bubblewrap not found on PATH"
    cmd = [
        bwrap,
        "--unshare-user-try",
        "--ro-bind", "/usr", "/usr",
        "--ro-bind", "/bin", "/bin",
        "--ro-bind", "/lib", "/lib",
        "--ro-bind-try", "/lib64", "/lib64",
        "--dev", "/dev",
        "--proc", "/proc",
        "/usr/bin/true",
    ]
    try:
        proc = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=2,
            check=False,
        )
    except Exception as exc:
        return False, f"{type(exc).__name__}: {exc}"
    if proc.returncode == 0:
        return True, f"{Path(bwrap).name} smoke probe passed"
    detail = (proc.stderr or proc.stdout or "").strip()
    return False, detail or f"{Path(bwrap).name} exited {proc.returncode}"


def _pid1_finding() -> CheckResult:
    comm = _read_text(Path("/proc/1/comm")).strip()
    cmdline = _read_text(Path("/proc/1/cmdline")).replace("\x00", " ").strip()
    rendered = cmdline or comm or "<unknown>"
    if _is_sleep_infinity(rendered):
        return CheckResult(
            severity="WARN",
            category=CATEGORY,
            id="runtime.pid1-sleep-infinity",
            message=(
                f"PID 1 is {rendered!r}; use container init/keepalive rather "
                "than sleep infinity so signal handling and process reaping work"
            ),
            suggested_fix="regenerate the devcontainer with init enabled",
        )
    return CheckResult(
        severity="INFO",
        category=CATEGORY,
        id="runtime.pid1-ok",
        message=f"PID 1 runtime process is {rendered}",
    )


def _is_sleep_infinity(command: str) -> bool:
    normalized = " ".join(command.split()).strip()
    return normalized == "sleep" or normalized.startswith("sleep infinity")


def _parse_int_file(path: Path) -> int | None:
    text = _read_text(path).strip()
    if not text or text == "max":
        return None
    try:
        return int(text)
    except ValueError:
        return None


def _parse_key_value_ints(text: str) -> dict[str, int]:
    events: dict[str, int] = {}
    for line in text.splitlines():
        key, _, value = line.partition(" ")
        try:
            events[key] = int(value)
        except ValueError:
            continue
    return events


def _cgroup_findings() -> list[CheckResult]:
    results: list[CheckResult] = []
    memory_current = _parse_int_file(Path("/sys/fs/cgroup/memory.current"))
    if memory_current is not None:
        results.append(CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="runtime.cgroup-memory-current",
            message=f"cgroup memory.current is {memory_current} bytes",
        ))
    events = _parse_key_value_ints(_read_text(Path("/sys/fs/cgroup/memory.events")))
    oom_kill = events.get("oom_kill", 0)
    if oom_kill > 0:
        results.append(CheckResult(
            severity="WARN",
            category=CATEGORY,
            id="runtime.cgroup-oom-kill",
            message=f"cgroup memory.events reports {oom_kill} OOM kill(s)",
            suggested_fix="increase container memory or reduce live process pressure",
        ))
    elif events:
        results.append(CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="runtime.cgroup-oom-ok",
            message="cgroup memory.events reports no OOM kills",
        ))

    pids_current = _parse_int_file(Path("/sys/fs/cgroup/pids.current"))
    pids_max = _parse_int_file(Path("/sys/fs/cgroup/pids.max"))
    if pids_current is not None:
        if pids_max and pids_current >= int(pids_max * 0.9):
            results.append(CheckResult(
                severity="WARN",
                category=CATEGORY,
                id="runtime.cgroup-pids-pressure",
                message=f"cgroup pids.current is {pids_current}/{pids_max}",
                suggested_fix="stop stale agent/MCP processes or raise the pids limit",
            ))
        else:
            suffix = f"/{pids_max}" if pids_max else ""
            results.append(CheckResult(
                severity="INFO",
                category=CATEGORY,
                id="runtime.cgroup-pids-current",
                message=f"cgroup pids.current is {pids_current}{suffix}",
            ))
    return results


def _process_counts() -> list[CheckResult]:
    total = 0
    processkit_python = 0
    for proc_dir in Path("/proc").iterdir():
        if not proc_dir.name.isdigit():
            continue
        total += 1
        cmdline = _read_text(proc_dir / "cmdline").replace("\x00", " ")
        if "python" in cmdline and "processkit" in cmdline:
            processkit_python += 1
    severity = "WARN" if processkit_python > 50 else "INFO"
    return [
        CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="runtime.process-count",
            message=f"runtime process count is {total}",
        ),
        CheckResult(
            severity=severity,  # type: ignore[arg-type]
            category=CATEGORY,
            id="runtime.processkit-python-count",
            message=f"processkit Python process count is {processkit_python}",
            suggested_fix=(
                "restart stale MCP servers or the gateway"
                if severity == "WARN"
                else None
            ),
        ),
    ]


def _write_probe(path: Path) -> CheckResult:
    try:
        path.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(
            prefix=".pk-doctor-write-",
            dir=path,
            delete=False,
        ) as handle:
            probe = Path(handle.name)
            handle.write(b"ok\n")
        probe.unlink(missing_ok=True)
        return CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="runtime-home.write-ok",
            message=f"{path} is writable",
        )
    except Exception as exc:
        return CheckResult(
            severity="WARN",
            category=CATEGORY,
            id="runtime-home.write-failed",
            message=f"{path} is not writable: {type(exc).__name__}: {exc}",
            suggested_fix=(
                "repair the runtime-home mount or ask the owner to "
                "regenerate host runtime configuration outside the container"
            ),
        )


def _runtime_home_findings(repo_root: Path) -> list[CheckResult]:
    repo_home = repo_root / ".aibox-home"
    bases = [repo_home] if repo_home.exists() else [Path.home()]
    rels = [
        Path(".config"),
        Path(".cache"),
        Path(".local"),
        Path(".tmux"),
    ]
    results: list[CheckResult] = []
    seen: set[Path] = set()
    for base in bases:
        for rel in rels:
            path = (base / rel).resolve()
            if path in seen or not path.exists():
                continue
            seen.add(path)
            results.append(_write_probe(path))
    if not results:
        results.append(CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="runtime-home.not-detected",
            message="no runtime-home mount paths were detected",
        ))
    return results


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    results: list[CheckResult] = []

    if not _is_container():
        return [CheckResult(
            severity="INFO",
            category=CATEGORY,
            id="runtime.container-skip",
            message="not running inside a container; container-local probes skipped",
        )]

    results.append(CheckResult(
        severity="INFO",
        category=CATEGORY,
        id="runtime.container-detected",
        message="container runtime detected; running in-container health probes",
    ))

    if _configured_lnav(repo_root):
        lnav = shutil.which("lnav")
        results.append(CheckResult(
            severity="INFO" if lnav else "WARN",
            category=CATEGORY,
            id="runtime.lnav-available" if lnav else "runtime.lnav-missing",
            message=(
                f"lnav available at {lnav}"
                if lnav
                else "lnav is not on PATH for the Prefix L structured log viewer"
            ),
            suggested_fix="install lnav in the container image" if not lnav else None,
        ))

    sqlite_ok, sqlite_detail = _sqlite_vec_probe()
    results.append(CheckResult(
        severity="INFO" if sqlite_ok else "WARN",
        category=CATEGORY,
        id="runtime.sqlite-vec-ok" if sqlite_ok else "runtime.sqlite-vec-unavailable",
        message=sqlite_detail,
        suggested_fix=(
            "install sqlite-vec in the pk-doctor/MCP uv runtime"
            if not sqlite_ok
            else None
        ),
    ))

    if _codex_enabled(repo_root):
        bwrap_ok, bwrap_detail = _bwrap_smoke()
        results.append(CheckResult(
            severity="INFO" if bwrap_ok else "WARN",
            category=CATEGORY,
            id="runtime.codex-bwrap-ok" if bwrap_ok else "runtime.codex-bwrap-failed",
            message=bwrap_detail,
            suggested_fix=(
                "repair bubblewrap/user-namespace support for Codex sandboxing"
                if not bwrap_ok
                else None
            ),
        ))

    results.append(_pid1_finding())
    results.extend(_cgroup_findings())
    results.extend(_process_counts())

    if _powerkit_configured(repo_root):
        helpers = [
            Path.home() / ".local" / "bin" / "aibox-powerkit-render-list",
            Path.home() / ".local" / "bin" / "aibox-powerkit-render-session",
        ]
        status_plugins = [
            POWERKIT_ROOT / "src" / "plugins" / "aibox_ai.sh",
            POWERKIT_ROOT / "src" / "plugins" / "aibox_log.sh",
            POWERKIT_ROOT / "src" / "plugins" / "aibox_mcp.sh",
            POWERKIT_ROOT / "src" / "plugins" / "aibox_mig.sh",
            POWERKIT_ROOT / "src" / "plugins" / "aibox_oom.sh",
            POWERKIT_ROOT / "src" / "plugins" / "aibox_proc.sh",
        ]
        missing = [
            str(path)
            for path in [POWERKIT_ENTRYPOINT, *status_plugins, *helpers]
            if not path.is_file()
        ]
        results.append(CheckResult(
            severity="INFO" if not missing else "WARN",
            category=CATEGORY,
            id="runtime.powerkit-ok" if not missing else "runtime.powerkit-missing",
            message=(
                "PowerKit image plugin tree, status plugins, and local render "
                "helpers are present"
                if not missing
                else f"PowerKit runtime files missing: {', '.join(missing)}"
            ),
            suggested_fix=(
                "rebuild/regenerate the aibox runtime image"
                if missing
                else None
            ),
            extra={"missing": missing} if missing else {},
        ))

    if _aibox_runtime_markers(repo_root):
        results.extend(_runtime_home_findings(repo_root))

    return results
