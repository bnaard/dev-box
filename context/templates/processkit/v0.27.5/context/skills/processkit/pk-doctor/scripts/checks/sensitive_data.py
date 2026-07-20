"""Repository sensitive-data advisory check.

This check is deliberately conservative. It emits deterministic WARN/ERROR
findings only for high-signal secret/PII formats, then emits one INFO briefing
that tells the derived-project agent what still needs probabilistic review.
"""

from __future__ import annotations

import math
import re
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path

from .common import CheckResult


_MAX_FILE_BYTES = 512_000
_MAX_FINDINGS_PER_KIND = 20
_TEXT_SUFFIXES = {
    ".cfg", ".conf", ".css", ".csv", ".env", ".html", ".ini", ".js",
    ".json", ".jsx", ".log", ".md", ".mjs", ".py", ".rs", ".sh",
    ".toml", ".ts", ".tsx", ".txt", ".yaml", ".yml",
}
_SKIP_PARTS = {
    ".git", ".venv", "__pycache__", "node_modules", "dist", "build",
    "target", ".mypy_cache", ".pytest_cache",
}
_EXAMPLE_PARTS = {
    "test", "tests", "fixtures", "fixture", "examples", "example",
    "docs", "docs-site", "references", "templates",
}


@dataclass(frozen=True)
class Pattern:
    id: str
    label: str
    regex: re.Pattern[str]
    severity: str
    kind: str
    advice: str
    group: int = 0
    entropy_min: float | None = None


