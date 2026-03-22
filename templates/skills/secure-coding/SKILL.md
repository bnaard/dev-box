---
name: secure-coding
description: Secure coding practices based on OWASP Top 10. Injection prevention, XSS mitigation, CSRF protection, input validation, and security headers. Use when reviewing code for security, implementing auth, or hardening web applications.
allowed-tools: Bash Read Write
---

# Secure Coding

Apply OWASP Top 10 security practices when writing, reviewing, or hardening code.
Identify vulnerabilities and implement concrete fixes.

## When to Use

- Reviewing code for security vulnerabilities.
- Implementing input validation or output encoding.
- Adding security headers to web applications.
- Preventing injection attacks (SQL, command, XSS).
- Implementing CSRF protection.
- Auditing code for secrets or sensitive data exposure.
- Hardening an application before production deployment.

## Instructions

### Input Validation

Validate all input at the boundary — never trust client-supplied data:

- **Allowlist over denylist** — define what IS valid, not what isn't.
- **Validate type, length, range, and format** before processing.
- **Reject unexpected input** — fail closed, not open.
- **Validate on the server** — client-side validation is for UX only.

```python
import re

def validate_username(username: str) -> str:
    """Allowlist approach: only permit safe characters."""
    if not re.match(r'^[a-zA-Z0-9_]{3,30}$', username):
        raise ValueError("Invalid username format")
    return username
```

### Output Encoding

Encode output based on context to prevent XSS:

| Context       | Encoding             | Example                          |
|---------------|----------------------|----------------------------------|
| HTML body     | HTML entity encoding | `&lt;script&gt;`                 |
| HTML attribute| Attribute encoding   | `" -> &quot;`                    |
| JavaScript    | JS hex encoding      | `\x3Cscript\x3E`                |
| URL parameter | Percent encoding     | `%3Cscript%3E`                   |
| CSS           | CSS hex encoding     | `\3C script\3E`                  |

Use your framework's built-in template escaping — never build HTML with string
concatenation.

### Injection Prevention

**SQL Injection:** Always use parameterized queries.

```python
# Safe: parameterized query
cursor.execute("SELECT * FROM users WHERE id = %s", (user_id,))
```

**Command Injection:** Avoid shell commands. Use library functions or subprocess
with argument lists (never shell=True with user input).

```python
# Safe: argument list prevents injection
subprocess.run(["convert", filename, "output.png"], check=True)
```

### CSRF Protection

- Use anti-CSRF tokens (synchronizer token pattern) for state-changing requests.
- Set `SameSite=Lax` or `SameSite=Strict` on session cookies.
- Verify `Origin` and `Referer` headers as defense in depth.
- Never use GET for state-changing operations.

### Security Headers

Apply these headers on every response:

```
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; frame-ancestors 'none'
Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: camera=(), microphone=(), geolocation=()
```

Start with a strict CSP and relax only as needed. Never use `unsafe-eval`.

### Secrets Management

- **Never hardcode** secrets, API keys, or credentials in source code.
- Use environment variables or a secrets manager (Vault, AWS Secrets Manager).
- Add patterns to `.gitignore`: `.env`, `*.pem`, `credentials.json`.
- Rotate secrets periodically and after any suspected exposure.
- Use different secrets for each environment (dev, staging, prod).

### Dependency Security

- Audit dependencies regularly: `npm audit`, `pip-audit`, `cargo audit`.
- Pin dependency versions in lock files.
- Update vulnerable dependencies promptly.
- Minimize the dependency tree — fewer deps means fewer attack vectors.

### Security Review Checklist

1. All user input is validated (type, length, format).
2. SQL queries use parameterized statements.
3. HTML output is context-encoded (no raw string interpolation).
4. CSRF tokens protect state-changing endpoints.
5. Security headers are set (CSP, HSTS, X-Frame-Options).
6. No secrets in source code or logs.
7. Authentication uses bcrypt/argon2, not MD5/SHA1.
8. File uploads validate type, size, and store outside webroot.
9. Error messages do not leak stack traces or internal details.
10. Dependencies are audited and up to date.

See `references/owasp-checklist.md` for the full OWASP Top 10 with prevention
checklists and secure code patterns.

## Examples

### Example 1: Security review of a web application

```
User: Review this Express.js app for security issues.

Agent: Reads the codebase and identifies:
  - SQL queries built with string concatenation -> switches to parameterized queries.
  - User input rendered without escaping in EJS templates -> adds output encoding.
  - No CSRF tokens on POST routes -> adds csurf middleware.
  - Missing security headers -> adds helmet middleware with strict CSP.
  - API key hardcoded in config.js -> moves to environment variable.
  Writes fixes for each issue.
```

### Example 2: Harden API input validation

```
User: Add input validation to our user registration endpoint.

Agent: Reads the endpoint, then adds:
  - Email format validation with regex.
  - Password strength requirements (min 12 chars, complexity).
  - Username allowlist validation (alphanumeric, 3-30 chars).
  - Rate limiting on registration endpoint.
  - Sanitization of all fields before database insertion.
  Uses a validation library (Joi, Zod, or pydantic) for consistency.
```

### Example 3: Fix XSS vulnerability

```
User: We found an XSS vulnerability in the search results page.

Agent: Reads the search feature code and finds:
  - Search query is reflected in the page without encoding.
  - Template uses raw HTML interpolation instead of encoded output.
  Fixes:
  - Switches to encoded output in the template engine.
  - Adds Content-Security-Policy header to block inline scripts.
  - Adds input validation to reject HTML/script characters in search.
  - Tests the fix with common XSS payloads.
```
