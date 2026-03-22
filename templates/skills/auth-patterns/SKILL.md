---
name: auth-patterns
description: Authentication and authorization patterns including OAuth2, JWT, session management, and RBAC/ABAC. Use when implementing login flows, securing APIs, managing tokens, or designing permission systems.
allowed-tools: Bash Read Write
---

# Authentication & Authorization Patterns

Implement and review authentication flows, token management, and permission systems
using industry-standard protocols and best practices.

## When to Use

- Implementing OAuth2 login flows (authorization code, PKCE, client credentials).
- Working with JWTs — creating, validating, refreshing tokens.
- Designing session management for web applications.
- Building role-based (RBAC) or attribute-based (ABAC) permission systems.
- Securing API endpoints with API keys, bearer tokens, or OAuth scopes.
- Reviewing authentication code for security issues.

## Instructions

### OAuth2 Flow Selection

Choose the correct flow based on the client type:

| Client Type              | Flow                  | Token Storage          |
|--------------------------|-----------------------|------------------------|
| Server-side web app      | Authorization Code    | Server-side session    |
| Single-page app (SPA)    | Auth Code + PKCE      | Memory (not localStorage) |
| Mobile / native app      | Auth Code + PKCE      | Secure device storage  |
| Service-to-service       | Client Credentials    | Environment variable   |
| CLI / limited-input device | Device Authorization | Display user code      |

See `references/oauth-flows.md` for detailed flow diagrams and implementation steps.

### JWT Best Practices

- **Always validate**: signature, `exp`, `iss`, `aud` claims.
- **Keep payloads small** — JWTs are sent on every request.
- **Never store secrets** in JWT payloads — they are base64-encoded, not encrypted.
- **Use short expiry** for access tokens (5-15 minutes).
- **Use `RS256` or `ES256`** for production — avoid `HS256` for public APIs.
- **Rotate signing keys** periodically via JWKS endpoint.

See `references/jwt-reference.md` for structure, claims, and validation checklist.

### Token Refresh Pattern

```
Client                    Auth Server                 API
  |                           |                        |
  |--- Request (access token) ----------------------->|
  |<-- 401 Unauthorized -------------------------------|
  |                           |                        |
  |--- Refresh token -------->|                        |
  |<-- New access + refresh --|                        |
  |                           |                        |
  |--- Retry (new access token) --------------------->|
  |<-- 200 OK ----------------------------------------|
```

Refresh tokens must be:
- Stored securely (httpOnly cookie or secure device storage).
- Single-use (rotated on every refresh).
- Revocable (server-side denylist or family tracking).

### Session Management

For server-rendered apps using sessions:

- Generate cryptographically random session IDs (min 128 bits).
- Store session data server-side (Redis, database) — never in the cookie itself.
- Set cookie flags: `HttpOnly`, `Secure`, `SameSite=Lax` (or `Strict`).
- Implement idle timeout (15-30 min) and absolute timeout (8-24 hours).
- Regenerate session ID after login to prevent session fixation.

### RBAC (Role-Based Access Control)

Assign permissions to roles, assign roles to users:

```python
ROLES = {
    "viewer": ["read:projects", "read:reports"],
    "editor": ["read:projects", "read:reports", "write:projects"],
    "admin":  ["read:projects", "read:reports", "write:projects",
               "manage:users", "manage:settings"],
}

def has_permission(user, permission):
    return any(permission in ROLES[role] for role in user.roles)
```

### ABAC (Attribute-Based Access Control)

Evaluate policies based on subject, resource, action, and context attributes:

```python
def can_access(subject, action, resource, context):
    # Owners can always edit their own resources
    if action == "edit" and resource.owner_id == subject.id:
        return True
    # Admins can do anything in their department
    if "admin" in subject.roles and resource.department == subject.department:
        return True
    # Time-based: maintenance window access
    if action == "deploy" and not context.is_maintenance_window:
        return False
    return False
```

Use ABAC when RBAC is too coarse — when access depends on resource ownership,
department, time, location, or other contextual attributes.

### API Key Patterns

- Prefix keys for identification: `sk_live_abc123` (secret), `pk_live_xyz789` (public).
- Hash keys before storage (SHA-256). Show the full key only once at creation.
- Scope keys to specific permissions — never grant full access by default.
- Include key ID in the key itself for lookup without hashing every key.

### CORS Configuration

```javascript
// Express.js example
app.use(cors({
  origin: ["https://app.example.com"],     // Never use "*" with credentials
  methods: ["GET", "POST", "PUT", "DELETE"],
  allowedHeaders: ["Authorization", "Content-Type"],
  credentials: true,                        // Allow cookies
  maxAge: 86400,                            // Cache preflight 24h
}));
```

Never set `Access-Control-Allow-Origin: *` when using credentials.

### Security Review Checklist

1. Authentication tokens are transmitted only over HTTPS.
2. Passwords are hashed with bcrypt/argon2 (never MD5/SHA1).
3. JWT signatures are validated; `alg: none` is rejected.
4. Refresh tokens are rotated on use and revocable.
5. Session IDs are regenerated after authentication.
6. CORS origins are explicitly listed (no wildcards with credentials).
7. Rate limiting is applied to login and token endpoints.
8. Failed login attempts trigger progressive delays or lockout.

## Examples

### Example 1: Implement OAuth2 PKCE flow for a SPA

```
User: Add Google OAuth login to our React SPA.

Agent: Reads the frontend codebase, then implements:
  1. Generate code_verifier (random 43-128 chars) and code_challenge (SHA256 + base64url).
  2. Redirect to Google authorize endpoint with code_challenge.
  3. Handle callback — exchange authorization code + code_verifier for tokens.
  4. Store access token in memory (not localStorage).
  5. Set up silent refresh via hidden iframe or refresh token rotation.
  6. Add logout that revokes tokens and clears state.
```

### Example 2: Add JWT authentication to an Express API

```
User: Secure our Express API with JWT.

Agent: Reads existing route structure, then:
  1. Creates auth middleware that extracts Bearer token from Authorization header.
  2. Validates JWT signature using RS256 public key from JWKS endpoint.
  3. Checks exp, iss, aud claims.
  4. Attaches decoded user to request context.
  5. Creates role-checking middleware: requireRole("admin").
  6. Applies middleware to protected routes.
  7. Adds token refresh endpoint with rotation.
```

### Example 3: Design RBAC permission system

```
User: We need a permission system for our multi-tenant SaaS.

Agent: Designs a role-permission model:
  - Roles are scoped to organizations (org admin vs global admin).
  - Permissions follow resource:action format (projects:write).
  - Creates database schema: users, roles, permissions, user_roles, role_permissions.
  - Implements middleware that checks user's org-scoped roles.
  - Adds permission caching with invalidation on role changes.
  - Writes migration scripts and seed data for default roles.
```