_PATTERNS = [
    Pattern(
        "sensitive-data.private-key",
        "private key block",
        re.compile(
            r"-----BEGIN (?:RSA |DSA |EC |OPENSSH |PGP )?PRIVATE KEY-----",
            re.IGNORECASE,
        ),
        "ERROR",
        "secret",
        "Remove the private key from the repo and rotate/revoke it.",
    ),
    Pattern(
        "sensitive-data.aws-access-key",
        "AWS access key id",
        re.compile(r"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b"),
        "ERROR",
        "secret",
        "Rotate the AWS key and move credentials to a secret manager.",
    ),
    Pattern(
        "sensitive-data.github-token",
        "GitHub token",
        re.compile(r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9_]{36,255}\b"),
        "ERROR",
        "secret",
        "Revoke the GitHub token and replace it with scoped runtime injection.",
    ),
    Pattern(
        "sensitive-data.openai-key",
        "OpenAI-style API key",
        re.compile(r"\bsk-[A-Za-z0-9][A-Za-z0-9_-]{20,}\b"),
        "ERROR",
        "secret",
        "Revoke the API key and load it from the runtime secret store.",
    ),
    Pattern(
        "sensitive-data.slack-token",
        "Slack token",
        re.compile(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b"),
        "ERROR",
        "secret",
        "Revoke the Slack token and keep bot/user tokens out of git.",
    ),
    Pattern(
        "sensitive-data.jwt",
        "JWT-like token",
        re.compile(
            r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}"
            r"\.[A-Za-z0-9_-]{10,}\b"
        ),
        "WARN",
        "secret",
        "Treat committed JWTs as credentials; verify expiry and rotate if live.",
    ),
    Pattern(
        "sensitive-data.url-credential",
        "URL embedded credential",
        re.compile(r"\b[a-z][a-z0-9+.-]*://[^/\s:@]+:[^/\s:@]+@"),
        "ERROR",
        "secret",
        "Remove credentials from URLs and rotate the exposed password/token.",
    ),
    Pattern(
        "sensitive-data.generic-assigned-secret",
        "assigned high-entropy secret",
        re.compile(
            r"(?i)\b(?:api[_-]?key|secret|token|password|passwd|pwd|"
            r"client[_-]?secret|private[_-]?key)\b\s*[:=]\s*"
            r"['\"]?([A-Za-z0-9_./+=-]{20,})['\"]?"
        ),
        "WARN",
        "secret",
        "Review the assigned value; rotate it if it is real and move it out of git.",
        group=1,
        entropy_min=3.5,
    ),
    Pattern(
        "sensitive-data.email-address",
        "email address",
        re.compile(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b"),
        "WARN",
        "personal_data",
        "Confirm the email is synthetic or intentionally public; otherwise redact it.",
    ),
    Pattern(
        "sensitive-data.us-ssn",
        "US social security number",
        re.compile(r"\b(?!000|666|9\d\d)\d{3}-(?!00)\d{2}-(?!0000)\d{4}\b"),
        "ERROR",
        "personal_data",
        "Remove the SSN-like value and treat the commit as a privacy incident.",
    ),
    Pattern(
        "sensitive-data.phone-number",
        "phone number",
        re.compile(
            r"(?<!\w)(?:\+?\d{1,3}[-.\s]?)?"
            r"(?:\(?\d{3}\)?[-.\s]?)\d{3}[-.\s]?\d{4}(?!\w)"
        ),
        "WARN",
        "personal_data",
        "Confirm the phone number is synthetic or intentionally public; otherwise redact it.",
    ),
]

_CREDIT_CARD_RE = re.compile(
    r"\b(?:\d[ -]*?){13,19}\b"
)
_RISKY_NAME_RE = re.compile(
    r"(?i)\b(?:full[_ -]?name|legal[_ -]?name|customer\w*[_ -]?name|"
    r"patient[_ -]?name|employee[_ -]?name)\b"
)
_RISKY_FILE_RE = re.compile(
    r"(?i)(?:^|[-_.])(?:secret|credential|password|passwd|token|api[_-]?key|"
    r"private[_-]?key|customer|contacts|users|dump|backup|export)(?:[-_.]|$)"
)
_PROBABILISTIC_REVIEW = [
    "Real personal names and aliases in free text, examples, screenshots, and commit messages.",
    "Home/work addresses, precise locations, birth dates, customer IDs, and account IDs.",
    "Medical, financial, education, employment, or support-ticket data linked to a person.",
    "Secrets that do not match known provider formats, especially short passwords and shared keys.",
    "Binary/media files, generated archives, logs, database dumps, and copied production exports.",
]
_DETERMINISTIC_EXAMPLES = [
    {
        "class": "provider tokens",
        "look_for": [
            "AKIA[20-character AWS access key id]",
            "ghp_[GitHub token body]",
            "sk-[API key body]",
            "xoxb-[Slack bot token body]",
        ],
    },
    {
        "class": "assigned secrets",
        "look_for": [
            "API_KEY=[long non-example value]",
            "client_secret: [long non-example value]",
            "password = [long non-example value]",
        ],
    },
    {
        "class": "embedded credentials",
        "look_for": [
            "https://[user]:[password]@[host]/...",
            "postgres://[user]:[password]@[host]/[database]",
        ],
    },
    {
        "class": "personal data",
        "look_for": [
            "[person]@[company].[tld]",
            "+[country code] [phone number]",
            "[US SSN pattern]",
            "[payment card number passing checksum]",
        ],
    },
]
_PROBABILISTIC_EXAMPLES = [
    {
        "class": "human identity data",
        "look_for": [
            "full_name: [real person]",
            "customer_name: [real customer]",
            "employee: [real employee]",
        ],
    },
    {
        "class": "private life or account data",
        "look_for": [
            "home_address: [street/city/postal code]",
            "birth_date: [date tied to a person]",
            "customer_id/account_id: [real account reference]",
        ],
    },
    {
        "class": "domain-sensitive records",
        "look_for": [
            "medical, financial, school, employment, or support-ticket data",
            "screenshots, logs, exports, dumps, archives, or generated reports",
            "short shared passwords, PINs, recovery codes, or ad-hoc bearer values",
        ],
    },
]


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    since_files = ctx.get("since_files")
    results: list[CheckResult] = []
    seen_by_id: dict[str, int] = {}
    scanned = 0
    skipped_examples = 0
    weak_signal_files: list[str] = []
    email_allowlist = _email_allowlist(repo_root)

    for path in _candidate_files(repo_root, since_files):
        rel = _rel(repo_root, path)
        if _is_example_path(path):
            skipped_examples += 1
            continue
        text = _read_text(path)
        if text is None:
            continue
        scanned += 1
        if _RISKY_FILE_RE.search(path.name) or _RISKY_NAME_RE.search(text):
            if len(weak_signal_files) < 20:
                weak_signal_files.append(rel)
        for pattern in _PATTERNS:
            for lineno, excerpt, secret in _matches(
                path, text, pattern, email_allowlist
            ):
                if pattern.entropy_min is not None:
                    if _entropy(secret or excerpt) < pattern.entropy_min:
                        continue
                if seen_by_id.get(pattern.id, 0) >= _MAX_FINDINGS_PER_KIND:
                    break
                seen_by_id[pattern.id] = seen_by_id.get(pattern.id, 0) + 1
                results.append(_finding(repo_root, path, lineno, excerpt, pattern))
        for lineno, excerpt in _credit_cards(text):
            fid = "sensitive-data.credit-card"
            if seen_by_id.get(fid, 0) >= _MAX_FINDINGS_PER_KIND:
                break
            seen_by_id[fid] = seen_by_id.get(fid, 0) + 1
            results.append(CheckResult(
                severity="ERROR",
                category="sensitive_data",
                id=fid,
                message=(
                    f"credit-card-like number with valid checksum in {rel}:{lineno}: "
                    f"{_redact(excerpt)}"
                ),
                suggested_fix=(
                    "Remove the card-like value and treat the commit as a privacy incident."
                ),
                action_kind="user_confirmation_needed",
                default_agent_action="ask_user",
                requires_user_confirmation=True,
                extra={"path": rel, "line": lineno, "kind": "personal_data"},
            ))

    results.append(CheckResult(
        severity="INFO",
        category="sensitive_data",
        id="sensitive-data.probabilistic-briefing",
        message=(
            "Deterministic sensitive-data scan complete; agent must still review "
            "context-dependent personal data and non-standard secrets."
        ),
        action_required=False,
        extra={
            "scanned_files": scanned,
            "skipped_example_files": skipped_examples,
            "deterministic_findings": len(results),
            "weak_signal_files": weak_signal_files,
            "probabilistic_review": _PROBABILISTIC_REVIEW,
            "deterministic_classes": [
                "private keys", "provider tokens", "JWTs", "URL credentials",
                "assigned high-entropy secret values", "email addresses",
                "phone numbers", "SSNs", "credit cards with Luhn checksum",
            ],
            "deterministic_examples": _DETERMINISTIC_EXAMPLES,
            "probabilistic_examples": _PROBABILISTIC_EXAMPLES,
        },
    ))
    return results


def _candidate_files(repo_root: Path, since_files: set[Path] | None) -> list[Path]:
    if since_files is not None:
        files = [p for p in since_files if p.is_file()]
    else:
        files = _git_files(repo_root)
        if files is None:
            files = [p for p in repo_root.rglob("*") if p.is_file()]
    return [
        p for p in sorted(files)
        if _is_scannable(repo_root, p)
    ]


def _git_files(repo_root: Path) -> list[Path] | None:
    try:
        out = subprocess.run(
            [
                "git", "-C", str(repo_root), "ls-files", "--cached",
                "--others", "--exclude-standard", "-z",
            ],
            capture_output=True,
            check=True,
        ).stdout
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None
    return [
        (repo_root / part.decode("utf-8", "replace")).resolve()
        for part in out.split(b"\0")
        if part
    ]


def _is_scannable(repo_root: Path, path: Path) -> bool:
    try:
        rel = path.relative_to(repo_root)
    except ValueError:
        return False
    if any(part in _SKIP_PARTS for part in rel.parts):
        return False
    if path.suffix.lower() not in _TEXT_SUFFIXES and path.name not in {
        ".env", ".env.local", ".npmrc", ".pypirc", "Dockerfile",
    }:
        return False
    try:
        return path.stat().st_size <= _MAX_FILE_BYTES
    except OSError:
        return False


def _is_example_path(path: Path) -> bool:
    parts = {part.lower() for part in path.parts}
    if parts & _EXAMPLE_PARTS:
        return True
    name = path.name.lower()
    return name.startswith("test_") or name.endswith("_test.py")


def _read_text(path: Path) -> str | None:
    try:
        data = path.read_bytes()
    except OSError:
        return None
    if b"\0" in data[:4096]:
        return None
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def _matches(
    path: Path,
    text: str,
    pattern: Pattern,
    email_allowlist: set[str] | None = None,
):
    for match in pattern.regex.finditer(text):
        line = text.count("\n", 0, match.start()) + 1
        excerpt = match.group(0)
        secret = match.group(pattern.group) if pattern.group else excerpt
        line_text = _line_at(text, line)
        if _is_false_positive(
            path, pattern, excerpt, secret, line_text, email_allowlist
        ):
            continue
        yield line, excerpt, secret


def _credit_cards(text: str):
    for match in _CREDIT_CARD_RE.finditer(text):
        raw = match.group(0)
        digits = re.sub(r"\D", "", raw)
        if 13 <= len(digits) <= 19 and _luhn(digits):
            line = text.count("\n", 0, match.start()) + 1
            yield line, raw


def _luhn(digits: str) -> bool:
    total = 0
    parity = len(digits) % 2
    for idx, char in enumerate(digits):
        n = int(char)
        if idx % 2 == parity:
            n *= 2
            if n > 9:
                n -= 9
        total += n
    return total % 10 == 0


def _entropy(value: str) -> float:
    if not value:
        return 0.0
    counts = {char: value.count(char) for char in set(value)}
    length = len(value)
    return -sum((count / length) * math.log2(count / length) for count in counts.values())


def _line_at(text: str, line: int) -> str:
    lines = text.splitlines()
    if 1 <= line <= len(lines):
        return lines[line - 1]
    return ""


def _is_false_positive(
    path: Path,
    pattern: Pattern,
    excerpt: str,
    secret: str,
    line_text: str,
    email_allowlist: set[str] | None = None,
) -> bool:
    lowered_line = line_text.lower()
    lowered_excerpt = excerpt.lower()

    if pattern.id == "sensitive-data.url-credential":
        if "[" in excerpt and "]" in excerpt:
            return True

    if pattern.id == "sensitive-data.email-address":
        if _team_member_pii_opted_in(path):
            return True
        synthetic = {
            "alice@example.com",
            "alex@example.com",
            "bob@company.com",
            "git@github.com",
            "release@aibox.local",
            "you@example.com",
        }
        if lowered_excerpt in synthetic:
            return True
        if lowered_excerpt in (email_allowlist or set()):
            return True
        if "identity.toml" in lowered_line:
            return True

    if pattern.id == "sensitive-data.phone-number":
        if path.suffix == ".md" and "skills" in path.parts:
            return True
        digits = re.sub(r"\D", "", excerpt)
        if re.fullmatch(r"20\d{10}", digits):
            return True
        if "/" in excerpt:
            return True
        if "http://" in lowered_line or "https://" in lowered_line:
            return True
        if "phone number" in lowered_line or "working_hours" in lowered_line:
            return True

    if pattern.id == "sensitive-data.generic-assigned-secret":
        if "lexical_token_from_id" in line_text:
            return True
        if secret.endswith("_id") or secret.endswith("-id"):
            return True

    return False


def _email_allowlist(repo_root: Path) -> set[str]:
    """Read intentional public/synthetic email addresses from project config."""
    path = repo_root / ".pk-doctor-allowlist.toml"
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError):
        return set()
    section = data.get("sensitive_data", {})
    allowlist = section.get("email_allowlist", {}) if isinstance(section, dict) else {}
    addresses = allowlist.get("addresses", []) if isinstance(allowlist, dict) else []
    return {
        address.strip().lower()
        for address in addresses
        if isinstance(address, str) and address.strip()
    }


def _team_member_pii_opted_in(path: Path) -> bool:
    parts = path.parts
    try:
        idx = parts.index("team-members")
    except ValueError:
        return False
    if len(parts) <= idx + 2:
        return False
    tm_root = Path(*parts[:idx + 2])
    entity_path = tm_root / "team-member.md"
    try:
        text = entity_path.read_text(encoding="utf-8")
    except OSError:
        return False
    if not text.startswith("---"):
        return False
    try:
        _start, rest = text.split("---", 1)
        block, _body = rest.split("---", 1)
    except ValueError:
        return False
    try:
        import yaml

        data = yaml.safe_load(block)
    except Exception:
        return False
    if not isinstance(data, dict):
        return False
    spec = data.get("spec") if isinstance(data.get("spec"), dict) else {}
    privacy = spec.get("privacy") if isinstance(spec.get("privacy"), dict) else {}
    return privacy.get("allow_committed_pii") is True


def _finding(
    repo_root: Path,
    path: Path,
    lineno: int,
    excerpt: str,
    pattern: Pattern,
) -> CheckResult:
    rel = _rel(repo_root, path)
    return CheckResult(
        severity=pattern.severity,  # type: ignore[arg-type]
        category="sensitive_data",
        id=pattern.id,
        message=f"{pattern.label} in {rel}:{lineno}: {_redact(excerpt)}",
        suggested_fix=pattern.advice,
        action_kind="user_confirmation_needed",
        default_agent_action="ask_user",
        requires_user_confirmation=True,
        extra={"path": rel, "line": lineno, "kind": pattern.kind},
    )


def _redact(value: str) -> str:
    compact = " ".join(value.strip().split())
    if len(compact) <= 12:
        return "<redacted>"
    return f"{compact[:6]}...{compact[-4:]}"


def _rel(repo_root: Path, path: Path) -> str:
    try:
        return str(path.relative_to(repo_root))
    except ValueError:
        return str(path)
